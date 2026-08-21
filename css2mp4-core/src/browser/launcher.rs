use std::path::PathBuf;

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;

use crate::error::{Error, Result};
use crate::options::RenderOptions;

/// ヘッドレス Chromium インスタンスを起動・初期化するモジュール。
pub struct BrowserLauncher;

impl BrowserLauncher {
    /// 与えられたレンダリング設定に基づき、ヘッドレスブラウザを起動します。
    pub async fn launch(opts: &RenderOptions) -> Result<(Browser, tokio::task::JoinHandle<()>)> {
        let mut builder = BrowserConfig::builder()
            .window_size(opts.width, opts.height)
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: opts.width,
                height: opts.height,
                device_scale_factor: Some(1.0),
                ..Default::default()
            })
            .arg("--hide-scrollbars")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-renderer-backgrounding")
            .arg("--force-device-scale-factor=1");

        if let Some(ref path) = opts.chrome_path {
            builder = builder.chrome_executable(path);
        } else if let Some(detected) = find_default_chrome_path() {
            builder = builder.chrome_executable(detected);
        }

        let config = builder
            .build()
            .map_err(|e| Error::BrowserLaunch(format!("BrowserConfigの生成に失敗しました: {e}")))?;

        let (browser, mut handler) = Browser::launch(config).await?;

        // chromiumoxide は Handler を poll し続けないと CDP メッセージが処理されないため、
        // バックグラウンドタスクとして回す。
        let handle = tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });

        Ok((browser, handle))
    }
}

/// OSごとの標準的な Google Chrome のインストールパスを探索します。
fn find_default_chrome_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Users\harun\AppData\Local\Google\Chrome\Application\chrome.exe",
        ];
        for path_str in candidates {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome");
        if path.exists() {
            return Some(path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = ["/usr/bin/google-chrome", "/usr/bin/chromium-browser", "/usr/bin/chromium"];
        for path_str in candidates {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

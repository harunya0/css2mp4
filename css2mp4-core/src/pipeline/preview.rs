use std::io::Write;
use std::path::PathBuf;

use tempfile::NamedTempFile;

use crate::browser::FrameCapturer;
use crate::error::Result;
use crate::options::RenderOptions;

/// プレビュー設定。
#[derive(Debug, Clone)]
pub struct PreviewOptions {
    /// HTML 文字列またはローカル HTML ファイルパス。
    pub input_html: Option<String>,
    pub input_path: Option<PathBuf>,
    /// キャプチャ対象時刻（秒）。
    pub time_seconds: f64,
    /// ビューポート幅。
    pub width: u32,
    /// ビューポート高さ。
    pub height: u32,
    /// 透過背景とするか。
    pub transparent: bool,
    /// Chromium 実行ファイルのパス（省略時は自動検出）。
    pub chrome_path: Option<PathBuf>,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            input_html: None,
            input_path: None,
            time_seconds: 0.0,
            width: 1920,
            height: 1080,
            transparent: false,
            chrome_path: None,
        }
    }
}

/// 単一フレームのプレビュー PNG 画像（バイト列）を生成する。
pub async fn preview_frame(opts: &PreviewOptions) -> Result<Vec<u8>> {
    // HTML文字列が指定されている場合は一時ファイルを作成
    let (_temp_file, input_path) = match (&opts.input_html, &opts.input_path) {
        (Some(html), _) => {
            let mut temp = NamedTempFile::new()?;
            temp.write_all(html.as_bytes())?;
            let path = temp.path().to_path_buf();
            (Some(temp), path)
        }
        (None, Some(path)) => (None, path.clone()),
        (None, None) => {
            return Err(crate::error::Error::BrowserLaunch(
                "入力 HTML またはパスが指定されていません".to_string(),
            ));
        }
    };

    let render_opts = RenderOptions {
        input: input_path,
        output: PathBuf::new(),
        fps: 60,
        duration: opts.time_seconds.max(1.0),
        width: opts.width,
        height: opts.height,
        transparent: opts.transparent,
        ffmpeg_path: PathBuf::from("ffmpeg"),
        chrome_path: opts.chrome_path.clone(),
    };

    render_opts.validate()?;
    let capturer = FrameCapturer::launch(&render_opts).await?;
    let png_bytes = capturer
        .capture_frame_png(opts.time_seconds, opts.transparent)
        .await?;

    Ok(png_bytes)
}

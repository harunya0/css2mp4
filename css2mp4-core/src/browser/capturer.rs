use chromiumoxide::cdp::browser_protocol::emulation::SetDefaultBackgroundColorOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::{Browser, Page};

use crate::browser::animation::AnimationController;
use crate::browser::launcher::BrowserLauncher;
use crate::browser::sampler::ComputedSample;
use crate::browser::url::to_file_url;
use crate::error::Result;
use crate::options::RenderOptions;

/// ヘッドレス Chromium を操作し、ページの仮想時間シークと PNG キャプチャを行うキャプチャエンジン。
pub struct FrameCapturer {
    _browser: Browser,
    page: Page,
    animation_ids: Vec<String>,
    _handle: tokio::task::JoinHandle<()>,
}

impl FrameCapturer {
    /// 与えられたレンダリング設定に基づき、ブラウザを起動して対象ページをロードします。
    pub async fn launch(opts: &RenderOptions) -> Result<Self> {
        let (browser, handle) = BrowserLauncher::launch(opts).await?;
        let page = browser.new_page("about:blank").await?;

        // 透過背景が必要な場合は、ブラウザの背景色を透明に設定
        if opts.transparent {
            let bg_override = SetDefaultBackgroundColorOverrideParams {
                color: Some(chromiumoxide::cdp::browser_protocol::dom::Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: Some(0.0),
                }),
            };
            page.execute(bg_override).await?;
        }

        // ページをロードし、アニメーション ID を収集＆自動再生停止
        let target_url = to_file_url(&opts.input);
        let animation_ids =
            AnimationController::initialize_and_collect_ids(&page, &target_url).await?;

        Ok(FrameCapturer {
            _browser: browser,
            page,
            animation_ids,
            _handle: handle,
        })
    }

    /// 指定された時間（秒）へアニメーションをシークし、PNG スクリーンショットを取得します。
    pub async fn capture_frame(&self, time_seconds: f64) -> Result<Vec<u8>> {
        AnimationController::seek(&self.page, &self.animation_ids, time_seconds * 1000.0).await?;

        let screenshot_params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .from_surface(true)
            .build();

        let png_bytes = self.page.screenshot(screenshot_params).await?;
        Ok(png_bytes)
    }

    /// `capture_frame` のエイリアス。
    pub async fn capture_frame_png(
        &self,
        time_seconds: f64,
        _transparent: bool,
    ) -> Result<Vec<u8>> {
        self.capture_frame(time_seconds).await
    }

    /// 指定セレクタの要素の Computed Style を指定時刻でサンプリングします。
    pub async fn sample_style(&self, selector: &str, time_seconds: f64) -> Result<ComputedSample> {
        AnimationController::seek(&self.page, &self.animation_ids, time_seconds * 1000.0).await?;
        crate::browser::sampler::sample_element_style(&self.page, selector).await
    }

    /// 検出されたアニメーションの件数を返します。
    pub fn animation_count(&self) -> usize {
        self.animation_ids.len()
    }
}

use std::collections::HashSet;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::animation::{
    EnableParams as AnimationEnableParams, EventAnimationStarted, SeekAnimationsParams,
    SetPlaybackRateParams,
};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::{Page, ScreenshotParams};
use futures::StreamExt;

use crate::browser::sampler::{build_sampling_script, RawStyleSample, StyleSample};
use crate::browser::url::to_file_url;
use crate::error::{Error, Result};
use crate::options::RenderOptions;

/// CSS アニメーションを仕込んだページを開き、CDP の `Animation` ドメインを使って
/// 決定論的にフレームをキャプチャ・サンプリングするためのハンドル。
pub struct FrameCapturer {
    _browser: Browser,
    _handler_task: tokio::task::JoinHandle<()>,
    page: Page,
    animation_ids: Vec<String>,
}

impl FrameCapturer {
    /// ヘッドレスブラウザを起動し、対象ページのアニメーションを検出・停止させた状態で初期化する。
    pub async fn launch(opts: &RenderOptions) -> Result<Self> {
        let mut builder = BrowserConfig::builder()
            .window_size(opts.width, opts.height)
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: opts.width,
                height: opts.height,
                device_scale_factor: Some(1.0),
                ..Default::default()
            });

        if let Some(chrome) = &opts.chrome_path {
            builder = builder.chrome_executable(chrome);
        }

        let config = builder.build().map_err(Error::BrowserLaunch)?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| Error::BrowserLaunch(e.to_string()))?;

        // chromiumoxide は Handler を poll し続けないと CDP メッセージが処理されないため、
        // バックグラウンドタスクとして回す。
        let handler_task = tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });

        let input_url = to_file_url(&opts.input);
        let page = browser.new_page("about:blank").await?;

        // ページ読み込み前に Animation ドメインを有効化
        page.execute(AnimationEnableParams {}).await?;

        let mut animation_events = page.event_listener::<EventAnimationStarted>().await?;

        page.goto(input_url.as_str()).await?;
        page.wait_for_navigation().await?;

        // アニメーション ID を収集
        let mut animation_ids: HashSet<String> = HashSet::new();
        let collect_deadline = tokio::time::sleep(Duration::from_millis(300));
        tokio::pin!(collect_deadline);
        loop {
            tokio::select! {
                _ = &mut collect_deadline => break,
                maybe_event = animation_events.next() => {
                    match maybe_event {
                        Some(evt) => {
                            animation_ids.insert(evt.animation.id.clone());
                        }
                        None => break,
                    }
                }
            }
        }

        // タイムラインの自動再生を停止（以降は seekAnimations でシーク制御）
        page.execute(SetPlaybackRateParams { playback_rate: 0.0 })
            .await?;

        Ok(Self {
            _browser: browser,
            _handler_task: handler_task,
            page,
            animation_ids: animation_ids.into_iter().collect(),
        })
    }

    /// `time_seconds` 時点までアニメーションをシークし、PNG スクリーンショットを取得する。
    pub async fn capture_frame_png(&self, time_seconds: f64, transparent: bool) -> Result<Vec<u8>> {
        self.seek_to(time_seconds).await?;

        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .omit_background(transparent)
            .build();

        let png = self.page.screenshot(params).await?;
        Ok(png)
    }

    /// `time_seconds` 時点までアニメーションをシークし、指定セレクタの要素からスタイルをサンプリングする。
    pub async fn sample_style(&self, selector: &str, time_seconds: f64) -> Result<StyleSample> {
        self.seek_to(time_seconds).await?;

        let script = build_sampling_script(selector);
        let raw: RawStyleSample = self.page.evaluate(script).await?.into_value()?;
        Ok(raw.into())
    }

    /// アニメーションを指定秒数位置へシークする。
    async fn seek_to(&self, time_seconds: f64) -> Result<()> {
        if !self.animation_ids.is_empty() {
            self.page
                .execute(SeekAnimationsParams {
                    animations: self.animation_ids.clone(),
                    current_time: time_seconds * 1000.0,
                })
                .await?;
        }
        Ok(())
    }

    /// 検出されたアニメーションの件数。
    pub fn animation_count(&self) -> usize {
        self.animation_ids.len()
    }
}

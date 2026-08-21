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

use crate::error::{Error, Result};
use crate::options::RenderOptions;

/// CSSアニメーションを仕込んだページを開き、CDPの `Animation` ドメインを使って
/// 決定論的にフレームをキャプチャするためのハンドル。
///
/// アプローチ:
/// 1. `Animation.enable` を有効にした状態でページを読み込み、発火する
///    `animationStarted` イベントからアニメーションIDを収集する。
/// 2. `Animation.setPlaybackRate(0)` でドキュメントタイムラインを止める。
/// 3. フレームごとに `Animation.seekAnimations` で狙った時刻へシークしてから
///    `Page.captureScreenshot` でPNGを取得する。
///
/// これによりリアルタイム再生に依存せず、fpsに関わらず正確な時刻のフレームを
/// 取得できる（Web Animations APIではなく純粋なCSSアニメーション/トランジションが対象）。
pub struct FrameCapturer {
    _browser: Browser,
    // Browser を drop するとハンドラも終了してしまうため保持しておく。
    _handler_task: tokio::task::JoinHandle<()>,
    page: Page,
    animation_ids: Vec<String>,
}

impl FrameCapturer {
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

        let config = builder
            .build()
            .map_err(Error::BrowserLaunch)?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| Error::BrowserLaunch(e.to_string()))?;

        // chromiumoxideはHandlerを誰かがpollし続けないとCDPメッセージが
        // 処理されないため、バックグラウンドタスクとして回し続ける。
        let handler_task = tokio::spawn(async move {
            while let Some(_event) = handler.next().await {
                // イベントはpage.event_listener側でも別途subscribeする。
            }
        });

        let input_url = to_file_url(&opts.input);
        let page = browser.new_page("about:blank").await?;

        // ページ読み込み前にAnimationドメインを有効化しておくことで、
        // ロード直後に発火するアニメーションも取りこぼさないようにする。
        page.execute(AnimationEnableParams {}).await?;

        let mut animation_events = page.event_listener::<EventAnimationStarted>().await?;

        page.goto(input_url.as_str()).await?;
        page.wait_for_navigation().await?;

        // アニメーションが発火し切るのを短時間待って収集する。
        // （CSSアニメーションはロード直後〜数フレーム以内に開始される前提）
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

        // タイムラインを固定し、以降は seekAnimations で明示的に時刻制御する。
        page.execute(SetPlaybackRateParams { playback_rate: 0.0 })
            .await?;

        Ok(Self {
            _browser: browser,
            _handler_task: handler_task,
            page,
            animation_ids: animation_ids.into_iter().collect(),
        })
    }

    /// `time_seconds` の時点までアニメーションをシークし、PNGフレームを1枚取得する。
    pub async fn capture_frame_png(&self, time_seconds: f64, transparent: bool) -> Result<Vec<u8>> {
        if !self.animation_ids.is_empty() {
            self.page
                .execute(SeekAnimationsParams {
                    animations: self.animation_ids.clone(),
                    current_time: time_seconds * 1000.0,
                })
                .await?;
        }

        let params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .omit_background(transparent)
            .build();

        let png = self.page.screenshot(params).await?;
        Ok(png)
    }

    /// キャプチャ対象として検出されたアニメーション数（デバッグ/ログ用）。
    pub fn animation_count(&self) -> usize {
        self.animation_ids.len()
    }

    /// `time_seconds` の時点までアニメーションをシークし、指定セレクタの
    /// 要素から `transform` / `opacity` をサンプリングする。
    ///
    /// `transform` は `getComputedStyle` から得られる行列
    /// （`matrix(a, b, c, d, e, f)` または `matrix3d(...)`）を分解し、
    /// 平行移動量(px)・回転角(度)・スケールに変換する。
    /// せん断（skew）を伴う変形は正しく分解できない点に注意。
    pub async fn sample_style(
        &self,
        selector: &str,
        time_seconds: f64,
    ) -> Result<StyleSample> {
        if !self.animation_ids.is_empty() {
            self.page
                .execute(SeekAnimationsParams {
                    animations: self.animation_ids.clone(),
                    current_time: time_seconds * 1000.0,
                })
                .await?;
        }

        let script = format!(
            r#"(() => {{
                const el = document.querySelector({selector});
                if (!el) {{ throw new Error('element not found: ' + {selector}); }}
                const cs = getComputedStyle(el);
                const opacity = parseFloat(cs.opacity);
                let tx = 0, ty = 0, angleDeg = 0, scale = 1;
                const t = cs.transform;
                if (t && t !== 'none') {{
                    const m = new DOMMatrixReadOnly(t);
                    tx = m.m41;
                    ty = m.m42;
                    scale = Math.sqrt(m.m11 * m.m11 + m.m12 * m.m12);
                    angleDeg = Math.atan2(m.m12, m.m11) * (180 / Math.PI);
                }}
                return {{ opacity, tx, ty, angleDeg, scale }};
            }})()"#,
            selector = serde_json::to_string(selector).unwrap_or_else(|_| "\"body\"".to_string()),
        );

        let result: RawStyleSample = self.page.evaluate(script).await?.into_value()?;

        Ok(StyleSample {
            opacity: result.opacity,
            translate_x: result.tx,
            translate_y: result.ty,
            rotation_deg: result.angle_deg,
            scale: result.scale,
        })
    }
}

/// [`FrameCapturer::sample_style`] が1フレーム分返すサンプル値。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleSample {
    /// CSS `opacity`（0.0〜1.0）。
    pub opacity: f64,
    /// `transform` 行列から取り出した水平移動量（px）。
    pub translate_x: f64,
    /// `transform` 行列から取り出した垂直移動量（px）。
    pub translate_y: f64,
    /// `transform` 行列から取り出した回転角（度）。
    pub rotation_deg: f64,
    /// `transform` 行列から取り出した拡大率（1.0 = 等倍）。
    pub scale: f64,
}

#[derive(serde::Deserialize)]
struct RawStyleSample {
    opacity: f64,
    tx: f64,
    ty: f64,
    #[serde(rename = "angleDeg")]
    angle_deg: f64,
    scale: f64,
}

fn to_file_url(path: &std::path::Path) -> String {
    let abs = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

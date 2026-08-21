//! フェーズ1で定義した各モジュール（`capture` / `encode` / `ymmp`）を
//! 組み合わせた高レベルAPI。CLIはこのモジュールの関数を呼び出すだけで
//! よいようにする。

use std::path::Path;

use crate::capture::FrameCapturer;
use crate::encode::FfmpegEncoder;
use crate::error::Result;
use crate::options::{RenderOptions, VideoFormat};
use crate::ymmp::{MotionSamples, YmmpProject};

/// レンダリングタスクの進捗を受け取るコールバック。
/// `(現在のフレーム番号, 総フレーム数)` を渡す。
pub trait ProgressSink {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32);
}

/// 何もしない進捗シンク（デフォルト）。
pub struct NoopProgress;
impl ProgressSink for NoopProgress {
    fn on_frame(&mut self, _frame_index: u32, _total_frames: u32) {}
}

impl<F: FnMut(u32, u32)> ProgressSink for F {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32) {
        self(frame_index, total_frames)
    }
}

/// `render` サブコマンド相当: CSSアニメーションをMP4/透過WebMへレンダリングする。
pub async fn render_video(opts: &RenderOptions, mut progress: impl ProgressSink) -> Result<()> {
    opts.validate()?;
    let format = VideoFormat::from_output_path(&opts.output, opts.transparent)?;

    let capturer = FrameCapturer::launch(opts).await?;
    let mut encoder = FfmpegEncoder::spawn(opts, format)?;

    let total_frames = opts.total_frames();
    for frame_index in 0..total_frames {
        let time_seconds = frame_index as f64 / opts.fps as f64;
        let png = capturer
            .capture_frame_png(time_seconds, format.is_transparent())
            .await?;
        encoder.write_frame(&png).await?;
        progress.on_frame(frame_index + 1, total_frames);
    }

    encoder.finish().await?;
    Ok(())
}

/// `export-ymmp` サブコマンド相当: 既存の `.ymmp` 内の指定アイテムへ、
/// CSSアニメーションからサンプリングしたモーションを上書きする。
///
/// `select` でタイムライン内のアイテムを選ぶ（インデックス指定のみ対応。
/// `Remark` 指定は [`YmmpProject::item_by_remark_mut`] を直接使う）。
pub async fn overwrite_ymmp_motion(
    opts: &RenderOptions,
    selector: &str,
    ymmp_input: impl AsRef<Path>,
    ymmp_output: impl AsRef<Path>,
    timeline_index: usize,
    item_index: usize,
    mut progress: impl ProgressSink,
) -> Result<()> {
    opts.validate()?;

    let capturer = FrameCapturer::launch(opts).await?;

    let total_frames = opts.total_frames();
    let mut samples = MotionSamples::default();
    for frame_index in 0..total_frames {
        let time_seconds = frame_index as f64 / opts.fps as f64;
        let style = capturer.sample_style(selector, time_seconds).await?;
        samples.translate_x.push(style.translate_x);
        samples.translate_y.push(style.translate_y);
        samples.zoom_percent.push(style.scale * 100.0);
        samples.rotation_deg.push(style.rotation_deg);
        samples.opacity_percent.push(style.opacity * 100.0);
        progress.on_frame(frame_index + 1, total_frames);
    }

    let mut project = YmmpProject::load(&ymmp_input)?;
    let item = project
        .item_mut(timeline_index, item_index)
        .ok_or_else(|| crate::error::Error::InputNotFound(ymmp_input.as_ref().to_path_buf()))?;

    samples.overwrite_item(item, opts.duration)?;
    project.save(&ymmp_output)?;

    Ok(())
}

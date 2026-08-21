use std::path::Path;

use crate::browser::FrameCapturer;
use crate::error::{Error, Result};
use crate::options::RenderOptions;
use crate::pipeline::progress::ProgressSink;
use crate::ymmp::{MotionSamples, YmmpProject};

/// 既存の `.ymmp` 内の指定アイテムへ、CSS アニメーションからサンプリングしたモーションを上書きするパイプライン。
pub async fn overwrite_ymmp_motion(
    opts: &RenderOptions,
    selector: &str,
    ymmp_input: impl AsRef<Path>,
    ymmp_output: impl AsRef<Path>,
    timeline_index: usize,
    item_index: usize,
    progress: impl ProgressSink,
) -> Result<()> {
    overwrite_ymmp_motion_with_tolerance(
        opts,
        selector,
        ymmp_input,
        ymmp_output,
        timeline_index,
        item_index,
        0.5,
        progress,
    )
    .await
}

/// 許容誤差 `tolerance` を指定してモーションを上書きするパイプライン。
pub async fn overwrite_ymmp_motion_with_tolerance(
    opts: &RenderOptions,
    selector: &str,
    ymmp_input: impl AsRef<Path>,
    ymmp_output: impl AsRef<Path>,
    timeline_index: usize,
    item_index: usize,
    tolerance: f64,
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
        .ok_or_else(|| Error::InputNotFound(ymmp_input.as_ref().to_path_buf()))?;

    samples.overwrite_item_with_tolerance(item, opts.duration, tolerance)?;
    project.save(&ymmp_output)?;

    Ok(())
}

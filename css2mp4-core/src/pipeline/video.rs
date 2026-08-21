use crate::browser::FrameCapturer;
use crate::encoder::{FfmpegEncoder, VideoFormat};
use crate::error::Result;
use crate::options::RenderOptions;
use crate::pipeline::progress::ProgressSink;

/// CSS アニメーションを MP4 または透過 WebM 動画としてレンダリングするパイプライン。
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

use anyhow::Result;
use css2mp4_core::options::RenderOptions;
use css2mp4_core::pipeline::render_video;

use crate::args::RenderArgs;
use crate::ui::CliProgress;

/// `render` サブコマンドの実行処理。
pub async fn handle_render(args: RenderArgs) -> Result<()> {
    let opts = RenderOptions {
        input: args.input,
        output: args.output,
        fps: args.fps,
        duration: args.duration,
        width: args.width,
        height: args.height,
        transparent: args.transparent,
        ffmpeg_path: args.ffmpeg,
        chrome_path: args.chrome,
    };

    let progress = CliProgress::new();
    render_video(&opts, progress).await?;

    println!("レンダリングが完了しました: {}", opts.output.display());
    Ok(())
}

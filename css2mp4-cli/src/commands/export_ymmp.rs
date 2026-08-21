use std::path::PathBuf;

use anyhow::Result;
use css2mp4_core::options::RenderOptions;
use css2mp4_core::pipeline::overwrite_ymmp_motion;

use crate::args::ExportYmmpArgs;
use crate::ui::CliProgress;

/// `export-ymmp` サブコマンドの実行処理。
pub async fn handle_export_ymmp(args: ExportYmmpArgs) -> Result<()> {
    let opts = RenderOptions {
        input: args.input,
        output: PathBuf::new(), // export-ymmp では動画出力を行わないため未使用
        fps: args.fps,
        duration: args.duration,
        width: args.width,
        height: args.height,
        transparent: false,
        ffmpeg_path: PathBuf::from("ffmpeg"),
        chrome_path: args.chrome,
    };

    let output_path = args.output.unwrap_or_else(|| args.ymmp.clone());
    let progress = CliProgress::new();

    overwrite_ymmp_motion(
        &opts,
        &args.selector,
        &args.ymmp,
        &output_path,
        args.timeline_index,
        args.item_index,
        progress,
    )
    .await?;

    println!(
        ".ymmpへのモーション書き込みが完了しました: {}",
        output_path.display()
    );
    Ok(())
}

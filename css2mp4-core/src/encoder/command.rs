use std::path::Path;

use crate::encoder::format::VideoFormat;
use crate::options::RenderOptions;

/// FFmpeg コマンドライン引数の生成を担当するビルダー。
pub struct FfmpegCommandBuilder;

impl FfmpegCommandBuilder {
    /// 与えられた設定から FFmpeg の起動引数リストを構築します。
    pub fn build_args(opts: &RenderOptions, format: VideoFormat, output_path: &Path) -> Vec<String> {
        let mut args = vec![
            "-y".to_string(),
            "-f".to_string(),
            "image2pipe".to_string(),
            "-vcodec".to_string(),
            "png".to_string(),
            "-r".to_string(),
            opts.fps.to_string(),
            "-i".to_string(),
            "-".to_string(),
        ];

        for &arg in format.ffmpeg_args() {
            args.push(arg.to_string());
        }

        args.push(output_path.to_string_lossy().to_string());
        args
    }
}

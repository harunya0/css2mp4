use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use crate::error::{Error, Result};
use crate::options::{RenderOptions, VideoFormat};

/// PNGフレーム列をstdin経由でffmpegに流し込み、動画ファイルへエンコードするパイプ。
///
/// `image2pipe` デマルチプレクサでPNGフレームを連結ストリームとして渡すため、
/// フレームを一度もディスクへ書き出さずに済む。
pub struct FfmpegEncoder {
    child: Child,
    format: VideoFormat,
}

impl FfmpegEncoder {
    pub fn spawn(opts: &RenderOptions, format: VideoFormat) -> Result<Self> {
        let mut cmd = Command::new(&opts.ffmpeg_path);
        cmd.arg("-y") // 出力ファイルを上書き
            .args(["-loglevel", "error"])
            .args(["-f", "image2pipe"])
            .args(["-framerate", &opts.fps.to_string()])
            .args(["-i", "-"]);

        match format {
            VideoFormat::Mp4 => {
                cmd.args(["-c:v", "libx264"])
                    .args(["-pix_fmt", "yuv420p"])
                    .args(["-movflags", "+faststart"]);
            }
            VideoFormat::WebmTransparent => {
                cmd.args(["-c:v", "libvpx-vp9"])
                    .args(["-pix_fmt", "yuva420p"])
                    // アルファチャンネルを保持するために alt-ref を無効化する。
                    .args(["-auto-alt-ref", "0"]);
            }
        }

        cmd.arg(&opts.output);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(Error::FfmpegSpawn)?;

        Ok(Self { child, format })
    }

    /// PNGフレームを1枚書き込む。
    pub async fn write_frame(&mut self, png_bytes: &[u8]) -> Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .expect("stdin was configured as piped");
        stdin.write_all(png_bytes).await.map_err(Error::FrameWrite)?;
        Ok(())
    }

    /// stdinを閉じてffmpegの終了を待ち、成功したかどうかを確認する。
    pub async fn finish(mut self) -> Result<()> {
        // stdin をdropしてEOFを送る。
        drop(self.child.stdin.take());

        let output = self
            .child
            .wait_with_output()
            .await
            .map_err(Error::FfmpegSpawn)?;

        if !output.status.success() {
            return Err(Error::FfmpegExit {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let _ = self.format; // フォーマットは主にログ/デバッグ用に保持している
        Ok(())
    }
}

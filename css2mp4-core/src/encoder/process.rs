use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};

use crate::encoder::command::FfmpegCommandBuilder;
use crate::encoder::format::VideoFormat;
use crate::error::{Error, Result};
use crate::options::RenderOptions;

/// FFmpeg 子プロセスのライフサイクル管理および stdin ストリーミングを行うエンコーダ。
pub struct FfmpegProcess {
    child: Child,
    stdin: ChildStdin,
}

impl FfmpegProcess {
    /// FFmpeg プロセスを起動し、stdin への書き込みパイプを初期化します。
    pub fn spawn(opts: &RenderOptions, format: VideoFormat) -> Result<Self> {
        let args = FfmpegCommandBuilder::build_args(opts, format, &opts.output);

        let mut child = Command::new(&opts.ffmpeg_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Ffmpeg(format!("FFmpegプロセスの起動に失敗しました: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Ffmpeg("FFmpegの標準入力を取得できませんでした".to_string()))?;

        Ok(FfmpegProcess { child, stdin })
    }

    /// 1 フレーム分の PNG データを FFmpeg の stdin に書き込みます。
    pub async fn write_frame(&mut self, png_bytes: &[u8]) -> Result<()> {
        self.stdin
            .write_all(png_bytes)
            .await
            .map_err(|e| Error::Ffmpeg(format!("フレームデータの書き込みに失敗しました: {e}")))?;
        Ok(())
    }

    /// stdin をクローズし、FFmpeg の終了を待機します。
    pub async fn finish(self) -> Result<()> {
        drop(self.stdin);

        let output = self
            .child
            .wait_with_output()
            .await
            .map_err(|e| Error::Ffmpeg(format!("FFmpegプロセスの待機に失敗しました: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Ffmpeg(format!(
                "FFmpegの実行がエラーで終了しました (code: {:?}):\n{stderr}",
                output.status.code()
            )));
        }

        Ok(())
    }
}

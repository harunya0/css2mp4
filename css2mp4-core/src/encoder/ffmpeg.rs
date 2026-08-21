use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use crate::encoder::format::VideoFormat;
use crate::error::{Error, Result};
use crate::options::RenderOptions;

/// PNG フレーム列を stdin 経由で FFmpeg に流し込み、動画ファイルへエンコードするパイプライン。
///
/// `image2pipe` デマルチプレクサで PNG フレームを連結ストリームとして渡すため、
/// フレームをディスクへ一切書き出さずに高速処理できます。
pub struct FfmpegEncoder {
    child: Child,
    format: VideoFormat,
}

impl FfmpegEncoder {
    /// FFmpeg プロセスを子プロセスとして起動し、入力パイプを開く。
    pub fn spawn(opts: &RenderOptions, format: VideoFormat) -> Result<Self> {
        let mut cmd = Command::new(&opts.ffmpeg_path);
        cmd.arg("-y") // 上書き許可
            .args(["-loglevel", "error"])
            .args(["-f", "image2pipe"])
            .args(["-framerate", &opts.fps.to_string()])
            .args(["-i", "-"]);

        // コーデック・ピクセルフォーマット引数を追加
        cmd.args(format.ffmpeg_args());

        cmd.arg(&opts.output);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(Error::FfmpegSpawn)?;

        Ok(Self { child, format })
    }

    /// PNG フレームを 1 枚 FFmpeg の stdin に書き込む。
    pub async fn write_frame(&mut self, png_bytes: &[u8]) -> Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .expect("stdin was configured as piped");
        stdin.write_all(png_bytes).await.map_err(Error::FrameWrite)?;
        Ok(())
    }

    /// stdin を閉じて FFmpeg の終了を待ち、エンコード結果を検証する。
    pub async fn finish(mut self) -> Result<()> {
        // stdin を drop して EOF を送信
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

        let _ = self.format;
        Ok(())
    }
}

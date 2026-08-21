use std::path::PathBuf;

use crate::error::{Error, Result};
pub use crate::encoder::VideoFormat;

/// レンダリングタスクの設定値。
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// 入力 HTML ファイルへのパス。
    pub input: PathBuf,
    /// 出力ファイルパス（.mp4 / .webm / .ymmp）。
    pub output: PathBuf,
    /// フレームレート (fps)。
    pub fps: u32,
    /// 出力する長さ（秒）。
    pub duration: f64,
    /// 出力解像度の幅 (px)。
    pub width: u32,
    /// 出力解像度の高さ (px)。
    pub height: u32,
    /// 透過動画として出力するか（WebM VP9 + alpha）。
    pub transparent: bool,
    /// FFmpeg 実行ファイルのパス（省略時は PATH 上の `ffmpeg`）。
    pub ffmpeg_path: PathBuf,
    /// Chromium 実行ファイルのパス（省略時は自動検出）。
    pub chrome_path: Option<PathBuf>,
}

impl RenderOptions {
    /// 入力ファイルの存在チェックなど、最低限のバリデーションを行う。
    pub fn validate(&self) -> Result<()> {
        if !self.input.exists() {
            return Err(Error::InputNotFound(self.input.clone()));
        }
        Ok(())
    }

    /// 総フレーム数（fps × duration を切り上げ）。
    pub fn total_frames(&self) -> u32 {
        (self.fps as f64 * self.duration).ceil() as u32
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            input: PathBuf::new(),
            output: PathBuf::new(),
            fps: 60,
            duration: 3.0,
            width: 1920,
            height: 1080,
            transparent: false,
            ffmpeg_path: PathBuf::from("ffmpeg"),
            chrome_path: None,
        }
    }
}

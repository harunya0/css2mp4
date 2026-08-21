use std::path::PathBuf;

use clap::Parser;

/// `render` サブコマンドの引数定義。
#[derive(Parser, Debug)]
pub struct RenderArgs {
    /// 入力HTMLファイル。
    pub input: PathBuf,
    /// 出力ファイル（拡張子で .mp4 / .webm を判別）。
    #[arg(short, long)]
    pub output: PathBuf,
    /// フレームレート。
    #[arg(long, default_value_t = 60)]
    pub fps: u32,
    /// 長さ（秒）。
    #[arg(long, default_value_t = 3.0)]
    pub duration: f64,
    /// 出力解像度の幅。
    #[arg(long, default_value_t = 1920)]
    pub width: u32,
    /// 出力解像度の高さ。
    #[arg(long, default_value_t = 1080)]
    pub height: u32,
    /// 透過動画として出力する（WebM VP9 + アルファチャンネル）。
    #[arg(long, default_value_t = false)]
    pub transparent: bool,
    /// ffmpeg実行ファイルのパス。
    #[arg(long, default_value = "ffmpeg")]
    pub ffmpeg: PathBuf,
    /// Chromium/Chrome実行ファイルのパス（省略時は自動検出）。
    #[arg(long)]
    pub chrome: Option<PathBuf>,
}

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "css2mp4-cli",
    about = "CSSアニメーションをMP4/透過WebM/YMM4プロジェクトへレンダリングするCLI",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// CSSアニメーションをMP4または透過WebM動画としてレンダリングする。
    Render(RenderArgs),

    /// 既存の.ymmpファイル内のアイテムに、CSSアニメーションから
    /// サンプリングしたモーション（X/Y/Zoom/Rotation/Opacity）を上書きする。
    ExportYmmp(ExportYmmpArgs),

    /// （フェーズ3・未実装）フロントエンド連携用のローカルAPIサーバーを起動する。
    Serve(ServeArgs),
}

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

#[derive(Parser, Debug)]
pub struct ExportYmmpArgs {
    /// 入力HTMLファイル。
    pub input: PathBuf,
    /// モーションのサンプリング元となるCSSセレクタ（例: "#target"）。
    #[arg(long)]
    pub selector: String,
    /// 上書き対象の既存 .ymmp ファイル。
    #[arg(long)]
    pub ymmp: PathBuf,
    /// 書き出し先の .ymmp ファイル（省略時は --ymmp を上書き）。
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// 対象タイムラインのインデックス（0始まり）。
    #[arg(long, default_value_t = 0)]
    pub timeline_index: usize,
    /// 対象アイテムのインデックス（0始まり）。
    #[arg(long, default_value_t = 0)]
    pub item_index: usize,
    /// フレームレート（サンプリング解像度）。
    #[arg(long, default_value_t = 60)]
    pub fps: u32,
    /// 長さ（秒）。
    #[arg(long, default_value_t = 3.0)]
    pub duration: f64,
    /// レンダリング用ビューポートの幅。
    #[arg(long, default_value_t = 1920)]
    pub width: u32,
    /// レンダリング用ビューポートの高さ。
    #[arg(long, default_value_t = 1080)]
    pub height: u32,
    /// Chromium/Chrome実行ファイルのパス（省略時は自動検出）。
    #[arg(long)]
    pub chrome: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// バインドするホストアドレス。
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// リッスンするポート番号。
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}


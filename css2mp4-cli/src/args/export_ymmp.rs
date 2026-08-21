use std::path::PathBuf;

use clap::Parser;

/// `export-ymmp` サブコマンドの引数定義。
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
    /// キーフレーム（中間点）削減の許容誤差（px / %）。0を指定すると全フレームを中間点として出力します。
    #[arg(long, default_value_t = 0.5)]
    pub tolerance: f64,
}

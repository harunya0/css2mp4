pub mod export_ymmp;
pub mod render;
pub mod serve;

pub use export_ymmp::ExportYmmpArgs;
pub use render::RenderArgs;
pub use serve::ServeArgs;

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

    /// フロントエンド連携用のローカルAPIサーバーを起動する。
    Serve(ServeArgs),
}

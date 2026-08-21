use std::path::PathBuf;

use clap::{Parser, Subcommand};
use css2mp4_core::options::RenderOptions;
use css2mp4_core::render::{self, ProgressSink};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(
    name = "css2mp4-cli",
    about = "CSSアニメーションをMP4/透過WebM/YMM4プロジェクトへレンダリングするCLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// CSSアニメーションをMP4または透過WebM動画としてレンダリングする。
    Render {
        /// 入力HTMLファイル。
        input: PathBuf,
        /// 出力ファイル（拡張子で .mp4 / .webm を判別）。
        #[arg(short, long)]
        output: PathBuf,
        /// フレームレート。
        #[arg(long, default_value_t = 60)]
        fps: u32,
        /// 長さ（秒）。
        #[arg(long, default_value_t = 3.0)]
        duration: f64,
        /// 出力解像度の幅。
        #[arg(long, default_value_t = 1920)]
        width: u32,
        /// 出力解像度の高さ。
        #[arg(long, default_value_t = 1080)]
        height: u32,
        /// 透過動画として出力する（WebM VP9 + アルファチャンネル）。
        #[arg(long, default_value_t = false)]
        transparent: bool,
        /// ffmpeg実行ファイルのパス。
        #[arg(long, default_value = "ffmpeg")]
        ffmpeg: PathBuf,
        /// Chromium/Chrome実行ファイルのパス（省略時は自動検出）。
        #[arg(long)]
        chrome: Option<PathBuf>,
    },

    /// 既存の.ymmpファイル内のアイテムに、CSSアニメーションから
    /// サンプリングしたモーション（X/Y/Zoom/Rotation/Opacity）を上書きする。
    ExportYmmp {
        /// 入力HTMLファイル。
        input: PathBuf,
        /// モーションのサンプリング元となるCSSセレクタ（例: "#target"）。
        #[arg(long)]
        selector: String,
        /// 上書き対象の既存 .ymmp ファイル。
        #[arg(long)]
        ymmp: PathBuf,
        /// 書き出し先の .ymmp ファイル（省略時は --ymmp を上書き）。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 対象タイムラインのインデックス（0始まり）。
        #[arg(long, default_value_t = 0)]
        timeline_index: usize,
        /// 対象アイテムのインデックス（0始まり）。
        #[arg(long, default_value_t = 0)]
        item_index: usize,
        /// フレームレート（サンプリング解像度）。
        #[arg(long, default_value_t = 60)]
        fps: u32,
        /// 長さ（秒）。
        #[arg(long, default_value_t = 3.0)]
        duration: f64,
        /// レンダリング用ビューポートの幅。
        #[arg(long, default_value_t = 1920)]
        width: u32,
        /// レンダリング用ビューポートの高さ。
        #[arg(long, default_value_t = 1080)]
        height: u32,
        /// Chromium/Chrome実行ファイルのパス（省略時は自動検出）。
        #[arg(long)]
        chrome: Option<PathBuf>,
    },

    /// （フェーズ3・未実装）フロントエンド連携用のローカルAPIサーバーを起動する。
    Serve {
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

struct CliProgress {
    bar: ProgressBar,
}

impl ProgressSink for CliProgress {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32) {
        self.bar.set_length(total_frames as u64);
        self.bar.set_position(frame_index as u64);
    }
}

fn new_progress_bar() -> CliProgress {
    let bar = ProgressBar::new(0);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} フレーム ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    CliProgress { bar }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Render {
            input,
            output,
            fps,
            duration,
            width,
            height,
            transparent,
            ffmpeg,
            chrome,
        } => {
            let opts = RenderOptions {
                input,
                output,
                fps,
                duration,
                width,
                height,
                transparent,
                ffmpeg_path: ffmpeg,
                chrome_path: chrome,
            };
            let progress = new_progress_bar();
            render::render_video(&opts, progress).await?;
            println!("レンダリングが完了しました: {}", opts.output.display());
        }

        Command::ExportYmmp {
            input,
            selector,
            ymmp,
            output,
            timeline_index,
            item_index,
            fps,
            duration,
            width,
            height,
            chrome,
        } => {
            let opts = RenderOptions {
                input,
                output: PathBuf::new(), // export-ymmpでは動画出力を行わないため未使用。
                fps,
                duration,
                width,
                height,
                transparent: false,
                ffmpeg_path: PathBuf::from("ffmpeg"),
                chrome_path: chrome,
            };
            let output_path = output.unwrap_or_else(|| ymmp.clone());
            let progress = new_progress_bar();
            render::overwrite_ymmp_motion(
                &opts,
                &selector,
                &ymmp,
                &output_path,
                timeline_index,
                item_index,
                progress,
            )
            .await?;
            println!(".ymmpへのモーション書き込みが完了しました: {}", output_path.display());
        }

        Command::Serve { port } => {
            eprintln!(
                "serve コマンドはフェーズ3（未実装）です。ポート指定: {port}\n\
                 TODOリスト フェーズ3「Axumを用いたローカルHTTP/WebSocket APIエンドポイントの実装」を参照。"
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

use std::path::PathBuf;

/// css-motion-core 全体で使われるエラー型。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("入力ファイルが見つかりません: {0}")]
    InputNotFound(PathBuf),

    #[error("出力形式を拡張子から判別できません: {0}")]
    UnknownOutputFormat(PathBuf),

    #[error("ヘッドレスChromiumの起動に失敗しました: {0}")]
    BrowserLaunch(String),

    #[error("CDP通信でエラーが発生しました: {0}")]
    Cdp(#[from] chromiumoxide::error::CdpError),

    #[error("ffmpeg の起動に失敗しました。PATHにffmpegがあるか確認してください: {0}")]
    FfmpegSpawn(#[source] std::io::Error),

    #[error("ffmpeg がエラー終了しました (code={code:?}):\n{stderr}")]
    FfmpegExit { code: Option<i32>, stderr: String },

    #[error("フレームの書き込みに失敗しました: {0}")]
    FrameWrite(#[source] std::io::Error),

    #[error("JSONのシリアライズに失敗しました: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IOエラー: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

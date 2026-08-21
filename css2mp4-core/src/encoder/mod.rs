pub mod command;
pub mod format;
pub mod process;

pub use format::VideoFormat;
pub use process::FfmpegProcess;
// 互換性のための型エイリアス
pub type FfmpegEncoder = FfmpegProcess;

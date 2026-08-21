use std::path::Path;

use crate::error::{Error, Result};

/// 出力する動画コンテナ / コーデック形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4,
    WebmTransparent,
}

impl VideoFormat {
    /// 出力パスの拡張子と `--transparent` フラグから動画形式を判別する。
    pub fn from_output_path(path: &Path, transparent: bool) -> Result<Self> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
        {
            Some(ext) if ext == "mp4" => Ok(VideoFormat::Mp4),
            Some(ext) if ext == "webm" => Ok(VideoFormat::WebmTransparent),
            _ if transparent => Ok(VideoFormat::WebmTransparent),
            _ => Err(Error::UnknownOutputFormat(path.to_path_buf())),
        }
    }

    /// 透過背景を必要とする形式かどうか。
    pub fn is_transparent(&self) -> bool {
        matches!(self, VideoFormat::WebmTransparent)
    }

    /// FFmpeg に渡すコーデック・ピクセルフォーマット等の引数リストを生成する。
    pub fn ffmpeg_args(&self) -> &'static [&'static str] {
        match self {
            VideoFormat::Mp4 => &[
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
            ],
            VideoFormat::WebmTransparent => &[
                "-c:v",
                "libvpx-vp9",
                "-pix_fmt",
                "yuva420p",
                "-auto-alt-ref",
                "0",
            ],
        }
    }
}

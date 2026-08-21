pub mod browser;
pub mod encoder;
pub mod error;
pub mod options;
pub mod pipeline;
pub mod ymmp;

// 後方互換性および既存コードとの互換のためのエイリアス
pub mod capture {
    pub use crate::browser::*;
}
pub mod encode {
    pub use crate::encoder::*;
}
pub mod render {
    pub use crate::pipeline::*;
}

pub use browser::{FrameCapturer, StyleSample};
pub use encoder::{FfmpegEncoder, VideoFormat};
pub use error::{Error, Result};
pub use options::RenderOptions;
pub use pipeline::{
    overwrite_ymmp_motion, preview_frame, render_video, NoopProgress, PreviewOptions, ProgressSink,
};
pub use ymmp::{MotionSamples, YmmpProject};

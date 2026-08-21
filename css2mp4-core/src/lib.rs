pub mod browser;
pub mod encoder;
pub mod error;
pub mod options;
pub mod pipeline;
pub mod ymmp;

// 後方互換性のためのモジュールエイリアス
pub mod capture {
    pub use crate::browser::*;
}
pub mod encode {
    pub use crate::encoder::*;
}
pub mod render {
    pub use crate::pipeline::*;
}

pub use browser::{ComputedSample, FrameCapturer};
pub type StyleSample = ComputedSample;

pub use encoder::{FfmpegEncoder, VideoFormat};
pub use error::{Error, Result};
pub use options::RenderOptions;
pub use pipeline::{
    overwrite_ymmp_motion, overwrite_ymmp_motion_with_tolerance, preview_frame, render_video,
    NoopProgress, PreviewOptions, ProgressSink,
};
pub use ymmp::{MotionSamples, YmmpProject};

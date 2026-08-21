use css2mp4_core::pipeline::ProgressSink;
use indicatif::{ProgressBar, ProgressStyle};

/// `indicatif` を利用した CLI 向け進捗バー。
pub struct CliProgress {
    bar: ProgressBar,
}

impl CliProgress {
    /// 新しいプログレスバーを生成する。
    pub fn new() -> Self {
        let bar = ProgressBar::new(0);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} フレーム ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        Self { bar }
    }
}

impl Default for CliProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressSink for CliProgress {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32) {
        self.bar.set_length(total_frames as u64);
        self.bar.set_position(frame_index as u64);
    }
}

/// レンダリングやサンプリングタスクの進捗を受け取るコールバックトレイト。
///
/// `(現在のフレーム番号, 総フレーム数)` を受け取ります。
pub trait ProgressSink {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32);
}

/// 何もしないデフォルトの進捗シンク。
pub struct NoopProgress;

impl ProgressSink for NoopProgress {
    fn on_frame(&mut self, _frame_index: u32, _total_frames: u32) {}
}

impl<F: FnMut(u32, u32)> ProgressSink for F {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32) {
        self(frame_index, total_frames)
    }
}

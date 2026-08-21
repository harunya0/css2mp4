use std::io::Write;
use std::path::PathBuf;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use css2mp4_core::options::RenderOptions;
use css2mp4_core::pipeline::{render_video, ProgressSink};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::server::state::{AppState, TaskStatus};

#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    /// HTML / CSS のソースコード（インライン文字列）。
    pub html: String,
    /// 出力フォーマット ("mp4" または "webm")。
    #[serde(default = "default_format")]
    pub format: String,
    /// フレームレート（デフォルト: 60）。
    #[serde(default = "default_fps")]
    pub fps: u32,
    /// アニメーション長（秒、デフォルト: 3.0）。
    #[serde(default = "default_duration")]
    pub duration: f64,
    /// 幅（デフォルト: 1920）。
    #[serde(default = "default_width")]
    pub width: u32,
    /// 高さ（デフォルト: 1080）。
    #[serde(default = "default_height")]
    pub height: u32,
    /// 透過背景（デフォルト: false）。
    #[serde(default)]
    pub transparent: bool,
}

fn default_format() -> String {
    "mp4".to_string()
}
fn default_fps() -> u32 {
    60
}
fn default_duration() -> f64 {
    3.0
}
fn default_width() -> u32 {
    1920
}
fn default_height() -> u32 {
    1080
}

#[derive(Serialize)]
pub struct RenderResponse {
    pub task_id: String,
    pub status: TaskStatus,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

struct TaskProgressReporter {
    task_id: String,
    state: AppState,
}

impl ProgressSink for TaskProgressReporter {
    fn on_frame(&mut self, frame_index: u32, total_frames: u32) {
        let task_id = self.task_id.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            state.update_progress(&task_id, frame_index, total_frames).await;
        });
    }
}

pub async fn render_handler(
    State(state): State<AppState>,
    Json(payload): Json<RenderRequest>,
) -> Response {
    let task_id = Uuid::new_v4().to_string();

    // 一時ファイル（入力HTMLと出力動画）を準備
    let temp_input = match NamedTempFile::new() {
        Ok(mut f) => {
            if let Err(e) = f.write_all(payload.html.as_bytes()) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("一時HTMLファイルの書き込みに失敗: {e}"),
                    }),
                )
                    .into_response();
            }
            f
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("一時ファイルの作成に失敗: {e}"),
                }),
            )
                .into_response();
        }
    };

    let ext = if payload.format.to_ascii_lowercase() == "webm" || payload.transparent {
        "webm"
    } else {
        "mp4"
    };

    let output_dir = std::env::temp_dir().join("css2mp4_renders");
    let _ = std::fs::create_dir_all(&output_dir);
    let output_path = output_dir.join(format!("{}.{}", task_id, ext));

    let total_frames = (payload.fps as f64 * payload.duration).ceil() as u32;

    state
        .create_task(task_id.clone(), output_path.clone(), total_frames)
        .await;

    // 非同期バックグラウンドタスクとしてレンダリングを実行
    let opts = RenderOptions {
        input: temp_input.path().to_path_buf(),
        output: output_path,
        fps: payload.fps,
        duration: payload.duration,
        width: payload.width,
        height: payload.height,
        transparent: payload.transparent,
        ffmpeg_path: PathBuf::from("ffmpeg"),
        chrome_path: None,
    };

    let task_id_clone = task_id.clone();
    let state_clone = state.clone();

    tokio::spawn(async move {
        // temp_input をクロージャ内で保持してスコープ終了まで消えないようにする
        let _keep_temp = temp_input;
        let progress = TaskProgressReporter {
            task_id: task_id_clone.clone(),
            state: state_clone.clone(),
        };

        match render_video(&opts, progress).await {
            Ok(()) => {
                state_clone.complete_task(&task_id_clone).await;
            }
            Err(e) => {
                state_clone.fail_task(&task_id_clone, e.to_string()).await;
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(RenderResponse {
            task_id,
            status: TaskStatus::Queued,
        }),
    )
        .into_response()
}

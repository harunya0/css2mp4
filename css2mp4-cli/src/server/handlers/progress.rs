use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::server::state::{AppState, TaskStatus};

#[derive(Serialize)]
pub struct TaskStatusResponse {
    pub id: String,
    pub status: TaskStatus,
    pub current_frame: u32,
    pub total_frames: u32,
    pub percent: f64,
    pub error: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// タスクの現在ステータスを即時取得するエンドポイント。
pub async fn get_task_status_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Response {
    let tasks = state.tasks.read().await;
    if let Some(task) = tasks.get(&task_id) {
        let percent = if task.total_frames > 0 {
            (task.current_frame as f64 / task.total_frames as f64) * 100.0
        } else {
            0.0
        };
        Json(TaskStatusResponse {
            id: task.id.clone(),
            status: task.status.clone(),
            current_frame: task.current_frame,
            total_frames: task.total_frames,
            percent,
            error: task.error.clone(),
        })
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("タスクが見つかりません: {task_id}"),
            }),
        )
            .into_response()
    }
}

/// SSE によるリアルタイム進捗イベント配信エンドポイント。
pub async fn render_progress_sse_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Response {
    let tasks = state.tasks.read().await;
    let task = match tasks.get(&task_id) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("タスクが見つかりません: {task_id}"),
                }),
            )
                .into_response();
        }
    };

    let rx = task.tx.subscribe();
    drop(tasks);

    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(event) => match serde_json::to_string(&event) {
            Ok(json) => Some(Ok::<_, Infallible>(Event::default().data(json))),
            Err(_) => None,
        },
        Err(_) => None,
    });

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

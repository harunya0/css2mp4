use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::server::state::{AppState, TaskStatus};

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn download_handler(
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

    if task.status != TaskStatus::Completed {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("タスクはまだ完了していません (現在の状態: {:?})", task.status),
            }),
        )
            .into_response();
    }

    let output_path = task.output_path.clone();
    drop(tasks);

    let file = match File::open(&output_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("出力ファイルを開けませんでした: {e}"),
                }),
            )
                .into_response();
        }
    };

    let filename = output_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.mp4");

    let content_type = if filename.ends_with(".webm") {
        "video/webm"
    } else {
        "video/mp4"
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(body)
        .unwrap()
}

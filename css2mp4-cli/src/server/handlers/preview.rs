use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use css2mp4_core::pipeline::{preview_frame, PreviewOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    /// HTML / CSS のソースコード（インライン文字列）。
    pub html: String,
    /// キャプチャ対象秒数（デフォルト: 0.0）。
    #[serde(default)]
    pub time_seconds: f64,
    /// 出力幅（デフォルト: 1920）。
    #[serde(default = "default_width")]
    pub width: u32,
    /// 出力高さ（デフォルト: 1080）。
    #[serde(default = "default_height")]
    pub height: u32,
    /// 透過背景（デフォルト: false）。
    #[serde(default)]
    pub transparent: bool,
}

fn default_width() -> u32 {
    1920
}
fn default_height() -> u32 {
    1080
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn preview_handler(Json(payload): Json<PreviewRequest>) -> Response {
    let opts = PreviewOptions {
        input_html: Some(payload.html),
        input_path: None,
        time_seconds: payload.time_seconds,
        width: payload.width,
        height: payload.height,
        transparent: payload.transparent,
        chrome_path: None,
    };

    match preview_frame(&opts).await {
        Ok(png_bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(png_bytes))
            .unwrap(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

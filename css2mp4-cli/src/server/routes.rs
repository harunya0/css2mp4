use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::server::handlers::{
    download_handler, get_task_status_handler, health_handler, preview_handler,
    render_progress_sse_handler, render_handler,
};
use crate::server::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/preview", post(preview_handler))
        .route("/api/render", post(render_handler))
        .route("/api/render/:id", get(get_task_status_handler))
        .route("/api/render/:id/events", get(render_progress_sse_handler))
        .route("/api/render/:id/download", get(download_handler))
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = create_router(AppState::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}


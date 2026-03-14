use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{convert, formats, health, parse};
use crate::types::ApiResult;

pub type ApiResponse<T> = (StatusCode, Json<ApiResult<T>>);

pub fn create_router() -> Router {
    const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    Router::new()
        .route("/api/health", get(health))
        .route("/api/formats", get(formats))
        .route("/api/parse", post(parse))
        .route("/api/convert", post(convert))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

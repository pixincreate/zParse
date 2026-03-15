use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, StatusCode},
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{convert, formats, health, parse};
use crate::types::ApiResult;

pub type ApiResponse<T> = (StatusCode, Json<ApiResult<T>>);

pub fn create_router() -> Router {
    const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    // CORS: Allow specific origin via env var, or default to restrictive (no CORS headers)
    // Set ZPARSE_CORS_ORIGIN=* to allow any origin (dev mode)
    let cors = match std::env::var("ZPARSE_CORS_ORIGIN").ok() {
        Some(origin) if origin == "*" => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
        Some(origin) => {
            let header_value = origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| panic!("Invalid ZPARSE_CORS_ORIGIN: {}", origin));
            CorsLayer::new()
                .allow_origin(header_value)
                .allow_methods(Any)
                .allow_headers(Any)
        }
        None => CorsLayer::new(), // Restrictive: no CORS headers (same-origin only)
    };

    Router::new()
        .route("/api/health", get(health))
        .route("/api/formats", get(formats))
        .route("/api/parse", post(parse))
        .route("/api/convert", post(convert))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(cors)
}

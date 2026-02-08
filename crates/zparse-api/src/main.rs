#![forbid(unsafe_code)]

use axum::{routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
struct ParseRequest {
    content: String,
    format: ApiFormat,
}

#[derive(Debug, Deserialize)]
struct ConvertRequest {
    content: String,
    from: ApiFormat,
    to: ApiFormat,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ApiFormat {
    Json,
    Toml,
    Yaml,
    Xml,
}

impl From<ApiFormat> for zparse::Format {
    fn from(value: ApiFormat) -> Self {
        match value {
            ApiFormat::Json => zparse::Format::Json,
            ApiFormat::Toml => zparse::Format::Toml,
            ApiFormat::Yaml => zparse::Format::Yaml,
            ApiFormat::Xml => zparse::Format::Xml,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum ApiResponse {
    Ok { data: serde_json::Value },
    Err { error: String },
}

#[derive(Debug, Serialize)]
struct ConvertResponse {
    status: &'static str,
    content: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/formats", get(formats))
        .route("/api/parse", post(parse))
        .route("/api/convert", post(convert))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let host = std::env::var("ZPARSE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("ZPARSE_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("failed to bind {addr}: {err}");
            return;
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("server error: {err}");
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn formats() -> Json<Vec<&'static str>> {
    Json(vec!["json", "toml", "yaml", "xml"])
}

async fn parse(Json(payload): Json<ParseRequest>) -> Json<ApiResponse> {
    match parse_to_json(&payload.content, payload.format) {
        Ok(data) => Json(ApiResponse::Ok { data }),
        Err(err) => Json(ApiResponse::Err { error: err }),
    }
}

async fn convert(Json(payload): Json<ConvertRequest>) -> Json<ConvertResponse> {
    let result = zparse::convert(&payload.content, payload.from.into(), payload.to.into());
    match result {
        Ok(content) => Json(ConvertResponse {
            status: "ok",
            content,
        }),
        Err(err) => Json(ConvertResponse {
            status: "error",
            content: err.to_string(),
        }),
    }
}

fn parse_to_json(input: &str, format: ApiFormat) -> Result<serde_json::Value, String> {
    let json = zparse::convert(input, format.into(), zparse::Format::Json)
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&json).map_err(|err| err.to_string())
}

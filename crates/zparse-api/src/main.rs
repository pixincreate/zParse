#![forbid(unsafe_code)]

use axum::{Json, Router, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
struct ParseRequest {
    content: String,
    format: InputFormat,
}

#[derive(Debug, Deserialize)]
struct ConvertRequest {
    content: String,
    from: InputFormat,
    to: OutputFormat,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum InputFormat {
    Json,
    Jsonc,
    Csv,
    Toml,
    Yaml,
    Xml,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    Json,
    Csv,
    Toml,
    Yaml,
    Xml,
}

impl From<InputFormat> for zparse::Format {
    fn from(value: InputFormat) -> Self {
        match value {
            InputFormat::Json => zparse::Format::Json,
            InputFormat::Jsonc => zparse::Format::Json,
            InputFormat::Csv => zparse::Format::Csv,
            InputFormat::Toml => zparse::Format::Toml,
            InputFormat::Yaml => zparse::Format::Yaml,
            InputFormat::Xml => zparse::Format::Xml,
        }
    }
}

impl From<OutputFormat> for zparse::Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Json => zparse::Format::Json,
            OutputFormat::Csv => zparse::Format::Csv,
            OutputFormat::Toml => zparse::Format::Toml,
            OutputFormat::Yaml => zparse::Format::Yaml,
            OutputFormat::Xml => zparse::Format::Xml,
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
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

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
    Json(vec!["json", "jsonc", "csv", "toml", "yaml", "xml"])
}

async fn parse(Json(payload): Json<ParseRequest>) -> Json<ApiResponse> {
    match parse_to_json(&payload.content, payload.format) {
        Ok(data) => Json(ApiResponse::Ok { data }),
        Err(err) => Json(ApiResponse::Err { error: err }),
    }
}

async fn convert(Json(payload): Json<ConvertRequest>) -> Json<ConvertResponse> {
    let result = if matches!(payload.from, InputFormat::Jsonc) {
        let config = zparse::JsonConfig {
            allow_comments: true,
            allow_trailing_commas: true,
            ..zparse::JsonConfig::default()
        };
        zparse::convert_with_options(
            &payload.content,
            payload.from.into(),
            payload.to.into(),
            &zparse::ConvertOptions {
                json: config,
                ..Default::default()
            },
        )
    } else {
        zparse::convert(&payload.content, payload.from.into(), payload.to.into())
    };

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

fn parse_to_json(input: &str, format: InputFormat) -> Result<serde_json::Value, String> {
    let json = if matches!(format, InputFormat::Jsonc) {
        let config = zparse::JsonConfig {
            allow_comments: true,
            allow_trailing_commas: true,
            ..zparse::JsonConfig::default()
        };
        zparse::convert_with_options(
            input,
            format.into(),
            zparse::Format::Json,
            &zparse::ConvertOptions {
                json: config,
                ..Default::default()
            },
        )
    } else {
        zparse::convert(input, format.into(), zparse::Format::Json)
    };

    let json: String = json.map_err(|err| err.to_string())?;
    serde_json::from_str(&json).map_err(|err| err.to_string())
}

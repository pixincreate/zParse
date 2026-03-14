#![forbid(unsafe_code)]

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
struct ParseRequest {
    content: String,
    format: InputFormat,
    csv_delimiter: Option<char>,
}

#[derive(Debug, Deserialize)]
struct ConvertRequest {
    content: String,
    from: InputFormat,
    to: OutputFormat,
    csv_delimiter: Option<char>,
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

/// Unified API response type for both parse and convert endpoints
#[derive(Debug, Serialize)]
struct ApiResult<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResult<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

type ApiResponse<T> = (StatusCode, Json<ApiResult<T>>);

#[tokio::main]
async fn main() {
    // 10 MB request body limit (prevents DoS via huge payloads)
    const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    let app = Router::new()
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

/// Parse content and return as JSON
/// Returns 200 on success, 400 on parse error, 422 on invalid input
async fn parse(Json(payload): Json<ParseRequest>) -> ApiResponse<serde_json::Value> {
    match parse_to_json(&payload.content, payload.format, payload.csv_delimiter) {
        Ok(data) => (StatusCode::OK, Json(ApiResult::ok(data))),
        Err(err) => {
            // Return 400 for validation errors, 422 for parse failures
            let status = if err.starts_with("CSV delimiter") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (status, Json(ApiResult::err(err)))
        }
    }
}

/// Convert between formats
/// Returns 200 on success, 400 on validation error, 422 on conversion error
async fn convert(Json(payload): Json<ConvertRequest>) -> ApiResponse<String> {
    let csv_config = match csv_config_from_delimiter(payload.csv_delimiter) {
        Ok(config) => config,
        Err(err) => {
            return (StatusCode::BAD_REQUEST, Json(ApiResult::err(err)));
        }
    };

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
                csv: csv_config,
            },
        )
    } else if matches!(payload.from, InputFormat::Csv) && payload.csv_delimiter.is_some() {
        zparse::convert_with_options(
            &payload.content,
            payload.from.into(),
            payload.to.into(),
            &zparse::ConvertOptions {
                csv: csv_config,
                ..Default::default()
            },
        )
    } else {
        zparse::convert(&payload.content, payload.from.into(), payload.to.into())
    };

    match result {
        Ok(content) => (StatusCode::OK, Json(ApiResult::ok(content))),
        Err(err) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResult::err(err.to_string())),
        ),
    }
}

fn csv_config_from_delimiter(delimiter: Option<char>) -> Result<zparse::CsvConfig, String> {
    match delimiter {
        None => Ok(zparse::CsvConfig::default()),
        Some(ch) => {
            if !ch.is_ascii() {
                return Err(format!(
                    "CSV delimiter must be ASCII, got '{ch}' (U+{:04X})",
                    ch as u32
                ));
            }
            let b = ch as u8;
            if matches!(b, b'\n' | b'\r' | b'"') {
                return Err(format!(
                    "Invalid CSV delimiter: '{ch}' ({}) is reserved",
                    if b == b'\n' {
                        "newline"
                    } else if b == b'\r' {
                        "carriage return"
                    } else {
                        "quote"
                    }
                ));
            }
            Ok(zparse::CsvConfig::default().with_delimiter(b))
        }
    }
}

fn parse_to_json(
    input: &str,
    format: InputFormat,
    csv_delimiter: Option<char>,
) -> Result<serde_json::Value, String> {
    let csv_config = csv_config_from_delimiter(csv_delimiter)?;
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
                csv: csv_config,
            },
        )
    } else if matches!(format, InputFormat::Csv) && csv_delimiter.is_some() {
        zparse::convert_with_options(
            input,
            format.into(),
            zparse::Format::Json,
            &zparse::ConvertOptions {
                csv: csv_config,
                ..Default::default()
            },
        )
    } else {
        zparse::convert(input, format.into(), zparse::Format::Json)
    };

    let json: String = json.map_err(|err| err.to_string())?;
    serde_json::from_str(&json).map_err(|err| err.to_string())
}

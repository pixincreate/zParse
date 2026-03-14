use axum::{Json, http::StatusCode};
use serde_json::json;

use crate::router::ApiResponse;
use crate::types::{ApiResult, ConvertRequest, InputFormat, ParseRequest};

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

pub async fn formats() -> Json<Vec<&'static str>> {
    Json(vec!["json", "jsonc", "csv", "toml", "yaml", "xml"])
}

pub async fn parse(Json(payload): Json<ParseRequest>) -> ApiResponse<serde_json::Value> {
    match parse_to_json(&payload.content, payload.format, payload.csv_delimiter) {
        Ok(data) => (StatusCode::OK, Json(ApiResult::ok(data))),
        Err(err) => {
            let status = if err.contains("CSV delimiter") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            (status, Json(ApiResult::err(err)))
        }
    }
}

pub async fn convert(Json(payload): Json<ConvertRequest>) -> ApiResponse<String> {
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

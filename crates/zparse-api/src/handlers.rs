use axum::{Json, http::StatusCode};
use serde_json::{Value as JsonValue, json};

use crate::router::ApiResponse;
use crate::types::{ApiResult, ConvertRequest, InputFormat, ParseRequest};

pub async fn health() -> Json<JsonValue> {
    Json(json!({"status": "ok"}))
}

pub async fn formats() -> Json<JsonValue> {
    Json(json!(["json", "jsonc", "csv", "toml", "yaml", "xml"]))
}

pub async fn parse(Json(payload): Json<ParseRequest>) -> ApiResponse<JsonValue> {
    match parse_to_json(&payload.content, payload.format, payload.csv_delimiter) {
        Ok(data) => (StatusCode::OK, Json(ApiResult::ok(data))),
        Err(err) => {
            let status =
                if err.starts_with("CSV delimiter") || err.starts_with("Invalid CSV delimiter") {
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

    // Use native zparse convert for all format conversions
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
) -> Result<JsonValue, String> {
    let csv_config = csv_config_from_delimiter(csv_delimiter)?;

    // Use zparse's native parsers directly based on format
    let value = match format {
        InputFormat::Json => {
            let input_data = zparse::Input::from_str(input);
            let mut parser = zparse::JsonParser::new(input_data.as_bytes());
            parser.parse_value()
        }
        InputFormat::Jsonc => {
            let config = zparse::JsonConfig {
                allow_comments: true,
                allow_trailing_commas: true,
                ..zparse::JsonConfig::default()
            };
            let input_data = zparse::Input::from_str(input);
            let mut parser = zparse::JsonParser::with_config(input_data.as_bytes(), config);
            parser.parse_value()
        }
        InputFormat::Csv => {
            let mut parser = zparse::CsvParser::with_config(input.as_bytes(), csv_config);
            parser.parse()
        }
        InputFormat::Toml => zparse::from_toml_str(input),
        InputFormat::Yaml => zparse::from_yaml_str(input),
        InputFormat::Xml => {
            // XML returns XmlDocument, convert via zparse::convert to JSON string, then parse
            zparse::convert(input, zparse::Format::Xml, zparse::Format::Json)
                .and_then(|json_str| zparse::from_str(&json_str))
        }
    };

    // Convert zparse::Value to serde_json::Value for HTTP response
    value
        .map_err(|err| err.to_string())
        .and_then(zparse_value_to_json_value)
}

// Convert zparse::Value to serde_json::Value for HTTP serialization
fn zparse_value_to_json_value(value: zparse::Value) -> Result<JsonValue, String> {
    match value {
        zparse::Value::Null => Ok(JsonValue::Null),
        zparse::Value::Bool(b) => Ok(JsonValue::Bool(b)),
        zparse::Value::Number(n) => {
            if n.is_finite() {
                serde_json::Number::from_f64(n)
                    .map(JsonValue::Number)
                    .ok_or_else(|| "Invalid number".to_string())
            } else {
                Ok(JsonValue::Null) // Non-finite numbers become null
            }
        }
        zparse::Value::String(s) => Ok(JsonValue::String(s)),
        zparse::Value::Array(arr) => {
            let json_arr: Result<Vec<_>, _> =
                arr.into_iter().map(zparse_value_to_json_value).collect();
            Ok(JsonValue::Array(json_arr?))
        }
        zparse::Value::Object(obj) => {
            let json_obj: Result<serde_json::Map<_, _>, _> = obj
                .into_iter()
                .map(|(k, v)| zparse_value_to_json_value(v).map(|jv| (k, jv)))
                .collect();
            Ok(JsonValue::Object(json_obj?))
        }
        zparse::Value::Datetime(dt) => {
            let s = format!("{:?}", dt);
            Ok(JsonValue::String(s))
        }
    }
}

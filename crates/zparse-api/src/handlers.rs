use axum::{http::StatusCode, response::IntoResponse, Json as AxumJson, response::Response};

use crate::json::{json, json_error, JsonError, JsonResponse};
use crate::types::{ConvertRequest, InputFormat, ParseRequest};

enum ApiResponse {
    Ok(JsonResponse),
    Err(JsonError),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Ok(r) => r.into_response(),
            ApiResponse::Err(e) => e.into_response(),
        }
    }
}

pub async fn health() -> JsonResponse {
    json(r#"{"status":"ok"}"#)
}

pub async fn formats() -> JsonResponse {
    json(r#"["json","jsonc","csv","toml","yaml","xml"]"#)
}

pub async fn parse(AxumJson(payload): AxumJson<ParseRequest>) -> impl IntoResponse {
    match parse_to_json(&payload.content, payload.format, payload.csv_delimiter) {
        Ok(data) => ApiResponse::Ok(json(&data)),
        Err(err) => {
            let status = if err.starts_with("CSV delimiter") || err.starts_with("Invalid CSV delimiter") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::UNPROCESSABLE_ENTITY
            };
            ApiResponse::Err(json_error(&err, status))
        }
    }
}

pub async fn convert(AxumJson(payload): AxumJson<ConvertRequest>) -> impl IntoResponse {
    let csv_config = match csv_config_from_delimiter(payload.csv_delimiter) {
        Ok(config) => config,
        Err(err) => {
            return ApiResponse::Err(json_error(&err, StatusCode::BAD_REQUEST));
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
        Ok(content) => ApiResponse::Ok(json(&content)),
        Err(err) => ApiResponse::Err(json_error(&err.to_string(), StatusCode::UNPROCESSABLE_ENTITY)),
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
) -> Result<String, String> {
    let csv_config = csv_config_from_delimiter(csv_delimiter).map_err(|e| e.to_string())?;

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
            zparse::convert(input, zparse::Format::Xml, zparse::Format::Json)
                .and_then(|json_str| zparse::from_str(&json_str))
        }
    };

    value
        .map_err(|err| err.to_string())
        .map(|v| native_value_to_json(v))
}

fn native_value_to_json(value: zparse::Value) -> String {
    match value {
        zparse::Value::Null => "null".to_string(),
        zparse::Value::Bool(b) => b.to_string(),
        zparse::Value::Number(n) => {
            if n.is_finite() {
                n.to_string()
            } else {
                "null".to_string()
            }
        }
        zparse::Value::String(s) => format!("\"{}\"", escape_json(&s)),
        zparse::Value::Array(arr) => {
            let items: Vec<String> = arr.into_iter().map(native_value_to_json).collect();
            format!("[{}]", items.join(","))
        }
        zparse::Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .into_iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_json(&k), native_value_to_json(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        zparse::Value::Datetime(dt) => {
            format!("\"{:?}\"", dt)
        }
    }
}

fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0C}' => result.push_str("\\f"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result
}

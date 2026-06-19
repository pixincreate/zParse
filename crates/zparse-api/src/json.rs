//! Native JSON response types for Axum (no serde_json dependency)

use axum::{
    body::Body,
    http::{StatusCode, header::HeaderValue},
    response::{IntoResponse, Response},
};

/// A JSON string response - wrapper that sets correct content-type header
#[derive(Debug, Clone)]
pub struct JsonResponse(pub String);

impl IntoResponse for JsonResponse {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", HeaderValue::from_static("application/json"))
            .body(Body::from(self.0))
            .unwrap()
    }
}

/// Create a JSON response from a pre-serialized JSON string
pub fn json(s: &str) -> JsonResponse {
    JsonResponse(s.to_string())
}

/// A JSON response for errors
pub struct JsonError(pub String, pub StatusCode);

impl IntoResponse for JsonError {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self.1)
            .header("content-type", HeaderValue::from_static("application/json"))
            .body(Body::from(self.0))
            .unwrap()
    }
}

/// Create an error JSON response
pub fn json_error(message: &str, status: StatusCode) -> JsonError {
    JsonError(
        format!(
            r#"{{"success":false,"data":null,"error":{}}}"#,
            escape_json_string(message)
        ),
        status,
    )
}

fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

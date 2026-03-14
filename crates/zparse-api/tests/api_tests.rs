use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// Helper to create the app for testing
fn app() -> axum::Router {
    zparse_api::router::create_router()
}

#[tokio::test]
async fn test_health_check() {
    let app = app();

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

#[tokio::test]
async fn test_formats_list() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/formats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_json() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": r#"{"name": "test", "value": 42}"#,
                        "format": "json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_csv_with_delimiter() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "name\tage\nAlice\t30\n",
                        "format": "csv",
                        "csv_delimiter": "\t"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_invalid_csv_delimiter() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "test",
                        "format": "csv",
                        "csv_delimiter": "\n"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_convert_json_to_toml() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": r#"{"name": "test"}"#,
                        "from": "json",
                        "to": "toml"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_convert_csv_to_json() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "name,age\nAlice,30\n",
                        "from": "csv",
                        "to": "json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_convert_invalid_delimiter() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "test",
                        "from": "csv",
                        "to": "json",
                        "csv_delimiter": "é"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_convert_invalid_json() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "not valid json",
                        "from": "json",
                        "to": "toml"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_parse_yaml() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "name: test\nvalue: 42",
                        "format": "yaml"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_xml() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "<root><name>test</name></root>",
                        "format": "xml"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_parse_toml() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/parse")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "content": "name = \"test\"\nvalue = 42",
                        "format": "toml"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

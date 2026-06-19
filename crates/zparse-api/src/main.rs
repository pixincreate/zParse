#![forbid(unsafe_code)]

use zparse_api::router::create_router;

#[tokio::main]
async fn main() {
    let app = create_router();

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

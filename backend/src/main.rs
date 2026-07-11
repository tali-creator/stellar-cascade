mod routes;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

/// Builds and returns the application router.
///
/// Extracted from `main` so tests can call `router()` directly without
/// binding a real port.
pub fn router() -> Router {
    Router::new().route("/health", get(routes::health::handler))
}

#[tokio::main]
async fn main() {
    // Initialise tracing — respects RUST_LOG env var, defaults to "info".
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Port is configurable via PORT env var; default 3000.
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    info!("cascade backend listening on {addr}");

    axum::serve(listener, router()).await.expect("server error");
}

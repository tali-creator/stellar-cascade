mod config;
mod routes;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

/// Builds and returns the application router.
///
/// Extracted from `main` so tests can call `router()` directly without
/// binding a real port.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
        // Log every request: method, path, status, latency.
        // Respects the active tracing subscriber — controlled via RUST_LOG.
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    // Load config first — dotenvy picks up .env, then real env vars win.
    let cfg = config::Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {e}");
        std::process::exit(1);
    });

    // Initialise tracing using the RUST_LOG value from config.
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.rust_log)),
        )
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    info!("cascade backend listening on {addr}");

    axum::serve(listener, router()).await.expect("server error");
}

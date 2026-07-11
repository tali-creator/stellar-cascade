mod config;
mod db;
mod routes;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

/// Builds and returns the application router with a live Postgres pool
/// attached as shared state.
///
/// Route handlers access the pool via `axum::extract::State<PgPool>`.
/// The pool is created once at startup and shared for the process lifetime.
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

/// Builds a router without Postgres state, for use in unit tests that do not
/// exercise any DB code (e.g. /health).  Avoids requiring a real database
/// connection in the test suite.
#[cfg(test)]
pub fn router_for_test() -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
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

    // Connect to Postgres — fail fast if unreachable.
    let pool = db::build_pool(&cfg.database_url).await.unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    info!("connected to Postgres (pool max_size=5)");

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    info!("cascade backend listening on {addr}");

    axum::serve(listener, router(pool))
        .await
        .expect("server error");
}

mod config;
mod db;
mod event_decode;
mod routes;
mod soroban_rpc;
mod sync_poll;
mod sync_state;
mod sync_worker;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

use sync_state::SyncState;

/// Builds and returns the application router with a live Postgres pool and
/// shared sync state attached.
///
/// `pool` is not yet used directly by any route — future API routes will
/// extract it from state. It is accepted here so the signature is stable
/// and callers don't need updating when those routes are added.
#[allow(unused_variables)]
pub fn router(pool: PgPool, sync_state: Arc<SyncState>) -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
        .layer(TraceLayer::new_for_http())
        // Axum extracts tuple state via individual State<T> extractors.
        .with_state(sync_state)
}

/// Builds a router without Postgres state, for use in unit tests.
/// Accepts a `SyncState` so health-handler tests can exercise sync status.
#[cfg(test)]
pub fn router_for_test(sync_state: Arc<SyncState>) -> Router {
    Router::new()
        .route("/health", get(routes::health::handler))
        .layer(TraceLayer::new_for_http())
        .with_state(sync_state)
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
    info!(
        soroban_rpc_url = %cfg.soroban_rpc_url,
        registry_contract_id = %cfg.registry_contract_id,
        poll_interval_secs = cfg.sync_poll_interval_secs,
        "Soroban RPC configured"
    );

    // Shared sync state — written by the polling loop, read by /health.
    let sync_state = SyncState::new_shared();

    // Spawn the background sync worker — runs independently of the HTTP server.
    sync_poll::spawn(
        pool.clone(),
        cfg.soroban_rpc_url.clone(),
        cfg.registry_contract_id.clone(),
        Duration::from_secs(cfg.sync_poll_interval_secs),
        Arc::clone(&sync_state),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    info!("cascade backend listening on {addr}");

    axum::serve(listener, router(pool, sync_state))
        .await
        .expect("server error");
}

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Maximum number of connections in the Postgres pool.
///
/// 5 is intentionally conservative for local dev and CI.  Raise via an env
/// var in a later issue once load characteristics are known.
const MAX_CONNECTIONS: u32 = 5;

/// Build a [`PgPool`] from the given connection URL.
///
/// Fails fast with a descriptive error if the database is unreachable — the
/// caller should treat this as fatal at startup rather than catching and
/// retrying silently.
///
/// The pool is created with [`PgPoolOptions::connect`] (not `connect_lazy`),
/// which establishes at least one real connection immediately.  This surfaces
/// misconfigured credentials and unreachable hosts at boot time, not on the
/// first request under load.
pub async fn build_pool(database_url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect(database_url)
        .await
        .map_err(|e| {
            format!(
                "Failed to connect to Postgres at the configured DATABASE_URL.\n\
                 Check that the server is running and the URL is correct.\n\
                 Error: {e}"
            )
        })
}

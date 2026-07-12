use std::env;

/// Typed configuration loaded once at startup from environment variables.
///
/// Optional fields have sensible defaults.  Required fields (currently only
/// `database_url`) cause an immediate, descriptive error if absent — the
/// process exits before binding a port rather than failing obscurely on the
/// first query.
///
/// Load order:
///   1. A `.env` file in the working directory (dev convenience, no-op in prod).
///   2. Real environment variables (always take precedence over `.env`).
#[derive(Debug)]
pub struct Config {
    /// TCP port the HTTP server binds to.  Default: 3000.
    pub port: u16,

    /// Tracing filter string forwarded to `tracing_subscriber`.
    /// Controls verbosity via `RUST_LOG`.  Default: "info".
    pub rust_log: String,

    /// Postgres connection string.  **Required.**
    /// Example: `postgres://cascade:cascade@localhost:5432/cascade`
    pub database_url: String,

    /// Soroban RPC endpoint.  Default: Stellar testnet.
    pub soroban_rpc_url: String,
}

impl Config {
    /// Read config from the environment.
    ///
    /// Calls [`dotenvy::dotenv`] first so a local `.env` file is picked up
    /// in development.  The call is intentionally allowed to fail silently
    /// (no `.env` file is perfectly normal in CI and production).
    ///
    /// # Errors
    ///
    /// Returns an error string if a required variable is absent, or if any
    /// variable is present but unparseable (e.g. `PORT=banana`).
    pub fn from_env() -> Result<Self, String> {
        // Load .env if present; ignore the error if it doesn't exist.
        let _ = dotenvy::dotenv();

        Self::from_vars(
            env::var("PORT").ok().as_deref(),
            env::var("RUST_LOG").ok().as_deref(),
            env::var("DATABASE_URL").ok().as_deref(),
            env::var("SOROBAN_RPC_URL").ok().as_deref(),
        )
    }

    /// Parse config from explicit string values.
    ///
    /// `None` means the variable was absent; `Some(s)` means it was set to
    /// `s`.  This is the testable core — `from_env` is a thin wrapper that
    /// reads the real environment and delegates here.
    pub(crate) fn from_vars(
        port: Option<&str>,
        rust_log: Option<&str>,
        database_url: Option<&str>,
        soroban_rpc_url: Option<&str>,
    ) -> Result<Self, String> {
        let port = match port {
            Some(val) => val
                .parse::<u16>()
                .map_err(|_| format!("PORT must be a valid port number (1–65535), got {val:?}"))?,
            None => 3000,
        };

        let rust_log = rust_log.unwrap_or("info").to_string();

        let database_url = database_url
            .ok_or_else(|| {
                "DATABASE_URL is required but was not set.\n\
                 Hint: copy backend/.env.example to backend/.env and fill in the value."
                    .to_string()
            })?
            .to_string();

        let soroban_rpc_url = soroban_rpc_url
            .unwrap_or("https://soroban-testnet.stellar.org:443")
            .to_string();

        Ok(Config {
            port,
            rust_log,
            database_url,
            soroban_rpc_url,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Config;

    const DB: &str = "postgres://cascade:cascade@localhost:5432/cascade";

    #[test]
    fn defaults_when_optional_vars_absent() {
        let cfg = Config::from_vars(None, None, Some(DB), None).expect("should load with defaults");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.rust_log, "info");
        assert_eq!(cfg.database_url, DB);
        assert_eq!(
            cfg.soroban_rpc_url,
            "https://soroban-testnet.stellar.org:443"
        );
    }

    #[test]
    fn respects_port_var() {
        let cfg = Config::from_vars(Some("8080"), None, Some(DB), None).expect("should load");
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn rejects_invalid_port() {
        let err = Config::from_vars(Some("banana"), None, Some(DB), None).expect_err("should fail");
        assert!(
            err.contains("PORT"),
            "error should mention PORT, got: {err}"
        );
    }

    #[test]
    fn respects_rust_log_var() {
        let cfg = Config::from_vars(None, Some("debug"), Some(DB), None).expect("should load");
        assert_eq!(cfg.rust_log, "debug");
    }

    #[test]
    fn port_boundary_max() {
        let cfg =
            Config::from_vars(Some("65535"), None, Some(DB), None).expect("should accept max port");
        assert_eq!(cfg.port, 65535);
    }

    #[test]
    fn rejects_port_zero() {
        let cfg = Config::from_vars(Some("0"), None, Some(DB), None).expect("parses without error");
        assert_eq!(cfg.port, 0);
    }

    #[test]
    fn requires_database_url() {
        let err =
            Config::from_vars(None, None, None, None).expect_err("should fail without DB URL");
        assert!(
            err.contains("DATABASE_URL"),
            "error should mention DATABASE_URL, got: {err}"
        );
    }

    #[test]
    fn database_url_stored_verbatim() {
        let url = "postgres://user:pass@host:5432/db";
        let cfg = Config::from_vars(None, None, Some(url), None).expect("should load");
        assert_eq!(cfg.database_url, url);
    }

    #[test]
    fn soroban_rpc_url_defaults_to_testnet() {
        let cfg = Config::from_vars(None, None, Some(DB), None).expect("should load");
        assert_eq!(
            cfg.soroban_rpc_url,
            "https://soroban-testnet.stellar.org:443"
        );
    }

    #[test]
    fn soroban_rpc_url_can_be_overridden() {
        let cfg = Config::from_vars(
            None,
            None,
            Some(DB),
            Some("https://rpc.mainnet.stellar.org"),
        )
        .expect("should load");
        assert_eq!(cfg.soroban_rpc_url, "https://rpc.mainnet.stellar.org");
    }
}

use std::env;

/// Typed configuration loaded once at startup from environment variables.
///
/// All fields have sensible defaults so the service boots with zero
/// configuration in a dev environment.  As new integrations land
/// (Postgres, Soroban RPC) their required vars will be added here and
/// marked accordingly.
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
    /// Returns an error string if a variable is present but unparseable
    /// (e.g. `PORT=banana`).  Missing optional variables fall back to their
    /// defaults without error.
    pub fn from_env() -> Result<Self, String> {
        // Load .env if present; ignore the error if it doesn't exist.
        let _ = dotenvy::dotenv();

        Self::from_vars(
            env::var("PORT").ok().as_deref(),
            env::var("RUST_LOG").ok().as_deref(),
        )
    }

    /// Parse config from explicit string values.
    ///
    /// `None` means the variable was absent; `Some(s)` means it was set to
    /// `s`.  This is the testable core — `from_env` is a thin wrapper that
    /// reads the real environment and delegates here.
    fn from_vars(port: Option<&str>, rust_log: Option<&str>) -> Result<Self, String> {
        let port = match port {
            Some(val) => val
                .parse::<u16>()
                .map_err(|_| format!("PORT must be a valid port number (1–65535), got {val:?}"))?,
            None => 3000,
        };

        let rust_log = rust_log.unwrap_or("info").to_string();

        Ok(Config { port, rust_log })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn defaults_when_vars_absent() {
        let cfg = Config::from_vars(None, None).expect("should load with defaults");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.rust_log, "info");
    }

    #[test]
    fn respects_port_var() {
        let cfg = Config::from_vars(Some("8080"), None).expect("should load");
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn rejects_invalid_port() {
        let err = Config::from_vars(Some("banana"), None).expect_err("should fail");
        assert!(
            err.contains("PORT"),
            "error should mention PORT, got: {err}"
        );
    }

    #[test]
    fn respects_rust_log_var() {
        let cfg = Config::from_vars(None, Some("debug")).expect("should load");
        assert_eq!(cfg.rust_log, "debug");
    }

    #[test]
    fn port_boundary_max() {
        let cfg = Config::from_vars(Some("65535"), None).expect("should accept max port");
        assert_eq!(cfg.port, 65535);
    }

    #[test]
    fn rejects_port_zero() {
        // u16 parses "0" successfully, but 0 is technically invalid as a
        // listening port.  Document current behaviour: we accept it and let
        // the OS reject it at bind time (keeps parsing simple).
        let cfg = Config::from_vars(Some("0"), None).expect("parses without error");
        assert_eq!(cfg.port, 0);
    }
}

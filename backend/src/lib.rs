//! Library target exposing internal modules for integration tests.
//!
//! The binary (`main.rs`) is the production entry point.  This file exists
//! solely so that `tests/` integration tests can import crate internals via
//! `use backend::...` without making everything `pub` in the binary.
//!
//! All modules required to satisfy internal `use crate::` references must be
//! declared here, even if not directly used by tests.  Modules that tests
//! import directly are `pub`; internal-only dependencies are private.

pub mod db;
pub mod event_decode;
pub mod sync_worker;

// Required by sync_worker (use crate::soroban_rpc::RpcError).
// Not exported to tests — they don't call the RPC directly.
// get_events is only called from sync_poll, which is excluded from the lib
// target; suppress the dead_code lint rather than pulling in the full poll loop.
#[allow(dead_code)]
mod soroban_rpc;

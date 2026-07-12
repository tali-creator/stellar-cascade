//! Thin wrapper around [`stellar_rpc_client::Client`] for fetching contract
//! events from Soroban RPC.
//!
//! The rest of the codebase should import this module rather than calling
//! `stellar_rpc_client` directly, so the RPC boundary is in one place and
//! easy to mock or swap later.

#![allow(dead_code)] // used by the sync worker in a later issue

use stellar_rpc_client::{Client, Event, EventStart, EventType};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("failed to create RPC client: {0}")]
    ClientInit(#[from] stellar_rpc_client::Error),

    #[error("getEvents RPC call failed: {0}")]
    GetEvents(stellar_rpc_client::Error),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Re-export so callers don't need to depend on `stellar-rpc-client` directly.
pub type RawEvent = Event;

/// Fetch contract events from Soroban RPC starting at `start_ledger`,
/// filtered to `contract_id`.
///
/// Returns up to `limit` events (default 100 if `None`).  The caller is
/// responsible for paginating by advancing `start_ledger` to
/// `last_event.ledger + 1` after each batch.
///
/// # Contract
///
/// Only events emitted by `contract_id` are returned.  The topic filter is
/// intentionally left empty (match all topics) so every event type
/// (`register`, `update`) is included in a single call.
pub async fn get_events(
    rpc_url: &str,
    contract_id: &str,
    start_ledger: u32,
    limit: Option<usize>,
) -> Result<Vec<RawEvent>, RpcError> {
    let client = Client::new(rpc_url)?;

    let response = client
        .get_events(
            EventStart::Ledger(start_ledger),
            Some(EventType::Contract),
            &[contract_id.to_string()],
            &[], // no topic filter — fetch all event types
            Some(limit.unwrap_or(100)),
        )
        .await
        .map_err(RpcError::GetEvents)?;

    Ok(response.events)
}

// ---------------------------------------------------------------------------
// Integration test (network-dependent, ignored in normal CI)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Calls live testnet and confirms the registered contract events are
    /// visible.
    ///
    /// Run manually with:
    ///   cargo test -p backend soroban_rpc::tests::live_testnet -- --ignored
    ///
    /// Skipped in normal CI because it requires outbound network access.
    #[tokio::test]
    #[ignore = "requires network access to Stellar testnet"]
    async fn live_testnet_get_events_returns_register_event() {
        const RPC_URL: &str = "https://soroban-testnet.stellar.org:443";
        // The registry contract deployed in B1 / DEPLOYMENTS.md.
        const CONTRACT_ID: &str = "CC4BP273CO73T6AXOMILHWXF37EJ5B766JEOKCYTET4YBHE3FX46GYNI";
        // Start well before the deployment ledger to guarantee we catch the
        // register event.  Testnet ledger at time of deployment: ~2026-07-11.
        // Use ledger 1 to scan from genesis (small testnet, fast).
        const START_LEDGER: u32 = 1;

        let events = get_events(RPC_URL, CONTRACT_ID, START_LEDGER, Some(50))
            .await
            .expect("getEvents should succeed against live testnet");

        assert!(
            !events.is_empty(),
            "expected at least one event from the deployed registry contract"
        );

        // Every event must be from our contract.
        for event in &events {
            assert_eq!(
                event.contract_id, CONTRACT_ID,
                "unexpected contract_id in event: {event:?}"
            );
        }

        // At least one event should have "register" as its first topic value.
        let has_register = events.iter().any(|e| {
            e.topic
                .first()
                .map(|t| t.contains("cmVnaXN0ZXI")) // base64 of ScSymbol("register")
                .unwrap_or(false)
        });

        // Print all events for manual inspection when running with --nocapture.
        for e in &events {
            println!("event: id={} topic={:?} value={}", e.id, e.topic, e.value);
        }

        assert!(
            has_register,
            "expected a 'register' event from the deployed contract; got: {events:#?}"
        );
    }
}

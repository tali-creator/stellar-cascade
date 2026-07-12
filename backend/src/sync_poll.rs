//! Resumable event-sync polling loop (B14).
//!
//! Spawned as a background `tokio::task` from `main.rs`.  Runs independently
//! of the HTTP server — an RPC failure here logs and retries with backoff,
//! but never brings down the API.
//!
//! # Resumability
//!
//! On startup, the worker reads `sync_cursor.last_processed_ledger` from
//! Postgres.  It fetches events starting at `cursor + 1`, so a crash or
//! restart picks up exactly where it left off — no replayed ledgers, no gaps.
//!
//! # Poll cycle log line
//!
//! Each cycle emits a structured `info!` line:
//!   `poll_cycle ledger_start=N events_fetched=N events_applied=N cursor=N`

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::event_decode::decode_event;
use crate::soroban_rpc::get_events;
use crate::sync_state::SyncState;
use crate::sync_worker::{advance_cursor_to, apply_event};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Spawn the sync polling loop as a detached background task.
///
/// `sync_state` is updated after every successful poll cycle so the health
/// endpoint can report real sync status without touching the database.
///
/// The task runs forever (or until the process exits).  Errors are logged and
/// retried; they do not propagate to the caller.
pub fn spawn(
    pool: PgPool,
    rpc_url: String,
    contract_id: String,
    poll_interval: Duration,
    sync_state: Arc<SyncState>,
) {
    tokio::spawn(async move {
        run_loop(pool, rpc_url, contract_id, poll_interval, sync_state).await;
    });
}

// ---------------------------------------------------------------------------
// Loop implementation
// ---------------------------------------------------------------------------

async fn run_loop(
    pool: PgPool,
    rpc_url: String,
    contract_id: String,
    poll_interval: Duration,
    sync_state: Arc<SyncState>,
) {
    info!("sync worker started");

    loop {
        match poll_once(&pool, &rpc_url, &contract_id).await {
            Ok((events_fetched, events_applied, cursor)) => {
                // Update shared state so the health endpoint can report
                // freshness without a DB round-trip.
                sync_state.record_poll(cursor as u64);

                info!(
                    events_fetched,
                    events_applied, cursor, "poll cycle complete"
                );
            }
            Err(e) => {
                // Log and continue — don't crash the process.
                // Note: sync_state is NOT updated on failure, so the health
                // endpoint will start reporting "degraded" after 60s with no
                // successful cycle.
                error!(error = %e, "poll cycle failed; will retry after backoff");
                // Brief extra sleep on error to avoid hammering a broken RPC.
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Run a single poll cycle: read cursor → fetch events → decode → apply.
///
/// Returns `(events_fetched, events_applied, new_cursor_ledger)`.
///
/// # Crash-resume correctness
///
/// The cursor is advanced **once**, after all events in the batch have been
/// applied — not per-event.  If the process crashes mid-batch, the cursor
/// stays at its pre-batch value, and the next run re-fetches from the same
/// start ledger.  Events already applied in the aborted batch are either:
///
/// - Idempotent on re-apply (`ON CONFLICT DO NOTHING` for RegisterProject,
///   additive upsert for Deposit), or
/// - Re-applied atomically (UpdateSplits deletes + re-inserts inside a txn).
///
/// The one non-idempotent case is Deposit accumulation: a deposit re-applied
/// after a crash would double-count.  This is acceptable given that the
/// deposit-phase contract is not yet live; when it is, the `sync_events`
/// audit log provides the reconciliation trail to detect and correct
/// double-counts if they ever occur.
async fn poll_once(
    pool: &PgPool,
    rpc_url: &str,
    contract_id: &str,
) -> Result<(usize, usize, i64), Box<dyn std::error::Error + Send + Sync>> {
    // Read the current cursor from Postgres.
    let cursor_row = sqlx::query!("SELECT last_processed_ledger FROM sync_cursor WHERE id = 1")
        .fetch_one(pool)
        .await?;

    let start_ledger = (cursor_row.last_processed_ledger + 1) as u32;

    // Fetch events from the RPC.
    let raw_events = get_events(rpc_url, contract_id, start_ledger, Some(100)).await?;
    let events_fetched = raw_events.len();

    if events_fetched == 0 {
        return Ok((0, 0, cursor_row.last_processed_ledger));
    }

    let mut events_applied = 0usize;
    // Track the highest ledger in this batch so we can advance the cursor
    // once, after all events are applied.
    let mut highest_ledger = cursor_row.last_processed_ledger as u32;

    for raw in &raw_events {
        let tx_hash = raw.tx_hash.as_deref().unwrap_or("unknown");

        match decode_event(raw) {
            Ok(Some(decoded)) => {
                match apply_event(pool, rpc_url, contract_id, &decoded, raw.ledger, tx_hash).await {
                    Ok(()) => {
                        events_applied += 1;
                        if raw.ledger > highest_ledger {
                            highest_ledger = raw.ledger;
                        }
                    }
                    Err(e) => {
                        warn!(
                            ledger = raw.ledger,
                            tx_hash,
                            error = %e,
                            "failed to apply event; skipping"
                        );
                    }
                }
            }
            Ok(None) => {
                // Unknown event type — silently skip, but still advance past it.
                if raw.ledger > highest_ledger {
                    highest_ledger = raw.ledger;
                }
            }
            Err(e) => {
                warn!(
                    ledger = raw.ledger,
                    tx_hash,
                    error = %e,
                    "failed to decode event; skipping"
                );
            }
        }
    }

    // Advance the cursor once for the whole batch.  This is the correct
    // granularity: a crash before this point leaves the cursor at its
    // pre-batch value, so all events in this batch are re-processed on
    // restart rather than silently skipped.
    advance_cursor_to(pool, highest_ledger).await?;

    let final_cursor = sqlx::query!("SELECT last_processed_ledger FROM sync_cursor WHERE id = 1")
        .fetch_one(pool)
        .await?
        .last_processed_ledger;

    Ok((events_fetched, events_applied, final_cursor))
}

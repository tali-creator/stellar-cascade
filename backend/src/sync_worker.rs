//! Sync worker: applies decoded contract events to the Postgres database.
//!
//! # Design notes
//!
//! ## Why contract storage reads are required
//!
//! The registry contract's events deliberately emit minimal data:
//!   - `register` → (owner_address, receiver_count)
//!   - `update`   → (receiver_count)
//!
//! Neither event includes the actual receiver list (addresses + percentages),
//! so the sync worker must read the full `Project` struct from contract
//! persistent storage via `get_ledger_entries` after each event to populate
//! the `splits` table correctly.
//!
//! ## Transaction discipline
//!
//! Every `apply_event` call wraps ALL of the following in a single Postgres
//! transaction:
//!   1. State write (projects / splits INSERT/UPDATE)
//!   2. Append to `sync_events` (audit log)
//!   3. Advance `sync_cursor.last_processed_ledger`
//!
//! A crash mid-apply leaves the transaction uncommitted, so on restart the
//! worker re-processes that event cleanly.

use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use tracing::{debug, warn};

use crate::event_decode::DecodedEvent;
use crate::soroban_rpc::RpcError;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("RPC error fetching project from chain: {0}")]
    Rpc(#[from] RpcError),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Apply a single decoded event to the database.
///
/// Wraps all writes (state + audit log + cursor advance) in a single
/// transaction so the DB is never left in a partially-applied state.
pub async fn apply_event(
    pool: &PgPool,
    rpc_url: &str,
    contract_id: &str,
    event: &DecodedEvent,
    ledger: u32,
    tx_hash: &str,
) -> Result<(), SyncError> {
    let mut txn = pool.begin().await?;

    match event {
        DecodedEvent::RegisterProject {
            project_id,
            owner_address,
            ..
        } => {
            apply_register(&mut txn, rpc_url, contract_id, project_id, owner_address, ledger)
                .await?;

            append_sync_event(
                &mut txn,
                ledger,
                tx_hash,
                "register_project",
                &json!({
                    "project_id": project_id,
                    "owner": owner_address,
                }),
            )
            .await?;
        }

        DecodedEvent::UpdateSplits { project_id, .. } => {
            apply_update(&mut txn, rpc_url, contract_id, project_id, ledger).await?;

            append_sync_event(
                &mut txn,
                ledger,
                tx_hash,
                "update_splits",
                &json!({ "project_id": project_id }),
            )
            .await?;
        }

        DecodedEvent::Deposit {
            project_id,
            token_address,
            amount,
        } => {
            apply_deposit(&mut txn, project_id, token_address, *amount, ledger).await?;

            append_sync_event(
                &mut txn,
                ledger,
                tx_hash,
                "deposit",
                &json!({
                    "project_id": project_id,
                    "token_address": token_address,
                    // amount as string to avoid JSON number precision loss for large i128 values
                    "amount": amount.to_string(),
                }),
            )
            .await?;
        }
    }

    advance_cursor(&mut txn, ledger).await?;
    txn.commit().await?;

    debug!(ledger, tx_hash, "applied event");
    Ok(())
}

// ---------------------------------------------------------------------------
// Per-event apply helpers
// ---------------------------------------------------------------------------

async fn apply_register(
    txn: &mut Transaction<'_, Postgres>,
    rpc_url: &str,
    contract_id: &str,
    project_id: &str,
    owner_address: &str,
    ledger: u32,
) -> Result<(), SyncError> {
    // INSERT the project row. ON CONFLICT DO NOTHING matches the contract's
    // own behaviour: register_project returns ProjectAlreadyExists if the
    // project_id is already registered, so a duplicate event should be a no-op
    // on our side too.
    sqlx::query!(
        r#"
        INSERT INTO projects (id, owner_address, last_synced_ledger, created_at, updated_at)
        VALUES ($1, $2, $3, now(), now())
        ON CONFLICT (id) DO NOTHING
        "#,
        project_id,
        owner_address,
        ledger as i64,
    )
    .execute(&mut **txn)
    .await?;

    // Fetch the full receiver list from contract storage and upsert splits.
    upsert_splits(txn, rpc_url, contract_id, project_id, ledger).await?;

    Ok(())
}

async fn apply_update(
    txn: &mut Transaction<'_, Postgres>,
    rpc_url: &str,
    contract_id: &str,
    project_id: &str,
    ledger: u32,
) -> Result<(), SyncError> {
    // Update the project's staleness timestamp.
    sqlx::query!(
        r#"
        UPDATE projects
        SET last_synced_ledger = $2, updated_at = now()
        WHERE id = $1
        "#,
        project_id,
        ledger as i64,
    )
    .execute(&mut **txn)
    .await?;

    // Replace splits by fetching the current receiver list from the chain.
    upsert_splits(txn, rpc_url, contract_id, project_id, ledger).await?;

    Ok(())
}

/// Apply a `Deposit` event: additively accumulate the deposited amount into
/// the `balances` table.
///
/// This is an **additive upsert**, not an overwrite — subsequent deposits for
/// the same (project_id, token_address) pair increment the stored balance.
/// The `balances.amount` column is `NUMERIC(39,0)`, which safely covers the
/// full i128 range without precision loss.
async fn apply_deposit(
    txn: &mut Transaction<'_, Postgres>,
    project_id: &str,
    token_address: &str,
    amount: i128,
    ledger: u32,
) -> Result<(), SyncError> {
    // Convert i128 to a Decimal for sqlx / NUMERIC binding.
    // We use a string intermediate because sqlx's NUMERIC support expects
    // rust_decimal::Decimal, which we bind via its Display impl.
    let amount_str = amount.to_string();
    sqlx::query!(
        r#"
        INSERT INTO balances (project_id, token_address, amount, updated_at)
        VALUES ($1, $2, $3::NUMERIC, now())
        ON CONFLICT (project_id, token_address)
        DO UPDATE SET
            amount     = balances.amount + EXCLUDED.amount,
            updated_at = now()
        "#,
        project_id,
        token_address,
        amount_str as _,
    )
    .execute(&mut **txn)
    .await?;

    // Also update the project's staleness timestamp to reflect that something
    // changed in this ledger.
    sqlx::query!(
        r#"
        UPDATE projects
        SET last_synced_ledger = GREATEST(last_synced_ledger, $2), updated_at = now()
        WHERE id = $1
        "#,
        project_id,
        ledger as i64,
    )
    .execute(&mut **txn)
    .await?;

    Ok(())
}

/// Fetch the current `Project` from contract storage and replace the `splits`
/// rows for that project atomically (DELETE + INSERT within the caller's txn).
async fn upsert_splits(
    txn: &mut Transaction<'_, Postgres>,
    rpc_url: &str,
    contract_id: &str,
    project_id: &str,
    ledger: u32,
) -> Result<(), SyncError> {
    // Fetch current receiver list from chain.
    let project = crate::soroban_rpc::fetch_project(rpc_url, contract_id, project_id).await?;

    let Some(on_chain) = project else {
        // Project not found on chain (e.g. expired entry) — log and skip splits.
        warn!(
            project_id,
            ledger, "project not found in contract storage; skipping splits upsert"
        );
        return Ok(());
    };

    // Delete existing splits for this project (all-or-nothing replacement).
    sqlx::query!(
        "DELETE FROM splits WHERE project_id = $1",
        project_id
    )
    .execute(&mut **txn)
    .await?;

    // Re-insert the fresh receiver list.
    for (position, receiver) in on_chain.receivers.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO splits (project_id, receiver_address, percentage_bps, position)
            VALUES ($1, $2, $3, $4)
            "#,
            project_id,
            receiver.address,
            receiver.percentage_bps as i32,
            position as i16,
        )
        .execute(&mut **txn)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Audit log + cursor
// ---------------------------------------------------------------------------

async fn append_sync_event(
    txn: &mut Transaction<'_, Postgres>,
    ledger: u32,
    tx_hash: &str,
    event_type: &str,
    raw_data: &serde_json::Value,
) -> Result<(), SyncError> {
    sqlx::query!(
        r#"
        INSERT INTO sync_events (ledger_sequence, tx_hash, event_type, raw_data, processed_at)
        VALUES ($1, $2, $3, $4, now())
        "#,
        ledger as i64,
        tx_hash,
        event_type,
        raw_data,
    )
    .execute(&mut **txn)
    .await?;
    Ok(())
}

async fn advance_cursor(
    txn: &mut Transaction<'_, Postgres>,
    ledger: u32,
) -> Result<(), SyncError> {
    // Only advance if the new ledger is strictly greater than the stored one,
    // so a batch processed out of order never moves the cursor backwards.
    sqlx::query!(
        r#"
        UPDATE sync_cursor
        SET last_processed_ledger = GREATEST(last_processed_ledger, $1),
            updated_at = now()
        WHERE id = 1
        "#,
        ledger as i64,
    )
    .execute(&mut **txn)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for apply_event require a live DB + RPC and are covered by
    // the integration test suite run against a real testnet event.
    // The pure-logic paths (advance_cursor idempotency, append_sync_event
    // schema) are verified via the migration sanity checks in B7-B10.
    //
    // Full integration tests with a live pool are added in a later issue
    // once the polling loop (B14) is wired up and there are real events
    // to process.
    #[test]
    fn sync_error_display() {
        let e = SyncError::Db(sqlx::Error::RowNotFound);
        assert!(e.to_string().contains("database error"));
    }
}

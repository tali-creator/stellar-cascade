-- Migration: add event_id to sync_events for idempotent event application
--
-- Context (B16):
--   The B14 crash-resume fix made cursor advancement batch-level: if the
--   process dies mid-batch, the cursor stays at its pre-batch value and the
--   whole batch is re-fetched on restart.  For most event types this is safe
--   (RegisterProject is ON CONFLICT DO NOTHING; UpdateSplits replaces state
--   atomically).  But Deposit is an additive accumulation — re-applying the
--   same deposit event would double-count real value.
--
--   This migration adds the column and constraint that `apply_event` uses as
--   its idempotency gate: before touching any state, it attempts to INSERT a
--   row keyed by event_id.  If the INSERT produces 0 rows (conflict), the
--   event was already processed and the state write is skipped entirely.
--
-- Source of event_id:
--   Soroban RPC returns a stable `id` field on every Event object.  Its
--   format is `{ledger_sequence}-{tx_index_within_ledger}-{event_index_within_tx}`
--   (e.g. "0000000050-0000000001-0000000000").  It uniquely identifies a
--   single contract event across the whole chain and is always non-null,
--   making it a better idempotency key than (tx_hash, response_array_index).
--
-- Why ALTER TABLE rather than recreating:
--   The original migration created sync_events without this column.  Adding
--   it here keeps the migration history linear and avoids data loss on
--   databases that have already run the original migration.
--
--   Existing rows (if any, e.g. from testnet runs) will have event_id = ''
--   due to the DEFAULT below, which is safe: the unique constraint only
--   fires on new INSERT attempts, and a fresh testnet run will start with an
--   empty table.

-- Add the column.  Default '' so the ALTER doesn't fail on existing rows.
-- New application code always supplies the real event_id, so '' is only
-- a migration-time sentinel for any pre-existing rows.
ALTER TABLE sync_events
    ADD COLUMN event_id TEXT NOT NULL DEFAULT '';

-- Remove the default immediately — new rows must always supply a value.
ALTER TABLE sync_events
    ALTER COLUMN event_id DROP DEFAULT;

-- Unique constraint that drives idempotent application.
-- Named explicitly for clarity in error messages and \d output.
ALTER TABLE sync_events
    ADD CONSTRAINT sync_events_event_id_unique UNIQUE (event_id);

-- Covering index for the idempotency-check INSERT ON CONFLICT lookup.
-- Redundant with the unique constraint's implicit index, but named
-- explicitly so intent is clear in EXPLAIN output.
-- (Postgres creates a unique index automatically; this comment documents
--  why we don't need a separate CREATE INDEX here.)

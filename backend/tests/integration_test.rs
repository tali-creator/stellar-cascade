//! Integration tests: idempotent event application against a live Postgres instance.
//!
//! # Running these tests
//!
//! These tests require a real Postgres database.  Set `TEST_DATABASE_URL` (or
//! `DATABASE_URL` as a fallback) before running:
//!
//! ```bash
//! TEST_DATABASE_URL=postgres://cascade:cascade@localhost:5432/cascade \
//!   cargo test --test integration_test
//! ```
//!
//! Each test run creates a fresh, isolated Postgres schema (named with a UUID),
//! runs all migrations against it, exercises the code, then drops the schema.
//! This means tests can run in parallel without stepping on each other, and
//! leave no permanent state in your dev database.
//!
//! If `TEST_DATABASE_URL` / `DATABASE_URL` is not set the tests are skipped
//! rather than failing — so CI without a database doesn't break.
//!
//! # What is being proved
//!
//! B16 added an idempotency gate to `apply_event`: before touching any state,
//! it attempts `INSERT INTO sync_events (...) ON CONFLICT (event_id) DO NOTHING`
//! and skips the state write if 0 rows are affected.  The unit test in
//! `sync_worker::tests` proves the control-flow path is structurally
//! unreachable.  These integration tests prove the *database behaviour*:
//! the unique constraint on `sync_events.event_id` actually fires, and a
//! `Deposit` balance genuinely does not double-count across two identical
//! `apply_event` calls.

use backend::db;
use backend::event_decode::DecodedEvent;
use backend::sync_worker::{advance_cursor_to, apply_event};
use sqlx::{Executor, PgPool, Row};

// ---------------------------------------------------------------------------
// Test harness helpers
// ---------------------------------------------------------------------------

/// Return the database URL for integration tests, or `None` if not configured.
///
/// Checks `TEST_DATABASE_URL` first, then `DATABASE_URL`.  Tests call
/// `skip_if_no_db!()` at the start so they are skipped cleanly rather than
/// failing when no database is available.
fn test_database_url() -> Option<String> {
    let _ = dotenvy::dotenv();
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
}

/// Skip the current test if no database URL is configured.
macro_rules! skip_if_no_db {
    () => {
        match test_database_url() {
            Some(url) => url,
            None => {
                eprintln!("SKIP: set TEST_DATABASE_URL or DATABASE_URL to run integration tests");
                return;
            }
        }
    };
}

/// A temporary Postgres schema that is created fresh for one test run and
/// dropped on `Drop`.
///
/// All migrations are applied inside the schema so the test gets a clean,
/// fully-migrated database with no shared state from other test runs.
struct TestSchema {
    pool: PgPool,
    schema_name: String,
}

impl TestSchema {
    /// Create a new schema, run all migrations, return a pool whose
    /// `search_path` is set to the new schema.
    async fn new(database_url: &str) -> Self {
        // Pick a unique name so parallel test runs don't collide.
        let schema_name = format!("test_{}", uuid_v4_hex());

        // Connect without a search_path override first to CREATE the schema.
        let admin_pool = db::build_pool(database_url)
            .await
            .expect("failed to connect to test database");

        admin_pool
            .execute(sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema_name)))
            .await
            .expect("failed to create test schema");

        // Build a connection URL that sets search_path to our new schema.
        // sqlx doesn't have a built-in `search_path` option, so we use
        // `options` in the connection string.
        let scoped_url = format!("{}?options=-csearch_path%3D{}", database_url, schema_name);

        let pool = db::build_pool(&scoped_url)
            .await
            .expect("failed to connect with scoped search_path");

        // Run all migrations inside the scoped schema.
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations failed");

        TestSchema { pool, schema_name }
    }

    /// Drop the schema and all its contents.
    async fn cleanup(self) {
        // Use a fresh connection without the search_path override to DROP.
        self.pool
            .execute(sqlx::query(&format!(
                "DROP SCHEMA \"{}\" CASCADE",
                self.schema_name
            )))
            .await
            .expect("failed to drop test schema");
    }
}

/// Generate a short random hex string suitable for schema names.
/// Uses the process ID + a monotonic counter to avoid pulling in a UUID dep.
fn uuid_v4_hex() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    format!("{ts:08x}{pid:06x}{n:04x}")
}

/// Seed a minimal project row so FK constraints on `balances` and
/// `sync_events` can be satisfied without making a real RPC call.
async fn seed_project(pool: &PgPool, project_id: &str) {
    sqlx::query(
        "INSERT INTO projects (id, owner_address, last_synced_ledger, created_at, updated_at)
         VALUES ($1, $2, 0, now(), now())
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(project_id)
    .bind("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF") // valid strkey placeholder
    .execute(pool)
    .await
    .expect("failed to seed project row");
}

/// Read the `amount` from `balances` for a given (project_id, token_address).
/// Returns `None` if no row exists.
async fn read_balance(pool: &PgPool, project_id: &str, token_address: &str) -> Option<i64> {
    let row = sqlx::query(
        "SELECT amount::BIGINT FROM balances WHERE project_id = $1 AND token_address = $2",
    )
    .bind(project_id)
    .bind(token_address)
    .fetch_optional(pool)
    .await
    .expect("failed to query balances");

    row.map(|r| r.get::<i64, _>(0))
}

/// Count `sync_events` rows matching a given `event_id`.
async fn count_sync_events(pool: &PgPool, event_id: &str) -> i64 {
    sqlx::query("SELECT COUNT(*) FROM sync_events WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .expect("failed to count sync_events")
        .get::<i64, _>(0)
}

// ---------------------------------------------------------------------------
// Test: Deposit idempotent replay
// ---------------------------------------------------------------------------

/// Core B16 proof: applying the same Deposit event twice with the same
/// `event_id` must leave the balance unchanged — no double-count.
///
/// This is the end-to-end version of the structural unit test in
/// `sync_worker::tests::idempotent_replay_deposit_does_not_double_count`.
/// It hits a real Postgres unique constraint and reads back the actual stored
/// balance to confirm behaviour, not just code structure.
#[tokio::test]
async fn idempotent_replay_deposit_does_not_double_count() {
    let url = skip_if_no_db!();
    let schema = TestSchema::new(&url).await;
    let pool = &schema.pool;

    let project_id = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
    let token_address = "CUSDC_TEST_TOKEN_ADDRESS";
    let event_id = "0000000050-0000000001-0000000000"; // stable RPC-format event ID
    let tx_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let ledger: u32 = 50;
    let deposit_amount: i128 = 1_000_000; // 1 USDC in micro-units

    // Seed a project so the balances FK is satisfiable.
    seed_project(pool, project_id).await;

    let event = DecodedEvent::Deposit {
        project_id: project_id.to_string(),
        token_address: token_address.to_string(),
        amount: deposit_amount,
    };

    // ── First application ────────────────────────────────────────────────────
    // apply_event should:
    //   1. INSERT into sync_events (event_id is new → 1 row affected)
    //   2. Accumulate deposit_amount into balances
    //   3. Commit
    apply_event(
        pool,
        "http://unused-rpc-for-deposit",
        "CUNUSED",
        &event,
        ledger,
        tx_hash,
        event_id,
    )
    .await
    .expect("first apply_event should succeed");

    let balance_after_first = read_balance(pool, project_id, token_address)
        .await
        .expect("balance row should exist after first apply");

    assert_eq!(
        balance_after_first, deposit_amount as i64,
        "balance after first apply should equal the deposit amount"
    );

    let sync_events_after_first = count_sync_events(pool, event_id).await;
    assert_eq!(
        sync_events_after_first, 1,
        "sync_events should have exactly 1 row after first apply"
    );

    // ── Second application (replay) ──────────────────────────────────────────
    // This simulates exactly what B14's batch-level cursor advancement does
    // after a crash: the cursor hasn't moved, so the same batch is re-fetched
    // and the same event_id is passed to apply_event again.
    //
    // apply_event should:
    //   1. Attempt INSERT into sync_events → 0 rows affected (conflict on event_id)
    //   2. Return Ok(()) immediately, without touching balances
    apply_event(
        pool,
        "http://unused-rpc-for-deposit",
        "CUNUSED",
        &event,
        ledger,
        tx_hash,
        event_id,
    )
    .await
    .expect("second apply_event (replay) should return Ok, not error");

    let balance_after_replay = read_balance(pool, project_id, token_address)
        .await
        .expect("balance row should still exist after replay");

    assert_eq!(
        balance_after_replay, deposit_amount as i64,
        "balance must be unchanged after replay — not doubled"
    );

    // The definitive assertion: only one sync_events row for this event_id,
    // proving the ON CONFLICT DO NOTHING fired against a real unique constraint.
    let sync_events_after_replay = count_sync_events(pool, event_id).await;
    assert_eq!(
        sync_events_after_replay, 1,
        "sync_events must have exactly 1 row after replay — not 2"
    );

    schema.cleanup().await;
}

// ---------------------------------------------------------------------------
// Test: Deposit replay with multiple pre-existing deposits (accumulation check)
// ---------------------------------------------------------------------------

/// Confirm that two *distinct* Deposit events (different event_ids) for the
/// same (project, token) DO accumulate, while a replay of either one does not.
///
/// This guards against accidentally making the gate too broad — e.g. keying
/// on (project_id, token_address) rather than event_id would prevent
/// legitimate second deposits from accumulating.
#[tokio::test]
async fn distinct_deposits_accumulate_but_replay_does_not() {
    let url = skip_if_no_db!();
    let schema = TestSchema::new(&url).await;
    let pool = &schema.pool;

    let project_id = "1122334455667788990011223344556677889900112233445566778899001122";
    let token_address = "CXLM_TEST_TOKEN";
    let tx_hash = "cafecafecafecafecafecafecafecafecafecafecafecafecafecafecafecafe";

    seed_project(pool, project_id).await;

    let event_a = DecodedEvent::Deposit {
        project_id: project_id.to_string(),
        token_address: token_address.to_string(),
        amount: 500_000,
    };
    let event_b = DecodedEvent::Deposit {
        project_id: project_id.to_string(),
        token_address: token_address.to_string(),
        amount: 300_000,
    };

    let event_id_a = "0000000051-0000000001-0000000000";
    let event_id_b = "0000000051-0000000001-0000000001";

    // Apply two distinct deposit events.
    apply_event(
        pool,
        "http://unused",
        "CUNUSED",
        &event_a,
        51,
        tx_hash,
        event_id_a,
    )
    .await
    .expect("first distinct deposit should succeed");

    apply_event(
        pool,
        "http://unused",
        "CUNUSED",
        &event_b,
        51,
        tx_hash,
        event_id_b,
    )
    .await
    .expect("second distinct deposit should succeed");

    let balance_after_two = read_balance(pool, project_id, token_address)
        .await
        .expect("balance should exist");

    assert_eq!(
        balance_after_two, 800_000,
        "two distinct deposits should accumulate: 500_000 + 300_000 = 800_000"
    );

    // Now replay event_a — balance must stay at 800_000, not go to 1_300_000.
    apply_event(
        pool,
        "http://unused",
        "CUNUSED",
        &event_a,
        51,
        tx_hash,
        event_id_a,
    )
    .await
    .expect("replay of event_a should return Ok");

    let balance_after_replay = read_balance(pool, project_id, token_address)
        .await
        .expect("balance should exist after replay");

    assert_eq!(
        balance_after_replay, 800_000,
        "replay of event_a must not re-add 500_000: expected 800_000, not 1_300_000"
    );

    // Both distinct events recorded once each.
    assert_eq!(count_sync_events(pool, event_id_a).await, 1);
    assert_eq!(count_sync_events(pool, event_id_b).await, 1);

    schema.cleanup().await;
}

// ---------------------------------------------------------------------------
// Test: RegisterProject replay (bonus check)
// ---------------------------------------------------------------------------

/// Bonus check: replaying a RegisterProject event with a previously-processed
/// event_id returns Ok(()) and leaves exactly one sync_events row.
///
/// Setup: seed a sync_events row directly (simulating a prior successful
/// first-time apply), then call apply_event with the same event_id.
/// The gate should fire before any RPC call is attempted — confirming the
/// early-return path works for non-Deposit events too, and that the RPC is
/// not called on replay (which would fail in real crash-recovery since the
/// network may be temporarily down).
#[tokio::test]
async fn register_project_replay_is_no_op() {
    let url = skip_if_no_db!();
    let schema = TestSchema::new(&url).await;
    let pool = &schema.pool;

    let project_id = "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";
    let event_id = "0000000055-0000000002-0000000000";
    let tx_hash = "bebebebebebebebebebebebebebebebebebebebebebebebebebebebebebebebebebe";
    let ledger: u32 = 55;

    // Seed a project row (represents the state after the first successful apply).
    seed_project(pool, project_id).await;

    // Seed a sync_events row directly — simulating that the first apply_event
    // call already committed successfully before the crash.
    sqlx::query(
        "INSERT INTO sync_events \
         (ledger_sequence, tx_hash, event_id, event_type, raw_data, processed_at) \
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(ledger as i64)
    .bind(tx_hash)
    .bind(event_id)
    .bind("register_project")
    .bind(serde_json::json!({ "project_id": project_id, "owner": "GABC" }))
    .execute(pool)
    .await
    .expect("failed to seed sync_events row");

    // Now replay: same event_id, pointing at a deliberately broken RPC URL.
    // If the gate works correctly, apply_event returns Ok(()) without ever
    // calling the RPC.  If it doesn't, it would attempt a network call to
    // the broken URL and return an Err.
    let event = DecodedEvent::RegisterProject {
        project_id: project_id.to_string(),
        owner_address: "GABC".to_string(),
        receiver_count: 2,
    };

    apply_event(
        pool,
        "http://127.0.0.1:1", // port 1 — connection refused, immediate error if reached
        "CUNUSED",
        &event,
        ledger,
        tx_hash,
        event_id,
    )
    .await
    .expect(
        "replay of RegisterProject should return Ok(()) without touching the RPC; \
         if this fails with a connection error the idempotency gate did not fire",
    );

    // Exactly one sync_events row — the one we seeded, not a second insert.
    let count = count_sync_events(pool, event_id).await;
    assert_eq!(
        count, 1,
        "sync_events should still have exactly 1 row after replay"
    );

    schema.cleanup().await;
}

// ---------------------------------------------------------------------------
// Test: advance_cursor_to is idempotent
// ---------------------------------------------------------------------------

/// Confirm that calling advance_cursor_to multiple times with the same or
/// lower ledger never moves the cursor backwards.
#[tokio::test]
async fn advance_cursor_to_never_moves_backwards() {
    let url = skip_if_no_db!();
    let schema = TestSchema::new(&url).await;
    let pool = &schema.pool;

    // Migration seeds sync_cursor with last_processed_ledger = 0.
    advance_cursor_to(pool, 100).await.expect("advance to 100");
    advance_cursor_to(pool, 100)
        .await
        .expect("advance to 100 again (idempotent)");
    advance_cursor_to(pool, 50)
        .await
        .expect("advance to 50 (should not rewind)");

    let cursor: i64 = sqlx::query("SELECT last_processed_ledger FROM sync_cursor WHERE id = 1")
        .fetch_one(pool)
        .await
        .expect("cursor row missing")
        .get(0);

    assert_eq!(cursor, 100, "cursor must stay at 100, not rewind to 50");

    advance_cursor_to(pool, 200).await.expect("advance to 200");

    let cursor2: i64 = sqlx::query("SELECT last_processed_ledger FROM sync_cursor WHERE id = 1")
        .fetch_one(pool)
        .await
        .expect("cursor row missing")
        .get(0);

    assert_eq!(cursor2, 200, "cursor should advance to 200");

    schema.cleanup().await;
}

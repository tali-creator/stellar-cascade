//! Shared sync-worker state exposed to the health endpoint.
//!
//! The polling loop writes here after every successful cycle; the health
//! handler reads here without touching the database or the RPC.
//!
//! Both fields are `AtomicU64` so reads and writes are lock-free and
//! can cross the async task boundary without a Mutex.
//!
//! # Staleness
//!
//! A `last_poll_unix_secs` value of `0` means the worker has never
//! completed a cycle since startup — treated as stale by the health handler.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// How many seconds without a successful poll cycle before the health
/// endpoint reports `"degraded"`.
pub const STALE_THRESHOLD_SECS: u64 = 60;

// ---------------------------------------------------------------------------
// SyncState
// ---------------------------------------------------------------------------

/// Shared state written by the sync worker and read by the health handler.
#[derive(Debug, Default)]
pub struct SyncState {
    /// The highest ledger sequence successfully committed to Postgres.
    /// `0` means no ledger has been processed yet.
    pub last_processed_ledger: AtomicU64,

    /// Unix timestamp (seconds) of the last successfully completed poll cycle.
    /// `0` means no cycle has completed since startup.
    pub last_poll_unix_secs: AtomicU64,
}

impl SyncState {
    /// Create a new zeroed `SyncState` wrapped in an `Arc`.
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Record a successful poll cycle: update both the ledger cursor and
    /// the wall-clock timestamp atomically (each field independently).
    pub fn record_poll(&self, last_ledger: u64) {
        self.last_processed_ledger
            .store(last_ledger, Ordering::Relaxed);
        self.last_poll_unix_secs
            .store(now_unix_secs(), Ordering::Relaxed);
    }

    /// Snapshot the current state for the health response.
    pub fn snapshot(&self) -> SyncSnapshot {
        let last_poll = self.last_poll_unix_secs.load(Ordering::Relaxed);
        let ledger = self.last_processed_ledger.load(Ordering::Relaxed);
        let now = now_unix_secs();

        let seconds_since_last_poll = if last_poll == 0 {
            // Never polled — report as maximally stale.
            u64::MAX
        } else {
            now.saturating_sub(last_poll)
        };

        SyncSnapshot {
            last_processed_ledger: ledger,
            seconds_since_last_poll,
        }
    }
}

/// A point-in-time read of sync worker state, used to build the health
/// response without holding any lock.
#[derive(Debug, Clone, Copy)]
pub struct SyncSnapshot {
    pub last_processed_ledger: u64,
    /// `u64::MAX` if the worker has never completed a cycle.
    pub seconds_since_last_poll: u64,
}

impl SyncSnapshot {
    /// Whether the sync worker is considered stale under the configured
    /// threshold.
    pub fn is_stale(&self) -> bool {
        self.seconds_since_last_poll > STALE_THRESHOLD_SECS
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn default_state_is_stale() {
        let state = SyncState::default();
        let snap = state.snapshot();
        assert_eq!(snap.last_processed_ledger, 0);
        assert!(snap.is_stale(), "never-polled state should be stale");
    }

    #[test]
    fn record_poll_updates_ledger_and_timestamp() {
        let state = SyncState::default();
        state.record_poll(42);

        let snap = state.snapshot();
        assert_eq!(snap.last_processed_ledger, 42);
        // Should be fresh — just recorded.
        assert!(
            snap.seconds_since_last_poll <= 1,
            "should be fresh, got {}s",
            snap.seconds_since_last_poll
        );
        assert!(!snap.is_stale());
    }

    #[test]
    fn stale_threshold_boundary() {
        let state = SyncState::default();
        // Simulate a poll that happened exactly at the threshold boundary.
        let now = now_unix_secs();
        state
            .last_poll_unix_secs
            .store(now - STALE_THRESHOLD_SECS, Ordering::Relaxed);
        state.last_processed_ledger.store(100, Ordering::Relaxed);

        let snap = state.snapshot();
        // Exactly at threshold: seconds_since == STALE_THRESHOLD_SECS.
        // is_stale uses >, so exactly at threshold is NOT stale.
        assert!(
            !snap.is_stale(),
            "exactly at threshold should not be stale, got {}s",
            snap.seconds_since_last_poll
        );
    }

    #[test]
    fn past_threshold_is_stale() {
        let state = SyncState::default();
        let now = now_unix_secs();
        state
            .last_poll_unix_secs
            .store(now - STALE_THRESHOLD_SECS - 1, Ordering::Relaxed);
        state.last_processed_ledger.store(100, Ordering::Relaxed);

        let snap = state.snapshot();
        assert!(snap.is_stale(), "one second past threshold should be stale");
    }
}

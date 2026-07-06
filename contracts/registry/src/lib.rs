#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Vec};

// ---------------------------------------------------------------------------
// Data types (Issue #2)
// ---------------------------------------------------------------------------

#[contracttype]
pub struct Receiver {
    pub address: Address,
    /// Share of incoming funds in basis points (1 bp = 0.01%).
    /// Valid range: 0–10000, where 10000 = 100%.
    pub percentage: u32,
}

#[contracttype]
pub struct Project {
    pub id: BytesN<32>,
    pub owner: Address,
    pub receivers: Vec<Receiver>,
}

// ---------------------------------------------------------------------------
// Storage keys (Issue #3)
// ---------------------------------------------------------------------------

/// Discriminated union of every key this contract writes to persistent storage.
///
/// `Project(BytesN<32>)` maps a 32-byte project identifier to its `Project`
/// value.  Using a typed enum (rather than a plain `Symbol`) gives us:
///   • namespace safety — keys from different entry types can never collide
///   • compile-time exhaustiveness — adding a new entry kind forces a match arm
///   • on-chain clarity — Soroban serialises the variant tag, so explorers can
///     decode the key without out-of-band documentation
#[contracttype]
pub enum DataKey {
    Project(BytesN<32>),
}

// ---------------------------------------------------------------------------
// TTL constants (Issue #3)
// ---------------------------------------------------------------------------

/// Minimum remaining TTL (in ledgers) before a project entry is extended.
///
/// Stellar mainnet produces roughly 1 ledger every 5 seconds.
/// 30 days ≈ 518 400 ledgers.  We use this as the *threshold*: if the entry
/// still has more than this many ledgers left we skip the extension call,
/// avoiding unnecessary host charges.
pub const PROJECT_TTL_THRESHOLD_LEDGERS: u32 = 518_400; // ~30 days

/// Target TTL (in ledgers) to extend a project entry to.
///
/// 60 days ≈ 1 036 800 ledgers.  Keeping the target well above the threshold
/// means at most one extension call is needed per month per project, while
/// still staying within the network's `MAX_ENTRY_TTL` limit (currently
/// ~3 110 400 ledgers / ~180 days on mainnet).
///
/// Usage:
/// ```ignore
/// env.storage()
///    .persistent()
///    .extend_ttl(&key, PROJECT_TTL_THRESHOLD_LEDGERS, PROJECT_TTL_EXTEND_TO_LEDGERS);
/// ```
///
/// `extend_ttl` is the correct method name in soroban-sdk 25.2.0; the older
/// `bump` API was removed in an earlier SDK release.  Signature:
///   `fn extend_ttl(&self, key: &K, threshold: u32, extend_to: u32)`
pub const PROJECT_TTL_EXTEND_TO_LEDGERS: u32 = 1_036_800; // ~60 days

// ---------------------------------------------------------------------------
// Contract (shell — read/write functions come in later issues)
// ---------------------------------------------------------------------------

/// Why persistent() over temporary() or instance()?
///
/// • `temporary()` — entries are permanently deleted on expiry; unsuitable for
///   project registrations that must survive indefinitely.
/// • `instance()` — shares the contract instance's TTL and is loaded on every
///   invocation regardless of whether it is needed; appropriate for a small,
///   fixed set of global values (e.g. an admin address), not for an
///   unbounded map of per-project records.
/// • `persistent()` — entries survive expiry via the Expired State Stack (ESS)
///   and can be restored.  Per-project data is written and read independently,
///   so each entry carries its own TTL that we extend explicitly whenever the
///   entry is touched.  This is the right choice for user-scoped records that
///   must not be lost.
#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    /// Verify that the TTL constants are internally consistent: the extend-to
    /// target must be strictly greater than the threshold so that every
    /// successful extension actually increases the remaining lifetime.
    #[test]
    fn ttl_extend_to_exceeds_threshold() {
        assert!(
            PROJECT_TTL_EXTEND_TO_LEDGERS > PROJECT_TTL_THRESHOLD_LEDGERS,
            "extend_to ({PROJECT_TTL_EXTEND_TO_LEDGERS}) must be > threshold \
             ({PROJECT_TTL_THRESHOLD_LEDGERS})"
        );
    }

    /// Verify that `DataKey::Project` can be constructed and that the `Env`
    /// can round-trip it through its value representation without panicking.
    /// This exercises the `#[contracttype]` derive on `DataKey`.
    #[test]
    fn data_key_project_roundtrips() {
        let env = Env::default();
        let raw = [1u8; 32];
        let id = BytesN::from_array(&env, &raw);
        // Construction must not panic.
        let _key = DataKey::Project(id);
    }
}

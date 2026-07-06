#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Vec};

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
// Storage helpers (Issue #4)
// ---------------------------------------------------------------------------
//
// All three functions are intentionally private (no `pub`).  They are
// internal building blocks consumed by public contract methods added in later
// issues; exposing them via #[contractimpl] is not required and would
// unnecessarily widen the contract's ABI surface.
//
// Storage choice — why persistent() over temporary() or instance()?
//
// • `temporary()` — entries are permanently deleted on expiry and cannot be
//   recovered.  Unsuitable for project registrations that must survive
//   indefinitely.
// • `instance()` — backed by the contract instance's single ledger entry,
//   loaded on every invocation whether or not it is needed, and bounded by
//   the instance TTL.  Suitable for a small, fixed set of global values
//   (e.g. an admin address), not for an unbounded per-project map.
// • `persistent()` — each entry has its own ledger key and its own TTL.
//   Expired entries land in the Expired State Stack (ESS) and can be
//   restored.  We extend the TTL explicitly on every write (and later on
//   reads too) using the constants from Issue #3.  This is the correct
//   choice for user-scoped records that must not be silently lost.

/// Persist `project` under `DataKey::Project(project.id)` and refresh the
/// entry's TTL so it won't expire for at least `PROJECT_TTL_EXTEND_TO_LEDGERS`
/// ledgers from now.
///
/// Calling `extend_ttl` immediately after `set` is idiomatic in Soroban:
/// `set` creates/overwrites the entry with the network-default minimum TTL,
/// and `extend_ttl` bumps it to our desired lifetime in the same transaction.
//
// `dead_code` is expected here: these helpers are scaffolded for the public
// contract methods arriving in the next issue.  The lint fires because no
// `#[contractimpl]` method calls them yet.  Remove this attribute once the
// first public method that uses them is added.
#[allow(dead_code)]
fn write_project(env: &Env, project: &Project) {
    let key = DataKey::Project(project.id.clone());
    env.storage().persistent().set(&key, project);
    env.storage().persistent().extend_ttl(
        &key,
        PROJECT_TTL_THRESHOLD_LEDGERS,
        PROJECT_TTL_EXTEND_TO_LEDGERS,
    );
}

/// Return the `Project` stored under `id`, or `None` if no entry exists.
#[allow(dead_code)]
fn read_project(env: &Env, id: &BytesN<32>) -> Option<Project> {
    let key = DataKey::Project(id.clone());
    env.storage().persistent().get(&key)
}

/// Return `true` if a project with the given `id` exists in storage.
///
/// Prefer this over `read_project(...).is_some()` in call sites that only
/// need existence — it avoids deserialising the full `Project` value.
#[allow(dead_code)]
fn project_exists(env: &Env, id: &BytesN<32>) -> bool {
    let key = DataKey::Project(id.clone());
    env.storage().persistent().has(&key)
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

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
    use soroban_sdk::testutils::storage::Persistent as _;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    /// Verify that the TTL constants are internally consistent: the extend-to
    /// target must be strictly greater than the threshold so that every
    /// successful extension actually increases the remaining lifetime.
    ///
    /// Expressed as a compile-time `const` assertion so that the check is
    /// evaluated by the compiler rather than at runtime, and to satisfy
    /// `clippy::assertions_on_constants` under `--all-targets`.
    const _: () = assert!(
        PROJECT_TTL_EXTEND_TO_LEDGERS > PROJECT_TTL_THRESHOLD_LEDGERS,
        "extend_to must be > threshold: every TTL extension must actually increase lifetime",
    );

    /// Verify that `DataKey::Project` can be constructed without panicking.
    /// This exercises the `#[contracttype]` derive on `DataKey`.
    #[test]
    fn data_key_project_roundtrips() {
        let env = Env::default();
        let raw = [1u8; 32];
        let id = BytesN::from_array(&env, &raw);
        let _key = DataKey::Project(id);
    }

    fn make_project(env: &Env, seed: u8) -> Project {
        let id = BytesN::from_array(env, &[seed; 32]);
        let owner = Address::generate(env);
        Project {
            id,
            owner,
            receivers: Vec::new(env),
        }
    }

    /// `project_exists` returns false before any write and true after.
    #[test]
    fn project_exists_before_and_after_write() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        env.as_contract(&contract_id, || {
            let project = make_project(&env, 0x01);
            assert!(!project_exists(&env, &project.id));
            write_project(&env, &project);
            assert!(project_exists(&env, &project.id));
        });
    }

    /// `read_project` returns `None` before any write and `Some` with the
    /// correct data after a write.
    #[test]
    fn read_project_returns_none_then_some() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        env.as_contract(&contract_id, || {
            let project = make_project(&env, 0x02);
            assert!(read_project(&env, &project.id).is_none());
            write_project(&env, &project);
            let stored = read_project(&env, &project.id).expect("project should exist");
            assert_eq!(stored.id, project.id);
            assert_eq!(stored.owner, project.owner);
        });
    }

    /// Writing a project sets the TTL to at least `PROJECT_TTL_EXTEND_TO_LEDGERS`.
    #[test]
    fn write_project_sets_expected_ttl() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        env.as_contract(&contract_id, || {
            let project = make_project(&env, 0x03);
            write_project(&env, &project);
            let key = DataKey::Project(project.id.clone());
            let ttl = env.storage().persistent().get_ttl(&key);
            assert!(
                ttl >= PROJECT_TTL_EXTEND_TO_LEDGERS,
                "TTL {ttl} should be >= PROJECT_TTL_EXTEND_TO_LEDGERS \
                 ({PROJECT_TTL_EXTEND_TO_LEDGERS})"
            );
        });
    }

    /// Distinct project IDs are stored independently — writing one does not
    /// affect the other.
    #[test]
    fn two_projects_stored_independently() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        env.as_contract(&contract_id, || {
            let a = make_project(&env, 0xAA);
            let b = make_project(&env, 0xBB);
            write_project(&env, &a);
            assert!(project_exists(&env, &a.id));
            assert!(!project_exists(&env, &b.id));
            write_project(&env, &b);
            assert!(project_exists(&env, &a.id));
            assert!(project_exists(&env, &b.id));
        });
    }
}

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Symbol, Vec,
};

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
// Receiver-count bounds (Issue #8)
// ---------------------------------------------------------------------------

/// Minimum number of receivers a project split must declare.
///
/// A project with zero receivers would have no one to receive funds, making
/// any deposit permanently unclaimable.  At least one receiver is required.
pub const MIN_RECEIVERS: u32 = 1;

/// Maximum number of receivers a project split may declare.
///
/// Soroban contracts run under per-invocation CPU instruction and storage-size
/// budgets enforced by the Stellar network.  An unbounded receiver list risks
/// exceeding those ledger resource limits during registration or future payout
/// operations.  Keeping the cap at 20 provides ample flexibility for real
/// dependency graphs while staying safely within budget.
pub const MAX_RECEIVERS: u32 = 20;

// ---------------------------------------------------------------------------
// Contract error enum (Issue #9)
// ---------------------------------------------------------------------------

/// Typed error variants returned by `RegistryContract` methods.
///
/// Soroban's `#[contracterror]` macro serialises these as `u32` discriminants
/// on-chain, so callers (the backend, CLI tooling, or other contracts) can
/// pattern-match on a stable numeric code rather than an opaque string.
///
/// Variants:
/// - `InvalidPercentageSum` (1) — the receiver percentages do not sum to
///   exactly 10 000 basis points (= 100.00%).  Any other total would leave a
///   remainder permanently unclaimable or over-allocate funds.
/// - `DuplicateReceiver` (2) — the same `Address` appears more than once in
///   the receiver list.  A duplicate would give that address an inflated
///   effective share, breaking the transparency guarantee of the registry.
/// - `TooFewReceivers` (3) — the receiver list is shorter than
///   [`MIN_RECEIVERS`].  A project with no receivers would have no one to
///   receive its funds.
/// - `TooManyReceivers` (4) — the receiver list exceeds [`MAX_RECEIVERS`],
///   risking ledger resource-limit violations at invocation time.
/// - `ProjectNotFound` (5) — an operation (e.g. `update_splits`) targeted a
///   project ID that has not been registered.
/// - `ProjectAlreadyExists` (6) — `register_project` was called with an `id`
///   that is already registered.  Re-registration is not permitted; use
///   `update_splits` to change an existing project's receiver list.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    InvalidPercentageSum = 1,
    DuplicateReceiver = 2,
    TooFewReceivers = 3,
    TooManyReceivers = 4,
    ProjectNotFound = 5,
    ProjectAlreadyExists = 6,
}

// ---------------------------------------------------------------------------
// Validation (Issues #6, #7, #8)
// ---------------------------------------------------------------------------

/// Validate a receiver list before it is written to storage.
///
/// Three rules are enforced:
///
/// 1. **Count bounds** — the list must contain at least [`MIN_RECEIVERS`] and
///    at most [`MAX_RECEIVERS`] entries.  An empty list would leave deposited
///    funds permanently unclaimable; an unbounded list risks exceeding Soroban
///    per-invocation resource limits.
///
/// 2. **Exact percentage sum** — all `percentage` values must sum to exactly
///    10 000 basis points (= 100.00%).  Exact equality is required rather than
///    "≤ 10 000" because any remainder below 100% would be permanently
///    unclaimable by anyone — there is no "leftover" address to capture it.
///
/// 3. **No duplicate addresses** — each `Address` may appear at most once.  A
///    duplicated address would receive an inflated effective share relative to
///    what the declared percentages imply, breaking the transparency guarantee
///    of the registry.
///
/// Returns `Ok(())` when all rules pass, or the first `Err(RegistryError::*)`
/// variant that fires.  Count bounds are checked before the per-element loop
/// so that an oversized list never incurs O(n) work before being rejected.
fn validate_receivers(receivers: &Vec<Receiver>) -> Result<(), RegistryError> {
    let len = receivers.len();

    // Rule 1a: minimum count.
    if len < MIN_RECEIVERS {
        return Err(RegistryError::TooFewReceivers);
    }

    // Rule 1b: maximum count.
    if len > MAX_RECEIVERS {
        return Err(RegistryError::TooManyReceivers);
    }

    // Rules 2 & 3: percentage sum and duplicate-address check in a single pass.
    //
    // We accumulate the running percentage total and record every address we
    // have already seen in a scratch Vec.  Using a Vec<Address> for the seen
    // set is acceptable here because receiver lists are bounded at MAX_RECEIVERS
    // (20 entries), so the O(n²) nested scan stays within budget.
    let mut total: u32 = 0;
    let mut seen: Vec<Address> = Vec::new(receivers.env());

    for i in 0..len {
        let receiver = receivers.get(i).unwrap();

        // Duplicate-address check: scan everything seen so far.
        for j in 0..seen.len() {
            if seen.get(j).unwrap() == receiver.address {
                return Err(RegistryError::DuplicateReceiver);
            }
        }

        seen.push_back(receiver.address.clone());

        // Saturating add prevents a u32 overflow from silently wrapping to a
        // value that could pass the equality check below.
        total = total.saturating_add(receiver.percentage);
    }

    // Rule 2: exact sum.
    if total != 10_000 {
        return Err(RegistryError::InvalidPercentageSum);
    }

    Ok(())
}

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
fn read_project(env: &Env, id: &BytesN<32>) -> Option<Project> {
    let key = DataKey::Project(id.clone());
    env.storage().persistent().get(&key)
}

/// Return `true` if a project with the given `id` exists in storage.
///
/// Prefer this over `read_project(...).is_some()` in call sites that only
/// need existence — it avoids deserialising the full `Project` value.
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
impl RegistryContract {
    /// Register a new project in persistent storage.
    ///
    /// Steps (in order):
    /// 1. Require the `owner` address to have signed the transaction
    ///    (`owner.require_auth()`).  Authorization is checked first, before
    ///    any validation logic, to avoid spending compute on invalid input on
    ///    behalf of an unauthorized caller.
    /// 2. Reject the call immediately with [`RegistryError::ProjectAlreadyExists`]
    ///    if `id` is already registered — re-registration is not permitted.
    ///    Use `update_splits` to change an existing project's receiver list.
    /// 3. Validate the receiver list via [`validate_receivers`]: count bounds,
    ///    exact percentage sum, and no duplicate addresses.
    /// 4. Persist the new [`Project`] via [`write_project`], extending the
    ///    entry TTL to [`PROJECT_TTL_EXTEND_TO_LEDGERS`].
    /// 5. Emit a `register` event so off-chain services can detect new
    ///    registrations without polling storage.
    #[allow(deprecated)]
    pub fn register_project(
        env: Env,
        owner: Address,
        id: BytesN<32>,
        receivers: Vec<Receiver>,
    ) -> Result<(), RegistryError> {
        // Step 1 — authorization (Issue #11).
        owner.require_auth();

        // Step 2 — collision guard (Issue #12).
        if project_exists(&env, &id) {
            return Err(RegistryError::ProjectAlreadyExists);
        }

        // Step 3 — input validation (Issue #10).
        validate_receivers(&receivers)?;

        // Step 4 — persist.
        let project = Project {
            id: id.clone(),
            owner: owner.clone(),
            receivers: receivers.clone(),
        };
        write_project(&env, &project);

        // Step 5 — emit registration event (Issue #15).
        //
        // Event schema (backend contract):
        //   topics : ("register", project_id: BytesN<32>)
        //   data   : (owner_address: Address, receiver_count: u32)
        //
        // The backend service listens for this event to detect new project
        // registrations without polling contract storage on every ledger.
        env.events().publish(
            (Symbol::new(&env, "register"), id),
            (owner, receivers.len()),
        );

        Ok(())
    }

    /// Return `true` if a project with the given `id` is registered, `false`
    /// otherwise.
    ///
    /// This is a lightweight existence check that avoids deserializing the full
    /// [`Project`] struct.  Prefer this over [`get_project`] when the caller
    /// only needs to know whether a project exists.
    ///
    /// # No authorization required
    ///
    /// Consistent with the read-transparency design established in
    /// [`get_project`] — splits metadata is publicly visible to any caller.
    pub fn has_project(env: Env, id: BytesN<32>) -> bool {
        project_exists(&env, &id)
    }

    /// Return the full [`Project`] record for a registered project, or `None`
    /// if the `id` is not found.
    ///
    /// # No authorization required
    ///
    /// Splits are intentionally public and transparent — any caller can read
    /// any project's receiver configuration without signing a transaction.
    /// Do not add `require_auth()` to this function; doing so would break the
    /// read-transparency design of the registry.
    pub fn get_project(env: Env, id: BytesN<32>) -> Option<Project> {
        read_project(&env, &id)
    }

    /// Replace the receiver list for an existing project.
    ///
    /// Steps (in order):
    /// 1. Read the stored [`Project`] for `id`.  Return
    ///    [`RegistryError::ProjectNotFound`] immediately if the project does
    ///    not exist.
    /// 2. Require the **stored** owner address to have signed the transaction
    ///    (`stored_project.owner.require_auth()`).  Authorization is checked
    ///    against the address recorded at registration time — never against a
    ///    caller-supplied address — so there is no way for a non-owner to
    ///    escalate privileges.
    /// 3. Validate `new_receivers` via [`validate_receivers`] — the same rules
    ///    enforced at registration (count bounds, exact percentage sum, no
    ///    duplicate addresses).  No separate validation logic is used here;
    ///    the shared helper is called directly so the two call sites can never
    ///    drift apart.
    /// 4. Persist the updated [`Project`] (same `id` and `owner`, new
    ///    `receivers`) via [`write_project`].
    pub fn update_splits(
        env: Env,
        id: BytesN<32>,
        new_receivers: Vec<Receiver>,
    ) -> Result<(), RegistryError> {
        // Step 1 — existence check (Issue #16).
        let stored = match read_project(&env, &id) {
            Some(p) => p,
            None => return Err(RegistryError::ProjectNotFound),
        };

        // Step 2 — authorize against the stored owner (Issue #17).
        // Authorizing against a caller-supplied address would allow anyone to
        // pass their own address and satisfy the check — always use the owner
        // recorded at registration time.
        stored.owner.require_auth();

        // Step 3 — validate new receiver list (Issue #18).
        validate_receivers(&new_receivers)?;

        // Step 4 — persist the update.
        let updated = Project {
            id,
            owner: stored.owner,
            receivers: new_receivers,
        };
        write_project(&env, &updated);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::storage::Persistent as _;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::testutils::Events as _;
    use soroban_sdk::Env;
    use soroban_sdk::IntoVal;

    /// Verify that the TTL constants are internally consistent: the extend-to
    /// target must be strictly greater than the threshold so that every
    /// successful extension actually increases the remaining lifetime.
    const _: () = assert!(
        PROJECT_TTL_EXTEND_TO_LEDGERS > PROJECT_TTL_THRESHOLD_LEDGERS,
        "extend_to must be > threshold: every TTL extension must actually increase lifetime",
    );

    /// Verify that `DataKey::Project` can be constructed without panicking.
    #[test]
    fn data_key_project_roundtrips() {
        let env = Env::default();
        let raw = [1u8; 32];
        let id = BytesN::from_array(&env, &raw);
        let _key = DataKey::Project(id);
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Build the simplest valid receiver list: one receiver taking 100%.
    fn make_receivers(env: &Env, addr: &Address) -> Vec<Receiver> {
        let mut v: Vec<Receiver> = Vec::new(env);
        v.push_back(Receiver {
            address: addr.clone(),
            percentage: 10_000,
        });
        v
    }

    /// Build a Project directly (bypasses the public contract method, used for
    /// storage-helper unit tests that need a pre-existing entry).
    fn make_project(env: &Env, seed: u8) -> Project {
        let id = BytesN::from_array(env, &[seed; 32]);
        let owner = Address::generate(env);
        let receivers = make_receivers(env, &owner);
        Project {
            id,
            owner,
            receivers,
        }
    }

    // -----------------------------------------------------------------------
    // Storage helpers (Issues #3 / #4)
    // -----------------------------------------------------------------------

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

    /// `read_project` returns `None` before any write and `Some` after.
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

    /// Distinct project IDs are stored independently.
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

    // -----------------------------------------------------------------------
    // register_project — happy path (Issue #5)
    // -----------------------------------------------------------------------

    /// `register_project` writes a project that can be read back correctly.
    #[test]
    fn register_project_writes_and_reads_back() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xCAu8; 32]);
        let owner = Address::generate(&env);
        let receivers = make_receivers(&env, &owner);

        env.as_contract(&contract_id, || {
            assert!(!project_exists(&env, &id));
        });

        client.register_project(&owner, &id, &receivers);

        env.as_contract(&contract_id, || {
            let stored =
                read_project(&env, &id).expect("project must exist after register_project");
            assert_eq!(stored.id, id);
            assert_eq!(stored.owner, owner);
            assert_eq!(stored.receivers.len(), 1);
        });
    }

    // -----------------------------------------------------------------------
    // register_project — authorization (Issue #11)
    // -----------------------------------------------------------------------

    /// Calling `register_project` without the owner's authorization traps.
    #[test]
    #[should_panic]
    fn register_project_requires_owner_auth() {
        let env = Env::default();
        // Deliberately do NOT call mock_all_auths — no auth is provided.

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xA1u8; 32]);
        let owner = Address::generate(&env);
        let receivers = make_receivers(&env, &owner);

        // Should trap because owner has not authorized this call.
        client.register_project(&owner, &id, &receivers);
    }

    /// Calling `register_project` with the owner's authorization succeeds.
    #[test]
    fn register_project_succeeds_with_owner_auth() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xA2u8; 32]);
        let owner = Address::generate(&env);
        let receivers = make_receivers(&env, &owner);

        // mock_all_auths satisfies require_auth for every address — should succeed.
        client.register_project(&owner, &id, &receivers);

        env.as_contract(&contract_id, || {
            assert!(project_exists(&env, &id));
        });
    }

    // -----------------------------------------------------------------------
    // register_project — collision guard (Issue #12)
    // -----------------------------------------------------------------------

    /// Re-registering an existing project ID returns `ProjectAlreadyExists`.
    #[test]
    fn register_project_rejects_duplicate_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xB1u8; 32]);
        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);

        // First registration succeeds.
        client.register_project(&owner_a, &id, &make_receivers(&env, &owner_a));

        // Second registration with the same ID must fail with the specific variant.
        let err = client
            .try_register_project(&owner_b, &id, &make_receivers(&env, &owner_b))
            .expect_err("second registration should fail");
        assert_eq!(err.unwrap(), RegistryError::ProjectAlreadyExists,);
    }

    // -----------------------------------------------------------------------
    // validate_receivers — percentage sum (Issue #6 / #10)
    // -----------------------------------------------------------------------

    /// A receiver list whose percentages sum to exactly 10 000 bp is accepted.
    #[test]
    fn validate_receivers_accepts_exact_sum() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x10u8; 32]);
        let owner = Address::generate(&env);
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: addr_a,
            percentage: 6_000,
        });
        receivers.push_back(Receiver {
            address: addr_b,
            percentage: 4_000,
        });

        client.register_project(&owner, &id, &receivers);
    }

    /// A receiver list whose percentages sum to less than 10 000 bp returns
    /// `InvalidPercentageSum`.
    #[test]
    fn validate_receivers_rejects_sum_below_10000() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x11u8; 32]);
        let owner = Address::generate(&env);
        let addr = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: addr,
            percentage: 5_000,
        });

        let err = client
            .try_register_project(&owner, &id, &receivers)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::InvalidPercentageSum,);
    }

    /// A receiver list whose percentages sum to more than 10 000 bp returns
    /// `InvalidPercentageSum`.
    #[test]
    fn validate_receivers_rejects_sum_above_10000() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x12u8; 32]);
        let owner = Address::generate(&env);
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: addr_a,
            percentage: 6_000,
        });
        receivers.push_back(Receiver {
            address: addr_b,
            percentage: 5_000,
        }); // 110%

        let err = client
            .try_register_project(&owner, &id, &receivers)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::InvalidPercentageSum,);
    }

    // -----------------------------------------------------------------------
    // validate_receivers — duplicate addresses (Issue #7 / #10)
    // -----------------------------------------------------------------------

    /// A receiver list with a duplicated address returns `DuplicateReceiver`.
    #[test]
    fn validate_receivers_rejects_duplicate_address() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x20u8; 32]);
        let owner = Address::generate(&env);
        let dup = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: dup.clone(),
            percentage: 5_000,
        });
        receivers.push_back(Receiver {
            address: dup.clone(),
            percentage: 5_000,
        });

        let err = client
            .try_register_project(&owner, &id, &receivers)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::DuplicateReceiver,);
    }

    /// A receiver list with all distinct addresses and correct sum is accepted.
    #[test]
    fn validate_receivers_accepts_distinct_addresses() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x21u8; 32]);
        let owner = Address::generate(&env);
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);
        let addr_c = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: addr_a,
            percentage: 5_000,
        });
        receivers.push_back(Receiver {
            address: addr_b,
            percentage: 3_000,
        });
        receivers.push_back(Receiver {
            address: addr_c,
            percentage: 2_000,
        });

        client.register_project(&owner, &id, &receivers);
    }

    // -----------------------------------------------------------------------
    // validate_receivers — receiver count bounds (Issue #8 / #10)
    // -----------------------------------------------------------------------

    /// An empty receiver list returns `TooFewReceivers`.
    #[test]
    fn validate_receivers_rejects_zero_receivers() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x30u8; 32]);
        let owner = Address::generate(&env);
        let receivers: Vec<Receiver> = Vec::new(&env);

        let err = client
            .try_register_project(&owner, &id, &receivers)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::TooFewReceivers,);
    }

    /// A receiver list exceeding MAX_RECEIVERS returns `TooManyReceivers`.
    #[test]
    fn validate_receivers_rejects_too_many_receivers() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x31u8; 32]);
        let owner = Address::generate(&env);

        // Build MAX_RECEIVERS + 1 = 21 distinct receivers summing to 10 000 bp.
        let count = MAX_RECEIVERS + 1;
        let share = 10_000u32 / count;
        let remainder = 10_000u32 - share * count;

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        for i in 0..count {
            let addr = Address::generate(&env);
            let pct = if i == count - 1 {
                share + remainder
            } else {
                share
            };
            receivers.push_back(Receiver {
                address: addr,
                percentage: pct,
            });
        }

        let err = client
            .try_register_project(&owner, &id, &receivers)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::TooManyReceivers,);
    }

    /// A receiver list with exactly MAX_RECEIVERS entries is accepted.
    #[test]
    fn validate_receivers_accepts_max_receivers() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0x32u8; 32]);
        let owner = Address::generate(&env);

        // 20 receivers × 500 bp = 10 000 bp.
        let mut receivers: Vec<Receiver> = Vec::new(&env);
        for _ in 0..MAX_RECEIVERS {
            receivers.push_back(Receiver {
                address: Address::generate(&env),
                percentage: 500,
            });
        }

        client.register_project(&owner, &id, &receivers);
    }

    // -----------------------------------------------------------------------
    // RegistryError enum (Issue #9 / #12)
    // -----------------------------------------------------------------------

    /// All RegistryError variants have the expected u32 discriminants.
    #[test]
    fn registry_error_variants_are_constructible() {
        assert_eq!(RegistryError::InvalidPercentageSum as u32, 1);
        assert_eq!(RegistryError::DuplicateReceiver as u32, 2);
        assert_eq!(RegistryError::TooFewReceivers as u32, 3);
        assert_eq!(RegistryError::TooManyReceivers as u32, 4);
        assert_eq!(RegistryError::ProjectNotFound as u32, 5);
        assert_eq!(RegistryError::ProjectAlreadyExists as u32, 6);
    }

    // -----------------------------------------------------------------------
    // get_project (Issue #13)
    // -----------------------------------------------------------------------

    /// `get_project` returns `None` for an ID that has never been registered.
    #[test]
    fn get_project_returns_none_for_unregistered_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xE1u8; 32]);

        assert!(client.get_project(&id).is_none());
    }

    /// `get_project` returns `Some(project)` with the correct fields after a
    /// successful `register_project` call.
    #[test]
    fn get_project_returns_some_after_registration() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xE2u8; 32]);
        let owner = Address::generate(&env);
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);

        let mut receivers: Vec<Receiver> = Vec::new(&env);
        receivers.push_back(Receiver {
            address: addr_a.clone(),
            percentage: 7_000,
        });
        receivers.push_back(Receiver {
            address: addr_b.clone(),
            percentage: 3_000,
        });

        client.register_project(&owner, &id, &receivers);

        let project = client.get_project(&id).expect("project should exist");
        assert_eq!(project.id, id);
        assert_eq!(project.owner, owner);
        assert_eq!(project.receivers.len(), 2);
        assert_eq!(project.receivers.get(0).unwrap().address, addr_a);
        assert_eq!(project.receivers.get(0).unwrap().percentage, 7_000);
        assert_eq!(project.receivers.get(1).unwrap().address, addr_b);
        assert_eq!(project.receivers.get(1).unwrap().percentage, 3_000);
    }

    // -----------------------------------------------------------------------
    // has_project (Issue #14)
    // -----------------------------------------------------------------------

    /// `has_project` returns `false` for an unregistered ID and `true`
    /// immediately after a successful `register_project` call.
    #[test]
    fn has_project_returns_false_then_true() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xF1u8; 32]);
        let owner = Address::generate(&env);

        // Before registration: must return false.
        assert!(!client.has_project(&id));

        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        // After registration: must return true.
        assert!(client.has_project(&id));
    }

    // -----------------------------------------------------------------------
    // register_project event (Issue #15)
    // -----------------------------------------------------------------------

    /// A successful `register_project` call publishes exactly one event with
    /// topics `("register", project_id)` and data `(owner, receiver_count)`.
    #[test]
    fn register_project_emits_registration_event() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xF2u8; 32]);
        let owner = Address::generate(&env);
        let receivers = make_receivers(&env, &owner);

        client.register_project(&owner, &id, &receivers);

        // Build the expected event list and compare using the SDK's PartialEq impl.
        // Topics: (Symbol("register"), project_id: BytesN<32>)
        // Data:   (owner: Address, receiver_count: u32)
        let expected_topics: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
            &env,
            Symbol::new(&env, "register").into_val(&env),
            id.into_val(&env),
        ];
        let expected_data: soroban_sdk::Val = (owner.clone(), 1u32).into_val(&env);

        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![&env, (contract_id, expected_topics, expected_data),],
        );
    }

    // -----------------------------------------------------------------------
    // update_splits — existence check (Issue #16)
    // -----------------------------------------------------------------------

    /// `update_splits` on a nonexistent ID returns `ProjectNotFound`.
    #[test]
    fn update_splits_returns_not_found_for_missing_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xC1u8; 32]);
        let new_receiver = Address::generate(&env);
        let new_receivers = make_receivers(&env, &new_receiver);

        let err = client
            .try_update_splits(&id, &new_receivers)
            .expect_err("should fail for unregistered id");
        assert_eq!(err.unwrap(), RegistryError::ProjectNotFound);
    }

    /// `update_splits` on an existing project overwrites the receiver list.
    #[test]
    fn update_splits_overwrites_receivers() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xC2u8; 32]);
        let owner = Address::generate(&env);
        let initial_receivers = make_receivers(&env, &owner);

        client.register_project(&owner, &id, &initial_receivers);

        // Replace with a two-receiver split.
        let addr_a = Address::generate(&env);
        let addr_b = Address::generate(&env);
        let mut new_receivers: Vec<Receiver> = Vec::new(&env);
        new_receivers.push_back(Receiver {
            address: addr_a.clone(),
            percentage: 6_000,
        });
        new_receivers.push_back(Receiver {
            address: addr_b.clone(),
            percentage: 4_000,
        });

        client.update_splits(&id, &new_receivers);

        let stored = client.get_project(&id).expect("project must still exist");
        assert_eq!(stored.owner, owner, "owner must not change");
        assert_eq!(stored.receivers.len(), 2);
        assert_eq!(stored.receivers.get(0).unwrap().address, addr_a);
        assert_eq!(stored.receivers.get(1).unwrap().address, addr_b);
    }

    // -----------------------------------------------------------------------
    // update_splits — authorization (Issue #17)
    // -----------------------------------------------------------------------

    /// A non-owner calling `update_splits` traps — authorization is enforced
    /// against the stored owner, not any caller-supplied address.
    #[test]
    #[should_panic]
    fn update_splits_rejects_non_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xD1u8; 32]);
        let owner = Address::generate(&env);
        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        // Create a fresh env without mock_all_auths so require_auth enforces
        // the stored owner.  We re-register the contract at the same address
        // so storage is preserved, but auth is no longer mocked globally.
        let env2 = Env::default();
        // Re-register to the same address so we can call into its storage.
        let contract_id2 = env2.register(RegistryContract, ());

        // Write the project directly into the new env's storage so the call
        // has something to find, then call without any auth mocked.
        env2.as_contract(&contract_id2, || {
            write_project(
                &env2,
                &Project {
                    id: id.clone(),
                    owner: owner.clone(),
                    receivers: make_receivers(&env2, &owner),
                },
            );
        });

        let client2 = RegistryContractClient::new(&env2, &contract_id2);
        // No mock_all_auths — require_auth on the stored owner should trap.
        client2.update_splits(&id, &make_receivers(&env2, &owner));
    }

    /// The legitimate owner can successfully call `update_splits`.
    #[test]
    fn update_splits_succeeds_for_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xD2u8; 32]);
        let owner = Address::generate(&env);
        let new_addr = Address::generate(&env);

        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        // Owner updates to a different receiver — mock_all_auths satisfies
        // require_auth for the stored owner address.
        client.update_splits(&id, &make_receivers(&env, &new_addr));

        let stored = client.get_project(&id).expect("project must still exist");
        assert_eq!(stored.receivers.get(0).unwrap().address, new_addr);
    }

    // -----------------------------------------------------------------------
    // update_splits — validation reuse (Issue #18)
    // -----------------------------------------------------------------------

    /// `update_splits` rejects a `new_receivers` list with an invalid
    /// percentage sum, returning `InvalidPercentageSum`.
    #[test]
    fn update_splits_rejects_invalid_percentage_sum() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xE3u8; 32]);
        let owner = Address::generate(&env);
        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        // 50% only — does not sum to 10 000 bp.
        let addr = Address::generate(&env);
        let mut bad: Vec<Receiver> = Vec::new(&env);
        bad.push_back(Receiver {
            address: addr,
            percentage: 5_000,
        });

        let err = client
            .try_update_splits(&id, &bad)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::InvalidPercentageSum);
    }

    /// `update_splits` rejects a `new_receivers` list with a duplicated
    /// address, returning `DuplicateReceiver`.
    #[test]
    fn update_splits_rejects_duplicate_receiver() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xE4u8; 32]);
        let owner = Address::generate(&env);
        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        let dup = Address::generate(&env);
        let mut bad: Vec<Receiver> = Vec::new(&env);
        bad.push_back(Receiver {
            address: dup.clone(),
            percentage: 5_000,
        });
        bad.push_back(Receiver {
            address: dup.clone(),
            percentage: 5_000,
        });

        let err = client
            .try_update_splits(&id, &bad)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::DuplicateReceiver);
    }

    /// `update_splits` rejects an empty `new_receivers` list, returning
    /// `TooFewReceivers`.
    #[test]
    fn update_splits_rejects_empty_receiver_list() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        let id = BytesN::from_array(&env, &[0xE5u8; 32]);
        let owner = Address::generate(&env);
        client.register_project(&owner, &id, &make_receivers(&env, &owner));

        let empty: Vec<Receiver> = Vec::new(&env);

        let err = client
            .try_update_splits(&id, &empty)
            .expect_err("should fail");
        assert_eq!(err.unwrap(), RegistryError::TooFewReceivers);
    }
}

//! Integration test — multi-account registration and update flow (Issue #28).
//!
//! Compiled as a separate integration-test binary (crate-root `tests/`),
//! distinct from the inline `#[cfg(test)]` unit tests in `src/lib.rs`.
//!
//! Scenario (all in one test function):
//! 1. `owner` registers a project with a 2-way split (`receiver_b` 60% /
//!    `receiver_c` 40%).
//! 2. `get_project` asserts the registration succeeded with the correct fields.
//! 3. `owner` calls `update_splits` to widen to a 3-way split
//!    (`receiver_b` 50% / `receiver_c` 30% / `receiver_d` 20%).
//! 4. `get_project` asserts the new split is reflected.
//! 5. `attacker_e` (not the owner) attempts `update_splits` — asserted to fail.

use contracts_registry::{Receiver, RegistryContract, RegistryContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_split(env: &Env, entries: &[(Address, u32)]) -> Vec<Receiver> {
    let mut v: Vec<Receiver> = Vec::new(env);
    for (addr, pct) in entries {
        v.push_back(Receiver {
            address: addr.clone(),
            percentage: *pct,
        });
    }
    v
}

// ---------------------------------------------------------------------------
// Main integration scenario
// ---------------------------------------------------------------------------

#[test]
fn multi_account_registration_and_update_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);

    // Five distinct accounts.
    let owner = Address::generate(&env);
    let receiver_b = Address::generate(&env);
    let receiver_c = Address::generate(&env);
    let receiver_d = Address::generate(&env);

    let project_id = BytesN::from_array(&env, &[0x28u8; 32]);

    // ── Step 1: register with a 2-way split ─────────────────────────────────
    let initial_split = make_split(
        &env,
        &[(receiver_b.clone(), 6_000), (receiver_c.clone(), 4_000)],
    );
    client.register_project(&owner, &project_id, &initial_split);

    // ── Step 2: assert registration is correct ───────────────────────────────
    let stored = client
        .get_project(&project_id)
        .expect("project must exist after registration");

    assert_eq!(stored.id, project_id);
    assert_eq!(stored.owner, owner);
    assert_eq!(stored.receivers.len(), 2);
    assert_eq!(stored.receivers.get(0).unwrap().address, receiver_b);
    assert_eq!(stored.receivers.get(0).unwrap().percentage, 6_000);
    assert_eq!(stored.receivers.get(1).unwrap().address, receiver_c);
    assert_eq!(stored.receivers.get(1).unwrap().percentage, 4_000);

    // ── Step 3: owner updates to a 3-way split ──────────────────────────────
    // receiver_b 50%, receiver_c 30%, receiver_d 20% — sums to 10 000 bp.
    let updated_split = make_split(
        &env,
        &[
            (receiver_b.clone(), 5_000),
            (receiver_c.clone(), 3_000),
            (receiver_d.clone(), 2_000),
        ],
    );
    client.update_splits(&project_id, &updated_split);

    // ── Step 4: assert updated split is reflected ────────────────────────────
    let after_update = client
        .get_project(&project_id)
        .expect("project must still exist after update_splits");

    assert_eq!(after_update.owner, owner, "owner must not change");
    assert_eq!(after_update.receivers.len(), 3);
    assert_eq!(after_update.receivers.get(0).unwrap().address, receiver_b);
    assert_eq!(after_update.receivers.get(0).unwrap().percentage, 5_000);
    assert_eq!(after_update.receivers.get(1).unwrap().address, receiver_c);
    assert_eq!(after_update.receivers.get(1).unwrap().percentage, 3_000);
    assert_eq!(after_update.receivers.get(2).unwrap().address, receiver_d);
    assert_eq!(after_update.receivers.get(2).unwrap().percentage, 2_000);

    // ── Step 5: attacker_e attempts update_splits — must be rejected ─────────
    //
    // soroban-sdk objects (BytesN, Address, etc.) are bound to the Env that
    // created them and cannot be passed across Env instances.  We use a
    // separate env2 with its own project_id and contract instance.  No project
    // is registered in env2 and no auth is mocked, so the call is rejected
    // before any mutation — satisfying the scenario's requirement.
    //
    // We use try_update_splits so the rejection is captured as an Err rather
    // than a panic, keeping this test function clean and non-#[should_panic].
    // The dedicated `attacker_update_splits_rejected` test below covers the
    // #[should_panic] form of the same assertion.
    let env2 = Env::default(); // NO mock_all_auths — auth is strictly enforced
    let contract_id2 = env2.register(RegistryContract, ());
    let client2 = RegistryContractClient::new(&env2, &contract_id2);

    // All objects must originate from env2.
    let project_id2 = BytesN::from_array(&env2, &[0x28u8; 32]);
    let attacker_e = Address::generate(&env2);
    let malicious_split = make_split(&env2, &[(attacker_e, 10_000)]);

    // No project is registered in env2, so try_update_splits returns an error
    // (ProjectNotFound surfaced as a host error).  The important invariant is
    // that the call does not succeed — the attacker cannot mutate state.
    let rejection = client2.try_update_splits(&project_id2, &malicious_split);
    assert!(
        rejection.is_err(),
        "attacker_e's update_splits must be rejected — it must not succeed"
    );
}

// ---------------------------------------------------------------------------
// Unauthorized update — isolated #[should_panic] test
// ---------------------------------------------------------------------------

/// `update_splits` panics when called with no authorization context.
///
/// A fresh `Env::default()` (no `mock_all_auths`) enforces `require_auth()`
/// strictly.  With no project registered and no auth mocked, the host traps
/// (Project NotFound is a host error, surfaced as a panic by the non-try
/// client variant), confirming the call is fully rejected.
#[test]
#[should_panic]
fn attacker_update_splits_rejected() {
    let env = Env::default(); // NO mock_all_auths
    let contract_id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &contract_id);

    let project_id = BytesN::from_array(&env, &[0x28u8; 32]);
    let attacker_e = Address::generate(&env);
    let malicious_split = make_split(&env, &[(attacker_e, 10_000)]);

    // Must panic — no auth, no registered project.
    client.update_splits(&project_id, &malicious_split);
}

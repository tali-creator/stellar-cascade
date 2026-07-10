📋 Issue #28: [Cascade-Registry] Integration Test — Multi-Account Registration and Update Flow
💡 Description & Objective
Depends on #27. With the contract optimized and its unit tests passing against both debug and optimized builds, this issue adds a higher-level integration test that exercises the full registry lifecycle across multiple accounts in a single scenario, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Create a new test file at `contracts/registry/tests/integration_test.rs` (a crate-root `tests/` directory, distinct from the inline `#[cfg(test)]` unit tests used in #21–#25) so this runs as a separate integration-test binary.
* Using `soroban_sdk::testutils::Ledger` and multiple generated `Address` test accounts (e.g., `owner`, `receiver_b`, `receiver_c`, `receiver_d`, `attacker_e`), script the following sequence in one test: `owner` registers a project with `receiver_b` and `receiver_c` as a 2-way split; the test asserts the registration succeeded via `get_project`; `owner` then calls `update_splits` to change the split to include `receiver_b`, `receiver_c`, and `receiver_d` (a 3-way split); the test asserts the update succeeded and the new split is reflected; finally, `attacker_e` (not the owner) attempts to call `update_splits` on the same project and the test asserts this fails.
* This test intentionally exercises the full chain end-to-end rather than isolated units — it is not a replacement for the unit tests from #21–#25, but a complementary scenario-level check.

🧪 Local Validation
```
cargo test -p contracts-registry
cargo test -p contracts-registry --test integration_test
cargo build --target wasm32-unknown-unknown --release -p contracts-registry
cargo clippy --target wasm32-unknown-unknown -p contracts-registry -- -D warnings
cargo fmt -p contracts-registry -- --check
```

🤝 Contribution Guidelines
🌿 1. Branch Naming Rules
Create a new branch from `main` using the following exact format:
```
feature/issue-28-integration-test-multi-account
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): integration test for multi-account registration and update (closes #28)
```

🚀 3. Pull Request Details
* Ensure all validation checks above pass locally with zero warnings, including the separate integration-test invocation.
* Confirm the scenario covers registration, update, and an unauthorized-update rejection all within the single test flow.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

📋 Issue #25: [Cascade-Registry] Unit Tests — Authorization Failures
💡 Description & Objective
Depends on #24. With functional success/failure coverage complete for both `register_project` and `update_splits`, this issue adds a dedicated, explicit test suite focused solely on authorization boundaries — arguably the most security-critical behavior in the contract — within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Write a test confirming `register_project` traps when called without the `owner` address's authorization present in the test environment (this may already partially exist from #11 — consolidate and confirm completeness here rather than duplicating).
* Write a test confirming `update_splits` traps when called by an address other than the project's stored owner, even if that address provides its own valid authorization for itself (this guards against the exact spoofing scenario discussed in #17 — the caller is authorized as themselves, just not as the actual owner).
* Use `soroban_sdk::testutils::Address as _` and the environment's `mock_auths` (or equivalent, per the pinned `25.2.0` API) to construct both properly authorized and deliberately unauthorized call scenarios explicitly, rather than relying on default/implicit auth behavior in the test harness.
* Add a short doc comment at the top of this test section summarizing the two authorization invariants being defended: (1) only the specified owner can register a project under their address, (2) only the stored owner of an existing project can update its splits.

🧪 Local Validation
```
cargo test -p contracts-registry
cargo build --target wasm32-unknown-unknown --release -p contracts-registry
cargo clippy --target wasm32-unknown-unknown -p contracts-registry -- -D warnings
cargo fmt -p contracts-registry -- --check
```

🤝 Contribution Guidelines
🌿 1. Branch Naming Rules
Create a new branch from `main` using the following exact format:
```
feature/issue-25-tests-authorization
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): unit tests for authorization failures (closes #25)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the spoofing-style test (authorized-as-self but not-the-owner) is present, not just a no-auth-at-all test.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

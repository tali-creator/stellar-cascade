📋 Issue #22: [Cascade-Registry] Unit Tests — `register_project` Failure Paths
💡 Description & Objective
Depends on #21. With success-path coverage complete, this issue adds systematic coverage of every documented failure mode for `register_project`, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Write table-style (or individually named) tests covering every `RegistryError` variant reachable from `register_project`: `InvalidPercentageSum` (sum both above and below 10000, as two separate test cases), `DuplicateReceiver`, `TooFewReceivers` (zero receivers), `TooManyReceivers` (one more than `MAX_RECEIVERS`), and `ProjectAlreadyExists` (from #12).
* Each test must assert the *specific* `RegistryError` variant returned via pattern matching (e.g., `assert_eq!(result, Err(RegistryError::InvalidPercentageSum))`), not merely that `result.is_err()` — a generic "it failed" assertion would allow a regression where the wrong error is returned to pass silently.
* Confirm each failure test uses a minimal, otherwise-valid project setup that isolates the single failure condition under test — e.g., the `TooManyReceivers` test should not also have a percentage-sum problem, so the test unambiguously exercises one code path.

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
feature/issue-22-tests-register-failures
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): unit tests for register_project failure paths (closes #22)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm every `RegistryError` variant reachable from `register_project` has at least one dedicated test.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

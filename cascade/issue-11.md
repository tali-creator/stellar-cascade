📋 Issue #11: [Cascade-Registry] Require Owner Authorization on Registration
💡 Description & Objective
Depends on #10. With typed errors now wired through validation, this issue adds an authorization requirement to `register_project` so that only the address signing the transaction can register a project under its own ownership, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add `owner.require_auth();` as the very first line inside `register_project`, before any validation logic runs — authorization should be checked before spending compute on validation.
* This relies on the `owner: Address` parameter already present in `register_project`'s signature since #5 — no signature change needed.
* Add a `testutils`-based test using `soroban_sdk::testutils::Address as _` and the test environment's mocked-auth utilities, confirming that calling `register_project` without the owner's authorization present in the test context causes the invocation to trap/fail.
* Add a second test confirming the happy path still works when authorization is correctly mocked for the owner address.

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
feature/issue-11-require-owner-auth-registration
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): require owner authorization on registration (closes #11)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm both the unauthorized-trap test and the authorized-success test are present and passing.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

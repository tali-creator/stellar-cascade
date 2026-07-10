📋 Issue #10: [Cascade-Registry] Wire Error Enum into Validation and Registration
💡 Description & Objective
Depends on #9. With `RegistryError` now defined, this issue connects it to the actual validation and registration flow, replacing any panic-based rejection from #6–#8 with proper typed error returns, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Change `validate_receivers`'s signature from `-> bool` (or panicking) to `-> Result<(), RegistryError>`.
* Map each failure path to its corresponding variant: percentage sum mismatch → `RegistryError::InvalidPercentageSum`, duplicate address → `RegistryError::DuplicateReceiver`, count out of bounds → `RegistryError::TooFewReceivers` or `RegistryError::TooManyReceivers`.
* Change `register_project`'s return type from `()` (as left in the #5 skeleton) to `Result<(), RegistryError>`, propagating whatever `validate_receivers` returns via `?` or explicit matching.
* Update every existing test written for #6, #7, and #8 to assert on the specific `RegistryError` variant returned, rather than only checking that a panic occurred — this is a breaking change to test assertions and must be handled in this same issue, not deferred.

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
feature/issue-10-wire-errors-into-registration
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): wire error enum into validation and registration (closes #10)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm every test from #6–#8 still passes after being updated to check specific error variants.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

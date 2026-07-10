📋 Issue #18: [Cascade-Registry] Reuse Split Validation Logic in `update_splits`
💡 Description & Objective
Depends on #17. With owner authorization now enforced on updates, this issue closes the last major gap in `update_splits`: it currently accepts any `new_receivers` list without checking percentage sums, duplicates, or count bounds — the same rules already enforced on registration. This is a refactor-and-extend issue, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Inside `update_splits`, call the existing `validate_receivers` function (originally from #6, extended through #8, and returning `Result<(), RegistryError>` since #10) on `new_receivers`, before constructing the updated `Project` struct.
* Propagate any validation error using `?` or explicit matching, exactly as `register_project` already does.
* This issue is explicitly a refactor-and-reuse task: confirm no validation logic is copy-pasted or reimplemented separately for `update_splits`. If you find yourself writing a second version of the percentage-sum or duplicate-check logic, stop — call the existing function instead.
* Add unit tests mirroring the failure cases from #6–#8 but exercised through `update_splits` instead of `register_project`: invalid percentage sum, duplicate receiver, and out-of-bounds receiver count, each asserting the correct `RegistryError` variant.

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
feature/issue-18-reuse-validation-in-update
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
refactor(contracts-registry): reuse split validation logic in update_splits (closes #18)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm no duplicated validation logic exists between `register_project` and `update_splits` — a reviewer should be able to diff the two functions and see the same function call, not similar-but-separate logic.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

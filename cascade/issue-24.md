📋 Issue #24: [Cascade-Registry] Unit Tests — `update_splits` Success and Failure
💡 Description & Objective
Depends on #23. With read-function coverage complete, this issue brings `update_splits` up to the same test-coverage standard already established for `register_project` in #21–#22, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Write a test confirming a successful `update_splits` call changes the stored `receivers` list, verifiable via a follow-up `get_project` call.
* Write a test confirming a successful update fires the `"update"` event (from #19) with correct topic and data.
* Write a test confirming `update_splits` called on a nonexistent project ID returns `RegistryError::ProjectNotFound` (from #16) specifically.
* Write tests reusing the same failure categories from #22 — `InvalidPercentageSum`, `DuplicateReceiver`, `TooFewReceivers`, `TooManyReceivers` — but exercised through `update_splits`'s `new_receivers` parameter instead of `register_project`'s `receivers` parameter, confirming the shared validation logic from #18 behaves identically in both entry points.

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
feature/issue-24-tests-update-splits
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): unit tests for update_splits success and failure (closes #24)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the `ProjectNotFound` test is present and distinct from the shared-validation failure tests.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

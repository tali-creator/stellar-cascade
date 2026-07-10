📋 Issue #14: [Cascade-Registry] Implement Public `has_project` Convenience Function
💡 Description & Objective
Depends on #13. With `get_project` now available for full reads, this issue adds a lighter-weight existence check for callers — like the planned Axum backend — that only need to know whether a project ID is registered, without paying the cost of deserializing the full `Project` struct, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add `pub fn has_project(env: Env, id: BytesN<32>) -> bool` to the `#[contractimpl]` block.
* Implement it as a thin public wrapper around the existing private `project_exists` helper from #4 — do not reimplement the existence check.
* No authorization required, consistent with the read-transparency design established in #13.
* Add a unit test confirming `has_project` returns `false` for an unregistered ID and `true` immediately after a successful `register_project` call.

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
feature/issue-14-implement-has-project
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): implement public has_project function (closes #14)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm both the `false` and `true` return-value test cases are present and passing.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

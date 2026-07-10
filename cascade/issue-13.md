📋 Issue #13: [Cascade-Registry] Implement Public `get_project` Read Function
💡 Description & Objective
Depends on #12. With registration now fully validated, authorized, and collision-protected, this issue adds the first public read function so external callers — including the backend service and Wave contributors testing the contract — can retrieve a project's stored splits, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add `pub fn get_project(env: Env, id: BytesN<32>) -> Option<Project>` to the `#[contractimpl] impl RegistryContract` block.
* Implement it as a thin wrapper around the private `read_project` helper already defined in #4 — do not duplicate the storage-read logic.
* No authorization check is required on this function — splits are intended to be publicly transparent and readable by anyone, matching the Drips-style fund-splitting philosophy behind Cascade.
* Add a doc comment noting explicitly that this function is intentionally unauthenticated, so future contributors don't mistakenly add a `require_auth()` call here.
* Add a unit test confirming `get_project` returns `None` for an unregistered ID, and a second test confirming it returns `Some(project)` with the exact receiver list and owner after a successful registration.

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
feature/issue-13-implement-get-project
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): implement public get_project read function (closes #13)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm both the "not found" and "found" test cases are present and passing.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

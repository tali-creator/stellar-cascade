📋 Issue #12: [Cascade-Registry] Prevent Re-registration of an Existing Project ID
💡 Description & Objective
Depends on #11. With owner authorization now enforced, this issue closes a remaining gap: nothing currently stops a caller from registering a second project under an ID that's already in use, silently overwriting the original split configuration. This issue adds that protection, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add a new variant to `RegistryError` (from #9): `ProjectAlreadyExists = 5`.
* Inside `register_project`, after the `owner.require_auth()` check (from #11) but before writing to storage, call `project_exists` (the private helper from #4) on the incoming `id`.
* If `project_exists` returns `true`, return `Err(RegistryError::ProjectAlreadyExists)` immediately — do not proceed to validation or storage writes.
* Add a unit test that registers a project successfully, then attempts to register a second project using the identical `id`, and asserts the second call returns `RegistryError::ProjectAlreadyExists` specifically.

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
feature/issue-12-prevent-project-id-collision
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): prevent re-registration of existing project id (closes #12)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the re-registration test asserts the specific `ProjectAlreadyExists` variant, not a generic failure.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

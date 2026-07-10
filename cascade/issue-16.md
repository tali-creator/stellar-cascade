📋 Issue #16: [Cascade-Registry] Implement `update_splits` Function Skeleton
💡 Description & Objective
Depends on #15. With registration fully validated, authorized, read-accessible, and event-emitting, this issue begins the second core capability of the registry: allowing a project owner to change their split configuration after initial registration. This issue implements the skeleton only, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add `pub fn update_splits(env: Env, id: BytesN<32>, new_receivers: Vec<Receiver>) -> Result<(), RegistryError>` to the `#[contractimpl]` block.
* Add a new `RegistryError` variant (extending the enum from #9): `ProjectNotFound = 6`.
* Inside the function, call `read_project` (from #4) on the given `id`. If it returns `None`, return `Err(RegistryError::ProjectNotFound)` immediately.
* If the project exists, construct an updated `Project` with the same `id` and `owner` but with `receivers` replaced by `new_receivers`, then call `write_project` to persist it.
* This is a skeleton issue only — no authorization check and no validation of `new_receivers` yet. Both are added in #17 and #18 respectively. Do not add them prematurely in this issue; keep the scope narrow so review stays focused.

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
feature/issue-16-update-splits-skeleton
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): implement update_splits function skeleton (closes #16)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm a test exists showing `update_splits` on a nonexistent ID returns `ProjectNotFound` and that a valid update overwrites the stored receivers.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

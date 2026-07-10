📋 Issue #26: [Cascade-Registry] Refactor `lib.rs` into Modules
💡 Description & Objective
Depends on #25. With the flat-splits registry now functionally complete and fully tested, this issue performs a structural cleanup before deployment: splitting the single, now-large `lib.rs` file into focused modules, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Create `contracts/registry/src/types.rs` containing `Project`, `Receiver`, and `DataKey` (from #2 and #3).
* Create `contracts/registry/src/errors.rs` containing the `RegistryError` enum (from #9, extended through #12, #16, and #20's audit).
* Create `contracts/registry/src/storage.rs` containing the private helper functions `write_project`, `read_project`, and `project_exists` (from #4).
* Create `contracts/registry/src/contract.rs` containing the `#[contract] RegistryContract` struct and its `#[contractimpl]` block with all public functions (`register_project`, `update_splits`, `get_project`, `has_project`).
* Slim `lib.rs` down to `#![no_std]`, module declarations (`mod types; mod errors; mod storage; mod contract;`), and necessary `pub use` re-exports so external code (tests, future backend integration) can still reference types at the crate root if needed.
* This is a pure refactor — no behavior change. Every test written in #21–#25 must pass unmodified except for import path adjustments (e.g., `use crate::types::Project;` instead of implicit same-file access).

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
feature/issue-26-refactor-into-modules
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
refactor(contracts-registry): split lib.rs into modules (closes #26)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm test count before and after the refactor is identical — no tests silently dropped during the file split.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

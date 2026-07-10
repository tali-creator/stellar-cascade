📋 Issue #20: [Cascade-Registry] Consistent `ProjectNotFound` Handling Across Public Functions
💡 Description & Objective
Depends on #19. With both `register_project` and `update_splits` now feature-complete, this issue is a consistency and cleanup pass: confirm that "project not found" is handled the same way everywhere it's relevant, rather than each function inventing its own convention, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Audit every public function implemented so far: `register_project`, `get_project`, `has_project`, `update_splits`.
* Confirm `get_project` and `has_project` intentionally return `Option<Project>` / `bool` (not `Result`) for "not found," since these are read functions where "not found" is a normal, expected outcome, not an error — this is consistent with the design intent from #13.
* Confirm `update_splits` uses `RegistryError::ProjectNotFound` (introduced in #16) consistently, and that no code path anywhere falls back to an `unwrap()` or `expect()` on an `Option<Project>` that could legitimately be `None`.
* This is a cleanup/audit issue — no new public functions should be added. If an inconsistency is found, fix it in place and note what was changed in the PR description.
* Add a short section to internal code comments (not public docs — that's #30) summarizing the convention: reads return `Option`/`bool`, mutating calls return `Result<_, RegistryError>`.

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
feature/issue-20-consistent-not-found-handling
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
refactor(contracts-registry): consistent project-not-found error handling (closes #20)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Explicitly list in the PR description any inconsistency found and fixed, or confirm none were found.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

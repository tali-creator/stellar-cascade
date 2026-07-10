📋 Issue #27: [Cascade-Registry] Add WASM Build Optimization Script
💡 Description & Objective
Depends on #26. With the codebase now cleanly modularized, this issue prepares the contract for deployment by introducing a repeatable build-optimization step, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add a script at `contracts/registry/optimize.sh` (or a `justfile`/`Makefile` target if the repo has already standardized on one elsewhere — check `CONTRIBUTING.md` first before introducing a new convention) that runs `soroban contract optimize` (or the current Stellar CLI equivalent — verify exact subcommand name against the CLI version referenced in the repo's docs) against the release WASM artifact produced by `cargo build --target wasm32-unknown-unknown --release -p contracts-registry`.
* Have the script print the resulting optimized `.wasm` file size to stdout for visibility.
* Run the full existing test suite (from #21–#25) against the optimized artifact where feasible, or at minimum confirm via `soroban contract invoke` (or CLI equivalent) that `register_project` and `get_project` still behave correctly when invoked against the optimized WASM — optimization should never change contract behavior, only binary size.
* Document the before/after size numbers in the PR description as a sanity-check data point for reviewers.

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
feature/issue-27-wasm-optimization-script
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): add wasm build optimization script (closes #27)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Include before/after WASM size numbers in the PR description.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

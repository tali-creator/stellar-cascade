📋 Issue #30: [Cascade-Registry] Document Registry Contract Usage in README
💡 Description & Objective
Depends on #29. With the Splits Registry contract now deployed to testnet and fully tested, this final issue in the flat-registry phase documents the contract for external readers — Wave reviewers, future contributors, and the backend team — within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add a "Splits Registry Contract" section either to the main repository README or a dedicated `contracts/registry/README.md` linked from the main README (match whichever convention the repo already uses for other component docs).
* Explicitly describe the flat-splits model in plain language: a project registers a fixed list of receivers with basis-point percentages summing to 100%; there is no support for nested dependency trees or multi-level splitting in this phase — that is intentionally deferred to a future phase, so readers don't assume more functionality exists than what's built.
* Document the full public function list with parameter descriptions: `register_project(owner, id, receivers)`, `update_splits(id, new_receivers)`, `get_project(id)`, `has_project(id)` — including which require authorization (per #11, #17) and which are public reads (per #13, #14).
* Include the testnet contract ID recorded in `DEPLOYMENTS.md` (from #29).
* Include a short example invocation using the Stellar CLI showing a `register_project` call with example parameters, so a reader can try it against testnet directly.
* This issue explicitly closes out the flat-registry MVP phase — do not add speculative documentation about deposit/claim functionality or nested trees, since those are out of scope for this phase and not yet implemented.

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
feature/issue-30-document-registry-readme
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
docs(contracts-registry): document registry contract usage in readme (closes #30)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above still pass locally with zero warnings (docs-only changes shouldn't affect them, but confirm nothing was accidentally broken).
* Confirm the documentation explicitly states the flat-splits scope boundary (no nested trees yet), so external readers don't misjudge project maturity.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

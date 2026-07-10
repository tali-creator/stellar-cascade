📋 Issue #17: [Cascade-Registry] Require Owner Authorization on `update_splits`
💡 Description & Objective
Depends on #16. With the `update_splits` skeleton in place, this issue closes a serious gap: as written, anyone can currently change any project's splits, regardless of ownership. This issue restricts updates to the project's actual registered owner, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Inside `update_splits`, after successfully reading the existing project via `read_project` (and confirming it exists, per #16) but before constructing the updated struct, call `.require_auth()` on the **stored** owner address from the retrieved `Project` — not on any caller-supplied address parameter.
* This distinction matters: authorizing against a caller-supplied address would allow anyone to claim to be "the owner" and pass their own address for authorization, defeating the purpose of the check. Always authorize against the address that was recorded at registration time.
* Add a unit test where Account A registers a project, then Account B (a different test address, not the owner) attempts to call `update_splits` on that project ID — confirm this traps/fails.
* Add a second test confirming the legitimate owner (Account A) can successfully update their own project's splits.

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
feature/issue-17-require-owner-auth-update
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): require owner authorization on update_splits (closes #17)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm both the non-owner-rejection test and the owner-success test are present and passing.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

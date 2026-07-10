📋 Issue #15: [Cascade-Registry] Emit Event on Project Registration
💡 Description & Objective
Depends on #14. With both read functions now in place, this issue adds an on-chain event emission to `register_project`, giving off-chain services — specifically the planned Axum backend that caches project metadata in Postgres — a way to detect new registrations without polling contract storage, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Inside `register_project`, immediately after the successful `write_project` call and before returning `Ok(())`, add an event emission using `env.events().publish((Symbol::new(&env, "register"), id.clone()), (owner.clone(), receivers.len() as u32));`.
* Confirm the exact `env.events().publish` signature and `Symbol` construction against the pinned `soroban-sdk = 25.2.0` API — do not assume syntax from memory or from other SDK versions.
* Add a doc comment directly above the emission documenting the event schema: topics are `("register", project_id)`, and data is `(owner_address, receiver_count)` — this is the contract the backend team will code against.
* Add a unit test using `env.events().all()` after a successful registration, confirming exactly one event was published with the expected topic and data structure.

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
feature/issue-15-emit-registration-event
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): emit event on project registration (closes #15)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the event-schema doc comment is present and matches the actual emitted topics/data exactly.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

📋 Issue #19: [Cascade-Registry] Emit Event on Splits Update
💡 Description & Objective
Depends on #18. With `update_splits` now authorized and validated to the same standard as registration, this issue mirrors #15 by emitting an on-chain event so off-chain services can detect split changes without polling storage, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Inside `update_splits`, immediately after the successful `write_project` call and before returning `Ok(())`, add an event emission using `env.events().publish((Symbol::new(&env, "update"), id.clone()), (new_receivers.len() as u32));`.
* Confirm the exact `env.events().publish` and `Symbol` construction syntax against the pinned `soroban-sdk = 25.2.0` API, consistent with how #15 verified it — do not assume the two emissions need different verification.
* Add a doc comment documenting this event's schema: topic `("update", project_id)`, data `(new_receiver_count)` — distinct from the `"register"` event schema documented in #15, so downstream listeners (the backend) can distinguish the two event types by topic.
* Add a unit test confirming the update event fires only when `update_splits` succeeds, and does **not** fire when the call fails validation (per #18) or authorization (per #17) — use `env.events().all()` to assert the event list is empty in the failure cases.

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
feature/issue-19-emit-update-event
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): emit event on splits update (closes #19)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the "no event on failure" test is present, not just the success-path event test.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

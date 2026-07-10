📋 Issue #8: [Cascade-Registry] Enforce Minimum and Maximum Receiver Count
💡 Description & Objective
Depends on #7. With percentage-sum and duplicate-address checks in place, this issue adds bounds on how many receivers a single project split can have, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add two crate-level constants near the top of `lib.rs`: `const MIN_RECEIVERS: u32 = 1;` and `const MAX_RECEIVERS: u32 = 20;`.
* Extend `validate_receivers` (from #6–#7) to reject any receiver vector whose length falls outside `[MIN_RECEIVERS, MAX_RECEIVERS]`.
* Add a doc comment explaining why an upper bound exists: Soroban contracts operate under CPU instruction and storage-size limits per invocation, so an unbounded receiver list risks exceeding ledger resource limits during registration or future payout operations.
* Keep `MIN_RECEIVERS` and `MAX_RECEIVERS` as named constants rather than inline magic numbers, so a future issue can adjust the cap without hunting through the validation logic.

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
feature/issue-8-enforce-receiver-count-bounds
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): enforce min/max receiver count (closes #8)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Add unit tests for both boundary violations: zero receivers, and a receiver count exceeding `MAX_RECEIVERS`.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

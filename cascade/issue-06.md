📋 Issue #6: [Cascade-Registry] Validate Split Percentages Sum to 100%
💡 Description & Objective
Depends on #5. With `register_project` wired up as a skeleton that writes any receiver list to storage without checks, this issue adds the first validation rule: split percentages must sum to exactly 100%, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Add a private `fn validate_receivers(receivers: &Vec<Receiver>) -> bool` (or have it panic for now — a proper `Result`-based error type is introduced in #9, so keep this simple until then).
* Sum all `Receiver.percentage` values across the vector.
* Confirm the total equals exactly `10000` (basis points, where 10000 = 100.00%, matching the convention documented in #2).
* Call `validate_receivers` from inside `register_project`, before `write_project` is invoked — reject (panic or early-return) if validation fails.
* Add a doc comment explaining why exact equality is required rather than "≤ 10000" — a partial split would leave a remainder of funds permanently unclaimable by anyone.

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
feature/issue-6-validate-percentage-sum
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): validate split percentages sum to 100% (closes #6)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Add at least one test proving a receiver list summing to less than or more than 10000 is rejected.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

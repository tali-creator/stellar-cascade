📋 Issue #7: [Cascade-Registry] Reject Duplicate Receiver Addresses
💡 Description & Objective
Depends on #6. With percentage-sum validation in place, this issue extends `validate_receivers` to reject a second class of invalid input: the same receiver address appearing more than once in a single project's split list, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Extend the existing `validate_receivers` function (from #6) to also check for duplicate `Address` entries across the `receivers` vector.
* Since `soroban_sdk::Vec` does not provide built-in dedup or set semantics, implement this via nested iteration (comparing each entry against every other entry) — acceptable at this scale, since receiver lists are bounded and small.
* Alternatively, use a scratch `Vec<Address>` to track addresses already seen while iterating once, rejecting as soon as a repeat is found.
* Keep this check inside the same `validate_receivers` function from #6 — do not create a second validation function.
* Add a doc comment explaining why duplicates are disallowed: a duplicated address would receive an inflated effective share relative to what the percentage list implies, breaking the transparency guarantee of the registry.

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
feature/issue-7-reject-duplicate-receivers
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): reject duplicate receiver addresses (closes #7)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Add a unit test with a deliberately duplicated address in the receiver list and confirm registration is rejected.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

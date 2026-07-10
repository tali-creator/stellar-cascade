📋 Issue #9: [Cascade-Registry] Define Contract Error Enum
💡 Description & Objective
Depends on #8. With three validation rules now implemented (percentage sum, duplicates, receiver-count bounds), this issue formalizes how failures are communicated to callers by introducing a proper Soroban contract error type, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Define `#[contracterror] #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)] #[repr(u32)] pub enum RegistryError { ... }`.
* Add one variant per failure case introduced so far: `InvalidPercentageSum = 1`, `DuplicateReceiver = 2`, `TooFewReceivers = 3`, `TooManyReceivers = 4`.
* Assign explicit `u32` discriminants starting at 1 — Soroban's `#[contracterror]` macro requires explicit values, and leaving gaps for future variants (e.g., skipping to 10, 20 for future categories) is acceptable if you want room to group related errors later, but sequential is fine for this MVP phase.
* This issue defines the enum only. Do not yet change `validate_receivers`'s return type or `register_project`'s signature — that wiring happens in #10.
* Add a doc comment on the enum itself summarizing what each variant means, since this will be the primary interface contributors and the backend team read to understand failure modes.

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
feature/issue-9-define-error-enum
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
feat(contracts-registry): define contract error enum (closes #9)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the enum compiles and derives all traits required by Soroban's `#[contracterror]` macro.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

📋 Issue #21: [Cascade-Registry] Unit Tests — `register_project` Success Path
💡 Description & Objective
Depends on #20. With the contract's core logic and conventions now stable, this issue begins a dedicated test-hardening phase, starting with comprehensive coverage of the `register_project` success path, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* If not already organized this way, ensure a clearly separated `#[cfg(test)] mod test { ... }` section exists (or a dedicated test module file, depending on how #26's later refactor is anticipated — for now, inline is fine).
* Write a test registering a valid project with exactly 2 receivers summing to 10000 bps, and confirm `get_project` returns the exact `owner`, `id`, and `receivers` that were submitted.
* Write a second test registering a project with the maximum allowed receiver count (`MAX_RECEIVERS` from #8) to confirm the upper boundary itself is a valid success case, not just values below it.
* Write a third test confirming that after a successful registration, exactly one `"register"` event (per #15) is present in `env.events().all()`, with the correct topic and data.
* These tests should be additive to whatever tests already exist from #6–#19 — do not delete or replace prior tests, only fill gaps in success-path coverage.

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
feature/issue-21-tests-register-success
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): unit tests for register_project success path (closes #21)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the boundary-case test (maximum receivers) is present, not just a typical 2–3 receiver case.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

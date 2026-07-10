📋 Issue #23: [Cascade-Registry] Unit Tests — `get_project` and `has_project`
💡 Description & Objective
Depends on #22. With registration's success and failure paths fully covered, this issue closes the test-coverage gap for the two read functions, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Write a test confirming `get_project` returns `None` for an ID that was never registered.
* Write a test confirming `get_project` returns `Some(project)` with exactly matching `owner`, `id`, and `receivers` fields after a successful registration.
* Write a test confirming `has_project` returns `false` for an unregistered ID and `true` immediately following successful registration.
* Write a test confirming both `get_project` and `has_project` remain callable with **no** authorization context mocked in the test environment — this explicitly verifies the read-transparency design decision from #13 and #14 (that these functions never call `require_auth()`), guarding against a future regression where auth is mistakenly added to a read path.

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
feature/issue-23-tests-get-has-project
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
test(contracts-registry): unit tests for get_project and has_project (closes #23)
```

🚀 3. Pull Request Details
* Ensure all four validation checks above pass locally with zero warnings.
* Confirm the no-auth-required test is present specifically — this is easy to skip since it tests for an *absence* of a check.
* Do not commit `target/` or unrelated `Cargo.lock` changes.
* Submit your branch for peer review.

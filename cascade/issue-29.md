📋 Issue #29: [Cascade-Registry] Deploy Registry Contract to Stellar Testnet
💡 Description & Objective
Depends on #28. With unit and integration test coverage now complete and passing against the optimized build, this issue takes the Splits Registry live on Stellar's public testnet for the first time, within the `contracts-registry` component scope.

🛠️ Step-by-Step Implementation Guide
* Verify the exact deployment command against whatever Stellar CLI version is specified in the repo's `CONTRIBUTING.md` — command names have shifted between `soroban contract deploy` (older CLI) and `stellar contract deploy` (current CLI naming); do not assume which applies without checking.
* Deploy the optimized WASM artifact produced in #27 to Stellar testnet using a dedicated deployer identity (not a personal/mainnet-funded key) — generate a fresh testnet identity if one doesn't already exist in the repo's tooling.
* Fund the deployer identity via Stellar's testnet friendbot if needed before deployment.
* Record the deployment outcome in a new file, `contracts/registry/DEPLOYMENTS.md`, including: the resulting contract ID, the network (explicitly labeled `testnet`), the deployment date, and the deployer's public address (not any secret key).
* Explicitly do not deploy to Stellar mainnet in this issue — mainnet deployment is out of scope for this MVP phase and would need its own dedicated review process.

🧪 Local Validation
```
cargo test -p contracts-registry
cargo test -p contracts-registry --test integration_test
cargo build --target wasm32-unknown-unknown --release -p contracts-registry
cargo clippy --target wasm32-unknown-unknown -p contracts-registry -- -D warnings
cargo fmt -p contracts-registry -- --check
```

🤝 Contribution Guidelines
🌿 1. Branch Naming Rules
Create a new branch from `main` using the following exact format:
```
feature/issue-29-deploy-testnet
```

💬 2. Commit Message Standard
Use Conventional Commits format when creating commits. Your final commit (or PR squash commit) must include:
```
chore(contracts-registry): deploy registry contract to stellar testnet (closes #29)
```

🚀 3. Pull Request Details
* Ensure all validation checks above still pass locally with zero warnings prior to deployment.
* Confirm `DEPLOYMENTS.md` contains the contract ID, network, date, and deployer public address — and no secret keys.
* Do not commit `target/`, unrelated `Cargo.lock` changes, or any private key material.
* Submit your branch for peer review.

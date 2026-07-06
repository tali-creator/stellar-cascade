# Contributing to Cascade

Welcome — this guide is written for people landing here cold, likely through a Drips Wave sprint, with no prior context on the repo. It covers everything you need to get a working local setup and submit a PR.

---

## 1. Before you start

Pick an open issue from the Issues tab. Issues are labeled by layer (`frontend`, `backend`, `contracts`) and difficulty — comment on the issue to claim it before starting work, so effort isn't duplicated during the sprint window.

---

## 2. Prerequisites

Install these once, regardless of which layer you're contributing to:

```bash
# Rust (needed for both backend/ and contracts/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none

# Stellar CLI — build/deploy/invoke Soroban contracts
cargo install --locked stellar-cli --features opt

# Node.js 18+ (needed for frontend/)
node -v   # should print v18 or higher
npm -v
```

Verify everything installed correctly:
```bash
cargo --version
stellar --version
node --version
```

---

## 3. Clone and install

```bash
git clone https://github.com/<your-username>/stellar-cascade.git
cd stellar-cascade
```

### Frontend dependencies
```bash
cd frontend
npm install
cd ..
```

### Backend and contracts
No separate install step — Cargo fetches and caches dependencies automatically the first time you build or test each crate (see below).

---

## 4. Working on `contracts/` (Soroban)

### 4.1 Set up a Stellar identity (one-time)

You need a funded testnet identity to deploy or invoke contracts:

```bash
stellar keys generate contributor --network testnet --fund
```

This creates a local keypair named `contributor` and funds it with testnet XLM via Friendbot automatically.

### 4.2 Configure the testnet network (one-time)

```bash
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### 4.3 Build a contract

```bash
cd contracts/<contract-name>
cargo build --target wasm32v1-none --release
```

### 4.4 Run tests

```bash
cargo test
```

### 4.5 Deploy to testnet (only if your issue requires live deployment)

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/<contract_name>.wasm \
  --source contributor \
  --network testnet
```

This returns a contract ID — you'll need it to invoke functions on the deployed contract for manual testing.

---

## 5. Working on `backend/` (Rust/Axum)

```bash
cd backend
cp .env.example .env   # fill in RPC URL, DB connection string
cargo run
```

Run tests:
```bash
cargo test
```

---

## 6. Working on `frontend/` (Next.js/TypeScript)

```bash
cd frontend
npm run dev
```

Visit `http://localhost:3000`.

Run tests:
```bash
npm run test
```

---

## 7. Where tests live

| Layer | Test location | Run with |
|---|---|---|
| `contracts/` | Inside each contract crate, typically `src/test.rs` or a `tests/` folder alongside `lib.rs` | `cargo test` from inside the contract's folder |
| `backend/` | `backend/src/**/*` as inline `#[cfg(test)]` modules, or `backend/tests/` for integration tests | `cargo test` from inside `backend/` |
| `frontend/` | Co-located with components as `*.test.tsx`, or under `frontend/__tests__/` | `npm run test` from inside `frontend/` |

If you add a new feature, add a corresponding test in the matching location — PRs without any test coverage for new logic will likely be asked for changes before merge.

---

## 8. Submitting a PR

1. Create a branch off `main`: `git checkout -b fix/short-description`
2. Keep the PR scoped to a single issue — don't bundle unrelated changes
3. Make sure CI passes locally before pushing:
```bash
   # contracts
   cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

   # backend
   cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

   # frontend
   cd frontend && npm run lint && npm run test
```
4. Reference the issue number in your PR description (e.g. `Closes #12`)
5. Push and open the PR against `main`

---

## 9. How review works

- A maintainer will review your PR, typically within the sprint window — reviews prioritize correctness and test coverage over style nitpicks
- If changes are requested, push additional commits to the same branch rather than opening a new PR
- Once approved, a maintainer merges it — you don't need merge access yourself
- If your PR sits without review for longer than expected during an active Wave sprint, a polite comment tagging a maintainer is fine

---

## 10. Questions

Open a new issue with the `question` label, or comment directly on the issue you're working on if it's specific to that task.
# Cascade

**Automated, streaming funding for the open-source software Stellar ecosystem depends on.**

**Live demo:** [https://stellar-cascade-henna.vercel.app/](https://stellar-cascade-henna.vercel.app/)

> **Note on scope:** the sections below describe Cascade's long-term vision cascading, multi-level dependency funding with continuous streaming. What's actually built today is narrower: a **flat, single-level splits registry** with **discrete deposits into a claimable balance**. See [Current build stage](#current-build-stage--july-2026) for exactly what exists right now.

## Current build stage July 2026

**Status:** Early development flat splits registry deployed to testnet, indexer functional, frontend UI in progress

| Layer | Status | Details |
|---|---|---|
| **Contracts Splits Registry** | ✅ Testnet deployed | Live on testnet (deployed 2026-07-11). Flat, single-level splits only a project declares receivers and percentages directly, not a nested dependency tree. Project registration and split configuration complete, all tests passing. |
| **Contracts Deposits** | 📝 Scoped, not yet implemented | Deposit-into-balance and balance-query functions are fully scoped (17 issues) but not yet written or deployed. Until this lands, projects cannot actually receive funds on-chain. |
| **Contracts Claim/Withdraw** | 📋 Not yet scoped | The last piece of the register → deposit → claim loop. No design work started. |
| **Backend** | 🚧 Indexer functional, API minimal | Soroban event indexer running and decoding contract events into Postgres, with a resumable/idempotent sync worker. Database schema in place for projects, splits, balances, and sync state. API layer currently only exposes a health check; project/balance endpoints and transaction-builder endpoints are scoped but not yet implemented. |
| **Frontend** | 🚧 Landing page + register form UI built | Static landing page complete. A register-project form component exists (split entry, CSV upload, live validation) but is not yet wired to a wallet or to the backend. Dashboard, public project pages, and wallet integration are not yet implemented. |

**What works right now:**
- The registry contract can be invoked directly (CLI or manual transaction) to register a project and configure flat splits on testnet this is **not yet possible through the website**, since the frontend form isn't wired up
- The backend indexes registry events (registration, split updates) into Postgres in real time, with crash-safe resumability
- The landing page explains the concept and links to the repo

**What's next, roughly in order:**
- Implement and deploy the deposit contract functions, so projects can actually receive funds
- Implement the backend's remaining endpoints: unsigned transaction builders for register/update-splits, and read endpoints for project/balance data
- Wire the frontend register form to a real wallet (e.g. Freighter) and to the backend's transaction-builder endpoint
- Build a project view/dashboard page once the backend read endpoints exist
- Scope and build claim/withdraw
- Longer-term: nested/cascading dependency trees and continuous streaming the features described in the vision sections above are not yet designed at the contract level

## The problem

Every protocol, wallet, and app here is built on a stack of open-source libraries, SDKs, and tools most of it maintained by a small number of unpaid or underpaid contributors. Funding those maintainers today is manual and occasional: a grant application here, a one-off donation there, decided by a small group of reviewers on their own timeline. There is no default, automated way for a project's success to flow back down to the dependencies that made it possible. When a maintainer burns out or moves on, everything built on top of their work inherits that risk silently.

## What Cascade aims to do

*(Vision see [Current build stage](#current-build-stage--july-2026) above for what's actually implemented.)*

Cascade is a funding-splitting protocol built natively on Soroban. The long-term goal is for a project to register its dependency tree on-chain declaring what it is built on and how funding should be split across that tree so that whenever money flows in (protocol revenue, a grant, a donation) it cascades automatically down the tree in real time, denominated in USDC, XLM.

Planned core features:
- **On-chain dependency graphs** projects declare dependencies and split percentages transparently, and can update them as their stack evolves. *(Currently: flat splits only, one level, no dependency graph.)*
- **Continuous micro-streaming** funds move in small, constant streams rather than lump-sum payouts. *(Currently: not implemented the deposit model is a discrete deposit into a claimable balance, not a stream.)*
- **Cascading splits** if Project A depends on Project B, which depends on Project C, funding cascades through the whole tree automatically, not just one layer deep. *(Currently: not implemented splits are single-level.)*
- **Any funding source** works whether the incoming money is protocol trading fees, a foundation grant, or a one-time community donation
- **Public, verifiable flows** anyone can see exactly how funds are moving, so there is no ambiguity about who is getting paid for what

## How it's meant to work, eventually

1. A project registers on-chain and declares its dependency tree who it depends on, and what percentage of incoming funds each dependency should get
2. Funds arrive (protocol revenue, a grant, a donation) into the project's Cascade balance
3. Those funds stream automatically to dependencies according to the declared splits and if a dependency has its own dependencies, funding cascades further down the tree
4. Anyone in the tree can claim their accrued balance at any time

**Today**, only a version of steps 1 (flat splits, not a tree) and 2 (once the deposit contract phase lands) exist. Step 3's cascading and step 4's claim are future work.

## What this unlocks

- A sustainable, automated income stream for the maintainers every other project here quietly depends on
- Reduced maintainer burnout and turnover, meaning fewer silent points of failure across shared tooling
- A transparent, on-chain alternative to manual grant allocation that scales without needing a review committee to grow in proportion
- A flywheel: as more projects launch and route funding through Cascade, the dependencies they share get stronger, making the next project easier to build

## Repo structure three layers

This repo is organized as three independently-buildable layers, each with its own responsibilities:

```
cascade/
├── frontend/     Next.js + TypeScript the web app
├── backend/      Rust (Axum) off-chain indexing and API
└── contracts/    Rust (Soroban) on-chain logic and source of truth
```

### `contracts/` Soroban smart contracts (Rust)

The source of truth. All financial logic registering projects, storing split configurations, accepting deposits, and (eventually) letting recipients claim lives here and is enforced on-chain. Nothing in the backend or frontend can move funds; they can only read state and submit transactions a user has signed.

**Deployed contracts**

| Contract | Network | Status | README | Deployments |
|---|---|---|---|---|
| Splits Registry | Testnet | ✅ Deployed | [`contracts/registry/README.md`](contracts/registry/README.md) | [`contracts/registry/DEPLOYMENTS.md`](contracts/registry/DEPLOYMENTS.md) |

Deposit functions are scoped but live in the same `contracts/registry` crate as an upcoming addition, not yet deployed.

### `backend/` Rust service (Axum)

Sits between the contracts and the frontend. Current and planned responsibilities:
- **Indexing** on-chain events (registration, split updates, deposits once deployed) into a database, so the frontend doesn't need to query the chain directly for every page load **implemented and running**
- **Serving an API** the frontend will consume for project data and unsigned transaction building **only a health check exists today; the rest is scoped, not built**
- **Wallet-based authentication** (sign a challenge message to prove address ownership, no password database) **not yet started, design-stage only**

### `frontend/` Next.js + TypeScript app

The web interface: a landing page (built), a register-project form (built, not wired up), and not yet started a project dashboard, public project pages, and wallet connection.

## Running locally

### Prerequisites

```bash
# Rust + Soroban tooling
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none
cargo install --locked stellar-cli --features opt

# Node.js 18+ and npm (or pnpm)
node -v
npm -v
```

### 1. Clone the repo

```bash
git clone https://github.com/<your-username>/stellar-cascade.git
cd stellar-cascade
```

### 2. Contracts

```bash
cd contracts
cargo build --target wasm32v1-none --release
cargo test
```

The registry contract is already deployed to testnet see [`contracts/registry/DEPLOYMENTS.md`](contracts/registry/DEPLOYMENTS.md) for the live contract ID. To deploy your own instance instead of using the existing one:

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/<contract_name>.wasm \
  --source <your-identity> \
  --network testnet
```

### 3. Backend

```bash
cd backend
cp .env.example .env   # fill in RPC URL, database connection, etc.
cargo run
```

The API server will start on the port configured in `.env` (default `8080`). As of this writing, only `GET /health` is implemented.

### 4. Frontend

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:3000`. The landing page is fully functional; the register form is present in the UI but not yet connected to a wallet or the backend.

---

## Contributing

This repo is structured to have well-scoped, independently workable issues across all three layers at any given time. Check the Issues tab for current tasks each is labeled by layer (`frontend`, `backend`, `contracts`) and difficulty. The status table above reflects what's actually implemented as of this writing; the Issues tab reflects what's scoped and in progress.
# Cascade

**Automated, streaming funding for the open-source software this ecosystem depends on.**

## Current build stage — July 2026

**Status:** Early development — core contract deployed to testnet, indexer functional, frontend in progress

| Layer | Status | Details |
|---|---|---|
| **Contracts** | ✅ Testnet deployed | Registry contract live on testnet (deployed 2026-07-11). Core logic complete: project registration, split configuration, balance tracking. All integration tests passing. |
| **Backend** | 🚧 Indexer functional, API minimal | Soroban event indexer running and decoding contract events into Postgres. Database schema in place for projects, splits, balances, and sync state. API layer under development (currently only health endpoint). |
| **Frontend** | 🚧 Landing page complete | Static landing page built with all sections (hero, how it works, funding modes). Dashboard, project pages, and wallet integration not yet implemented. |

**What works right now:**
- Deploy a project to the registry contract on testnet and configure splits
- Backend indexes all on-chain events (project registration, split updates, deposits) in real time
- Landing page explains the concept and links to the repo

**What's next:**
- Complete backend API (project listing, dependency graph traversal, balance queries)
- Build frontend dashboard (view your project, see incoming funds, claim balance)
- Wallet connection and transaction signing flow
- Public project pages and dependency graph explorer

## The problem

Every protocol, wallet, and app here is built on a stack of open-source libraries, SDKs, and tools — most of it maintained by a small number of unpaid or underpaid contributors. Funding those maintainers today is manual and occasional: a grant application here, a one-off donation there, decided by a small group of reviewers on their own timeline. There is no default, automated way for a projects success to flow back down to the dependencies that made it possible. When a maintainer burns out or moves on, everything built on top of their work inherits that risk silently.

## What Cascade does

Cascade is a funding-splitting protocol built natively on Soroban. A project registers its dependency tree on-chain — declaring what it is built on and how funding should be split across that tree. From then on, whenever money flows in (protocol revenue, a grant, a donation) it cascades automatically down the tree in real time, denominated in USDC, XLM or EURC.

Core features:
- **On-chain dependency graphs** — projects declare dependencies and split percentages transparently, and can update them as their stack evolves
- **Continuous micro-streaming** — funds move in small, constant streams rather than lump-sum payouts, giving maintainers a predictable ongoing income instead of a single check
- **Cascading splits** — if Project A depends on Project B, which depends on Project C, funding cascades through the whole tree automatically, not just one layer deep
- **Any funding source** — works whether the incoming money is protocol trading fees, a foundation grant, or a one-time community donation
- **Public, verifiable flows** — anyone can see exactly how funds are moving through the dependency graph, so there is no ambiguity about who is getting paid for what

## How it works, briefly

1. A project registers on-chain and declares its dependency tree — who it depends on, and what percentage of incoming funds each dependency should get
2. Funds arrive (protocol revenue, a grant, a donation) into the project's Cascade balance
3. Those funds stream automatically to dependencies according to the declared splits — and if a dependency has its own dependencies, funding cascades further down the tree
4. Anyone in the tree can claim their accrued balance at any time

## What this unlocks

- A sustainable, automated income stream for the maintainers every other project here quietly depends on
- Reduced maintainer burnout and turnover, meaning fewer silent points of failure across shared tooling
- A transparent, on-chain alternative to manual grant allocation that scales without needing a review committee to grow in proportion
- A flywheel: as more projects launch and route funding through Cascade, the dependencies they share get stronger, making the next project easier to build

## Repo structure — three layers

This repo is organized as three independently-buildable layers, each with its own responsibilities:
cascade/
├── frontend/     Next.js + TypeScript — the web app
├── backend/      Rust (Axum) — off-chain indexing and API
└── contracts/    Rust (Soroban) — on-chain logic and source of truth



### `contracts/` — Soroban smart contracts (Rust)

The source of truth. All financial logic — registering projects, storing split configurations, streaming/accruing funds, and letting recipients claim — lives here and is enforced on-chain. Nothing in the backend or frontend can move funds; they can only read state and submit transactions a user has signed.

**Deployed contracts**

| Contract | Network | README | Deployments |
|---|---|---|---|
| Splits Registry | Testnet | [`contracts/registry/README.md`](contracts/registry/README.md) | [`contracts/registry/DEPLOYMENTS.md`](contracts/registry/DEPLOYMENTS.md) |

### `backend/` — Rust service (Axum)

Sits between the contracts and the frontend. Responsibilities:
- **Indexing** on-chain events (new projects, split updates, deposits, claims) into a database so the frontend doesn't need to query the chain directly for every page load
- **Serving an API** the frontend consumes for project data, graph traversal, and funding history
- **Handling wallet-based authentication** — a user signs a challenge message to prove address ownership; there's no password database

### `frontend/` — Next.js + TypeScript app

The web interface: a landing page, a project dashboard (splits, incoming funds, claim button), a dependency graph explorer, and public project pages viewable without connecting a wallet.



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

To deploy to testnet (once contracts exist):
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

The API server will start on the port configured in `.env` (default `8080`).

### 4. Frontend

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:3000`.

---

## Contributing

This repo is structured to have well-scoped, independently workable issues across all three layers at any given time. Check the Issues tab for current tasks — each is labeled by layer (`frontend`, `backend`, `contracts`) and difficulty.


# Cascade — Architecture & Build Plan

**Status:** Planning
**Target:** Drips Wave submission (Stellar ecosystem)
**Stack:** Soroban (Rust) contracts · Rust (Axum) backend · Next.js/TypeScript frontend

---

## 1. Objective

Cascade is an automated funding-splitting protocol. A project registers its dependency tree on-chain; whenever funds arrive (revenue, grants, donations), they cascade down that tree automatically as continuous micro-streams in USDC/EURC, rather than requiring a manual, one-off payout decision.

**Primary goal:** give open-source maintainers a sustainable, automated income stream instead of relying on occasional manual grants.

**Secondary goal:** be a credible, well-scoped, issue-rich repository suitable for Drips Wave — meaning the architecture needs to produce a steady pipeline of concrete, independently workable issues across contracts, backend, and frontend.

---

## 2. Core Concepts

| Concept | Description |
|---|---|
| **Project** | An entity (person, team, or protocol) registered in Cascade, identified by a Stellar address |
| **Dependency Graph** | A tree/DAG describing which projects a given project depends on, and what % of incoming funds each dependency receives |
| **Split** | A single edge in the graph: "Project A sends X% of incoming funds to Project B" |
| **Stream** | A continuous, time-based transfer of funds from one party to another (as opposed to a lump-sum transfer) |
| **Funding Source** | Where incoming money originates — could be external revenue, a grant deposit, or a direct donation into a project's Cascade balance |
| **Claim** | The act of a recipient withdrawing their accumulated streamed balance |

---

## 3. High-Level System Architecture

```
                     ┌─────────────────────┐
                     │      Frontend        │
                     │  (Next.js / TS)       │
                     │  - Graph explorer     │
                     │  - Funding dashboard  │
                     │  - Wallet connect     │
                     └──────────┬───────────┘
                                │ REST/GraphQL
                     ┌──────────▼───────────┐
                     │       Backend         │
                     │   (Rust / Axum)       │
                     │  - Indexer            │
                     │  - API server         │
                     │  - Auth (wallet-based)│
                     └──────────┬───────────┘
                                │ soroban-client (RPC)
                     ┌──────────▼───────────┐
                     │      Contracts        │
                     │      (Soroban)        │
                     │  - Project Registry   │
                     │  - Split Graph        │
                     │  - Streaming Vault    │
                     │  - Claim Logic        │
                     └───────────────────────┘
```

**Design principle:** contracts hold the source of truth and enforce all financial logic. The backend never moves funds directly — it only reads on-chain state (via indexing) and submits transactions that users have signed. The frontend never talks to contracts directly for writes; it goes through the backend for a consistent API surface, but can read directly from Soroban RPC for live data if that proves simpler.

---

## 4. Contracts Layer (`/contracts`)

Four separate contract crates, mirroring how mature Soroban protocols separate concerns:

### 4.1 `project-registry`
- Registers a project: address, name, metadata URI (off-chain JSON with description, links, etc.)
- Stores ownership (who can modify this project's splits)
- Emits `ProjectRegistered` event for the indexer

### 4.2 `split-graph`
- Stores the dependency graph: for each project, a list of `(recipient_address, percentage)` pairs
- Enforces percentages sum to ≤100%
- Allows the project owner to update splits (with a timelock or event log, so changes are auditable — funders should be able to see a project didn't rug its dependencies right before a big payout)
- Emits `SplitUpdated` events

### 4.3 `streaming-vault`
- Holds deposited funds (USDC/EURC) for a project
- Streams funds to dependents continuously based on the split graph — practically, this usually means **lazy evaluation**: rather than pushing a transaction every second, the contract calculates "how much has accrued since last claim" whenever someone claims or checks a balance
- This is the most technically sensitive contract — model it closely on Soroban's existing token-streaming patterns rather than inventing streaming math from scratch

### 4.4 `claim`
- Lets any address in the split graph pull their accrued balance
- Pull-based, not push-based (avoids gas/griefing issues if the graph has many recipients)
- Could be merged into `streaming-vault` rather than being fully separate — decide once you're implementing, don't over-separate prematurely

---

## 5. Backend Layer (`/backend`)

Built in Rust with Axum. Responsibilities:

- **Indexer**: subscribes to contract events (`ProjectRegistered`, `SplitUpdated`, funds deposited/claimed) and writes them into a local database (Postgres is the standard choice) so the frontend doesn't need to query the chain directly for every page load
- **API server**: exposes REST (or GraphQL) endpoints for the frontend — project lookup, graph traversal, funding history
- **Auth**: wallet-based — user signs a challenge message with their Stellar wallet (via passkey or Freighter-style extension) to prove address ownership; no passwords, no email/password DB to secure
- **Notification/webhook layer** (later phase): notify a project owner when their split graph changes or a large funding event occurs

---

## 6. Frontend Layer (`/frontend`)

Next.js + TypeScript. Key screens:

1. **Landing page** — what Cascade is, call to action to connect wallet and register a project
2. **Project dashboard** — for a project owner: current splits, incoming funds, balance, claim button
3. **Dependency graph explorer** — visual graph (consider a library like `react-flow` or `d3`) showing how funds cascade through registered projects
4. **Public project page** — anyone can view a project's splits and funding history without connecting a wallet
5. **Wallet connect flow** — passkey-based, per Stellar's native smart wallet support

---

## 7. Data Model (indexed database, not on-chain)

```
projects
  - address (pk)
  - name
  - metadata_uri
  - owner_address
  - created_at

splits
  - id (pk)
  - project_address (fk)
  - recipient_address
  - percentage
  - updated_at

funding_events
  - id (pk)
  - project_address (fk)
  - amount
  - asset (USDC/EURC)
  - source (revenue/grant/donation)
  - tx_hash
  - created_at

claims
  - id (pk)
  - recipient_address
  - amount
  - tx_hash
  - created_at
```

---

## 8. Build Roadmap — One Piece at a Time

This is ordered so each step is independently shippable and demoable, and so you always have a working (if incomplete) product rather than a pile of disconnected parts.

### Phase 0 — Foundation
- [ ] Repo scaffold (already done)
- [ ] CI/CD: lint + build checks for all three layers
- [ ] Deploy a "hello world" Soroban contract to testnet, confirm the toolchain works end to end

### Phase 1 — Static UI (no backend, no contracts)
- [ ] Landing page (what Cascade is, static content)
- [ ] Project dashboard UI with mock/hardcoded data
- [ ] Dependency graph explorer UI with mock data
- [ ] Basic responsive layout, design system decisions (colors, typography)

### Phase 2 — Authentication
- [ ] Wallet connect flow (passkey-based, using Stellar's native smart wallet support)
- [ ] Sign-in-with-wallet challenge/response on the backend
- [ ] Session handling (JWT or similar, tied to a verified wallet address)
- [ ] "Connected wallet" state reflected across frontend

### Phase 3 — Project Registry (contracts + integration)
- [ ] Write and test `project-registry` contract
- [ ] Deploy to testnet
- [ ] Backend: read registry state, expose `/projects` API
- [ ] Frontend: "Register a project" flow, wired to the real contract

### Phase 4 — Split Graph
- [ ] Write and test `split-graph` contract
- [ ] Backend: index `SplitUpdated` events
- [ ] Frontend: let a project owner define/edit splits, visualize the graph with real data

### Phase 5 — Streaming Vault
- [ ] Write and test `streaming-vault` contract (accrual math, deposits)
- [ ] Backend: index funding/deposit events
- [ ] Frontend: funding dashboard showing real incoming funds and accrual over time

### Phase 6 — Claims
- [ ] Write and test `claim` logic
- [ ] Backend: index claim events, expose claim history
- [ ] Frontend: claim button + transaction flow, balance history

### Phase 7 — Polish & Wave Readiness
- [ ] Public project pages (no wallet required to view)
- [ ] Error states, loading states, edge cases (zero splits, expired sessions, etc.)
- [ ] Write comprehensive tests across all three layers
- [ ] Populate the repo with scoped GitHub issues across frontend/backend/contracts for Wave contributors
- [ ] Documentation pass (README, CONTRIBUTING.md, issue templates)

---

## 9. What "done" looks like for a Drips Wave submission

- All three layers build and pass CI
- At least the core loop works end-to-end on testnet: register a project → define splits → deposit funds → dependents claim
- A healthy backlog of well-scoped, independently workable issues exists across all three folders, so contributors have something concrete to pick up in a one-week sprint
- README and this architecture doc are both current and accurate

---

## 10. Open Design Questions (revisit before Phase 3)

- Should split percentage changes be timelocked, or just logged transparently? (Affects trust — funders may not want dependencies to change unexpectedly)
- Single funding asset at launch (USDC only) vs. multi-asset from day one? Recommend starting single-asset to reduce contract complexity, expand later
- Should unclaimed streamed funds accrue interest/yield while sitting in the vault (e.g., via Blend), or stay idle? Worth deferring past MVP

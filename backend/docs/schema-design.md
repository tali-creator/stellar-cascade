# Cascade Backend — DB Schema Design

**Status:** Approved design — migrations to follow in B6–B10.  
**Last updated:** 2026-07-11

---

## Design principles

1. **Event-sourced sync, not just a cache.** The sync worker maintains a durable
   ledger cursor and an append-only event log (`sync_cursor`, `sync_events`).
   This makes the worker resumable after a crash with no full resync, and gives
   a built-in audit trail for free.

2. **Splits as a proper child table, not JSON.** Each `(project_id, receiver_address,
   percentage_bps)` triple is its own row in `splits`. This keeps splits fully
   queryable — "which projects pay this receiver?" is a straight indexed lookup,
   not a JSONB scan.

3. **Balances keyed by `(project_id, token_address)`.** Mirrors the planned
   `DataKey::Balance(project_id, token_address)` composite key in the deposit-phase
   contract. Multi-token from day one; no retrofitting needed when EURC/XLM
   support lands alongside USDC.

4. **Schema doesn't assume flat-only forever.** `projects.parent_project_id` is a
   nullable self-referential FK, unused today, indexed at zero cost. When the
   dependency-tree phase arrives it won't require a breaking migration.

5. **`is_locked` included but unused.** Matches the immutability-toggle placeholder
   already in the frontend register form. Backend and frontend agree on the field
   name from the start.

---

## Tables

### `projects`

Stores one row per registered on-chain project. Updated by the sync worker
whenever a `RegisterProject` or `UpdateSplits` event is indexed.

```
projects
├── id                  TEXT        PRIMARY KEY
│                                   On-chain type: BytesN<32> (32-byte project
│                                   identifier). Stored as lowercase hex string
│                                   (64 chars) rather than BYTEA so it renders
│                                   correctly in psql, logs, and API responses
│                                   without manual encoding. The registry
│                                   contract treats IDs as opaque bytes; hex is
│                                   a lossless, human-readable encoding.
│
├── owner_address       TEXT        NOT NULL
│                                   Stellar G... address of the project owner.
│                                   Stored as the standard base32-encoded
│                                   strkey (56 chars). Never NULL — every
│                                   registered project must have an owner.
│
├── name                TEXT        NULLABLE
│                                   Off-chain display label. Not part of the
│                                   contract; supplied by the project owner
│                                   via the frontend at registration time and
│                                   stored only in the backend DB.
│
├── is_locked           BOOLEAN     NOT NULL  DEFAULT false
│                                   Placeholder for the contract's future
│                                   immutability toggle. Unused until the lock
│                                   feature is implemented. Included now so
│                                   frontend, backend, and contract agree on
│                                   the field name before it matters.
│
├── parent_project_id   TEXT        NULLABLE
│                                   Self-referential FK → projects(id).
│                                   Unused placeholder for future dependency-tree
│                                   support (the "cascading splits" graph).
│                                   Indexed (see below). Zero cost if never
│                                   populated; avoids a breaking migration when
│                                   the dependency phase arrives.
│
├── last_synced_ledger  BIGINT      NOT NULL  DEFAULT 0
│                                   Ledger sequence at which this row was last
│                                   updated by the sync worker. Lets the API
│                                   surface staleness ("last seen at ledger N")
│                                   and lets ops quickly spot stale projects
│                                   without joining to sync_cursor.
│
├── created_at          TIMESTAMPTZ NOT NULL  DEFAULT now()
└── updated_at          TIMESTAMPTZ NOT NULL  DEFAULT now()
```

**Indexes:**

| Index | Columns | Type | Reason |
|---|---|---|---|
| `projects_pkey` | `id` | UNIQUE (implicit PK) | Lookups by project ID |
| `projects_owner_address_idx` | `owner_address` | BTREE | "Projects owned by address X" — dashboard query |
| `projects_parent_project_id_idx` | `parent_project_id` | BTREE | Future dependency-tree traversal; partial index on `IS NOT NULL` keeps it tiny while unused |

---

### `splits`

One row per `(project, receiver)` pair. Replaced in bulk on every
`UpdateSplits` event — delete-where-project + insert is the canonical update
pattern (matches how the contract models a full receiver-list replacement).

```
splits
├── id                  BIGSERIAL   PRIMARY KEY
│
├── project_id          TEXT        NOT NULL  REFERENCES projects(id) ON DELETE CASCADE
│                                   FK to the owning project. CASCADE delete
│                                   means removing a project automatically
│                                   cleans up its splits — no orphan rows.
│
├── receiver_address    TEXT        NOT NULL
│                                   Stellar G... address of the receiver.
│
├── percentage_bps      INTEGER     NOT NULL
│                                   Share in basis points. Contract invariant:
│                                   all rows for a given project_id sum to
│                                   exactly 10 000 (= 100.00%). Stored as
│                                   INTEGER not NUMERIC because basis points are
│                                   always whole numbers and INTEGER arithmetic
│                                   is exact.
│                                   CHECK (percentage_bps > 0 AND percentage_bps <= 10000)
│
└── position            SMALLINT    NOT NULL
                                    Zero-based index preserving the order in
                                    which receivers were declared on-chain.
                                    Ensures consistent display order on the
                                    frontend without relying on insertion order.
```

**Constraints:**

| Constraint | Definition | Reason |
|---|---|---|
| `splits_project_receiver_uniq` | UNIQUE `(project_id, receiver_address)` | Mirrors contract validation: a receiver may appear at most once per project |
| `splits_percentage_bps_check` | CHECK `(percentage_bps > 0 AND percentage_bps <= 10000)` | Rejects zero-share and over-100% rows at the DB layer |

**Indexes:**

| Index | Columns | Reason |
|---|---|---|
| `splits_pkey` | `id` | Row-level identity |
| `splits_project_id_position_idx` | `(project_id, position)` | Ordered fetch of all receivers for a project |
| `splits_receiver_address_idx` | `receiver_address` | "Which projects pay receiver X?" — reverse lookup |

---

### `balances`

One row per `(project, token)` pair, updated on every `Deposit` event.
Multi-token from the start — USDC, XLM, and EURC are all first-class.

```
balances
├── project_id          TEXT        NOT NULL  REFERENCES projects(id) ON DELETE CASCADE
├── token_address       TEXT        NOT NULL
│                                   Stellar contract address (C... strkey) of
│                                   the token. Using the on-chain contract
│                                   address rather than a ticker symbol avoids
│                                   ambiguity across networks (testnet USDC ≠
│                                   mainnet USDC).
│
├── amount              NUMERIC(39,0)  NOT NULL  DEFAULT 0
│                                   Balance in the token's smallest unit
│                                   (stroops for XLM, micro-USDC for USDC, etc).
│                                   The deposit-phase contract uses i128 for
│                                   amounts. Postgres BIGINT is only 64 bits
│                                   (max ~9.2 × 10¹⁸); i128 reaches ~1.7 × 10³⁸.
│                                   NUMERIC(39,0) safely covers the full i128
│                                   range (38 decimal digits) with one digit of
│                                   headroom, and has exact integer arithmetic.
│
└── updated_at          TIMESTAMPTZ NOT NULL  DEFAULT now()
```

**Primary key:** `(project_id, token_address)` — composite, mirrors
`DataKey::Balance(project_id, token_address)` from the deposit contract.

**Indexes:**

| Index | Columns | Reason |
|---|---|---|
| `balances_pkey` | `(project_id, token_address)` | Direct lookup by project + token |
| `balances_project_id_idx` | `project_id` | "All token balances for project X" |

---

### `sync_cursor`

Single-row table tracking how far the sync worker has processed the chain.
Used at startup to resume from the last processed ledger rather than re-indexing
from genesis.

```
sync_cursor
├── id                       SMALLINT    PRIMARY KEY  DEFAULT 1
│                                        Enforced as a single-row table via
│                                        CHECK (id = 1). If multi-contract
│                                        tracking is ever needed, the PK
│                                        becomes contract_address TEXT and this
│                                        constraint is dropped.
│
├── last_processed_ledger    BIGINT      NOT NULL  DEFAULT 0
│                                        Sequence number of the last ledger
│                                        fully processed by the sync worker.
│                                        The worker resumes from
│                                        last_processed_ledger + 1 on startup.
│                                        0 means "never synced".
│
└── updated_at               TIMESTAMPTZ NOT NULL  DEFAULT now()
```

**Constraint:** `CHECK (id = 1)` — enforces single-row invariant at the DB
layer. An application-level upsert on `id = 1` is the only write pattern needed.

---

### `sync_events`

Append-only log of every on-chain event processed by the sync worker.
Serves three purposes:

1. **Audit trail** — immutable record of what happened and when it was indexed.
2. **Replay / debug** — re-derive derived state (`projects`, `splits`,
   `balances`) from raw events without touching the chain again.
3. **Incident investigation** — correlate a bad derived state with the exact
   raw event that caused it.

Rows are never updated or deleted. Application code must only ever INSERT.

```
sync_events
├── id                  BIGSERIAL   PRIMARY KEY
│
├── ledger_sequence     BIGINT      NOT NULL
│                                   The ledger in which this event was emitted.
│                                   Used for replay ordering and for advancing
│                                   sync_cursor after a batch commit.
│
├── tx_hash             TEXT        NOT NULL
│                                   Transaction hash (hex). Ties the event back
│                                   to the on-chain transaction for explorer
│                                   links and cross-referencing.
│
├── event_type          TEXT        NOT NULL
│                                   Discriminator matching the contract's
│                                   emitted event topics. Expected values:
│                                   "register_project", "update_splits",
│                                   "deposit". Stored as TEXT (not an ENUM) so
│                                   adding a new event type doesn't require an
│                                   ALTER TYPE migration.
│
├── raw_data            JSONB       NOT NULL
│                                   Full decoded event payload. Schema varies
│                                   by event_type — see event payload shapes
│                                   below. JSONB over JSON: indexed, binary
│                                   storage, supports GIN for ad-hoc queries.
│
└── processed_at        TIMESTAMPTZ NOT NULL  DEFAULT now()
```

**Indexes:**

| Index | Columns | Type | Reason |
|---|---|---|---|
| `sync_events_pkey` | `id` | BTREE | Sequential scan / cursor pagination |
| `sync_events_ledger_sequence_idx` | `ledger_sequence` | BTREE | Replay from a given ledger; correlate with sync_cursor |
| `sync_events_event_type_idx` | `event_type` | BTREE | Filter by event type during replay |
| `sync_events_raw_data_gin_idx` | `raw_data` | GIN | Ad-hoc JSONB queries during debugging (e.g. find all events for a project_id) |

**Event payload shapes (`raw_data`):**

```jsonc
// register_project
{
  "project_id": "<hex>",
  "owner": "G...",
  "receivers": [
    { "address": "G...", "percentage_bps": 5000 }
  ]
}

// update_splits
{
  "project_id": "<hex>",
  "receivers": [
    { "address": "G...", "percentage_bps": 5000 }
  ]
}

// deposit
{
  "project_id": "<hex>",
  "token":  "C...",
  "amount": "1000000"   // stringified i128 to avoid JSON number precision loss
}
```

---

## Entity-relationship diagram

```
projects ──────────────────────────────────────────────────────┐
  │ id (PK)                                                     │ (self-ref,
  │                                                             │  nullable)
  ├──< splits                                                   │
  │      project_id (FK)                                        │
  │      receiver_address                                       │
  │      percentage_bps                                         │
  │      position                                               │
  │                                                             │
  └──< balances               parent_project_id ───────────────┘
         project_id (FK)
         token_address
         amount (NUMERIC 39,0)

sync_cursor   (single row)
  last_processed_ledger

sync_events   (append-only)
  ledger_sequence
  tx_hash
  event_type
  raw_data (JSONB)
```

---

## Type decisions summary

| Value | Contract type | SQL type | Rationale |
|---|---|---|---|
| Project ID | `BytesN<32>` | `TEXT` (64-char hex) | Human-readable in logs/API; lossless encoding of 32 raw bytes |
| Address | `Address` | `TEXT` (56-char G... strkey) | Standard Stellar encoding; stable across SDK versions |
| Percentage | `u32` (bps) | `INTEGER` | Whole-number bps, fits comfortably in 32-bit int |
| Balance/amount | `i128` | `NUMERIC(39,0)` | i128 max ≈ 1.7×10³⁸; BIGINT only covers 9.2×10¹⁸ — would silently overflow |
| Token address | `Address` | `TEXT` (C... strkey) | Contract address; same reasoning as wallet address |
| Ledger sequence | `u32` on-chain | `BIGINT` | Headroom for sequence numbers growing past 2³¹; BIGINT is 64-bit |

---

## Open questions (to resolve before B6)

- [ ] **`name` field provenance** — is project name stored only in the backend
  DB (off-chain metadata supplied at registration via the frontend), or does the
  contract emit it? Current contract has no name field. Assumption: backend-only.
  Confirm before writing the migration.

- [ ] **`sync_cursor` single-row vs. contract-keyed** — today there is one
  contract (`registry`). The single-row design with `CHECK (id = 1)` is correct
  now. If a deposit contract is added later, promote `id` to `contract_address
  TEXT PRIMARY KEY` and drop the check constraint. No data migration needed,
  just an `ALTER TABLE`.

- [ ] **GIN index on `sync_events.raw_data`** — useful for debugging but adds
  write overhead on every event insert. Consider deferring this index until
  there is a concrete query that needs it.

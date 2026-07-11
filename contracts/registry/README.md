# Splits Registry Contract

This document covers the flat-registry MVP — the first and currently deployed
phase of the Cascade contracts layer.  See the [scope boundary](#scope-boundary)
section at the bottom before assuming any functionality exists beyond what is
described here.

---

## Overview

The Splits Registry is a Soroban smart contract that lets projects declare a
fixed, on-chain list of funding recipients and the percentage share each one
receives.  Once registered, the split configuration can be updated by the
project owner at any time, and the full record is readable by anyone without
authorization.

All financial logic (streaming, claiming) lives in future contract phases.
This contract only stores and validates split configurations.

---

## Flat-splits model

A project registers a **flat** list of receivers.  Each receiver has:

- an `address` — a Stellar `Address` (account or contract)
- a `percentage` — a share expressed in **basis points** (bp), where
  `10 000 bp = 100.00%`

The receiver list must satisfy three rules at registration and on every
update:

1. Between 1 and 20 receivers (inclusive).
2. All `percentage` values sum to exactly `10 000` bp.
3. No address appears more than once.

**There is no support for nested dependency trees or multi-level cascading
splits in this phase.**  A project can declare its direct receivers, but those
receivers cannot automatically re-split incoming funds further down a tree.
That behavior is deferred to a later phase.

---

## Testnet deployment

See [`DEPLOYMENTS.md`](./DEPLOYMENTS.md) for the current contract ID, deployer
address, and deployment date.

---

## Public functions

### `register_project(owner, id, receivers)`

Register a new project in persistent storage.

| Parameter   | Type            | Description                                              |
|-------------|-----------------|----------------------------------------------------------|
| `owner`     | `Address`       | The project's controlling address.  **Requires authorization** — the transaction must be signed by this address. |
| `id`        | `BytesN<32>`    | A 32-byte unique identifier for the project.             |
| `receivers` | `Vec<Receiver>` | The initial split configuration (1–20 entries, summing to 10 000 bp, no duplicate addresses). |

**Authorization required:** yes — `owner.require_auth()` is called before any
other logic.

**Returns:** `Result<(), RegistryError>`

**Errors:**
- `ProjectAlreadyExists (6)` — `id` is already registered.  Use
  `update_splits` to change an existing project.
- `TooFewReceivers (3)` — fewer than 1 receiver supplied.
- `TooManyReceivers (4)` — more than 20 receivers supplied.
- `InvalidPercentageSum (1)` — percentages do not sum to exactly 10 000 bp.
- `DuplicateReceiver (2)` — the same address appears more than once.

**Events emitted on success:**
```
topics : ("register", project_id: BytesN<32>)
data   : (owner: Address, receiver_count: u32)
```

---

### `update_splits(id, new_receivers)`

Replace the receiver list for an existing project.

| Parameter       | Type            | Description                                              |
|-----------------|-----------------|----------------------------------------------------------|
| `id`            | `BytesN<32>`    | The project to update.                                   |
| `new_receivers` | `Vec<Receiver>` | The replacement split configuration (same rules as registration). |

**Authorization required:** yes — the transaction must be signed by the address
recorded as `owner` at registration time.  The owner address is read from
storage; there is no caller-supplied owner parameter to prevent privilege
escalation.

**Returns:** `Result<(), RegistryError>`

**Errors:**
- `ProjectNotFound (5)` — `id` is not registered.
- `TooFewReceivers (3)`, `TooManyReceivers (4)`, `InvalidPercentageSum (1)`,
  `DuplicateReceiver (2)` — same validation rules as `register_project`.

**Events emitted on success:**
```
topics : ("update", project_id: BytesN<32>)
data   : (new_receiver_count: u32)
```

---

### `get_project(id)`

Return the full `Project` record for a registered project, or `None` if the
ID is not found.

| Parameter | Type         | Description              |
|-----------|--------------|--------------------------|
| `id`      | `BytesN<32>` | The project to look up.  |

**Authorization required:** no — splits are intentionally public.

**Returns:** `Option<Project>`

---

### `has_project(id)`

Lightweight existence check.  Returns `true` if a project with the given `id`
is registered, `false` otherwise.  Prefer this over `get_project` when you
only need to know whether a project exists, as it avoids deserializing the
full record.

| Parameter | Type         | Description              |
|-----------|--------------|--------------------------|
| `id`      | `BytesN<32>` | The project to check.    |

**Authorization required:** no.

**Returns:** `bool`

---

## Error codes

| Variant                 | Code | Meaning                                              |
|-------------------------|------|------------------------------------------------------|
| `InvalidPercentageSum`  | 1    | Receiver percentages do not sum to 10 000 bp.        |
| `DuplicateReceiver`     | 2    | The same address appears more than once.             |
| `TooFewReceivers`       | 3    | Fewer than 1 receiver in the list.                   |
| `TooManyReceivers`      | 4    | More than 20 receivers in the list.                  |
| `ProjectNotFound`       | 5    | The target project ID is not registered.             |
| `ProjectAlreadyExists`  | 6    | The project ID is already registered.                |

---

## Data types

```rust
pub struct Receiver {
    pub address: Address,
    pub percentage: u32,   // basis points: 10_000 = 100.00%
}

pub struct Project {
    pub id: BytesN<32>,
    pub owner: Address,
    pub receivers: Vec<Receiver>,
}
```

---

## Example: invoking `register_project` via the Stellar CLI

Replace `<CONTRACT_ID>` with the testnet contract ID from
[`DEPLOYMENTS.md`](./DEPLOYMENTS.md), and substitute real testnet addresses
for the placeholders.

```bash
# Ensure your identity is set up and funded (one-time)
stellar keys generate my-identity --network testnet --fund

# Register a project with two receivers
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-identity \
  --network testnet \
  -- \
  register_project \
  --owner GOWNER_ADDRESS_HERE \
  --id 0101010101010101010101010101010101010101010101010101010101010101 \
  --receivers '[
    {"address": "GRECEIVER_A_ADDRESS", "percentage": 7000},
    {"address": "GRECEIVER_B_ADDRESS", "percentage": 3000}
  ]'
```

The `--id` flag for `register_project` expects a 64-character hex string
representing the 32-byte project identifier.  Choose something meaningful to
your project (e.g. a hash of the package name and version).

To verify the registration:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-identity \
  --network testnet \
  -- \
  get_project \
  --id 0101010101010101010101010101010101010101010101010101010101010101
```

---

## Scope boundary

This is the **flat-registry MVP**.  The following are explicitly **not** built
in this phase and should not be assumed to exist:

- Nested dependency trees — a receiver cannot itself have registered splits
  that auto-cascade further.  Each project has one flat list of direct
  receivers.
- Multi-level fund streaming or cascading payouts.
- Deposit or claim functions — no funds move through this contract.

These features are planned for future phases.  Do not document or rely on
them as if they are present.

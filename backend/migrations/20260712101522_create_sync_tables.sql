-- Migration: create_sync_tables
-- Implements sync_cursor and sync_events tables as designed in
-- backend/docs/schema-design.md. These tables provide resumability and
-- auditability for the event listener (next roadmap phase).

-- ---------------------------------------------------------------------------
-- sync_cursor: single-row table tracking how far the sync worker has
-- processed the chain. Used at startup to resume from the last processed
-- ledger rather than re-indexing from genesis.
-- ---------------------------------------------------------------------------

CREATE TABLE sync_cursor (
    -- Enforced as a single-row table via CHECK constraint below.
    -- If multi-contract tracking is ever needed, promote this to
    -- contract_address TEXT PRIMARY KEY and drop the check.
    id                       SMALLINT    NOT NULL DEFAULT 1,

    -- Sequence number of the last ledger fully processed by the sync worker.
    -- The worker resumes from last_processed_ledger + 1 on startup.
    -- 0 means "never synced".
    last_processed_ledger    BIGINT      NOT NULL DEFAULT 0,

    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT sync_cursor_pkey PRIMARY KEY (id),

    -- Single-row invariant: only id=1 is allowed.
    CONSTRAINT sync_cursor_single_row CHECK (id = 1)
);

-- Seed the single row immediately so application code can always UPDATE
-- rather than needing upsert-or-insert logic.
INSERT INTO sync_cursor (id, last_processed_ledger, updated_at)
VALUES (1, 0, now());

-- ---------------------------------------------------------------------------
-- sync_events: append-only log of every on-chain event processed by the
-- sync worker. Serves as audit trail, replay/debug source, and incident
-- investigation tool. Rows are never updated or deleted — only INSERTs.
-- ---------------------------------------------------------------------------

CREATE TABLE sync_events (
    id                  BIGSERIAL       NOT NULL,

    -- The ledger in which this event was emitted. Used for replay ordering
    -- and for advancing sync_cursor after a batch commit.
    ledger_sequence     BIGINT          NOT NULL,

    -- Transaction hash (hex). Ties the event back to the on-chain transaction
    -- for explorer links and cross-referencing.
    tx_hash             TEXT            NOT NULL,

    -- Discriminator matching the contract's emitted event topics.
    -- Expected values: "register_project", "update_splits", "deposit".
    -- TEXT (not ENUM) so adding a new event type doesn't require ALTER TYPE.
    event_type          TEXT            NOT NULL,

    -- Full decoded event payload. Schema varies by event_type.
    -- JSONB over JSON: indexed, binary storage, supports GIN for ad-hoc queries.
    raw_data            JSONB           NOT NULL,

    processed_at        TIMESTAMPTZ     NOT NULL DEFAULT now(),

    CONSTRAINT sync_events_pkey PRIMARY KEY (id)
);

-- Sequential scan / cursor pagination.
-- Redundant with PK but makes intent explicit.
CREATE INDEX sync_events_id_idx ON sync_events (id);

-- Replay from a given ledger; correlate with sync_cursor.
CREATE INDEX sync_events_ledger_sequence_idx ON sync_events (ledger_sequence);

-- Filter by event type during replay or debugging.
CREATE INDEX sync_events_event_type_idx ON sync_events (event_type);

-- Ad-hoc JSONB queries during debugging (e.g. find all events for a project_id).
-- GIN index adds write overhead on every event insert, but the query flexibility
-- during incident investigation is worth it.
CREATE INDEX sync_events_raw_data_gin_idx ON sync_events USING gin (raw_data);

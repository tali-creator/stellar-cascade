-- Migration: create_splits_table
-- Implements the splits table exactly as designed in backend/docs/schema-design.md.
-- One row per (project, receiver) pair. Replaced in bulk on UpdateSplits events.

CREATE TABLE splits (
    -- Auto-incrementing row identity. Not used for any semantic purposes;
    -- exists purely so there's a stable PK for any future audit trail.
    id                  BIGSERIAL   NOT NULL,

    -- FK to the owning project. CASCADE delete means removing a project
    -- automatically cleans up its splits — no orphan rows.
    -- In practice projects are rarely hard-deleted, but this is a safety net.
    project_id          TEXT        NOT NULL,

    -- Stellar G... address (56-char base32 strkey) of the receiver.
    receiver_address    TEXT        NOT NULL,

    -- Share in basis points (1 bp = 0.01%). Contract invariant: all rows for
    -- a given project_id sum to exactly 10,000 (= 100.00%).
    -- INTEGER not NUMERIC because bps are always whole numbers and INTEGER
    -- arithmetic is exact and fast.
    percentage_bps      INTEGER     NOT NULL,

    -- Zero-based index preserving the order in which receivers were declared
    -- on-chain. Ensures consistent display order on the frontend without
    -- relying on insertion order or undefined ordering.
    position            SMALLINT    NOT NULL,

    CONSTRAINT splits_pkey PRIMARY KEY (id),

    CONSTRAINT splits_project_fk
        FOREIGN KEY (project_id) REFERENCES projects (id)
        ON DELETE CASCADE,

    -- Mirrors contract validation: a receiver may appear at most once per project.
    CONSTRAINT splits_project_receiver_uniq
        UNIQUE (project_id, receiver_address),

    -- Rejects zero-share and over-100% rows at the DB layer.
    -- The contract already enforces this, but defense in depth.
    CONSTRAINT splits_percentage_bps_check
        CHECK (percentage_bps > 0 AND percentage_bps <= 10000)
);

-- Ordered fetch of all receivers for a project, respecting their declared order.
-- Used when rendering the project's split configuration on the frontend.
CREATE INDEX splits_project_id_position_idx ON splits (project_id, position);

-- Reverse lookup: "Which projects pay receiver X?"
-- Used for a receiver's dashboard showing all funding sources.
CREATE INDEX splits_receiver_address_idx ON splits (receiver_address);

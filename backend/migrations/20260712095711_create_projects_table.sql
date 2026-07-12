-- Migration: create_projects_table
-- Implements the projects table exactly as designed in backend/docs/schema-design.md.

CREATE TABLE projects (
    -- On-chain identifier: BytesN<32> stored as a 64-character lowercase hex string.
    -- Chosen over BYTEA for human-readability in logs, psql output, and API responses
    -- without any encoding gymnastics. The encoding is lossless.
    id                  TEXT        NOT NULL,

    -- Stellar G... address (56-char base32 strkey) of the project owner.
    owner_address       TEXT        NOT NULL,

    -- Off-chain display label. Not part of the contract; supplied by the owner
    -- via the frontend at registration time. Nullable because it is optional metadata.
    name                TEXT,

    -- Placeholder for the contract's future immutability toggle.
    -- Unused until the lock feature is implemented on-chain; included now so
    -- frontend, backend, and contract agree on the field name before it matters.
    is_locked           BOOLEAN     NOT NULL DEFAULT false,

    -- Self-referential FK: unused placeholder for future dependency-tree support.
    -- Indexed below even while unused so the future cascade-splits graph phase
    -- does not require a schema change — only a data migration.
    parent_project_id   TEXT,

    -- Ledger sequence at which the sync worker last updated this row.
    -- Lets the API report staleness and ops spot lagging projects at a glance.
    last_synced_ledger  BIGINT      NOT NULL DEFAULT 0,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT projects_pkey PRIMARY KEY (id),
    CONSTRAINT projects_parent_fk
        FOREIGN KEY (parent_project_id) REFERENCES projects (id)
        ON DELETE SET NULL
        DEFERRABLE INITIALLY DEFERRED
);

-- owner_address: used when the register-tx-builder checks for existing projects
-- by owner, and for dashboard queries ("projects owned by address X").
CREATE INDEX projects_owner_address_idx ON projects (owner_address);

-- parent_project_id: partial index (IS NOT NULL) so it stays effectively empty
-- while unused, with zero ongoing write overhead for NULL rows.
-- Will be used for dependency-tree traversal in a future issue.
CREATE INDEX projects_parent_project_id_idx
    ON projects (parent_project_id)
    WHERE parent_project_id IS NOT NULL;

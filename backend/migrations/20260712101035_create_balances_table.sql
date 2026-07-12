-- Migration: create_balances_table
-- Implements the balances table exactly as designed in backend/docs/schema-design.md.
-- One row per (project, token) pair. Multi-token from day one.

CREATE TABLE balances (
    -- FK to the owning project. CASCADE delete for consistency with splits.
    project_id          TEXT            NOT NULL,

    -- Stellar contract address (C... strkey) of the token.
    -- Using on-chain contract address rather than ticker symbol avoids
    -- ambiguity across networks (testnet USDC ≠ mainnet USDC).
    token_address       TEXT            NOT NULL,

    -- Balance in the token's smallest unit (stroops for XLM, micro-units
    -- for SAC tokens, etc.). The deposit-phase contract uses i128 for amounts.
    -- Postgres BIGINT is only 64 bits (max ~9.2×10¹⁸); i128 reaches ~1.7×10³⁸.
    -- NUMERIC(39,0) safely covers the full i128 range (38 decimal digits)
    -- with one digit of headroom, and has exact integer arithmetic.
    amount              NUMERIC(39,0)   NOT NULL DEFAULT 0,

    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),

    -- Composite PK mirrors the contract's DataKey::Balance(project_id, token_address).
    CONSTRAINT balances_pkey PRIMARY KEY (project_id, token_address),

    CONSTRAINT balances_project_fk
        FOREIGN KEY (project_id) REFERENCES projects (id)
        ON DELETE CASCADE
);

-- "All token balances for project X" — used on the project dashboard.
-- Redundant with the PK's leading column, but makes the query plan explicit.
CREATE INDEX balances_project_id_idx ON balances (project_id);

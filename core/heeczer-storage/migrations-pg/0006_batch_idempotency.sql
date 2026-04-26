-- Migration 0006 — batch Idempotency-Key replay cache — PostgreSQL dialect.
-- Mirrors core/heeczer-storage/migrations/0006_batch_idempotency.sql (SQLite).

CREATE TABLE heec_idempotency_keys (
    workspace_id      TEXT NOT NULL REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    idempotency_key   TEXT NOT NULL,
    request_hash      TEXT NOT NULL,
    status_code       INTEGER NOT NULL,
    response_body     TEXT NOT NULL,
    created_at        TEXT NOT NULL DEFAULT to_char(clock_timestamp() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
    expires_at        TEXT NOT NULL,
    PRIMARY KEY (workspace_id, idempotency_key)
);

CREATE INDEX idx_heec_idempotency_keys_expiry
    ON heec_idempotency_keys(expires_at);

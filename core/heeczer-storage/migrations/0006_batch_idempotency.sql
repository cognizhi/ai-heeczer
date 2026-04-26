-- Migration 0006 — batch Idempotency-Key replay cache (plan 0004, PRD §12.19).

CREATE TABLE heec_idempotency_keys (
    workspace_id      TEXT NOT NULL REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    idempotency_key   TEXT NOT NULL,
    request_hash      TEXT NOT NULL,
    status_code       INTEGER NOT NULL,
    response_body     TEXT NOT NULL,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expires_at        TEXT NOT NULL,
    PRIMARY KEY (workspace_id, idempotency_key)
);

CREATE INDEX idx_heec_idempotency_keys_expiry
    ON heec_idempotency_keys(expires_at);

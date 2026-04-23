-- ai-heeczer initial schema (plan 0003, PRD §20).
-- Backend: SQLite. PostgreSQL parity migration ships in plan 0004.
--
-- PRAGMA foreign_keys / journal_mode are connection-scoped in SQLite and are
-- set by the storage layer in `after_connect`, not here, because journal_mode
-- cannot be changed inside a transaction (which sqlx wraps each migration in).

CREATE TABLE heec_workspaces (
    workspace_id   TEXT PRIMARY KEY,
    display_name   TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    settings_json  TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE heec_api_keys (
    api_key_id     TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    hashed_key     TEXT NOT NULL UNIQUE,
    label          TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    revoked_at     TEXT
);
CREATE INDEX idx_heec_api_keys_workspace ON heec_api_keys(workspace_id);

CREATE TABLE heec_scoring_profiles (
    scoring_profile_id  TEXT NOT NULL,
    version             TEXT NOT NULL,
    workspace_id        TEXT REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    profile_json        TEXT NOT NULL,
    effective_at        TEXT NOT NULL,
    superseded_at       TEXT,
    PRIMARY KEY (scoring_profile_id, version, workspace_id)
);
CREATE INDEX idx_heec_scoring_profiles_effective
    ON heec_scoring_profiles(workspace_id, effective_at);

CREATE TABLE heec_tiers (
    tier_set_id    TEXT NOT NULL,
    version        TEXT NOT NULL,
    workspace_id   TEXT REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    tiers_json     TEXT NOT NULL,
    effective_at   TEXT NOT NULL,
    superseded_at  TEXT,
    PRIMARY KEY (tier_set_id, version, workspace_id)
);

CREATE TABLE heec_rates (
    rates_id       TEXT NOT NULL,
    version        TEXT NOT NULL,
    workspace_id   TEXT REFERENCES heec_workspaces(workspace_id) ON DELETE CASCADE,
    rates_json     TEXT NOT NULL,
    currency       TEXT NOT NULL,
    effective_at   TEXT NOT NULL,
    superseded_at  TEXT,
    PRIMARY KEY (rates_id, version, workspace_id)
);

CREATE TABLE heec_events (
    event_id          TEXT NOT NULL,
    workspace_id      TEXT NOT NULL REFERENCES heec_workspaces(workspace_id),
    spec_version      TEXT NOT NULL,
    framework_source  TEXT NOT NULL,
    correlation_id    TEXT,
    payload           TEXT NOT NULL,            -- normalized canonical event JSON
    payload_hash      TEXT NOT NULL DEFAULT '', -- sha256 of canonical bytes (computed app-side)
    received_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (workspace_id, event_id)
);
CREATE INDEX idx_heec_events_workspace_time
    ON heec_events(workspace_id, received_at);
CREATE INDEX idx_heec_events_correlation
    ON heec_events(workspace_id, correlation_id);

-- Append-only enforcement (PRD §19.4). UPDATE/DELETE rejected at the engine.
CREATE TRIGGER heec_events_no_update
BEFORE UPDATE ON heec_events
BEGIN
    SELECT RAISE(ABORT, 'heec_events is append-only');
END;
CREATE TRIGGER heec_events_no_delete
BEFORE DELETE ON heec_events
BEGIN
    SELECT RAISE(ABORT, 'heec_events is append-only; use heec_tombstones');
END;

CREATE TABLE heec_scores (
    workspace_id        TEXT NOT NULL REFERENCES heec_workspaces(workspace_id),
    event_id            TEXT NOT NULL,
    scoring_version     TEXT NOT NULL,
    scoring_profile_id  TEXT NOT NULL,
    profile_version     TEXT NOT NULL,
    tier_id             TEXT NOT NULL,
    tier_version        TEXT NOT NULL,
    rates_version       TEXT NOT NULL,
    result_json         TEXT NOT NULL,
    final_minutes       TEXT NOT NULL,         -- Decimal as string
    final_fec           TEXT NOT NULL,         -- Decimal as string
    confidence          TEXT NOT NULL,
    confidence_band     TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version),
    FOREIGN KEY (workspace_id, event_id) REFERENCES heec_events(workspace_id, event_id)
);
CREATE INDEX idx_heec_scores_lookup
    ON heec_scores(workspace_id, scoring_profile_id, created_at);

CREATE TRIGGER heec_scores_no_update
BEFORE UPDATE ON heec_scores
BEGIN
    SELECT RAISE(ABORT, 'heec_scores is append-only; re-score by inserting a new version');
END;
CREATE TRIGGER heec_scores_no_delete
BEFORE DELETE ON heec_scores
BEGIN
    SELECT RAISE(ABORT, 'heec_scores is append-only');
END;

CREATE TABLE heec_jobs (
    job_id         TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL REFERENCES heec_workspaces(workspace_id),
    event_id       TEXT,
    state          TEXT NOT NULL CHECK (state IN ('pending','running','succeeded','failed','dead_letter')),
    attempts       INTEGER NOT NULL DEFAULT 0,
    last_error     TEXT,
    available_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    enqueued_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    finished_at    TEXT
);
CREATE INDEX idx_heec_jobs_state_available
    ON heec_jobs(state, available_at);

CREATE TABLE heec_audit_log (
    audit_id       TEXT PRIMARY KEY,
    workspace_id   TEXT REFERENCES heec_workspaces(workspace_id),
    actor          TEXT NOT NULL,
    action         TEXT NOT NULL,
    target_table   TEXT,
    target_id      TEXT,
    payload_json   TEXT NOT NULL DEFAULT '{}',
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_heec_audit_log_workspace_time
    ON heec_audit_log(workspace_id, created_at);

CREATE TABLE heec_daily_aggregates (
    workspace_id   TEXT NOT NULL REFERENCES heec_workspaces(workspace_id),
    day            TEXT NOT NULL,           -- YYYY-MM-DD UTC
    project_id     TEXT NOT NULL DEFAULT '',
    category       TEXT NOT NULL DEFAULT '',
    framework_source TEXT NOT NULL DEFAULT '',
    event_count    INTEGER NOT NULL DEFAULT 0,
    total_minutes  TEXT NOT NULL DEFAULT '0',
    total_fec      TEXT NOT NULL DEFAULT '0',
    PRIMARY KEY (workspace_id, day, project_id, category, framework_source)
);

CREATE TABLE heec_tombstones (
    workspace_id   TEXT NOT NULL REFERENCES heec_workspaces(workspace_id),
    event_id       TEXT NOT NULL,
    deleted_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    reason         TEXT NOT NULL,
    PRIMARY KEY (workspace_id, event_id)
);

-- Migration history table (ADR-0004). sqlx::migrate manages its own table by
-- default (`_sqlx_migrations`); we expose a stable view named per PRD §20.
CREATE VIEW heec_schema_migrations AS
    SELECT version, description, installed_on, success
    FROM _sqlx_migrations;

-- ai-heeczer initial schema — PostgreSQL dialect (plan 0004, PRD §20).
-- Mirrors core/heeczer-storage/migrations/0001_init.sql (SQLite).
--
-- Key differences from the SQLite dialect:
--   - DEFAULT values use NOW() / CURRENT_TIMESTAMP instead of strftime().
--   - Append-only triggers use PL/pgSQL trigger functions (BEFORE trigger per row).
--   - JSON stored as TEXT for app-layer parity; upgrade to JSONB is a v2 schema concern.
--
-- This file is consumed by the PostgreSQL migrator (src/pg.rs).

CREATE TABLE aih_workspaces (
    workspace_id   TEXT PRIMARY KEY,
    display_name   TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    settings_json  TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE aih_api_keys (
    api_key_id     TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL REFERENCES aih_workspaces(workspace_id) ON DELETE CASCADE,
    hashed_key     TEXT NOT NULL UNIQUE,
    label          TEXT NOT NULL,
    created_at     TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    revoked_at     TEXT
);
CREATE INDEX idx_aih_api_keys_workspace ON aih_api_keys(workspace_id);

CREATE TABLE aih_scoring_profiles (
    scoring_profile_id  TEXT NOT NULL,
    version             TEXT NOT NULL,
    workspace_id        TEXT REFERENCES aih_workspaces(workspace_id) ON DELETE CASCADE,
    profile_json        TEXT NOT NULL,
    effective_at        TEXT NOT NULL,
    superseded_at       TEXT,
    PRIMARY KEY (scoring_profile_id, version, workspace_id)
);
CREATE INDEX idx_aih_scoring_profiles_effective
    ON aih_scoring_profiles(workspace_id, effective_at);

CREATE TABLE aih_tiers (
    tier_set_id    TEXT NOT NULL,
    version        TEXT NOT NULL,
    workspace_id   TEXT REFERENCES aih_workspaces(workspace_id) ON DELETE CASCADE,
    tiers_json     TEXT NOT NULL,
    effective_at   TEXT NOT NULL,
    superseded_at  TEXT,
    PRIMARY KEY (tier_set_id, version, workspace_id)
);

CREATE TABLE aih_rates (
    rates_id       TEXT NOT NULL,
    version        TEXT NOT NULL,
    workspace_id   TEXT REFERENCES aih_workspaces(workspace_id) ON DELETE CASCADE,
    rates_json     TEXT NOT NULL,
    currency       TEXT NOT NULL,
    effective_at   TEXT NOT NULL,
    superseded_at  TEXT,
    PRIMARY KEY (rates_id, version, workspace_id)
);

CREATE TABLE aih_events (
    event_id          TEXT NOT NULL,
    workspace_id      TEXT NOT NULL REFERENCES aih_workspaces(workspace_id),
    spec_version      TEXT NOT NULL,
    framework_source  TEXT NOT NULL,
    correlation_id    TEXT,
    payload           TEXT NOT NULL,
    payload_hash      TEXT NOT NULL DEFAULT '',
    received_at       TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    PRIMARY KEY (workspace_id, event_id)
);
CREATE INDEX idx_aih_events_workspace_time
    ON aih_events(workspace_id, received_at);
CREATE INDEX idx_aih_events_correlation
    ON aih_events(workspace_id, correlation_id);

-- Append-only enforcement (PRD §19.4) — PostgreSQL trigger functions.
CREATE OR REPLACE FUNCTION _aih_raise_no_update()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'table % is append-only', TG_TABLE_NAME;
END;
$$;

CREATE OR REPLACE FUNCTION _aih_raise_no_delete()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'table % is append-only; use aih_tombstones', TG_TABLE_NAME;
END;
$$;

CREATE TRIGGER aih_events_no_update
    BEFORE UPDATE ON aih_events
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_update();

CREATE TRIGGER aih_events_no_delete
    BEFORE DELETE ON aih_events
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_delete();

CREATE TABLE aih_scores (
    workspace_id        TEXT NOT NULL REFERENCES aih_workspaces(workspace_id),
    event_id            TEXT NOT NULL,
    scoring_version     TEXT NOT NULL,
    scoring_profile_id  TEXT NOT NULL,
    profile_version     TEXT NOT NULL,
    tier_id             TEXT NOT NULL,
    tier_version        TEXT NOT NULL,
    rates_version       TEXT NOT NULL,
    result_json         TEXT NOT NULL,
    final_minutes       TEXT NOT NULL,
    final_fec           TEXT NOT NULL,
    confidence          TEXT NOT NULL,
    confidence_band     TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    PRIMARY KEY (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version),
    FOREIGN KEY (workspace_id, event_id) REFERENCES aih_events(workspace_id, event_id)
);
CREATE INDEX idx_aih_scores_lookup
    ON aih_scores(workspace_id, scoring_profile_id, created_at);

CREATE TRIGGER aih_scores_no_update
    BEFORE UPDATE ON aih_scores
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_update();

CREATE TRIGGER aih_scores_no_delete
    BEFORE DELETE ON aih_scores
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_delete();

CREATE TABLE aih_jobs (
    job_id         TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL REFERENCES aih_workspaces(workspace_id),
    event_id       TEXT,
    state          TEXT NOT NULL CHECK (state IN ('pending','running','succeeded','failed','dead_letter')),
    attempts       INTEGER NOT NULL DEFAULT 0,
    last_error     TEXT,
    available_at   TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    enqueued_at    TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    finished_at    TEXT
);
CREATE INDEX idx_aih_jobs_state_available
    ON aih_jobs(state, available_at);

CREATE TABLE aih_audit_log (
    audit_id       TEXT PRIMARY KEY,
    workspace_id   TEXT REFERENCES aih_workspaces(workspace_id),
    actor          TEXT NOT NULL,
    action         TEXT NOT NULL,
    target_table   TEXT,
    target_id      TEXT,
    payload_json   TEXT NOT NULL DEFAULT '{}',
    created_at     TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
);
CREATE INDEX idx_aih_audit_log_workspace_time
    ON aih_audit_log(workspace_id, created_at);

CREATE TABLE aih_daily_aggregates (
    workspace_id   TEXT NOT NULL REFERENCES aih_workspaces(workspace_id),
    day            TEXT NOT NULL,
    project_id     TEXT NOT NULL DEFAULT '',
    category       TEXT NOT NULL DEFAULT '',
    framework_source TEXT NOT NULL DEFAULT '',
    event_count    INTEGER NOT NULL DEFAULT 0,
    total_minutes  TEXT NOT NULL DEFAULT '0',
    total_fec      TEXT NOT NULL DEFAULT '0',
    PRIMARY KEY (workspace_id, day, project_id, category, framework_source)
);

CREATE TABLE aih_tombstones (
    workspace_id   TEXT NOT NULL REFERENCES aih_workspaces(workspace_id),
    event_id       TEXT NOT NULL,
    deleted_at     TEXT NOT NULL DEFAULT to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    reason         TEXT NOT NULL,
    PRIMARY KEY (workspace_id, event_id)
);

-- Migration history view (ADR-0004).
CREATE VIEW aih_schema_migrations AS
    SELECT version, description, installed_on, success
    FROM _sqlx_migrations;

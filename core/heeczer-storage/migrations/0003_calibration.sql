-- Migration 0003: Calibration data model (plan 0015, PRD §25).
-- Append-only with the same trigger pattern as heec_events / heec_scores.

CREATE TABLE IF NOT EXISTS heec_benchmark_packs (
    pack_id         TEXT    NOT NULL,
    workspace_id    TEXT    REFERENCES heec_workspaces(workspace_id),
    version         TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    description     TEXT    NOT NULL DEFAULT '',
    items_json      TEXT    NOT NULL DEFAULT '[]',  -- JSON array of BenchmarkItem
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (pack_id, version)
);

CREATE TABLE IF NOT EXISTS heec_calibration_runs (
    run_id              TEXT    PRIMARY KEY,
    workspace_id        TEXT    REFERENCES heec_workspaces(workspace_id),
    pack_id             TEXT    NOT NULL,
    pack_version        TEXT    NOT NULL,
    profile_id          TEXT    NOT NULL,
    profile_version     TEXT    NOT NULL,
    results_json        TEXT    NOT NULL DEFAULT '{}',  -- JSON map item_id → delta
    status              TEXT    NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending', 'running', 'complete', 'failed')),
    started_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    finished_at         TEXT,
    FOREIGN KEY (pack_id, pack_version)
        REFERENCES heec_benchmark_packs(pack_id, version)
);

-- Append-only: calibration_runs rows may only have status updated (not rows deleted).
-- A trigger prevents DELETE.
CREATE TRIGGER IF NOT EXISTS heec_calibration_runs_no_delete
BEFORE DELETE ON heec_calibration_runs
BEGIN
    SELECT RAISE(ABORT, 'heec_calibration_runs is append-only; use status=failed instead of DELETE');
END;

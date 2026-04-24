-- Migration 0003 (PostgreSQL): Calibration data model (plan 0015, PRD §25).

CREATE TABLE IF NOT EXISTS heec_benchmark_packs (
    pack_id         TEXT        NOT NULL,
    workspace_id    TEXT        REFERENCES heec_workspaces(workspace_id),
    version         TEXT        NOT NULL,
    name            TEXT        NOT NULL,
    description     TEXT        NOT NULL DEFAULT '',
    items_json      JSONB       NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pack_id, version)
);

CREATE TABLE IF NOT EXISTS heec_calibration_runs (
    run_id              TEXT        PRIMARY KEY,
    workspace_id        TEXT        REFERENCES heec_workspaces(workspace_id),
    pack_id             TEXT        NOT NULL,
    pack_version        TEXT        NOT NULL,
    profile_id          TEXT        NOT NULL,
    profile_version     TEXT        NOT NULL,
    results_json        JSONB       NOT NULL DEFAULT '{}',
    status              TEXT        NOT NULL DEFAULT 'pending'
                            CHECK (status IN ('pending', 'running', 'complete', 'failed')),
    started_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at         TIMESTAMPTZ,
    CONSTRAINT fk_pack FOREIGN KEY (pack_id, pack_version)
        REFERENCES heec_benchmark_packs(pack_id, version)
);

CREATE OR REPLACE FUNCTION heec_prevent_calibration_delete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'heec_calibration_runs is append-only; use status=failed instead of DELETE';
END;
$$;

CREATE TRIGGER heec_calibration_runs_no_delete
BEFORE DELETE ON heec_calibration_runs
FOR EACH ROW EXECUTE FUNCTION heec_prevent_calibration_delete();

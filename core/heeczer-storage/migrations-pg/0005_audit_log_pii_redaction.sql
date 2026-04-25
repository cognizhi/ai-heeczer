-- Migration 0005 — tombstone-authorized audit-log PII redaction — PostgreSQL dialect.
-- Mirrors core/heeczer-storage/migrations/0005_audit_log_pii_redaction.sql (SQLite).
--
-- Replaces the blanket heec_audit_log_no_update trigger (added in migration 0002)
-- with a tombstone-aware function that permits ONE specific mutation:
-- setting target_id to NULL when a tombstone row exists for that event.
--
-- The shared _heec_raise_no_update() function from migration 0001 is no longer
-- used for heec_audit_log; we install a per-table function here instead so the
-- shared function remains available for heec_scoring_profiles / heec_tiers / heec_rates.
--
-- Allowed UPDATE: target_id → NULL when
--   (a) new target_id IS NULL, old target_id IS NOT NULL
--   (b) all other columns are unchanged (IS NOT DISTINCT FROM for nullable cols)
--   (c) a heec_tombstones row exists for (workspace_id, old.target_id)

CREATE OR REPLACE FUNCTION _heec_audit_log_guard_update()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    -- Permit only: target_id → NULL, tombstone present, no other column change.
    IF  NEW.target_id IS NULL
        AND OLD.target_id IS NOT NULL
        AND NEW.audit_id     =               OLD.audit_id
        AND NEW.workspace_id IS NOT DISTINCT FROM OLD.workspace_id
        AND NEW.actor        =               OLD.actor
        AND NEW.action       =               OLD.action
        AND NEW.target_table IS NOT DISTINCT FROM OLD.target_table
        AND NEW.payload_json =               OLD.payload_json
        AND NEW.created_at   =               OLD.created_at
        AND EXISTS (
            SELECT 1 FROM heec_tombstones
            WHERE workspace_id = OLD.workspace_id
              AND event_id     = OLD.target_id
        )
    THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION
        'heec_audit_log is append-only; only tombstone-authorized target_id redaction is permitted';
END;
$$;

DROP TRIGGER IF EXISTS heec_audit_log_no_update ON heec_audit_log;
CREATE TRIGGER heec_audit_log_no_update
    BEFORE UPDATE ON heec_audit_log
    FOR EACH ROW EXECUTE FUNCTION _heec_audit_log_guard_update();

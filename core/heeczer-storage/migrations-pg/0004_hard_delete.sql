-- Migration 0004 — tombstone-authorized hard deletion — PostgreSQL dialect.
-- Mirrors core/heeczer-storage/migrations/0004_hard_delete.sql (SQLite).
--
-- Replaces the heec_events and heec_scores no-delete trigger functions with
-- tombstone-aware versions. The existing shared function _heec_raise_no_delete()
-- (installed in migration 0001) is still used by heec_audit_log, so we create
-- new per-table functions rather than replacing the shared one.
--
-- A heec_tombstones row for (workspace_id, event_id) acts as a capability
-- token: inserting it first within the same transaction unlocks the subsequent
-- DELETEs on events and scores.
--
-- Also adds append-only guards for heec_tombstones itself.
--
-- Recovery note: if this migration fails after DROP TRIGGER but before the new
-- CREATE TRIGGER statements, the original trigger bodies from migration 0001
-- must be re-applied before retrying.

-- Tombstone-aware delete function for heec_events.
CREATE OR REPLACE FUNCTION _heec_events_guard_no_delete()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM heec_tombstones
        WHERE workspace_id = OLD.workspace_id AND event_id = OLD.event_id
    ) THEN
        RAISE EXCEPTION 'heec_events is append-only; insert a heec_tombstones row first';
    END IF;
    RETURN OLD;
END;
$$;

DROP TRIGGER IF EXISTS heec_events_no_delete ON heec_events;
CREATE TRIGGER heec_events_no_delete
    BEFORE DELETE ON heec_events
    FOR EACH ROW EXECUTE FUNCTION _heec_events_guard_no_delete();

-- Tombstone-aware delete function for heec_scores.
CREATE OR REPLACE FUNCTION _heec_scores_guard_no_delete()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM heec_tombstones
        WHERE workspace_id = OLD.workspace_id AND event_id = OLD.event_id
    ) THEN
        RAISE EXCEPTION 'heec_scores is append-only; insert a heec_tombstones row first';
    END IF;
    RETURN OLD;
END;
$$;

DROP TRIGGER IF EXISTS heec_scores_no_delete ON heec_scores;
CREATE TRIGGER heec_scores_no_delete
    BEFORE DELETE ON heec_scores
    FOR EACH ROW EXECUTE FUNCTION _heec_scores_guard_no_delete();

-- Append-only guards for heec_tombstones (reuse shared trigger functions from 0001).
CREATE TRIGGER heec_tombstones_no_update
    BEFORE UPDATE ON heec_tombstones
    FOR EACH ROW EXECUTE FUNCTION _heec_raise_no_update();

CREATE TRIGGER heec_tombstones_no_delete
    BEFORE DELETE ON heec_tombstones
    FOR EACH ROW EXECUTE FUNCTION _heec_raise_no_delete();

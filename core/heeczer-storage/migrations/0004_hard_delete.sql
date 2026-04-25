-- Migration 0004 — tombstone-authorized hard deletion (plan 0003, PRD §12.17).
--
-- Replaces the blanket no-delete triggers on heec_events and heec_scores with
-- tombstone-aware equivalents. A tombstone row for (workspace_id, event_id)
-- acts as a capability token: inserting it first unlocks the subsequent DELETEs
-- inside the same transaction, preserving the append-only invariant for all
-- other callers.
--
-- Also adds append-only guards for heec_tombstones itself so tombstone rows
-- cannot be deleted or updated after insertion, which would reopen a deleted
-- event for re-scoring.
--
-- The heec_audit_log triggers are intentionally left untouched; full PII
-- redaction of audit-log target_id fields requires a separate migration with
-- its own security review (tracked in plan 0003 backlog).
--
-- Recovery note: if this migration fails after the DROP TRIGGER statements but
-- before the new CREATE TRIGGER statements, the original trigger bodies from
-- migration 0001 must be re-applied before retrying.

-- Tombstone-aware delete guard for heec_events.
-- Deletion is permitted only after an authorising heec_tombstones row exists.
DROP TRIGGER IF EXISTS heec_events_no_delete;
CREATE TRIGGER heec_events_no_delete
BEFORE DELETE ON heec_events
WHEN NOT EXISTS (
    SELECT 1 FROM heec_tombstones
    WHERE workspace_id = OLD.workspace_id AND event_id = OLD.event_id
)
BEGIN
    SELECT RAISE(ABORT, 'heec_events is append-only; insert a heec_tombstones row first');
END;

-- Tombstone-aware delete guard for heec_scores.
DROP TRIGGER IF EXISTS heec_scores_no_delete;
CREATE TRIGGER heec_scores_no_delete
BEFORE DELETE ON heec_scores
WHEN NOT EXISTS (
    SELECT 1 FROM heec_tombstones
    WHERE workspace_id = OLD.workspace_id AND event_id = OLD.event_id
)
BEGIN
    SELECT RAISE(ABORT, 'heec_scores is append-only; insert a heec_tombstones row first');
END;

-- Append-only guards for heec_tombstones.
-- Tombstone rows must be permanent so re-scoring jobs reliably detect deleted
-- events (PRD §12.17). Deletion or update of a tombstone would silently reopen
-- an event for re-scoring.
CREATE TRIGGER heec_tombstones_no_update
BEFORE UPDATE ON heec_tombstones
BEGIN
    SELECT RAISE(ABORT, 'heec_tombstones is append-only');
END;

CREATE TRIGGER heec_tombstones_no_delete
BEFORE DELETE ON heec_tombstones
BEGIN
    SELECT RAISE(ABORT, 'heec_tombstones is append-only');
END;

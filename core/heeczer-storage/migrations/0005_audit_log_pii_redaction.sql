-- Migration 0005 — tombstone-authorized audit-log PII redaction (plan 0003, PRD §12.17).
--
-- Replaces the blanket heec_audit_log_no_update trigger (added in migration 0002)
-- with a tombstone-aware version that permits ONE specific mutation:
-- setting target_id to NULL when a tombstone row exists for that event.
--
-- All other UPDATE attempts are still rejected, preserving the append-only
-- invariant for the entire audit log except for this narrow redaction path.
--
-- Allowed UPDATE: target_id → NULL when
--   (a) new target_id IS NULL, old target_id IS NOT NULL
--   (b) all other columns are unchanged
--   (c) a heec_tombstones row exists for (workspace_id, old.target_id)
--
-- This allows hard_delete_event() to satisfy PRD §12.17 by NULLing the
-- target_id on any pre-existing audit rows that reference a deleted event_id,
-- without exposing a general-purpose UPDATE path on the audit log.

DROP TRIGGER IF EXISTS heec_audit_log_no_update;

CREATE TRIGGER heec_audit_log_no_update
BEFORE UPDATE ON heec_audit_log
WHEN NOT (
    -- Permit only: target_id → NULL, tombstone present, no other column change.
    NEW.target_id IS NULL
    AND OLD.target_id IS NOT NULL
    AND NEW.audit_id      =    OLD.audit_id
    AND NEW.workspace_id  IS   OLD.workspace_id
    AND NEW.actor         =    OLD.actor
    AND NEW.action        =    OLD.action
    AND NEW.target_table  IS   OLD.target_table
    AND NEW.payload_json  =    OLD.payload_json
    AND NEW.created_at    =    OLD.created_at
    AND EXISTS (
        SELECT 1 FROM heec_tombstones
        WHERE workspace_id = OLD.workspace_id
          AND event_id     = OLD.target_id
    )
)
BEGIN
    SELECT RAISE(ABORT,
        'heec_audit_log is append-only; only tombstone-authorized target_id redaction is permitted');
END;

-- Migration 0002 — foundation backlog hardening.
--
-- Adds:
--   1. Append-only triggers for heec_audit_log (PRD §22.5; was missing in 0001).
--   2. Unique indexes that close the SQLite "two NULLs are distinct" hole on
--      the (scoring_profile_id, version, workspace_id) family of PKs. NULL
--      workspace_id represents a "global" row; without these indexes two
--      identical global rows can co-exist because SQLite's PK semantics treat
--      each NULL as distinct. COALESCE(workspace_id, '') gives a portable
--      sentinel and is enforceable on PostgreSQL with the same expression.
--
-- Refs: foundation backlog (Security), AGENT_HARNESS §3 row "Database schema".

-- 1) Append-only audit log
CREATE TRIGGER heec_audit_log_no_update
BEFORE UPDATE ON heec_audit_log
BEGIN
    SELECT RAISE(ABORT, 'heec_audit_log is append-only');
END;

CREATE TRIGGER heec_audit_log_no_delete
BEFORE DELETE ON heec_audit_log
BEGIN
    SELECT RAISE(ABORT, 'heec_audit_log is append-only');
END;

-- 2) Close the nullable-workspace_id PK hole
CREATE UNIQUE INDEX uq_heec_scoring_profiles_global
    ON heec_scoring_profiles (scoring_profile_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_heec_tiers_global
    ON heec_tiers (tier_set_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_heec_rates_global
    ON heec_rates (rates_id, version, COALESCE(workspace_id, ''));

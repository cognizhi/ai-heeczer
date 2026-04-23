-- Migration 0002 — foundation backlog hardening — PostgreSQL dialect.
-- Mirrors core/heeczer-storage/migrations/0002_append_only_audit_and_global_unique.sql.
--
-- Adds:
--   1. Append-only triggers for heec_audit_log (PRD §22.5).
--   2. Unique partial indexes that close the nullable-workspace_id PK hole.
--      PostgreSQL natively treats two NULLs as distinct in UNIQUE constraints
--      per SQL standard. We use the same COALESCE sentinel as the SQLite
--      dialect so the application logic is identical.
--
-- Refs: foundation backlog (Security), AGENT_HARNESS §3 row "Database schema".

-- 1) Append-only audit log (reuse the shared trigger functions from 0001)
CREATE TRIGGER heec_audit_log_no_update
    BEFORE UPDATE ON heec_audit_log
    FOR EACH ROW EXECUTE FUNCTION _heec_raise_no_update();

CREATE TRIGGER heec_audit_log_no_delete
    BEFORE DELETE ON heec_audit_log
    FOR EACH ROW EXECUTE FUNCTION _heec_raise_no_delete();

-- 2) Close the nullable-workspace_id PK hole
CREATE UNIQUE INDEX uq_heec_scoring_profiles_global
    ON heec_scoring_profiles (scoring_profile_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_heec_tiers_global
    ON heec_tiers (tier_set_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_heec_rates_global
    ON heec_rates (rates_id, version, COALESCE(workspace_id, ''));

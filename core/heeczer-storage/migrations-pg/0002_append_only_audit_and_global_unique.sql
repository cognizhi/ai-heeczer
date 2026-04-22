-- Migration 0002 — foundation backlog hardening — PostgreSQL dialect.
-- Mirrors core/heeczer-storage/migrations/0002_append_only_audit_and_global_unique.sql.
--
-- Adds:
--   1. Append-only triggers for aih_audit_log (PRD §22.5).
--   2. Unique partial indexes that close the nullable-workspace_id PK hole.
--      PostgreSQL natively treats two NULLs as distinct in UNIQUE constraints
--      per SQL standard. We use the same COALESCE sentinel as the SQLite
--      dialect so the application logic is identical.
--
-- Refs: foundation backlog (Security), AGENT_HARNESS §3 row "Database schema".

-- 1) Append-only audit log (reuse the shared trigger functions from 0001)
CREATE TRIGGER aih_audit_log_no_update
    BEFORE UPDATE ON aih_audit_log
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_update();

CREATE TRIGGER aih_audit_log_no_delete
    BEFORE DELETE ON aih_audit_log
    FOR EACH ROW EXECUTE FUNCTION _aih_raise_no_delete();

-- 2) Close the nullable-workspace_id PK hole
CREATE UNIQUE INDEX uq_aih_scoring_profiles_global
    ON aih_scoring_profiles (scoring_profile_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_aih_tiers_global
    ON aih_tiers (tier_set_id, version, COALESCE(workspace_id, ''));

CREATE UNIQUE INDEX uq_aih_rates_global
    ON aih_rates (rates_id, version, COALESCE(workspace_id, ''));

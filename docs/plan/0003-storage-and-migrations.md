# Plan 03 — Storage and migrations

- **Status:** Active
- **Owner:** Tech Lead + Security Engineer
- **PRD:** §20, §12.5, §12.20, §12.17
- **ADR:** ADR-0004, ADR-0005

## Goal

Provide a portable storage layer with SQLite (local/dev) and PostgreSQL (production) backends sharing a single migration history, append-only event/score tables, and tenant-scoped queries.

## Checklist

### Tables (PRD §20)

- [x] `aih_workspaces` — tenant root. (PR #1)
- [x] `aih_api_keys` — hashed keys per workspace. (PR #1)
- [x] `aih_events` — append-only raw normalized events keyed by `event_id`. (PR #1)
- [x] `aih_scores` — append-only, keyed by `(event_id, scoring_version, scoring_profile_id)`. (PR #1)
- [x] `aih_jobs` — queue rows for image-mode workers (ADR-0006). (PR #1)
- [x] `aih_tiers` — append-only with `effective_at`. (PR #1)
- [x] `aih_rates` — append-only with `effective_at`. (PR #1)
- [x] `aih_scoring_profiles` — append-only with `effective_at`. (PR #1)
- [x] `aih_audit_log` — every config change and re-scoring event. (PR #1)
- [x] `aih_daily_aggregates` — derived rollups. (PR #1)
- [x] `aih_tombstones` — for hard-deletion (PRD §12.17). (PR #1)
- [x] `aih_schema_migrations` — migration history view alias over `_sqlx_migrations` (ADR-0004). (PR #1)

### Indexes

- [x] `aih_events(workspace_id, timestamp)`, `aih_events(workspace_id, event_id)` unique. (PR #1)
- [x] `aih_scores(workspace_id, event_id, scoring_version)` unique. (PR #1)
- [x] `aih_scores(workspace_id, scoring_profile_id, created_at)` for dashboard rollups. (PR #1)
- [x] `aih_jobs(state, available_at)` partial index for queue scans. (PR #1)
- [x] `aih_audit_log(workspace_id, created_at)`. (PR #1)

### Migrations (ADR-0004)

- [x] Wire `sqlx_macros::migrate!` into the storage layer used by the ingestion service/CLI, with `sqlx-core` + `sqlx-sqlite` + `sqlx-postgres` split so the lockfile does not pull the unused MySQL driver into security scans. (CI hardening, April 2026)
- [x] Author `0001_init.sql` with SQLite dialect (PR #1); PostgreSQL parity migration deferred to plan 04.
- [x] Author `migrations-pg/0001_init.sql` — PostgreSQL dialect (PL/pgSQL triggers, `NOW()` defaults). (plan 0004)
- [x] Author `migrations-pg/0002_append_only_audit_and_global_unique.sql` — PostgreSQL dialect. (plan 0004)
- [x] Add `src/pg.rs` PostgreSQL backend module (`heeczer_storage::pg`). (plan 0004)
- [x] Add `heec migrate up|status|verify` CLI subcommands (per ADR-0010; supersedes the prior `heeczerctl` plan). (PR #1)
- [x] Document migration authoring guide in `docs/architecture/data-model.md`. (session Apr-2026)
- [x] Calibration tables migration: `core/heeczer-storage/migrations/0003_calibration.sql` (SQLite) and `migrations-pg/0003_calibration.sql` (PostgreSQL). (session Apr-2026)

### Multi-tenancy

- [ ] Every tenant-scoped query carries `workspace_id` parameter; lints flag missing scopes.
- [ ] Repository layer enforces workspace scoping at the type level (newtype wrapper).

### Append-only enforcement

- [x] `aih_events` and `aih_scores` have DB triggers preventing UPDATE/DELETE except via tombstone insert. (SQLite: migration 0001; PostgreSQL: PL/pgSQL functions in migrations-pg/0001)
- [x] Re-scoring path inserts new score rows; never updates. (PR #1)
- [x] `aih_audit_log` append-only trigger pair (PRD §22.5). (migration 0002, commit 9fb81aa)

### Retention and deletion (PRD §12.17)

- [ ] Background sweeper deletes events past retention; writes tombstone rows.
- [x] Hard-deletion storage API (`heeczer_storage::admin::hard_delete_event`) and CLI command (`heec admin delete-event`), admin only, with audit log entry and tombstone. (session Apr-2026, migration 0004)
- [x] Aggregates remain anonymized after raw deletion. (`heec_daily_aggregates` is never touched by the hard-delete path; verified in `hard_delete_preserves_daily_aggregates` test, session Apr-2026)

### Tests

- [ ] Migration test: fresh-install on SQLite and PostgreSQL. (partial: SQLite covered via CLI integration test PR #1; PostgreSQL pending)
- [x] PostgreSQL migration file presence tests in `core/heeczer-storage/tests/migration_pg.rs` — 4 tests verify directory existence, non-empty files, CREATE TABLE presence, and SQLite/PG parity. (session Apr-2026)
- [ ] Migration test: incremental upgrade from each prior version.
- [x] Unit: append-only guard rejects updates/deletes. (`aih_events`, `aih_scores`, `aih_audit_log` triggers all under test in `tests/hardening.rs`, commit 9fb81aa)
- [x] Unit: `current_version` matches the embedded migration count, FK enforcement, `aih_jobs.state` CHECK constraint, `open_path` round-trip. (foundation hardening, commit 9fb81aa)
- [x] Unit: global rows on profiles/tiers/rates cannot duplicate via unique-with-COALESCE indexes (closes nullable-`workspace_id` PK hole). (migration 0002, commit 9fb81aa)
- [x] Integration: dedup on duplicate `event_id` returns existing record (PRD §19.4). (session Cat-3)
- [ ] Integration: conflicting payload for same `event_id` rejected with `409 Conflict` and audit-log entry.
- [x] Integration: tombstone prevents re-scoring of deleted event. (session Apr-2026)
- [ ] Audit-log PII redaction: NULL the `target_id` on pre-existing `heec_audit_log` rows for a hard-deleted event (PRD §12.17). Requires loosening the `heec_audit_log_no_update` trigger; needs a separate migration and security review.

### Docs

- [x] `docs/architecture/data-model.md` with ERD. (session Apr-2026)
- [ ] Operational runbook for PostgreSQL vacuum/index tuning at scale.

## Acceptance

- Both backends pass the same integration suite.
- Append-only invariants verified in CI.

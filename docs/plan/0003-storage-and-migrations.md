# Plan 03 — Storage and migrations

- **Status:** Active
- **Owner:** Tech Lead + Security Engineer
- **PRD:** §20, §12.5, §12.20, §12.17
- **ADR:** ADR-0004, ADR-0005

## Goal
Provide a portable storage layer with SQLite (local/dev) and PostgreSQL (production) backends sharing a single migration history, append-only event/score tables, and tenant-scoped queries.

## Checklist

### Tables (PRD §20)
- [ ] `aih_workspaces` — tenant root.
- [ ] `aih_api_keys` — hashed keys per workspace.
- [ ] `aih_events` — append-only raw normalized events keyed by `event_id`.
- [ ] `aih_scores` — append-only, keyed by `(event_id, scoring_version, scoring_profile_id)`.
- [ ] `aih_jobs` — queue rows for image-mode workers (ADR-0006).
- [ ] `aih_tiers` — append-only with `effective_at`.
- [ ] `aih_rates` — append-only with `effective_at`.
- [ ] `aih_scoring_profiles` — append-only with `effective_at`.
- [ ] `aih_audit_log` — every config change and re-scoring event.
- [ ] `aih_daily_aggregates` — derived rollups.
- [ ] `aih_tombstones` — for hard-deletion (PRD §12.17).
- [ ] `aih_schema_migrations` — migration history (ADR-0004).

### Indexes
- [ ] `aih_events(workspace_id, timestamp)`, `aih_events(workspace_id, event_id)` unique.
- [ ] `aih_scores(workspace_id, event_id, scoring_version)` unique.
- [ ] `aih_scores(workspace_id, scoring_profile_id, created_at)` for dashboard rollups.
- [ ] `aih_jobs(state, available_at)` partial index for queue scans.
- [ ] `aih_audit_log(workspace_id, created_at)`.

### Migrations (ADR-0004)
- [ ] Wire `sqlx::migrate!` into the ingestion service crate.
- [ ] Author `0001_init.sql` with separate SQLite/PostgreSQL variants where dialects diverge.
- [ ] Add `heeczerctl migrate up|status|verify` CLI subcommands.
- [ ] Document migration authoring guide in `docs/architecture/data-model.md`.

### Multi-tenancy
- [ ] Every tenant-scoped query carries `workspace_id` parameter; lints flag missing scopes.
- [ ] Repository layer enforces workspace scoping at the type level (newtype wrapper).

### Append-only enforcement
- [ ] `aih_events` and `aih_scores` have DB triggers (PG) and runtime guards (SQLite + Rust) preventing UPDATE/DELETE except via tombstone insert.
- [ ] Re-scoring path inserts new score rows; never updates.

### Retention and deletion (PRD §12.17)
- [ ] Background sweeper deletes events past retention; writes tombstone rows.
- [ ] Hard-deletion API endpoint (admin only) with audit log entry.
- [ ] Aggregates remain anonymized after raw deletion.

### Tests
- [ ] Migration test: fresh-install on SQLite and PostgreSQL.
- [ ] Migration test: incremental upgrade from each prior version.
- [ ] Unit: append-only guard rejects updates/deletes.
- [ ] Integration: dedup on duplicate `event_id` returns existing record (PRD §19.4).
- [ ] Integration: conflicting payload for same `event_id` rejected with `409 Conflict` and audit-log entry.
- [ ] Integration: tombstone prevents re-scoring of deleted event.

### Docs
- [ ] `docs/architecture/data-model.md` with ERD.
- [ ] Operational runbook for PostgreSQL vacuum/index tuning at scale.

## Acceptance
- Both backends pass the same integration suite.
- Append-only invariants verified in CI.

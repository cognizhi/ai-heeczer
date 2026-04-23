# ADR-0004: Database Migration Tooling

- **Status:** Accepted (CLI surface amended by ADR-0010)
- **Date:** 2026-04-22
- **Related:** PRD §12.20, §20, ADR-0001, ADR-0005, ADR-0010

## Context
ai-heeczer persists raw events, scores, jobs, tiers, rates, profiles, audit logs, and aggregates across both SQLite (local/dev) and PostgreSQL (production) (PRD §20). Schema will evolve over time and we must support:
- forward-only, ordered, idempotent migrations
- a single migration history shared between SQLite and PostgreSQL
- migrations runnable from the same artifact as the ingestion service (no separate runtime)
- migrations testable in CI on both backends
- safe online migrations on PostgreSQL (no long table locks for hot tables)

The user explicitly asked whether **Alembic** is appropriate. Alembic is an excellent, mature SQLAlchemy-based migration framework — but it is Python-only and requires a Python runtime in the deploy footprint.

## Decision
Use **`sqlx::migrate!`** (compile-time embedded SQL migrations) as the primary migration tool, owned by the Rust ingestion service crate (`server/ingestion`). Migrations live under `server/ingestion/migrations/` as numbered, forward-only `.sql` files with paired SQLite and PostgreSQL variants where dialects diverge.

We do **not** adopt Alembic for the production data path. Rationale:
1. Per ADR-0005, the ingestion service is Rust. Adding Alembic would force a Python runtime into the production container purely for migrations, increasing image size, CVE surface, and operational complexity.
2. `sqlx` is already required by the ingestion service for typed queries against both SQLite and PostgreSQL; reusing its migration support keeps tooling cohesive.
3. Embedded migrations execute at service startup (or via `heec migrate`, see ADR-0010) without a separate orchestrator.

Alembic remains an explicitly approved choice **only** for any optional future Python-based tooling (e.g., calibration scripts, data-science notebooks against a read replica). It must never be the production schema authority.

For complex online PostgreSQL migrations that exceed `sqlx`'s pure-SQL ergonomics (concurrent index builds, table rewrites in chunks), the migration script may shell out to `psql` with `CONCURRENTLY` clauses; the migration test in CI verifies both fresh-install and incremental-upgrade paths.

## Alternatives Considered
- **Alembic (Python)** — mature, Pythonic autogeneration, large community. Rejected because it forces Python into the Rust runtime image; autogeneration also encourages drift between ORM models and hand-written migrations, which we don't want for a contract-first system.
- **`refinery` (Rust)** — solid, supports multiple backends, but smaller ecosystem and overlaps with `sqlx`.
- **Diesel migrations** — would require adopting Diesel as the query layer; larger blast radius.
- **Flyway / Liquibase** — JVM dependency in deploy footprint; rejected on the same grounds as Alembic.
- **Hand-rolled scripts** — fragile, no history table, no rollback discipline.

## Consequences
- Positive: zero non-Rust runtime dependency in the ingestion image; migrations versioned with the same git SHA as the service binary.
- Positive: migration tests run against both SQLite and PostgreSQL in CI from the same fixture set.
- Negative: no autogeneration — every migration is hand-written. We treat this as a feature for a contract-first product.
- Negative: dialect divergences must be handled explicitly. We standardize on a small subset of portable SQL plus dialect-specific files when necessary.
- Follow-ups: ship `heec migrate up|status|verify` CLI commands per ADR-0010 (delivered in the foundation slice); document the migration authoring guide in `docs/architecture/data-model.md`.

## Amendment 2026-04-23 — `aih_schema_migrations` view

`sqlx::migrate!` owns the underlying history table named `_sqlx_migrations`. PRD §20 and the storage README expose a stable view named `aih_schema_migrations` (created in migration `0001_init.sql`) that aliases the columns we promise externally. SDKs and the dashboard MUST query the view, not the underlying table; the view is the contract, the table is implementation detail. Migration scripts that change history-table semantics require an ADR amendment here.

## References
- PRD §12.20 Database Schema Migrations
- PRD §20 Storage and Data Model
- ADR-0005 Ingestion Service Language

# Data model

> Status: foundation slice. Updated each plan increment.
> Last reviewed: 2026-04-24. Owner: Tech Lead + Security Engineer.

ai-heeczer uses an **append-only, workspace-scoped event and score model**.
Events are immutable once written; scores are immutable per `(workspace_id, event_id, scoring_version, scoring_profile_id, profile_version)`.
Re-scoring inserts a new row rather than updating an old one.
Hard-deletion honors data-subject requests via the `heec_tombstones` table. The tombstone records that an event was removed; it does not mean the raw event row survives.

Two storage backends share one migration history:

| Backend    | Use case               | Dialect notes                                |
| ---------- | ---------------------- | -------------------------------------------- |
| SQLite     | Local development, CLI | `strftime` timestamps, trigger-based guards  |
| PostgreSQL | Production             | `NOW()` defaults, PL/pgSQL trigger functions |

References: [ADR-0004](../adr/0004-database-migration-tooling.md), [plan 0003](../plan/0003-storage-and-migrations.md)

---

## Table catalog

All table names carry the `heec_` prefix (the implementation prefix; earlier implementation prefix was `aih_`).

| Table                    | Purpose                                                            |
| ------------------------ | ------------------------------------------------------------------ |
| `heec_workspaces`        | Tenant root; every other table FK's to this                        |
| `heec_api_keys`          | Per-workspace hashed API keys with revocation                      |
| `heec_events`            | Append-only normalized events, keyed by `(workspace_id, event_id)` |
| `heec_scores`            | Append-only scored results, versioned per scoring config           |
| `heec_jobs`              | Queue rows for async/image-mode workers (ADR-0006)                 |
| `heec_idempotency_keys`  | Batch `Idempotency-Key` replay cache scoped by workspace           |
| `heec_tiers`             | Append-only tier sets with `effective_at` ranges                   |
| `heec_rates`             | Append-only rate tables with `effective_at` ranges                 |
| `heec_scoring_profiles`  | Append-only profiles with `effective_at` ranges                    |
| `heec_audit_log`         | Every config change and re-scoring event (PRD §22.5)               |
| `heec_daily_aggregates`  | Derived rollups (workspace, day, project, category)                |
| `heec_tombstones`        | Hard-deletion records for removed raw event rows (PRD §12.17)      |
| `heec_schema_migrations` | View over `_sqlx_migrations`; stable public contract               |

---

## Entity-relationship diagram

```mermaid
erDiagram
    heec_workspaces {
        TEXT workspace_id PK
        TEXT display_name
        TEXT created_at
        TEXT settings_json
    }

    heec_api_keys {
        TEXT api_key_id PK
        TEXT workspace_id FK
        TEXT hashed_key
        TEXT label
        TEXT created_at
        TEXT revoked_at
    }

    heec_events {
        TEXT workspace_id PK
        TEXT event_id PK
        TEXT spec_version
        TEXT framework_source
        TEXT correlation_id
        TEXT payload
        TEXT payload_hash
        TEXT received_at
    }

    heec_scores {
        TEXT workspace_id PK
        TEXT event_id PK
        TEXT scoring_version PK
        TEXT scoring_profile_id PK
        TEXT profile_version PK
        TEXT tier_id
        TEXT tier_version
        TEXT rates_version
        TEXT result_json
        TEXT final_minutes
        TEXT final_fec
        TEXT confidence
        TEXT confidence_band
        TEXT created_at
    }

    heec_jobs {
        TEXT job_id PK
        TEXT workspace_id FK
        TEXT event_id
        TEXT state
        INTEGER attempts
        TEXT last_error
        TEXT available_at
        TEXT enqueued_at
        TEXT finished_at
    }

    heec_idempotency_keys {
        TEXT workspace_id PK
        TEXT idempotency_key PK
        TEXT request_hash
        INTEGER status_code
        TEXT response_body
        TEXT created_at
        TEXT expires_at
    }

    heec_tiers {
        TEXT tier_set_id PK
        TEXT version PK
        TEXT workspace_id PK
        TEXT tiers_json
        TEXT effective_at
        TEXT superseded_at
    }

    heec_rates {
        TEXT rates_id PK
        TEXT version PK
        TEXT workspace_id PK
        TEXT rates_json
        TEXT currency
        TEXT effective_at
        TEXT superseded_at
    }

    heec_scoring_profiles {
        TEXT scoring_profile_id PK
        TEXT version PK
        TEXT workspace_id PK
        TEXT profile_json
        TEXT effective_at
        TEXT superseded_at
    }

    heec_audit_log {
        TEXT audit_id PK
        TEXT workspace_id FK
        TEXT actor
        TEXT action
        TEXT target_table
        TEXT target_id
        TEXT payload_json
        TEXT created_at
    }

    heec_daily_aggregates {
        TEXT workspace_id PK
        TEXT day PK
        TEXT project_id PK
        TEXT category PK
        TEXT framework_source PK
        INTEGER event_count
        TEXT total_minutes
        TEXT total_fec
    }

    heec_tombstones {
        TEXT workspace_id PK
        TEXT event_id PK
        TEXT deleted_at
        TEXT reason
    }

    heec_workspaces ||--o{ heec_api_keys : "has"
    heec_workspaces ||--o{ heec_events : "scopes"
    heec_workspaces ||--o{ heec_scores : "scopes"
    heec_workspaces ||--o{ heec_jobs : "queues"
    heec_workspaces ||--o{ heec_idempotency_keys : "caches"
    heec_workspaces ||--o{ heec_audit_log : "logs"
    heec_workspaces ||--o{ heec_daily_aggregates : "aggregates"
    heec_workspaces ||--o{ heec_tombstones : "tombstones"
    heec_events ||--o{ heec_scores : "scored by"
```

---

## Key design decisions

### Append-only enforcement

`heec_events`, `heec_scores`, and `heec_audit_log` carry `BEFORE UPDATE` and `BEFORE DELETE`
triggers that call `RAISE(ABORT, ...)` (SQLite) or `RAISE EXCEPTION` (PostgreSQL).
This makes accidental mutation fail loudly at the database level, independent of
application logic.

Re-scoring inserts a new `heec_scores` row with a different `scoring_version`
or `scoring_profile_id` — it never touches the prior row.

### Batch idempotency replay cache

`heec_idempotency_keys` stores the original batch request hash and response body for each `(workspace_id, idempotency_key)` pair. A replay with the same request hash returns the stored response byte-for-byte until `expires_at`; a replay with a different request hash returns `409 conflict`.

This table is intentionally not append-only because idempotency keys are a bounded replay cache, not a historical business record. Operational cleanup may delete expired rows without affecting event or score immutability.

### Composite primary key for scores

```sql
PRIMARY KEY (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version)
```

Each component of the key answers a distinct reproducibility question:

| Column               | Question answered                                   |
| -------------------- | --------------------------------------------------- |
| `workspace_id`       | Which tenant produced this score?                   |
| `event_id`           | Which event was scored?                             |
| `scoring_version`    | Which engine version ran?                           |
| `scoring_profile_id` | Which profile (component weights, multipliers) ran? |
| `profile_version`    | Which revision of that profile was active?          |

For ingestion with the default profile and tier set, `profile_version` is the profile semver. For explicit re-score requests with non-default profile, tier set, or tier override inputs, the ingestion service appends a deterministic configuration hash to `profile_version` so the append-only score identity includes all inputs that can affect the score.

### Hard-delete tombstones

`heec_tombstones` marks an event that was hard-deleted for data-subject or privacy reasons. Storage admin paths insert the tombstone, delete associated scores, delete the raw event row, and preserve an audit trail without retaining the raw event payload.

### Nullable `workspace_id` on config tables

`heec_scoring_profiles`, `heec_tiers`, and `heec_rates` allow a `NULL` `workspace_id`
to represent a **global (system-default) row**. SQLite treats two `NULL` values as
distinct in a `PRIMARY KEY`, creating a hole where two identical global rows can
coexist. Migration `0002` closes this with expression indexes:

```sql
CREATE UNIQUE INDEX uq_heec_scoring_profiles_global
    ON heec_scoring_profiles (scoring_profile_id, version, COALESCE(workspace_id, ''));
```

The same `COALESCE` sentinel works on PostgreSQL, keeping both dialects identical.

### Migration history view

`sqlx::migrate!` manages `_sqlx_migrations` internally. Rather than exposing
that implementation detail, migration `0001` creates:

```sql
CREATE VIEW heec_schema_migrations AS
    SELECT version, description, installed_on, success
    FROM _sqlx_migrations;
```

SDKs and the dashboard query `heec_schema_migrations`. The view is the contract;
`_sqlx_migrations` is an implementation detail of ADR-0004.

---

## Multi-tenancy

Every query that touches tenant data **must** carry a `workspace_id` parameter.
The target repository layer wraps raw `String` IDs in a `WorkspaceId` newtype so missing scopes become compile errors. That type-level enforcement is still tracked in plan 0003; until it lands, handler and storage queries must continue to pass explicit workspace scopes.

Tenant isolation checklist for new queries:

1. `WHERE workspace_id = $1` on every `SELECT`, `INSERT`, and `UPDATE`.
2. Foreign-key cascades in the schema prevent orphan rows but do not substitute for explicit scoping in reads.
3. `heec_audit_log` entries record the acting workspace even for global config changes.

---

## Migration authoring guide

### File naming

Migrations are **forward-only, sequentially numbered**:

```text
core/heeczer-storage/migrations/          ← SQLite dialect
    0001_init.sql
    0002_append_only_audit_and_global_unique.sql
    <NNNN>_<slug>.sql

core/heeczer-storage/migrations-pg/       ← PostgreSQL dialect
    0001_init.sql
    0002_append_only_audit_and_global_unique.sql
    <NNNN>_<slug>.sql
```

The `<NNNN>` counter must match between the two trees. Gaps are not allowed.

### Adding a new migration

1. Pick the next integer (e.g., `0003`).
2. Author `migrations/0003_<slug>.sql` for SQLite. Use:
    - `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` for timestamps (not `CURRENT_TIMESTAMP`)
    - `TEXT` for all temporal columns (ISO-8601 strings)
    - `RAISE(ABORT, '...')` in `BEFORE UPDATE/DELETE` triggers
3. Author `migrations-pg/0003_<slug>.sql` for PostgreSQL. Use:
    - `NOW()` for timestamps
    - `TIMESTAMPTZ` for temporal columns
    - PL/pgSQL `CREATE FUNCTION` + `CREATE TRIGGER` for append-only guards
4. Run `heec migrate up` locally against both backends.
5. Add or update tests in `core/heeczer-storage/tests/` to cover new tables.
6. The `migration.yml` CI workflow validates fresh-install and incremental-upgrade on both backends.

### Dialect portability rules

Stick to this portable subset whenever possible:

- `CREATE TABLE`, `CREATE INDEX`, `CREATE UNIQUE INDEX`, `CREATE VIEW`
- `INTEGER`, `TEXT` column types (SQLite) / `BIGINT`, `TEXT`, `TIMESTAMPTZ` (PostgreSQL)
- `PRIMARY KEY (col1, col2, ...)` composite keys
- `FOREIGN KEY ... REFERENCES ... ON DELETE CASCADE`
- `CHECK (col IN ('a', 'b', 'c'))` constraints

Diverge only where unavoidable (trigger syntax, `COALESCE` in indexes, `CONCURRENTLY` for PostgreSQL online builds). Mark divergent blocks with a `-- dialect: sqlite` or `-- dialect: pg` comment.

### What you must not do

- Do not use `ALTER TABLE ... DROP COLUMN` on SQLite (not supported before SQLite 3.35, and not safe to assume in all CI images).
- Do not `UPDATE` or `DELETE` from append-only tables in a migration — use a new insert + tombstone instead.
- Do not reference `_sqlx_migrations` directly from application code; use the `heec_schema_migrations` view.

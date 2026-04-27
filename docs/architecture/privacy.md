# Privacy architecture

> Status: foundation slice — reflects PRD §22 and plan 0014.
> Last reviewed: 2026-04-24
> Owner: Security Engineer

## Privacy by default

ai-heeczer is designed so that persisted events contain telemetry and metadata,
not raw prompt or model content. The canonical event schema leaves no open
content fields outside `meta.extensions`, and `meta.extensions` now rejects
privacy-sensitive key names for prompt, output, attachment, secret, token, and
API-key-like content (PRD §22).

The core assertion: **what an AI agent said or was told is never stored**.
What is stored is only the telemetry envelope — token counts, durations,
categories, and identifiers — sufficient to compute HEE (Human Equivalent
Effort) and FEC (Financial Equivalent Cost) without re-reading the
conversation.

## Data classification

| Tier | Label        | Example fields                                                    | Handling                                                                                                       |
| ---- | ------------ | ----------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| 1    | Restricted   | raw prompt text, model response text, user PII beyond identifiers | Not stored; typed schema fields exclude it and `meta.extensions` rejects privacy-sensitive key names on ingest |
| 2    | Confidential | `workspace_id`, `user_id`, API keys (hashed), audit log entries   | Encrypted at rest; access gated by RBAC admin                                                                  |
| 3    | Internal     | `event_id`, `task.category`, `scoring_version`, score breakdowns  | Stored in append-only tables; workspace-scoped                                                                 |
| 4    | Public       | aggregate statistics, benchmark reference payloads, schema JSON   | Published openly; no workspace attribution                                                                     |

## What is collected

The following fields are recorded per ingested event:

| Field                       | Type      | Notes                                              |
| --------------------------- | --------- | -------------------------------------------------- |
| `event_id`                  | UUID      | Idempotency key; deduplicates on ingest            |
| `workspace_id`              | UUID      | All queries scoped by this value                   |
| `user_id`                   | string    | Workspace-local identifier; no external PII lookup |
| `spec_version`              | string    | Schema version (e.g., `"1.0"`)                     |
| `task.category`             | string    | Enum or `uncategorized`; no free-form user text    |
| `task.outcome`              | string    | `success`, `failure`, `partial`, `timeout`         |
| `metrics.tokens_prompt`     | integer   | Token count only; no prompt content                |
| `metrics.tokens_completion` | integer   | Token count only; no completion content            |
| `metrics.duration_ms`       | integer   | Wall-clock milliseconds                            |
| `metrics.tool_calls`        | integer   | Count of tool invocations                          |
| `metrics.retry_count`       | integer   | Number of retries                                  |
| `created_at`                | timestamp | Server-assigned ingest time                        |

## What is NOT collected

- Prompt text or model response text.
- Free-form user messages, file contents, or clipboard data.
- User PII beyond the workspace-local `user_id` string (no email, name, or
  external account identifiers).
- Model provider credentials or API keys passed through event payloads.
- IP addresses of event originators (gateway may log for rate limiting; not
  forwarded to storage).

## Log and audit hygiene

- Application logs never include event payload bodies. Log lines reference
  `event_id` only.
- Structured log fields are limited to: `event_id`, `workspace_id`, `action`,
  `result`, `duration_ms`, and `timestamp`.
- Audit log entries for sensitive actions contain: `workspace_id`, `actor_id`,
  `action`, `target_id`, and `timestamp`. No payload content.
- Log pipelines are configured with an allow-list of fields; unknown fields are
  dropped, not forwarded.

## Retention policy

Retention is configurable per workspace. The defaults are:

| Data type         | Default retention window |
| ----------------- | ------------------------ |
| Raw event rows    | 90 days                  |
| Score rows        | 90 days                  |
| Audit log entries | 365 days                 |
| Tombstone records | Indefinite               |

Workspace administrators can shorten retention via the workspace settings API.
Extending beyond the default requires a platform-level configuration override.

### Hard-delete API and tombstone semantics (PRD §12.17)

When a data-subject deletion request arrives:

1. A tombstone row is inserted into `heec_tombstones` with `workspace_id`,
   `subject_id`, `requested_at`, and `reason`.
2. Score aggregates are anonymized: `user_id` replaced with a stable hash
   derived from the tombstone record. HEE/FEC totals are preserved for
   workspace accounting.
3. Raw `heec_events` and `heec_scores` rows for the subject are swept by the
   retention job and hard-deleted.
4. An audit log entry records the completion: `action=hard_delete`,
   `workspace_id`, `tombstone_id`, `deleted_row_count`, `timestamp`.

The `heec_tombstones` table is itself append-only and is never deleted. It
serves as the audit trail proving the deletion occurred.

## Workspace isolation

Every query against `heeczer-storage` is parameterized on `workspace_id`.
The storage layer enforces this at the query level — no cross-workspace
joins are possible in the schema. API endpoints validate that the
authenticated workspace matches the request path parameter before any query
is issued.

No shared mutable state exists across workspaces. Scoring profiles and tier
sets are workspace-scoped; there are no global mutable records.

## Data residency

| Deployment mode    | Storage location                                                                 |
| ------------------ | -------------------------------------------------------------------------------- |
| Local (`heec` CLI) | SQLite file on the developer's local disk                                        |
| Self-hosted server | PostgreSQL instance under the operator's control                                 |
| Cloud (future)     | Region-pinned per workspace; no cross-region replication without explicit opt-in |

SQLite deployments store data only where the CLI is run. PostgreSQL
deployments are operator-controlled; ai-heeczer does not prescribe a
cloud provider or region. The data residency commitment is that the platform
does not move data to a region other than the one configured at workspace
creation.

## References

- PRD §22 — Privacy and data classification requirements
- PRD §12.17 — Retention and deletion
- [Plan 0014 — Security and privacy](../plan/0014-security-and-privacy.md)
- [`heeczer-storage` migrations](../../core/heeczer-storage/migrations/)
- [Security architecture](security.md)

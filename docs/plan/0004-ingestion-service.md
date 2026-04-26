# Plan 04 — Ingestion Service

- **Status:** Implemented foundation slice; production gaps tracked below
- **Owner:** Tech Lead
- **PRD:** §12.4, §12.16, §12.18, §12.19, §19, §29
- **ADR:** ADR-0002, ADR-0005, ADR-0006

## Goal

Ship the Rust ingestion service that accepts events via HTTP and queue, validates against the canonical schema, scores via the core, persists, and responds with auditable envelopes.

## Checklist

### Service Scaffolding

- [x] `services/heeczer-ingest/` cargo binary using `axum`, `tokio`, and `sqlx`.
- [x] Layered config via `figment` with env/file/default support.
- [x] Structured JSON logging via `tracing` and `tracing-subscriber`; ingest, batch, and rescore paths emit structured workspace/event/correlation/request fields.
- [x] Prometheus metrics endpoint at `GET /metrics` via `axum-prometheus`.

### HTTP API (PRD §12.16)

- [x] `POST /v1/events` — single event ingest, sync ack, schema validation, scoring, persistence, duplicate replay, and conflict detection.
- [x] `POST /v1/events:batch` — batch ingest with partial-success semantics, 100 event cap, single transaction for accepted rows, and idempotency replay.
- [x] `GET /v1/events/{event_id}` — workspace-scoped event read.
- [x] `GET /v1/events/{event_id}/scores` — workspace-scoped score-version list with 404 for missing events.
- [x] `POST /v1/events/{event_id}/rescore` — append-only re-score for existing events. This uses a slash route rather than Google API-style colon notation because axum does not support colon custom methods on parameterized path segments.
- [x] `GET /v1/jobs/{job_id}` — workspace-scoped job status read.
- [x] `GET /healthz`, `GET /v1/ready`, `GET /v1/version`, and `GET /metrics`.
- [x] `POST /v1/test/score-pipeline` — ADR-0012 dashboard test-orchestration route, feature-flagged and gated by `x-heeczer-tester`.
- [x] `GET /openapi.yaml` serves a checked-in OpenAPI 3.1 contract under `services/heeczer-ingest/openapi.yaml`. Generation from code annotations remains open.
- [x] Unsupported `spec_version` values return `415 Unsupported Media Type` with `Supported-Spec-Versions: 1.0` on single-event and batch ingest.
- [x] ADR-0002 amendment: ratified `415 Unsupported Media Type`, `unsupported_spec_version`, and the `Supported-Spec-Versions` response header for unsupported `spec_version` values. (session Apr-2026)

### Auth

- [x] API-key middleware: `x-heeczer-api-key` SHA-256 hash lookup against `heec_api_keys.hashed_key`, revocation check, workspace scoping, `AuthContext` injection, and `auth_failed` audit rows.
- [x] mTLS deployment guidance documented: current production guidance is edge proxy/service mesh mTLS plus API-key auth; native Rust service mTLS termination is deferred to plan 0014.

### Rate Limiting And Quotas (PRD §12.18)

- [x] Per-API-key token bucket via `tower-governor`, plus deterministic per-key limiter for authenticated quota headers and tests.
- [x] Per-workspace daily quota check from `heec_workspaces.settings_json.daily_event_quota`, falling back to runtime config.
- [x] Payload limits: 64 KiB for single-event ingest and 1 MiB for batch before JSON parsing; router cap mirrors the batch cap.
- [x] 429 responses include `Retry-After`, `X-Heeczer-Quota-Limit`, `X-Heeczer-Quota-Remaining`, and `X-Heeczer-Quota-Reset-After`.

### Idempotency (PRD §12.19)

- [x] `event_id` dedup by primary key and payload hash: exact duplicates replay the existing score, conflicting duplicates return `409 conflict` and write `ingest_conflict` audit entries. Insert-race losers re-check the stored hash before returning.
- [x] Batch `Idempotency-Key` cache with 24h default TTL in `heec_idempotency_keys`.
- [x] Replayed batch responses are byte-equal to the original response body.

### Queue Worker (Image Mode, ADR-0006)

- [x] `JobQueue` trait and PostgreSQL `FOR UPDATE SKIP LOCKED` implementation for enqueue, claim, complete, fail, and stats.
- [x] Generic worker loop with idle backoff, retry handoff, DLQ transition through queue policy, and shutdown future hook.
- [ ] Queue worker startup is not wired into the binary.
- [ ] Visibility metrics for queue depth, queue age p95, retries, and DLQ count are not exported to Prometheus yet.
- [ ] Full in-flight drain orchestration is not implemented; the worker loop exits on shutdown but is not service-managed.

### Persistence (Plan 03)

- [x] Plan 04 HTTP handlers scope reads and writes by `workspace_id`; authenticated callers cannot cross workspaces.
- [x] Append-only invariants remain enforced in DB for events, scores, and audit log. The idempotency cache is intentionally expirable operational data.
- [x] Re-score creates a new score row when the scoring tuple is new and preserves the original; non-default profile/tier inputs use a deterministic profile-version identity key, and no-op duplicate re-score does not write a phantom audit entry.
- [ ] Runtime HTTP PostgreSQL pool switching is not wired; the binary still opens SQLite while PostgreSQL migrations and queue abstractions exist.

### Performance (PRD §29)

- [ ] Native-mode `track()` path benched; <2 ms p95.
- [ ] Image-mode async ack path benched; <50 ms p95 same-region.
- [ ] Throughput bench targets ≥10k accepted enqueues/s/node.
- [ ] Bench profile documented in `docs/architecture/benchmarks.md` with payload size, auth mode, durability mode, queue backend, storage backend, and hardware.

### Tests

- [x] 32 ingestion integration tests cover healthz, version, ready, ingest validation, oversized payloads, workspace ID validation, duplicate replay, conflict rejection/audit, auth required/audited, workspace scoping, rate-limit headers, workspace quota headers, batch byte-equal replay, OpenAPI exposure, single/batch 415 spec negotiation, dashboard test route gates, rescore success/404/tier-override persistence/no-op audit, job reads, correlation-id persistence, and missing-score 404 behavior.
- [x] Integration: end-to-end ingest -> score -> persist for single event, batch, rescore, and job-status read-back.
- [ ] Contract: OpenAPI spec validated against live handler responses. Static spec exposure is covered; full request/response contract tests remain open.
- [ ] Property: idempotency invariants under concurrent duplicate POSTs.
- [ ] Load test (k6) wired into nightly CI for trend tracking.

### Docs

- [x] `services/heeczer-ingest/README.md` documents endpoints, config, auth, rate/quota/idempotency behavior, queue status, and tests.
- [x] `docs/architecture/deployment-modes.md` documents native vs image flows plus `/healthz` liveness and `/v1/ready` readiness probe mapping with Kubernetes examples.
- [x] `docs/architecture/security.md` documents the implemented error envelope, auth behavior, rate/quota headers, conflict semantics, and current mTLS deployment stance.
- [x] `docs/architecture/data-model.md` documents `heec_idempotency_keys`.
- [x] `docs/runbooks/rescore.md` documents when to trigger re-score, how to verify audit entries, no-op behavior, and downstream notification expectations.

## Acceptance

- [ ] All NFR targets demonstrated under the documented bench profile.
- [ ] Required CI jobs green.
- [ ] Security review signs off on auth/rate-limit/idempotency semantics.
- [ ] OpenAPI contract passes generated client smoke tests (JS + Python).

## Remaining Production Gaps

- Runtime HTTP PostgreSQL pool switching is not wired; the binary still opens SQLite.
- Queue worker startup, queue Prometheus gauges, queue age p95, and in-flight drain orchestration are not connected to the service binary.
- OpenAPI is static YAML, not generated from code annotations.
- NFR/load testing and nightly k6 trend tracking are not implemented.
- Native Rust mTLS termination is not implemented; use edge proxy/service mesh mTLS for production until plan 0014 lands.

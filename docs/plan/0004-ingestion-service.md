# Plan 04 — Ingestion service

- **Status:** Active
- **Owner:** Tech Lead
- **PRD:** §12.4, §19, §29, §12.16, §12.18, §12.19
- **ADR:** ADR-0005, ADR-0006

## Goal
Ship the Rust ingestion service that accepts events via HTTP and queue, validates against the canonical schema, normalizes, scores via the core, persists, and responds — meeting the documented latency and throughput targets.

## Checklist

### Service scaffolding
- [x] `services/heeczer-ingest/` cargo binary using `axum` + `tokio` + `sqlx`. (skeleton landed; relocated under `services/` to match ADR-0007 layout)
- [ ] Layered config via `figment` (env + file + flags). (partial: env-only bootstrap shipped; file/flag layering pending)
- [x] Structured logging via `tracing` + `tracing-subscriber` JSON formatter. (foundation: env-filter format; JSON formatter and span propagation pending)
- [ ] Prometheus metrics endpoint via `axum-prometheus`.

### HTTP API (PRD §12.16)
- [x] `POST /v1/events` — single event ingest, sync ack. (validates against canonical schema, scores, persists `aih_events` + `aih_scores` in a single transaction; dedupe via PK + `INSERT OR IGNORE`)
- [ ] `POST /v1/events:batch` — batch ingest with `Idempotency-Key` (PRD §12.19).
- [ ] `GET /v1/events/{event_id}` — read.
- [ ] `GET /v1/events/{event_id}/scores` — list score versions.
- [ ] `POST /v1/events/{event_id}:rescore` — explicit re-score.
- [ ] `GET /v1/jobs/{job_id}` — job status.
- [x] `GET /healthz`, `GET /v1/version`. (`/metrics`, `/v1/ready` pending)
- [x] `POST /v1/test/score-pipeline` — dashboard test-orchestration entry point per ADR-0012; gated by `Features::test_orchestration` + `x-heeczer-tester` header. RBAC stub will be promoted to real role mapping in plan 0010 follow-up.
- [ ] OpenAPI spec under `services/heeczer-ingest/openapi.yaml`, generated from code annotations.
- [ ] `spec_version` negotiation: server advertises supported versions in `GET /v1/health`; events with unsupported `spec_version` are rejected with `415 Unsupported Media Type` and a `Supported-Spec-Versions` header.
- [ ] All tracing spans propagate `correlation_id` and `event_id` as span attributes.

### Auth
- [ ] API-key middleware: hashed lookup, workspace scoping, audit log entry on auth failure.
- [ ] Optional mTLS configuration documented.

### Rate limiting and quotas (PRD §12.18)
- [ ] Per-API-key token bucket via `tower-governor`.
- [ ] Per-workspace daily quota check from `aih_workspaces`.
- [ ] Payload size limits enforced at the body extractor.
- [ ] 429 responses include `Retry-After` and quota headers.

### Idempotency (PRD §12.19)
- [ ] `event_id` dedup by primary key.
- [ ] `Idempotency-Key` cache for batch endpoints (24h default TTL).
- [ ] Replayed responses are byte-equal to the original.

### Queue worker (image mode, ADR-0006)
- [ ] `JobQueue` trait; PostgreSQL `SKIP LOCKED` implementation.
- [ ] Worker loop with backoff, retry policy, DLQ.
- [ ] Visibility metrics: queue depth, queue age p95, retries, DLQ count.
- [ ] Graceful shutdown drains in-flight jobs.

### Persistence (Plan 03)
- [ ] Repository layer enforces workspace scoping.
- [ ] Append-only invariants enforced in code and DB.
- [ ] Re-score creates new score row; original preserved.

### Performance (PRD §29)
- [ ] Native-mode `track()` path benched; <2 ms p95.
- [ ] Image-mode async ack path benched; <50 ms p95 same-region.
- [ ] Throughput bench targets ≥10k accepted enqueues/s/node.
- [ ] Bench profile documented in `docs/architecture/benchmarks.md` (payload size, auth mode, durability mode, queue backend, storage backend, hardware).

### Tests
- [x] Unit per handler. (8 integration tests cover healthz, version, ingest happy path + invalid + missing workspace, test-pipeline feature-off / no-tester / happy)
- [x] Integration: end-to-end ingest → score → persist (single-event happy path; multi-event read endpoints pending)
- [ ] Contract: OpenAPI spec validated against handler responses.
- [ ] Property: idempotency invariants under concurrent duplicate POSTs.
- [ ] Load test (k6) wired into nightly CI for trend tracking.

### Docs
- [ ] `server/ingestion/README.md` with run, config, deploy.
- [ ] `docs/architecture/deployment-modes.md` updated for native vs image flows.

## Acceptance
- All NFR targets demonstrated under the documented bench profile.
- Required CI jobs green.

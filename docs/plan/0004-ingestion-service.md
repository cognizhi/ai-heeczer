# Plan 04 — Ingestion service

- **Status:** Active
- **Owner:** Tech Lead
- **PRD:** §12.4, §19, §29, §12.16, §12.18, §12.19
- **ADR:** ADR-0005, ADR-0006

## Goal
Ship the Rust ingestion service that accepts events via HTTP and queue, validates against the canonical schema, normalizes, scores via the core, persists, and responds — meeting the documented latency and throughput targets.

## Checklist

### Service scaffolding
- [ ] `server/ingestion/` cargo binary using `axum` + `tokio` + `sqlx`.
- [ ] Layered config via `figment` (env + file + flags).
- [ ] Structured logging via `tracing` + `tracing-subscriber` JSON formatter.
- [ ] Prometheus metrics endpoint via `axum-prometheus`.

### HTTP API (PRD §12.16)
- [ ] `POST /v1/events` — single event ingest, sync ack.
- [ ] `POST /v1/events:batch` — batch ingest with `Idempotency-Key` (PRD §12.19).
- [ ] `GET /v1/events/{event_id}` — read.
- [ ] `GET /v1/events/{event_id}/scores` — list score versions.
- [ ] `POST /v1/events/{event_id}:rescore` — explicit re-score.
- [ ] `GET /v1/jobs/{job_id}` — job status.
- [ ] `GET /v1/health`, `/v1/ready`, `/metrics`.
- [ ] OpenAPI spec under `server/ingestion/openapi.yaml`, generated from code annotations.
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
- [ ] Unit per handler.
- [ ] Integration: end-to-end ingest → score → persist → read.
- [ ] Contract: OpenAPI spec validated against handler responses.
- [ ] Property: idempotency invariants under concurrent duplicate POSTs.
- [ ] Load test (k6) wired into nightly CI for trend tracking.

### Docs
- [ ] `server/ingestion/README.md` with run, config, deploy.
- [ ] `docs/architecture/deployment-modes.md` updated for native vs image flows.

## Acceptance
- All NFR targets demonstrated under the documented bench profile.
- Required CI jobs green.

# ADR-0005: Ingestion Service Implementation Language

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §12.4, §19, §29, ADR-0001

## Context
The heeczer ingestion service (PRD §12.4, §19) accepts events via HTTP/webhook/queue, validates, normalizes, scores, persists, and responds to clients. NFRs require ≥10,000 accepted async enqueue req/s/node and <50 ms p95 ack latency in image mode (PRD §29).

## Decision
Implement the ingestion service in **Rust**, using:
- `axum` for HTTP
- `tokio` runtime
- `sqlx` for database access (PostgreSQL + SQLite)
- direct in-process linkage to `heeczer-core` (no FFI hop on the hot path)
- `tower-http` for tracing, CORS, rate-limiting middleware

## Alternatives Considered
- **Python (FastAPI / Litestar) + Alembic** — fastest team velocity, but FFI hop into the Rust core, GIL friction, weaker tail latency, larger image. Rejected.
- **Go** — strong concurrency, but second FFI surface in addition to the Go SDK; Rust eliminates that doubling.
- **Node** — comfortable for some maintainers, but unnecessary FFI hop and GC pauses against latency targets.

## Consequences
- Positive: single-process scoring, predictable tail latency, smallest container image, one toolchain shared with core.
- Negative: smaller pool of Rust web-service contributors than Python/Node.
- Follow-ups: contributor onboarding doc with ramp-up resources in CONTRIBUTING.md.

## References
- PRD §29 Non-Functional Requirements
- ADR-0001 Rust Core

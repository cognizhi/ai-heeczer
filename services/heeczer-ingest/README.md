# heeczer-ingest

The ai-heeczer ingestion service (ADR-0005, plan 0004).

Validates, scores, and persists developer-activity events.
All five SDK bindings (JS, Python, Go, Java, Rust) talk to this service.

## Endpoints

| Method | Path                            | Description                                                              |
| ------ | ------------------------------- | ------------------------------------------------------------------------ |
| `GET`  | `/healthz`                      | Liveness probe, process-only                                             |
| `GET`  | `/v1/ready`                     | Readiness probe, checks DB reachability                                  |
| `GET`  | `/v1/version`                   | Engine + spec versions                                                   |
| `GET`  | `/openapi.yaml`                 | OpenAPI 3.1 contract                                                     |
| `GET`  | `/metrics`                      | Prometheus metrics scrape endpoint                                       |
| `POST` | `/v1/events`                    | Validate + score + persist a single event                                |
| `POST` | `/v1/events:batch`              | Validate + score + persist up to 100 events, partial success             |
| `GET`  | `/v1/events/{event_id}`         | Read a stored event                                                      |
| `GET`  | `/v1/events/{event_id}/scores`  | List score versions for an event                                         |
| `POST` | `/v1/events/{event_id}/rescore` | Append-only re-score of an existing event                                |
| `GET`  | `/v1/jobs/{job_id}`             | Queue job status                                                         |
| `POST` | `/v1/test/score-pipeline`       | Score-without-persist, for the test-orchestration dashboard view (gated) |

## Request body — `POST /v1/events`

```text
{
    "workspace_id": "my-workspace",
    "event": {
        ... canonical event per core/schema/event.v1.json ...
    }
}
```

`workspace_id` is the tenant scope for storage and auth. It must be 1-64 ASCII alphanumeric, dot (`.`), dash (`-`), or underscore (`_`) characters. When API-key auth is enabled, it must match the workspace attached to `x-heeczer-api-key`. The service normalizes the nested canonical `event.workspace_id` to this wrapper value before validation and persistence.

## Request body — `POST /v1/events:batch`

```text
{
    "workspace_id": "my-workspace",
    "events": [
        ... canonical event per core/schema/event.v1.json ...
    ]
}
```

Batches are capped at 100 events and 1 MiB. `Idempotency-Key` is supported for 24-hour byte-equal response replay.

## Environment variables

| Variable                                | Default                      | Description                                     |
| --------------------------------------- | ---------------------------- | ----------------------------------------------- |
| `HEECZER_LISTEN`                        | `0.0.0.0:8080`               | TCP listen address                              |
| `HEECZER_DATABASE_URL`                  | `sqlite:heeczer.db?mode=rwc` | SQLite DSN used by the HTTP service             |
| `HEECZER_AUTH__ENABLED`                 | `true`                       | Require `x-heeczer-api-key` on protected routes |
| `HEECZER_RATE_LIMIT__REFILL_PER_SECOND` | `17`                         | Per-key token refill rate                       |
| `HEECZER_RATE_LIMIT__BURST_SIZE`        | `200`                        | Per-key token bucket burst                      |
| `HEECZER_PAYLOAD_LIMITS__EVENT_BYTES`   | `65536`                      | Single-event payload limit                      |
| `HEECZER_PAYLOAD_LIMITS__BATCH_BYTES`   | `1048576`                    | Batch payload limit                             |
| `HEECZER_IDEMPOTENCY__RETENTION_HOURS`  | `24`                         | Batch idempotency replay window                 |
| `HEECZER_QUOTAS__DAILY_EVENTS`          | `5000000`                    | Default workspace daily event quota             |
| `HEECZER_FEATURES__TEST_ORCHESTRATION`  | `false`                      | Enable `/v1/test/score-pipeline`                |
| `RUST_LOG`                              | `info`                       | Log level filter (tracing env filter syntax)    |

## Run locally

```bash
# SQLite (default — no external DB needed)
cargo run -p heeczer-ingest

# Local unauthenticated smoke test mode
HEECZER_AUTH__ENABLED=false cargo run -p heeczer-ingest

# PostgreSQL migrations and queue worker support live in heeczer-storage and
# the Postgres JobQueue implementation. The HTTP service still opens SQLite in
# this slice; production HTTP Postgres pool switching is tracked under plan 0003.
```

## Security notes

- Protected routes require `x-heeczer-api-key` when `HEECZER_AUTH__ENABLED=true`.
- API keys are stored as SHA-256 hashes in `heec_api_keys.hashed_key`; raw keys are never logged.
- Auth failures write `auth_failed` rows to `heec_audit_log` without storing raw key material.
- Request bodies are limited to **64 KiB** for `/v1/events` and **1 MiB** for batch endpoints.
- Storage and scoring errors are **not** surfaced in HTTP responses; they are logged server-side only.
- `workspace_id` is validated against an allowlist to prevent control-character injection.
- Duplicate `event_id` with the same normalized payload replays the stored score. Duplicate `event_id` with a different payload returns `409 conflict` and writes an `ingest_conflict` audit row.
- Per-key token buckets return `429` with `Retry-After` and quota headers. Workspace daily quota checks read `heec_workspaces.settings_json.daily_event_quota` and fall back to `HEECZER_QUOTAS__DAILY_EVENTS`.
- The `/v1/test/score-pipeline` endpoint requires both the process-level feature flag **and** the `x-heeczer-tester: 1` request header.

## Error envelope

All non-2xx responses follow this shape (ADR-0011):

```json
{
    "ok": false,
    "envelope_version": "1",
    "error": {
        "kind": "bad_request",
        "message": "human-readable detail"
    }
}
```

Valid `kind` values include `schema`, `bad_request`, `scoring`, `storage`, `not_found`, `unauthorized`, `forbidden`, `conflict`, `payload_too_large`, `rate_limit_exceeded`, `feature_disabled`, `unsupported_spec_version`, and `unavailable`.

Unsupported `spec_version` values return `415 Unsupported Media Type` with `Supported-Spec-Versions: 1.0`.

## Queue worker

ADR-0006's default queue backend is represented by library-level `PostgresJobQueue` and `run_worker` pieces in the service crate. They claim work with `FOR UPDATE SKIP LOCKED`, track attempts, retry failed jobs with backoff, and move exhausted jobs to `dead_letter`. The binary does not start the worker yet; runtime PostgreSQL pool switching, queue startup, and queue metrics remain production wiring work.

## Tests

```bash
cargo test -p heeczer-ingest
```

Integration tests construct a real in-memory SQLite pool and drive the router via `tower::ServiceExt::oneshot` with no network listener. Coverage includes auth, quota, batch idempotency, OpenAPI exposure, conflict audit logging, readiness, rescore, and job reads.

## Deploy

### Docker (single container)

```dockerfile
FROM rust:1.88-slim AS builder
WORKDIR /src
COPY . .
RUN cargo build -p heeczer-ingest --release

FROM debian:bookworm-slim
COPY --from=builder /src/target/release/heeczer-ingest /usr/local/bin/
EXPOSE 8080
CMD ["heeczer-ingest"]
```

### Docker Compose (SQLite volume)

```yaml
services:
    ingest:
        build: .
        ports:
            - "8080:8080"
        environment:
            HEECZER_DATABASE_URL: "sqlite:/data/heeczer.db?mode=rwc"
        volumes:
            - ingest-data:/data

volumes:
    ingest-data:
```

PostgreSQL dialect migrations and the `PostgresJobQueue` implementation are present, but runtime HTTP PostgreSQL pool switching is not wired in this slice.

See [docs/architecture/deployment-modes.md](../../docs/architecture/deployment-modes.md)
for the native vs image deployment trade-offs.

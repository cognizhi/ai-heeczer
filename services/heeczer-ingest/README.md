# heeczer-ingest

The ai-heeczer ingestion service (ADR-0005, plan 0004).

Validates, scores, and persists individual developer-activity events.
All five SDK bindings (JS, Python, Go, Java, Rust) talk to this service.

## Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/healthz` | Liveness probe — always returns `200 {"ok":true}` |
| `GET` | `/v1/version` | Engine + spec versions |
| `POST` | `/v1/events` | Validate + score + persist a single event |
| `GET` | `/metrics` | Prometheus metrics scrape endpoint |
| `POST` | `/v1/test/score-pipeline` | Score-without-persist, for the test-orchestration dashboard view (gated) |

## Request body — `POST /v1/events`

```json
{
  "workspace_id": "my-workspace",
  "event": { /* canonical event per core/schema/event.v1.json */ }
}
```

`workspace_id` must be 1–128 ASCII alphanumeric, dash (`-`), or underscore (`_`) characters.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `HEECZER_DATABASE_URL` | `sqlite::memory:` | SQLite or Postgres DSN |
| `HEECZER_INGEST_BIND` | `0.0.0.0:8080` | TCP listen address |
| `HEECZER_FEATURE_TEST_ORCHESTRATION` | _(unset)_ | Set to `1`, `true`, or `on` to enable `/v1/test/score-pipeline` |
| `RUST_LOG` | `info` | Log level filter (tracing env filter syntax) |

## Run locally

```bash
# SQLite (default — no external DB needed)
cargo run -p heeczer-ingest

# Postgres
HEECZER_DATABASE_URL=postgres://user:pass@localhost/heeczer \
  cargo run -p heeczer-ingest
```

## Security notes

- Request bodies are limited to **1 MiB** (`RequestBodyLimitLayer`).
- Storage and scoring errors are **not** surfaced in HTTP responses; they are logged server-side only.
- `workspace_id` is validated against an allowlist to prevent control-character injection.
- The `/v1/test/score-pipeline` endpoint requires both the process-level feature flag **and** the `x-heeczer-tester: 1` request header.
- Authentication (API-key middleware against `heec_api_keys.hashed_key`) is tracked in plan 0004 and is **not** active in the current bootstrap build.

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

Valid `kind` values: `schema`, `bad_request`, `scoring`, `storage`, `not_found`, `forbidden`, `feature_disabled`.

## Tests

```bash
cargo test -p heeczer-ingest
```

Integration tests construct a real in-memory SQLite pool and drive the router via `tower::ServiceExt::oneshot` with no network listener.

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

### Docker Compose (with PostgreSQL)

```yaml
services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: heeczer
      POSTGRES_USER: heeczer
      POSTGRES_PASSWORD: heeczer
    volumes:
      - pgdata:/var/lib/postgresql/data

  ingest:
    build: .
    ports:
      - "8080:8080"
    environment:
      HEECZER_DATABASE_URL: "postgres://heeczer:heeczer@db:5432/heeczer"
    depends_on:
      - db

volumes:
  pgdata:
```

See [docs/architecture/deployment-modes.md](../../docs/architecture/deployment-modes.md)
for the native vs image deployment trade-offs.

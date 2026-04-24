# Deployment Modes

ai-heeczer supports two scoring deployment modes:

|              | **Native (in-process)**          | **Image (HTTP)**                        |
| ------------ | -------------------------------- | --------------------------------------- |
| Latency      | < 1 ms (no network hop)          | < 10 ms (local LAN)                     |
| Isolation    | Same process as the caller       | Separate container / service            |
| Persistence  | None (caller-managed)            | SQLite or PostgreSQL via heeczer-ingest |
| Multi-tenant | No — single workspace per caller | Yes — `workspace_id` per request        |
| Auth         | N/A                              | API-key middleware (plan 0004 §auth)    |
| Scaling      | Vertical (thread pool)           | Horizontal (replica set)                |

---

## Native mode

The caller links `heeczer-core` (Rust) or the native SDK binding directly.
Scoring runs synchronously in-process with zero network overhead.

### When to use

- Single-workspace CLI tools and scripts.
- Performance-critical hot paths where < 1 ms matters.
- Environments without network egress.
- Development / local smoke-testing.

### How it works

````text
caller binary
  └── heeczer SDK (native feature)
        └── heeczer-core::score()   ← pure CPU, Decimal arithmetic
```text
### Rust example

```rust
use heeczer::{Client, IngestInput};

let client = Client::native();
let result = client.score_event(IngestInput {
    workspace_id: "ws_default".into(),
    event: my_event,
    profile: None,
    tier_set: None,
    tier_override: None,
})?;
```text
---

## Image (HTTP) mode

`heeczer-ingest` runs as a standalone HTTP service. Clients send events
over `POST /v1/events`; the service validates, scores, persists, and
returns the score envelope.

### When to use

- Multi-tenant SaaS deployments with many workspaces.
- Auditable pipelines (persistent event log in PostgreSQL).
- Dashboard-connected scoring (the Next.js dashboard reads from the same DB).
- Language-agnostic clients (JS, Python, Go, Java, Rust all speak the same HTTP API).

### How it works

```text
SDK client (any language)
  │
  │ POST /v1/events  {"workspace_id": "ws_…", "event": {…}}
  ▼
heeczer-ingest (axum HTTP service)
  ├── JSON schema validation (heeczer-core)
  ├── score() (heeczer-core)
  ├── INSERT OR IGNORE INTO scored_events (heeczer-storage / SQLx)
  └── 200 {"ok": true, "score": {…}}
```text
### Persistence backends

| Backend | URL format | Notes |
|---|---|---|
| SQLite | `sqlite:heeczer.db?mode=rwc` | Default. Single-file; fine for ≤ 100 rps. |
| PostgreSQL | `postgres://user:pass@host/db` | Recommended for production. |

Migrations are applied automatically at startup via `sqlx::migrate!()`.

### Metrics

Prometheus metrics are exposed at `GET /metrics`. Scrape interval
recommended: 15 s. Key metrics:

| Metric | Description |
|---|---|
| `axum_http_requests_total` | Total HTTP requests by method, path, status. |
| `axum_http_requests_duration_seconds` | Request latency histogram. |
| `axum_http_requests_pending` | In-flight requests gauge. |

---

## Choosing a mode

```text
Need persistent event log?         → Image mode
Multi-language callers?            → Image mode
< 1 ms scoring latency required?   → Native mode
No network in your environment?    → Native mode
Local development / CI tests?      → Native mode (or in-process SQLite)
```text
---

## Future: queue-backed ingestion (plan 0004 §queue)

ADR-0006 documents the planned NATS / Kafka queue backend. In that mode
the flow becomes:

```text
SDK client → POST /v1/events → heeczer-ingest → queue → scoring worker → DB
```text
The HTTP response returns immediately after enqueue; the score is
delivered asynchronously via a webhook or polling endpoint.
````

# Deployment Modes

ai-heeczer supports two scoring deployment modes.

|              | Native (in-process)              | Image (HTTP)                              |
| ------------ | -------------------------------- | ----------------------------------------- |
| Latency      | < 2 ms p95 target                | < 50 ms async ack p95 target              |
| Isolation    | Same process as caller           | Separate container / service              |
| Persistence  | Caller-managed SQLite or adapter | heeczer-ingest storage + queue tables     |
| Multi-tenant | Single workspace per caller      | `workspace_id` scoped by API key          |
| Auth         | Caller responsibility            | `x-heeczer-api-key` against hashed DB row |
| Scaling      | Host process scaling             | Horizontal service + worker replicas      |

## Native Mode

The caller links `heeczer-core` or a native SDK binding directly. Scoring runs synchronously in-process with no network hop.

Use native mode for single-workspace tools, local smoke tests, offline workloads, and paths where a network hop would dominate latency.

```text
caller binary
  -> heeczer SDK native mode
     -> heeczer-core::score()
```

## Image Mode

`heeczer-ingest` runs as a standalone HTTP service. Clients send canonical events to `/v1/events` or `/v1/events:batch`; the service validates, scores, persists, enforces auth/quotas/idempotency, and returns the score envelope.

```text
SDK client
  -> POST /v1/events
  -> heeczer-ingest
     -> schema validation
     -> heeczer-core::score()
     -> heec_events + heec_scores
```

Image mode is the production-readiness target for multi-tenant deployments, dashboard-connected scoring, and language-agnostic HTTP clients.

## Health Probes

Use `/healthz` for liveness and `/v1/ready` for readiness.

| Probe     | Endpoint    | Meaning                                    |
| --------- | ----------- | ------------------------------------------ |
| Liveness  | `/healthz`  | Process is alive; no dependency checks     |
| Readiness | `/v1/ready` | Service can accept traffic; database works |

Kubernetes example:

```yaml
livenessProbe:
    httpGet:
        path: /healthz
        port: http
    initialDelaySeconds: 10
    periodSeconds: 10
readinessProbe:
    httpGet:
        path: /v1/ready
        port: http
    initialDelaySeconds: 5
    periodSeconds: 5
```

## Persistence And Queueing

SQLite remains the local/default HTTP storage backend in this slice. PostgreSQL dialect migrations exist in `core/heeczer-storage/migrations-pg/`, and ADR-0006's default queue is implemented as `PostgresJobQueue` using `FOR UPDATE SKIP LOCKED` against `heec_jobs`.

Queue-backed image mode targets this flow after runtime PostgreSQL pool switching and worker startup are wired:

```text
SDK client
  -> enqueue event/job
  -> PostgreSQL heec_jobs
  -> worker claim with SKIP LOCKED
  -> score + persist
  -> succeeded / failed / dead_letter state
```

## Metrics

Prometheus metrics are exposed at `/metrics`. Scrape every 15 seconds by default.

| Metric                                | Description                                     |
| ------------------------------------- | ----------------------------------------------- |
| `axum_http_requests_total`            | Total HTTP requests by method, path, and status |
| `axum_http_requests_duration_seconds` | Request latency histogram                       |
| `axum_http_requests_pending`          | In-flight requests gauge                        |

Queue visibility is represented by queue stats (`pending`, `running`, `failed`, `dead_letter`, retries) from the library-level `JobQueue` implementation; exporting dedicated queue gauges and starting the worker from the binary are the next metrics/runtime wiring steps.

## Choosing A Mode

```text
Need persistent event log?       -> Image mode
Need multi-language HTTP?        -> Image mode
Need isolated auth/quotas?       -> Image mode
Need offline local scoring?      -> Native mode
Need lowest possible latency?    -> Native mode
```

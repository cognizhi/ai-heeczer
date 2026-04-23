# ADR-0006: Default Queue Backend for Image Mode

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §12.4, §19.4, §34 (Open Question 6)

## Context
Image-mode ingestion needs a queue with at-least-once delivery, dedup by `event_id`, retries, DLQ, visibility into depth/age, and operability across self-hosted and managed deployments.

## Decision
Default reference implementation: **PostgreSQL-backed work queue** using `SELECT ... FOR UPDATE SKIP LOCKED` against `heec_jobs`. Optional adapters: **Redis Streams** and **NATS JetStream** for higher-throughput deployments.

Rationale:
- PostgreSQL is already a hard dependency for production storage; making the queue PG-native means zero additional infra for >90% of deployments.
- `SKIP LOCKED` queues reliably handle the documented 10k req/s/node target with appropriate batching.
- Adapters keep the door open for high-fanout deployments without coupling the core.

## Alternatives Considered
- **Redis as default** — extra dependency for small deployments.
- **Kafka** — overkill for the median deployment; high operational burden.
- **In-memory only** — fails durability and replay requirements.

## Consequences
- Positive: minimal infra, transactional consistency between job state and persisted score row.
- Negative: PG queues require careful index and vacuum tuning at scale; documented in operational runbook.
- Follow-ups: queue adapter contract trait (`JobQueue`) so Redis/NATS implementations can be swapped without core changes.

## References
- PRD §12.4 Queueing and Processing
- PRD §19.4 Reliability

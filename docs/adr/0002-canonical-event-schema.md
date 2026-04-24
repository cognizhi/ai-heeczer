# ADR-0002: Canonical Event Schema and Versioning

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §13, §12.16

## Context

Heterogeneous AI frameworks emit different telemetry. Without a canonical contract, scoring parity, validation, and adapter testing are impossible. The schema must be evolvable without silently breaking SDKs.

## Decision

Adopt the v1 canonical event schema documented in PRD §13 as the wire and storage contract. The schema is published as:

- a JSON Schema draft 2020-12 file under `core/schema/event.v1.json`
- generated typed structs in every SDK from that single file
- shared fixtures under `core/schema/fixtures/` consumed by every language test suite

Versioning rules:

- `spec_version` is mandatory on every event.
- Additive optional fields are minor changes and must remain backward-compatible.
- Removing or repurposing a field is a major change and ships a new versioned schema file (`event.v2.json`); the prior version remains servable for at least one minor release.
- Unknown top-level fields are rejected in strict mode and accepted only inside `meta.extensions` per PRD §13 schema rules.

## Alternatives Considered

- **Protobuf / Avro** — better wire efficiency, but worse human ergonomics, harder fixture review, and higher friction for HTTP and webhook adapters.
- **Free-form JSON with per-SDK validators** — fastest to start, guaranteed drift.

## Consequences

- Positive: one schema file, one fixture set, one validation surface across languages.
- Negative: JSON validation overhead (mitigated by compiled validators).
- Follow-ups: contract test job that asserts every SDK rejects/accepts the same fixtures byte-for-byte.

## References

- PRD §13 Canonical Event Schema (v1)
- PRD §12.16 API Versioning

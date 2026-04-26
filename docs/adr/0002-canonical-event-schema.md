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

## v1 → v2 Evolution Policy

The following rules govern how the canonical event schema evolves over time.

### Backward-compatible changes (no new schema file required)

- Adding an optional field at any nesting level — existing events remain valid.
- Adding a new allowed value to `meta.extensions` — always allowed; `extensions` is open.

### Breaking changes (requires `event.v2.json`)

- Renaming, removing, or changing the type of any existing field.
- Repurposing the semantics of an existing field.
- Tightening a constraint (e.g. a field that was optional becoming required).
- Adding a new mandatory field.

### Versioning mechanics

1. `spec_version` is the routing key in the ingestion service. The service validates the incoming event against the schema whose version matches `spec_version`. Unknown versions are rejected immediately before any validation or storage with `415 Unsupported Media Type`, `error.kind = "unsupported_spec_version"`, and a `Supported-Spec-Versions` response header.
2. When v2 ships, the ingestion service validates v1 events against `event.v1.json` and v2 events against `event.v2.json` in parallel.
3. v1 remains fully servable for **at least one minor release** after v2 is published. SDKs are expected to migrate within that window.
4. The ingestion service emits `X-Heeczer-Spec-Version: <version>` on every response, reflecting the spec version used to process the request. This is always present as a machine-readable signal.
5. When v1 deprecation begins, the service additionally emits `Deprecation: true` and `Sunset: <RFC 7231 date>` headers on responses that processed v1 events. These headers follow [RFC 8594](https://www.rfc-editor.org/rfc/rfc8594) semantics.

### Migration path for SDK authors

1. Update SDK to construct and accept v2 events.
2. Gate on the `Deprecation` header (or monitor the `Sunset` date) to know when to drop v1 support.
3. After the Sunset date, the service rejects v1 events with `415 Unsupported Media Type` and a `Supported-Spec-Versions` response header.

## References

- PRD §13 Canonical Event Schema (v1)
- PRD §12.16 API Versioning
- [RFC 8594 — The Sunset HTTP Header Field](https://www.rfc-editor.org/rfc/rfc8594)

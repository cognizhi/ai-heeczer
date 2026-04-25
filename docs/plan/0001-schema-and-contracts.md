# Plan 01 — Schema and contracts

- **Status:** Done
- **Owner:** Tech Lead + SDK Engineer
- **PRD:** §13, §12.2, §12.16, §12.15
- **ADR:** ADR-0002

## Goal

Establish the canonical event schema as the single, versioned, fixture-driven contract consumed by the Rust core, every SDK, the ingestion service, the dashboard data layer, and all framework adapters.

## Deliverables

- `core/schema/event.v1.json` — JSON Schema draft 2020-12.
- `core/schema/fixtures/` — shared fixtures (valid, invalid, edge cases).
- Generated typed bindings per language (rust struct, TS interface, python dataclass/pydantic, go struct, java POJO).
- Contract test job in CI that asserts every binding accepts/rejects the same fixtures byte-for-byte.

## Checklist

### Schema definition

- [x] Author `core/schema/event.v1.json` matching PRD §13. (PR #1)
- [x] Add validation rules: `spec_version` mandatory, `meta.extensions` is the only unknown-field bucket, strict mode rejects all other unknowns. (PR #1)
- [x] Add fixture set: `valid/`, `invalid/`, `edge/` (min payload, max payload, unicode, missing optional fields, extension passthrough). (PR #1)
- [x] Expand `valid/` fixtures to cover representative use cases beyond the canonical PRD example: summarization HIL, RCA failure high-risk, planning-architecture partial, regulated decision support, drafting timeout, CI triage tool-heavy. (now 12 fixtures total; auto-discovered by `every_valid_fixture_validates` and `every_valid_fixture_scores_under_default_profile`.)
- [x] Additional golden fixtures `08-minimum-payload.json`, `09-outcome-failure.json`, `10-outcome-partial.json`, `11-high-confidence-band.json`, `12-low-confidence-band.json` added to `core/schema/fixtures/events/valid/`. (session Apr-2026)
- [x] Document the schema authoring guide in `docs/architecture/data-model.md`. (session Apr-2026)

### Generation and bindings

- [x] Generate Rust types via hand-written serde structs validated against the schema. (PR #1)
- [x] Generate TS types via hand-written interfaces in `bindings/heeczer-js/src/index.ts`. (session Apr-2026)
- [x] Generate Python types via hand-written TypedDicts in `bindings/heeczer-py/src/heeczer/client.py`. (session Apr-2026)
- [x] Generate Go types via hand-written structs in `bindings/heeczer-go/heeczer.go`. (session Apr-2026)
- [x] Generate Java POJOs via hand-written Jackson POJOs in `bindings/heeczer-java/src/main/java/io/github/cognizhi/heeczer/`. (session Apr-2026)

### Tests

- [x] Unit: schema validator round-trips every valid fixture. (PR #1)
- [x] Unit: schema validator rejects every invalid fixture with the documented error code. (PR #1)
- [x] Unit: `ProfileValidator` validates the embedded default profile and rejects unknown / missing fields. (commit 2d11a69)
- [x] Unit: `ScoringProfile` struct rejects unknown top-level fields via `serde(deny_unknown_fields)`. (commit 2d11a69)
- [x] Unit: `TierSetValidator` validates `tiers/default.v1.json` and activated `heec validate tier` surface. (ADR-0010 Phase 2)
- [x] Golden scoring tests for minimum payload, failure outcome, and partial success outcome in `tests/golden_scoring.rs`. (session Apr-2026)
- [x] Contract: all five bindings produce semantically equal JSON for every valid fixture (round-trip test). (session Apr-2026)
- [x] Contract: extension fields under `meta.extensions` survive round-trip; unknown top-level fields are rejected in strict mode. (session Apr-2026)

### Versioning

- [x] Document the v1 → v2 evolution policy in ADR-0002. (session Apr-2026)
- [x] Wire `spec_version` into the ingestion service as the routing key for parser selection. (session Apr-2026)
- [x] Add a "deprecated header" middleware skeleton for future v1-on-v2 deprecation. (session Apr-2026)

### Docs

- [x] Update root README schema section. (session Apr-2026)
- [x] Update `docs/architecture/data-model.md` with diagrams. (session Apr-2026)
- [x] Mark this plan Done in `0000-overview.md` once all items are `[x]`. (session Apr-2026)

## Acceptance

- All contract tests green on `main`.
- Schema and fixtures referenced from at least one test in every binding.

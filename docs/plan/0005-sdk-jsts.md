# Plan 05 — JS/TS SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23
- **ADR:** ADR-0001

## Goal
Ship `@ai-heeczer/sdk` for Node.js (CJS + ESM), with native bindings to `heeczer-core` via `napi-rs`, idiomatic public API, and full parity with the shared fixture suite.

## Checklist

### Package
- [x] `bindings/heeczer-js/` workspace package (path differs from the original `bindings/node/` placeholder; SDK ships HTTP-only first per plan revision below).
- [ ] `napi-rs` build with prebuilt binaries for linux x64/arm64, macOS x64/arm64, windows x64. (deferred: HTTP-first SDK ships now; in-process scoring binding follows once parity test rig lands)
- [ ] CJS + ESM dual export via `tshy` or equivalent. (foundation: ESM-only first; CJS dual export pending publish step)
- [x] TypeScript types covering the envelope contract (`ScoreResult`, `IngestEventResponse`, `VersionResponse`, `HeeczerApiError` with closed `kind` enum). Exact-shape generation from JSON Schema pending.

### Public API
- [x] `HeeczerClient` class with `healthz`, `version`, `ingestEvent`, `testScorePipeline`. (The plan's original `track`/`trackBatch`/`flush`/`close` shape predates the ingestion service; the HTTP-first surface is the foundation, with batching + flush + retry following the batch endpoint in plan 0004.)
- [ ] Mode selection: `native` (in-process score) and `image` (HTTP transport). (current SDK is HTTP-only / image mode)
- [x] Async `Promise<…>` return shapes throughout.
- [ ] Configurable timeout, retry policy, transport. (only `fetch` injection today)
- [x] `version()` reports SDK + service + engine versions.

### Validation
- [ ] Local schema validation before transport. (relies on server-side validation for now)
- [x] Typed errors mapped to the closed `kind` enum from the ingestion-service envelope.

### Tests
- [x] Unit: every public method (8 vitest cases covering baseUrl handling, version, ingest happy path, error envelope mapping, non-JSON error fallback, tester header always sent, api key forwarding).
- [ ] Contract: shared fixtures. (pending: needs the parity fixture rig in plan 0001 §Tests)
- [ ] Parity: byte-equal output vs Rust reference.
- [ ] Bench: `track()` p95 <2 ms in native mode. (depends on napi-rs binding above)
- [ ] Packaging: `npm pack` smoke test in CI matrix.

### Docs
- [x] `bindings/heeczer-js/README.md` with quickstart, configuration, methods table, error-kind matrix, and link to runnable example.
- [x] Example app under `examples/node/quickstart.mjs` (cross-language index in `examples/README.md`).
- [ ] CHANGELOG.md (managed by release-please).

## Acceptance
- Parity job green.
- Prebuilt binaries publish on release.

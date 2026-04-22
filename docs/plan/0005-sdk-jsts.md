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
- [ ] `bindings/node/` workspace package.
- [ ] `napi-rs` build with prebuilt binaries for linux x64/arm64, macOS x64/arm64, windows x64.
- [ ] CJS + ESM dual export via `tshy` or equivalent.
- [ ] TypeScript types generated from the JSON schema.

### Public API
- [ ] `Client` class with `track(event)`, `trackBatch(events)`, `flush()`, `close()`.
- [ ] Mode selection: `native` (in-process score) and `image` (HTTP transport).
- [ ] Async non-blocking `track()` returning a `Promise<TrackResult>`.
- [ ] Configurable timeout, retry policy, transport.
- [ ] `version()` reports SDK + core version.

### Validation
- [ ] Local schema validation before transport.
- [ ] Typed errors mapped per PRD §14.2.1 fallback contract.

### Tests
- [ ] Unit: every public API.
- [ ] Contract: shared fixtures.
- [ ] Parity: byte-equal output vs Rust reference.
- [ ] Bench: `track()` p95 <2 ms in native mode.
- [ ] Packaging: `npm pack` smoke test in CI matrix.

### Docs
- [ ] `bindings/node/README.md` with quickstart, API reference, examples.
- [ ] Example app under `examples/node/`.
- [ ] CHANGELOG.md (managed by release-please).

## Acceptance
- Parity job green.
- Prebuilt binaries publish on release.

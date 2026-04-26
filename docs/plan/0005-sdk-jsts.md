# Plan 05 — JS/TS SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-25
- **PRD:** §23
- **ADR:** ADR-0001

## Goal

Ship `@ai-heeczer/sdk` for Node.js (CJS + ESM), with native bindings to `heeczer-core` via `napi-rs`, idiomatic public API, and full parity with the shared fixture suite.

## Checklist

### Package

- [x] `bindings/heeczer-js/` workspace package (path differs from the original `bindings/node/` placeholder; SDK ships HTTP-only first per plan revision below).
- [ ] `napi-rs` build with prebuilt binaries for linux x64/arm64, macOS x64/arm64, windows x64. (deferred: HTTP-first SDK ships now; in-process scoring binding follows once parity test rig lands)
- [x] CJS + ESM dual export via `tsc` dual build. `package.json` exports `import` and `require`; `pnpm run build` emits `dist/index.js` and `dist/cjs/index.cjs`. (session Apr-2026)
- [x] TypeScript types covering the envelope contract (`ScoreResult`, `IngestEventResponse`, `VersionResponse`, `HeeczerApiError` with closed `kind` enum). Exact-shape generation from JSON Schema pending.

### Public API

- [x] `HeeczerClient` class with `healthz`, `version`, `ingestEvent`, `testScorePipeline`. (The plan's original `track`/`trackBatch`/`flush`/`close` shape predates the ingestion service; the HTTP-first surface is the foundation, with batching + flush + retry following the batch endpoint in plan 0004.)
- [x] Mode selection: `mode: "image" | "native"` is part of `HeeczerClientOptions`; image mode is implemented and native mode fails fast with an explicit napi-rs binding message. Native functionality remains gated by the unchecked napi-rs package item above. (session Apr-2026)
- [x] Async `Promise<…>` return shapes throughout.
- [x] Configurable timeout, retry policy, transport. (`timeoutMs`, `retry`, and `fetch` injection; transient status retry defaults to 408/429/5xx)
- [x] `version()` reports SDK + service + engine versions.

### Validation

- [x] Local schema validation before transport. `validateEvent()` enforces the v1 required fields, closed enums, strict unknown-field placement, and range/pattern constraints without adding a runtime schema dependency.
- [x] Typed errors mapped to the closed `kind` enum from the ingestion-service envelope.

### Tests

- [x] Unit: every public method (8 vitest cases covering baseUrl handling, version, ingest happy path, error envelope mapping, non-JSON error fallback, tester header always sent, api key forwarding).
- [x] Contract: shared fixtures. Vitest round-trips all shared valid fixtures and now also runs the local validator against them.
- [ ] Parity: byte-equal output vs Rust reference.
- [ ] Bench: `track()` p95 <2 ms in native mode. (depends on napi-rs binding above)
- [x] Packaging: `npm pack`/publish dry-run smoke is wired through `release-dry-run.yml`; local `pnpm run pack:smoke` passed on 2026-04-25.

### Docs

- [x] `bindings/heeczer-js/README.md` with quickstart, configuration, methods table, error-kind matrix, and link to runnable example.
- [x] Example app under `examples/node/quickstart.mjs` (cross-language index in `examples/README.md`).
- [ ] CHANGELOG.md (managed by release-please).

## Acceptance

- Parity job green.
- Prebuilt binaries publish on release.

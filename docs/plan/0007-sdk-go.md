# Plan 07 — Go SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23
- **ADR:** ADR-0001

## Goal
Ship the Go SDK as a Go module that consumes the Rust core via the C ABI (`heeczer-core-c`) using cgo, with full parity.

## Checklist

### Module
- [x] `bindings/heeczer-go/` Go module on a canonical-shaped path (`github.com/cognizhi/ai-heeczer/bindings/heeczer-go`). Path differs from the original `bindings/go/` placeholder; module is stdlib-only.
- [ ] cgo-linked against prebuilt static libs for linux/darwin × amd64/arm64. (deferred: HTTP-first SDK ships now; cgo binding to heeczer-core-c follows once parity test rig + napi-rs/pyo3 siblings land)
- [x] Pure-Go compile path (no cgo today). The future cgo binding will be an opt-in build tag rather than a breaking change.

### Public API
- [x] `Client` struct with `Healthz`, `Version`, `IngestEvent`, `TestScorePipeline` (HTTP-first; `Track`/`TrackBatch`/`Flush`/`Close` follow the batch endpoint in plan 0004).
- [x] Functional options (`WithAPIKey`, `WithHTTPClient`).
- [ ] Mode selection: `native` and `image`. (image-only today; depends on cgo binding above)

### Tests
- [x] Unit (`go test ./...`) — 8/8 pass against `httptest.NewServer` instead of mocks (per the user's TDD-with-emulation guidance).
- [ ] Contract: shared fixtures. (pending: needs the parity fixture rig in plan 0001 §Tests)
- [ ] Parity: byte-equal output vs Rust reference.
- [x] `go vet ./...` clean. (`golangci-lint` + `govulncheck` to be wired in plan 0012 CI work.)
- [ ] `govulncheck` clean.

### Docs
- [ ] `bindings/go/README.md` with quickstart, API reference.
- [ ] Example app under `examples/go/`.

## Acceptance
- Parity job green.
- Module tag published per release on the canonical path.

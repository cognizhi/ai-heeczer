# Plan 07 — Go SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-27
- **PRD:** §23
- **ADR:** ADR-0001

## Goal

Ship the Go SDK as a Go module that consumes the Rust core via the C ABI (`heeczer-core-c`) using cgo, with full parity.

## Checklist

### Module

- [x] `bindings/heeczer-go/` Go module on a canonical-shaped path (`github.com/cognizhi/ai-heeczer/bindings/heeczer-go`). Path differs from the original `bindings/go/` placeholder; module is stdlib-only.
- [ ] cgo-linked against prebuilt static libs for linux/darwin × amd64/arm64. (deferred: HTTP-first SDK ships now; cgo binding to heeczer-core-c follows after the HTTP parity gate is stable)
- [x] Pure-Go compile path (no cgo today). The future cgo binding will be an opt-in build tag rather than a breaking change.

### Public API

- [x] `Client` struct with `Healthz`, `Version`, `IngestEvent`, `TestScorePipeline` (HTTP-first; `Track`/`TrackBatch`/`Flush`/`Close` follow the batch endpoint in plan 0004).
- [x] Functional options (`WithAPIKey`, `WithHTTPClient`).
- [x] Mode selection: `WithMode(ModeImage | ModeNative)` is part of the functional-options surface; image mode is implemented and native mode fails fast with an explicit cgo binding message. Native functionality remains gated by the unchecked cgo item above. (session Apr-2026)

### Tests

- [x] Unit (`go test ./...`) — 8/8 pass against `httptest.NewServer` instead of mocks (per the user's TDD-with-emulation guidance).
- [x] Contract: shared fixtures. `go test ./...` round-trips all shared valid fixtures and enforces strict unknown top-level field rejection via `DisallowUnknownFields`.
- [x] Parity: byte-equal output vs Rust reference. `parity.yml` now generates Rust CLI reference `ScoreResult` JSON, starts `heeczer-ingest` with test orchestration enabled, and runs `go run ./cmd/parity` against every shared valid fixture. `ScoreResult` preserves the raw score object on decode so additive engine fields are not dropped before comparison. (session Apr-2026)
- [x] `go vet ./...` clean. (`golangci-lint` + `govulncheck` to be wired in plan 0012 CI work.)
- [x] `govulncheck` clean. (wired into `parity.yml` Go job, session Cat-3)

### Docs

- [x] `bindings/heeczer-go/README.md` with quickstart, error-kind matrix, functional-options table, and link to runnable example.
- [x] Example app under `examples/go/quickstart.go` (with local `replace` directive in `examples/go/go.mod` so the demo compiles before the module is published).

## Acceptance

- Parity job green.
- Module tag published per release on the canonical path. `release.yml` converts release-please's `heeczer-go-vX.Y.Z` trigger tag into the Go proxy tag `bindings/heeczer-go/vX.Y.Z`, and `release-dry-run.yml` validates the tag shape.

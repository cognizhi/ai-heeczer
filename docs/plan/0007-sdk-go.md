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
- [ ] `bindings/go/` Go module on the canonical path.
- [ ] cgo-linked against prebuilt static libs for linux/darwin × amd64/arm64.
- [ ] Pure-Go fallback compile path that returns "image-mode-only" error if static lib missing (for users who can't run cgo).

### Public API
- [ ] `Client` struct with `Track(ctx, event) (*Result, error)`, `TrackBatch`, `Flush`, `Close`.
- [ ] Functional options for config.
- [ ] Mode selection: `native` and `image`.

### Tests
- [ ] Unit (`go test ./...`).
- [ ] Contract: shared fixtures.
- [ ] Parity: byte-equal output vs Rust reference.
- [ ] `golangci-lint` clean.
- [ ] `govulncheck` clean.

### Docs
- [ ] `bindings/go/README.md` with quickstart, API reference.
- [ ] Example app under `examples/go/`.

## Acceptance
- Parity job green.
- Module tag published per release on the canonical path.

# Plan 08 — Rust SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23
- **ADR:** ADR-0001

## Goal
Ship `heeczer` crate as a thin idiomatic Rust SDK over `heeczer-core`.

## Checklist

- [ ] `bindings/rust/` workspace member.
- [ ] Public `Client` with sync and async (`tokio`) `track`/`track_batch`/`flush`/`close`.
- [ ] Mode selection: `native` and `image` (via `reqwest`).
- [ ] Re-export core types under a clean module path.
- [ ] Unit + contract + parity tests.
- [ ] `cargo doc` with examples.
- [ ] `bindings/rust/README.md`.
- [ ] Example under `examples/rust/`.

## Acceptance
- Parity job green.
- `cargo publish --dry-run` clean in CI.

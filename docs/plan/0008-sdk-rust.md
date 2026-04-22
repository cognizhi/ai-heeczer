# Plan 08 — Rust SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23
- **ADR:** ADR-0001

## Goal
Ship `heeczer` crate as a thin idiomatic Rust SDK over `heeczer-core`.

## Checklist

- [x] `bindings/heeczer-rs/` workspace member. Re-exports `Event`, `ScoreResult`, `ScoringProfile`, `TierSet`, `ConfidenceBand` from `heeczer-core`.
- [x] `Client::native()` — synchronous in-process scoring, zero network hop. Feature flag `native` (on by default).
- [ ] `Client::http()` — async HTTP client targeting the ingestion service. Feature flag `http` (optional, deps: `reqwest 0.12`, `tokio 1`). Scaffold in `src/http.rs` behind `#[cfg(feature = "http")]`; full impl deferred until ingestion service auth lands (plan 0004).
- [ ] Mode selection: `native` and `image`. Native ships today; image follows.
- [x] 2 integration tests in `tests/native.rs` against the canonical fixture (green).
- [ ] Contract: shared fixtures parity. (pending: needs the parity fixture rig in plan 0001 §Tests)
- [ ] `cargo publish --dry-run` clean in CI. (pending plan 0012 CI work)

## Acceptance
- Parity job green.
- `cargo publish --dry-run` clean in CI.

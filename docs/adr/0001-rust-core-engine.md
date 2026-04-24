# ADR-0001: Rust as the Core Scoring Engine

- **Status:** Accepted (C ABI surface governed by ADR-0011; `unsafe_code` boundary clarified below)
- **Date:** 2026-04-22
- **Related:** PRD §14, §19, §23, ADR-0011

## Context

ai-heeczer must deliver bit-identical scoring outputs across JS/TS, Python, Go, Rust, and Java SDKs (PRD §14.7, §23). Independently re-implementing scoring per language guarantees drift and fixture divergence. We need a single source of truth that is fast, embeddable, and FFI-friendly.

## Decision

Implement the canonical scoring engine, schema validation, normalization, confidence model, and explainability trace generation in a single Rust crate (`heeczer-core`). All language SDKs consume the same compiled core via:

- Rust: native crate
- Node: `napi-rs` N-API bindings
- Python: `pyo3` + `maturin` with `abi3` wheels
- Go: `cgo` over a stable C ABI exposed by `heeczer-core-c`
- Java: Foreign Function & Memory API on JDK 22+; JNI fallback only when required

## Alternatives Considered

- **Per-language reimplementation** — fastest to prototype, guaranteed long-term drift, fixture divergence, and double maintenance.
- **WASM core** — strong portability, but heavier startup cost, harder integration with native server runtimes, and immature toolchain for Java/Go embedding.
- **C/C++ core** — comparable performance, but weaker memory safety, weaker ergonomics, and more brittle build pipeline than Rust.

## Consequences

- Positive: single source of truth for arithmetic, fixed-point math, rounding, fallbacks, and confidence derivation; cross-language parity becomes a build-time guarantee, not a hope.
- Positive: native performance for the ingestion service path.
- Negative: every language binding adds FFI surface, packaging complexity, and per-platform CI matrix.
- Follow-ups: define and version the C ABI surface (ADR-0011, accepted 2026-04-23); golden-fixture suite shared across all bindings.

## Amendment 2026-04-23 — `unsafe_code` boundary

`heeczer-core-c` is the **single sanctioned `unsafe_code` boundary** in the workspace. The workspace lint `unsafe_code = "forbid"` applies everywhere except this crate, which overrides to `unsafe_code = "allow"` for `extern "C"` shims only. New `unsafe` outside this crate requires a fresh ADR.

## References

- PRD §14.7 Scoring Contract Requirements
- PRD §23 Cross-Language SDK Strategy

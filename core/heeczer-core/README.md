# heeczer-core

Deterministic, fixed-point scoring core for **ai-heeczer**. Single source of truth for
HEE (Human Equivalent Effort), FEC (Financial Equivalent Cost), confidence score, and
explainability trace. All language SDKs (JS/TS, Python, Go, Java) consume this crate
through FFI; see ADR-0001.

## Guarantees

- All math uses `rust_decimal::Decimal`. Floating-point arithmetic is forbidden in scoring
  paths via workspace `clippy` config and code review.
- Identical normalized input ⇒ byte-equal `ScoreResult` JSON across every binding.
- `SCORING_VERSION` and `SPEC_VERSION` are public constants; any change requires a
  fixture diff (PRD §14.7, ADR-0003).
- Round half away from zero; rounding occurs once per output field at the persisted-output
  boundary.

See `docs/architecture/scoring-engine.md`.

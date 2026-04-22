# Plan 02 — Scoring core

- **Status:** Active
- **Owner:** Tech Lead
- **PRD:** §14, §15, §16, §14.2.1, §14.7
- **ADR:** ADR-0001, ADR-0003

## Goal
Deliver the deterministic, fixed-point, versioned scoring engine in Rust that produces HEE, FEC, confidence score, confidence band, and explainability trace from a normalized event. This is the single source of truth for all SDKs.

## Deliverables
- `core/heeczer-core/` Rust crate with public `score(event, profile, tier_set, rate_table) -> ScoreResult` API.
- Fixed-point decimal arithmetic via `rust_decimal` (≥4 fractional digits intermediate; documented rounding rule for persisted output).
- Confidence model implementation per PRD §15.
- Explainability trace structure per PRD §16.
- Golden fixture suite under `core/schema/fixtures/scoring/` covering every component, multiplier, and edge case.
- C ABI surface `core/heeczer-core-c/` for Go and Java consumers.

## Checklist

### Crate scaffolding
- [ ] Create `core/heeczer-core/` cargo workspace member.
- [ ] Add `rust_decimal` dependency; ban `f32`/`f64` arithmetic in scoring code via `clippy.toml` `disallowed-types`.
- [ ] Define types: `Event`, `ScoringProfile`, `TierSet`, `RateTable`, `ScoreResult`, `ExplainabilityTrace`, `ConfidenceBand`.

### Normalization (PRD §14.2.1)
- [ ] Coerce missing optional numeric metrics to `0`, optional booleans to `false`, optional multipliers to `1.0`.
- [ ] Compute `total_tokens = tokens_prompt + tokens_completion`.
- [ ] Normalize missing `task.category` to `uncategorized` with default multiplier `1.0` and confidence penalty.
- [ ] Validate required non-derivable fields exist; return typed errors otherwise.

### Base scoring formula (PRD §14.2)
- [ ] Implement each component: token, duration, step, tool, artifact, output, review.
- [ ] Implement category multiplier lookup with profile override.
- [ ] Implement context multiplier composition (retry, ambiguity, risk, HIL, outcome).
- [ ] Implement tier adjustment (PRD §14.5) and FEC (PRD §14.6).

### Confidence (PRD §15)
- [ ] Implement deterministic completeness + calibration matrix.
- [ ] Implement penalties: missing category, repeated retries.
- [ ] Implement risk-based caps.
- [ ] Derive band from unrounded score.

### Explainability (PRD §16)
- [ ] Build the trace JSON structure.
- [ ] Include `scoring_version`, `bcu_breakdown`, multipliers, baseline minutes, tier block, final minutes, FEC, confidence.
- [ ] Provide a `human_summary` string builder.

### Determinism and rounding
- [ ] Single rounding helper applied at exactly the persisted-output boundary.
- [ ] Unit test: same input → same output across 10k randomized iterations.
- [ ] Unit test: float-equivalent inputs do not change persisted output.

### Versioning (ADR-0003)
- [ ] Embed `SCORING_VERSION` constant; build fails if changed without a fixture diff.
- [ ] Add a `scoring_version_check` script wired into CI.
- [ ] Define `ScoringProfile` JSON schema under `core/schema/scoring_profile.v1.json`; profiles are append-only with `effective_at` and `superseded_at` columns; `superseded_at` is the only mutable field.
- [ ] Property test: `Decimal` operations cannot overflow on PRD §29 maximum payload sizes.

### C ABI
- [ ] Create `core/heeczer-core-c/` with `cbindgen` headers.
- [ ] Expose `heeczer_score_json(input_json, profile_json, ...) -> *c_char` and `heeczer_free_string(*c_char)`.
- [ ] Add memory-leak test using `valgrind` in CI on Linux.

### Tests
- [ ] Unit tests for every component.
- [ ] Golden fixtures: minimum input, maximum input, every category, every outcome, every confidence band.
- [ ] Property tests with `proptest` for monotonicity (more tokens → ≥ same BCU).
- [ ] Benchmark: `score()` p50/p95 on a reference event.

### Docs
- [ ] Crate-level rustdoc with examples.
- [ ] `docs/architecture/scoring-engine.md` with formula diagrams.
- [ ] Update README scoring section.

## Acceptance
- All fixtures pass.
- C ABI test green.
- Bench published in `docs/architecture/benchmarks.md`.

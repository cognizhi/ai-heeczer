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
- [x] Create `core/heeczer-core/` cargo workspace member. (PR #1)
- [ ] Add `rust_decimal` dependency; ban `f32`/`f64` arithmetic in scoring code via `clippy.toml` `disallowed-types`. (partial: `rust_decimal` added PR #1; `clippy.toml` `disallowed-types` guard pending)
- [x] Define types: `Event`, `ScoringProfile`, `TierSet`, `RateTable`, `ScoreResult`, `ExplainabilityTrace`, `ConfidenceBand`. (PR #1)

### Normalization (PRD §14.2.1)
- [x] Coerce missing optional numeric metrics to `0`, optional booleans to `false`, optional multipliers to `1.0`. (PR #1)
- [x] Compute `total_tokens = tokens_prompt + tokens_completion`. (PR #1)
- [x] Normalize missing `task.category` to `uncategorized` with default multiplier `1.0` and confidence penalty. (PR #1)
- [x] Validate required non-derivable fields exist; return typed errors otherwise. (PR #1)

### Base scoring formula (PRD §14.2)
- [x] Implement each component: token, duration, step, tool, artifact, output, review. (PR #1)
- [x] Implement category multiplier lookup with profile override. (PR #1)
- [x] Implement context multiplier composition (retry, ambiguity, risk, HIL, outcome). (PR #1)
- [x] Implement tier adjustment (PRD §14.5) and FEC (PRD §14.6). (PR #1)

### Confidence (PRD §15)
- [x] Implement deterministic completeness + calibration matrix. (PR #1)
- [x] Implement penalties: missing category, repeated retries. (PR #1)
- [x] Implement risk-based caps. (PR #1)
- [x] Derive band from unrounded score. (PR #1)

### Explainability (PRD §16)
- [x] Build the trace JSON structure. (PR #1)
- [x] Include `scoring_version`, `bcu_breakdown`, multipliers, baseline minutes, tier block, final minutes, FEC, confidence. (PR #1)
- [x] Provide a `human_summary` string builder. (PR #1)

### Determinism and rounding
- [x] Single rounding helper applied at exactly the persisted-output boundary. (PR #1)
- [x] Unit test: same input → same output across 10k randomized iterations. (PR #1)
- [x] Unit test: float-equivalent inputs do not change persisted output. (PR #1)

### Versioning (ADR-0003)
- [ ] Embed `SCORING_VERSION` constant; build fails if changed without a fixture diff. (partial: constant embedded PR #1; CI fixture-diff guard pending)
- [ ] Add a `scoring_version_check` script wired into CI.
- [x] Define `ScoringProfile` JSON schema under `core/schema/scoring_profile.v1.json`; profiles are append-only with `effective_at` and `superseded_at` columns; `superseded_at` is the only mutable field. (PR #1)
- [x] Property test: rounding idempotence, scale preservation, score purity, JSON round-trip stability, confidence-band bounds, token-BCU linearity. (foundation hardening, commit cb06b1f)
- [ ] Property test: `Decimal` operations cannot overflow on PRD §29 maximum payload sizes.

### C ABI
- [ ] Create `core/heeczer-core-c/` with `cbindgen` headers. (partial: crate created and ABI functions implemented PR #1; `cbindgen` header generation pending)
- [x] Expose `heeczer_score_json(input_json, profile_json, ...) -> *c_char` and `heeczer_free_string(*c_char)`. (PR #1)
- [x] C ABI envelope contract written and accepted (ADR-0011). (commit 13d75f1)
- [x] ABI gap tests: `heeczer_versions_json`, `heeczer_free_string(NULL)` no-op, non-UTF-8 bytes, envelope-is-parseable-JSON. (foundation hardening, commit cb06b1f)
- [ ] Add a memory-leak test using `valgrind` in CI on Linux.

### Tests
- [x] Unit tests for every component. (PR #1 — determinism, golden fixture, and C ABI integration tests)
- [ ] Golden fixtures: minimum input, maximum input, every category, every outcome, every confidence band. (partial: PRD canonical fixture PR #1; per-outcome, per-category, and per-band fixtures pending)
- [x] Byte-stable golden ScoreResult JSON file under `core/schema/fixtures/golden/`. (foundation hardening, commit cb06b1f)
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

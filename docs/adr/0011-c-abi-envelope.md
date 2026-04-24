# ADR-0011: C ABI Envelope Contract for `heeczer-core-c`

- **Status:** Accepted
- **Date:** 2026-04-23
- **Deciders:** Tech Lead, SDK Engineer, Security Engineer
- **Related:** PRD §14.7, §23, ADR-0001, ADR-0002, ADR-0003

## Context

ADR-0001 made `heeczer-core` (Rust) the single source of truth for scoring and committed to consuming it from non-Rust SDKs (Go via cgo, Java via FFM, optional C/C++ embedders) through `heeczer-core-c`'s C ABI. ADR-0001 explicitly listed defining and versioning that C ABI surface as a follow-up — this ADR is that follow-up.

The implemented surface today is two functions:

```c
char* heeczer_score_json(const char* event_json,
                          const char* profile_json,
                          const char* tiers_json,
                          const char* tier_override);
char* heeczer_versions_json(void);
void  heeczer_free_string(char* s);
```

with all results encoded as a JSON envelope. Without a written contract, every binding will guess at field shape, error semantics, and panic behaviour, causing silent drift exactly like the reimplementation risk ADR-0001 was meant to eliminate.

## Decision

The C ABI surface is governed by the **stable JSON envelope** described below. The envelope is versioned independently of `SCORING_VERSION` and `SPEC_VERSION` and is part of the public contract surface tracked by `release-please` and CHANGELOG entries.

### 1. Memory ownership (unchanged)

- Every `char*` returned by the library is a NUL-terminated UTF-8 string owned by the caller; the caller MUST release it via `heeczer_free_string`.
- `heeczer_free_string(NULL)` is a documented no-op.
- All input pointers are borrowed; the library does not retain them past the call.
- Functions are panic-safe: a Rust panic is caught and converted to a `panic` error envelope.

### 2. Success envelope

```json
{
  "ok": true,
  "envelope_version": "1",
  "result": <ScoreResult>          // shape governed by ADR-0003 + SCORING_VERSION
}
```

- `result` is byte-equivalent to `serde_json::to_string(&ScoreResult)` of the canonical Rust core. Field order, decimal-as-string formatting, and key naming follow ADR-0003.

### 3. Error envelope

```json
{
  "ok": false,
  "envelope_version": "1",
  "error": {
    "kind": "schema|deserialise|score|panic|nul-input|invalid-utf8",
    "message": "<human-readable, redacted; no event payload echoed>"
  }
}
```

- `kind` is a closed enum; new kinds require a minor `envelope_version` bump.
- `message` is for operators; SDKs MUST NOT parse it for control flow.
- Sensitive content (event payload, profile JSON, tier names) MUST NOT be echoed in `message` to avoid log-injection / data-leak risk through SDK error wrapping.

> **Backwards-compat note for the bootstrap envelope:** the foundation slice
> shipped a flat `{"ok": false, "error": "..."}` shape. Bindings written against
> envelope_version `1` MUST tolerate a string `error` for backward read
> compatibility but SHOULD always emit the structured form. The flat string form
> is deprecated and is removed in envelope_version `2`.

### 4. Versioning

- `envelope_version` is a string (currently `"1"`).
- Additive fields under `result` follow ADR-0003 (driven by `SCORING_VERSION`).
- Additive top-level fields are minor; renames or removals require a major envelope bump and an ADR amendment.
- A new `envelope_version` is announced via per-binding CHANGELOG entries and a release-impact note (PRD §27).

### 5. Test surface

- `core/heeczer-core-c` ships ABI tests covering: success envelope is parseable JSON, error envelope is parseable JSON, NULL input handled, non-UTF-8 input handled (no panic), `heeczer_versions_json` matches Rust constants, `heeczer_free_string(NULL)` is a no-op.
- Each non-Rust SDK adds a parity test that decodes the envelope and asserts the same `ScoreResult` against the shared golden fixture under `core/schema/fixtures/golden/`.

## Alternatives Considered

- **Bare struct ABI** (return `ScoreResult`-shaped C struct): forces every binding to model decimal-as-string, optional fields, and explainability trees in C; updates would break every binding. Rejected.
- **Opaque handle + getters** (FFI handle, then per-field accessors): more allocations, more FFI crossings, and harder to keep aligned with `serde_json` representations on the Rust side. Rejected.
- **Protobuf wire format**: imposes protoc on every build; better for network boundaries, overkill for a same-process FFI envelope. Rejected.

## Consequences

- Positive: a single, stable, parsable contract for every non-Rust binding; arbitrary additive fields land without breaking older SDK builds.
- Positive: panic / non-UTF-8 / null-input behaviour is contractual, not incidental.
- Negative: each FFI call pays a JSON serialise/parse cost. Acceptable at the per-event call rate the SDKs target; optimised binary frame can ship as `envelope_version: "2"` if benchmarks force it.
- Follow-ups: ship a Java / Go parity test harness over `core/schema/fixtures/golden/` once those SDKs land (plans 0007, 0009).

## References

- PRD §14.7, §23
- ADR-0001 Rust core engine
- ADR-0003 Scoring versioning
- `core/heeczer-core-c/src/lib.rs`

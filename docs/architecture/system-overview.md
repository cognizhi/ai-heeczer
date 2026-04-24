# System overview

> Status: foundation slice. Updated each plan increment.

ai-heeczer is a deterministic scoring system. The architecture intentionally
keeps the scoring surface tiny and language-agnostic so every SDK, the CLI, and
the ingestion service produce **byte-identical** results from the same input.

## Layered responsibilities

```text
┌────────────────────────────────────────────────────────────────────────┐
│ Surfaces                                                               │
│   • SDKs (Rust / JS / Python / Go / Java)  ← plans 0005–0009          │
│   • Ingestion service                       ← plan 0004                │
│   • heec CLI                                 ← ADR-0010, this slice    │
│   • Dashboard                               ← plan 0010                │
└────────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────────┐
│ heeczer-core (this slice)                                              │
│   schema validator → normalizer → scoring orchestrator → result        │
│   versioned profiles + tier sets, deterministic Decimal math           │
└────────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────────┐
│ heeczer-storage                                                        │
│   sqlx migrations, append-only event/score tables, audit log           │
└────────────────────────────────────────────────────────────────────────┘
```

## Determinism boundary

Anything in `core/heeczer-core` is part of the determinism boundary. Changes
that alter scoring output **must** bump `SCORING_VERSION`, update fixtures,
and ship an ADR-0003 amendment.

The canonical contract is asserted by:

- `core/heeczer-core/tests/golden_scoring.rs` — PRD §29 hand-computed cases
- `core/heeczer-core/tests/schema_validation.rs` — every fixture under `core/schema/fixtures/events/`
- `core/heeczer-core/tests/determinism.rs` — version pinning + run-to-run stability

## Append-only storage contract

`heeczer-storage` enforces append-only semantics for `heec_events` and
`heec_scores` via SQLite triggers (`heec_events_no_update`, `heec_events_no_delete`,
matching pair on scores). Tombstones live in `heec_tombstones` so we can
honor data subject deletion requests without breaking the immutability of the
event log itself.

## Versioning

Two versions matter:

| Constant          | Defined in                               | Bumps when                                    |
| ----------------- | ---------------------------------------- | --------------------------------------------- |
| `SPEC_VERSION`    | `core/heeczer-core/src/version.rs`       | Schema shape changes; currently `"1.0"` (PRD `spec_version` field) |
| `SCORING_VERSION` | `core/heeczer-core/src/version.rs`       | Any math, rounding, or default change (semver)|

Both are baked into `ScoreResult` so every persisted score is reproducible.

## Open questions

Tracked as backlog items in the project tracker. None block the foundation
slice.

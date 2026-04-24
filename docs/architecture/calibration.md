# Calibration Architecture

Calibration is the process of tuning the scoring parameters — tier
boundaries, BCU multipliers, and confidence weights — to match observed
real-world effort data. Plan 0015 tracks the full calibration roadmap.

## Goals

1. **Accuracy** — scored estimates should converge with actual recorded effort.
2. **Reproducibility** — every calibration run is deterministic given the
   same input data and reference pack.
3. **Auditability** — parameter changes are versioned and linked to the data
   that drove them.

---

## Data model

### Reference pack

A *reference pack* is a JSON document that pairs canonical events with
ground-truth effort measurements:

```json
{
  "pack_id": "ref_v1_2024q4",
  "schema_version": "1",
  "created_at": "2024-12-01T00:00:00Z",
  "entries": [
    {
      "event": { /* canonical event — event.v1.json */ },
      "actual_minutes": 42
    }
  ]
}
```text
Reference packs are stored under `core/schema/profiles/` (sample packs)
and loaded by the calibration CLI sub-command (plan 0015).

### Scoring profile

A *scoring profile* (`scoring_profile.v1.json`) contains the multipliers
and thresholds that `heeczer-core::score()` applies. Profiles are versioned
by `scoring_version`.

### Tier set

A *tier set* (`tier_set.v1.json`) maps event categories to BCU ranges.
Tier sets are versioned independently from scoring profiles.

---

## Calibration workflow

```text
reference pack (actual_minutes per event)
  │
  ▼
heec calibrate --pack <path/to/pack.json> --profile <path/to/profile.json>
  │
  ├── score each event with the current profile
  ├── compute RMSE between estimated_minutes and actual_minutes
  ├── gradient-descent or grid-search over multiplier space
  └── write updated profile to <output>.json + calibration report
```text
The calibration sub-command is implemented in `core/heeczer-cli`
(plan 0015 §CLI calibration).

---

## Metrics

| Metric | Description |
|---|---|
| RMSE | Root-mean-square error between estimated and actual minutes. |
| MAE | Mean absolute error (minutes). |
| R² | Coefficient of determination; 1.0 = perfect fit. |
| Bias | Signed mean error; positive = systematic over-estimation. |

---

## Versioning policy

- Scoring profiles follow [semantic versioning](https://semver.org/).
  A change to a multiplier is a **patch**; a new top-level field is a
  **minor**; removal of a field is a **major**.
- Tier sets are versioned independently.
- `ScoreResult.scoring_version` and `ScoreResult.spec_version` always
  reflect the profile and tier set versions used, enabling reproducibility
  of historical scores.

See [ADR-0003](../adr/0003-scoring-versioning.md) for the full versioning
policy.

---

## Reference pack format schema

The reference pack format will be published as a JSON Schema alongside the
canonical event schema in `core/schema/` once the calibration CLI lands
(plan 0015 §schema).

---

## Roadmap (plan 0015)

| Item | Status |
|---|---|
| Reference pack format definition | Planned |
| `heec calibrate` CLI sub-command | Planned |
| RMSE / MAE / R² report output | Planned |
| Automated calibration CI job | Planned |
| Sample reference pack for `01-prd-canonical` fixture | Planned |

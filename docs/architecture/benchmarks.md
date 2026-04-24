# Benchmarks reference

> Status: foundation slice — reflects PRD §29, §25, and plan 0015.
> Last reviewed: 2026-04-24
> Owner: Tech Lead

## Performance targets (PRD §29)

| Metric                                         | Target           | Measurement point                          |
| ---------------------------------------------- | ---------------- | ------------------------------------------ |
| Native mode `track()` latency (p95)            | < 2 ms           | In-process call on reference hardware      |
| Image mode async ack latency (p95)             | < 50 ms          | Same-region ingest endpoint to 200/202 ack |
| Ingest throughput per node                     | ≥ 10,000 accepted enqueues/s | Single node, queue backend: none |

"Native mode" means the scoring engine is called in-process without network
hops. "Image mode" means the Docker-packaged ingestion service receiving HTTP
requests.

## Reference hardware profile

Bench runs in CI on a GitHub Actions `ubuntu-latest` runner (2 vCPU,
7 GB RAM, SSD-backed). The runner does not guarantee dedicated CPU; results
from CI should be treated as a lower bound. For absolute capacity planning,
reproduce locally on a dedicated Linux host.

Local reproductions should record: CPU model, core count, RAM, and OS. Attach
that profile when filing a benchmark regression.

## Reference payload

- **File:** [`core/schema/fixtures/events/valid/01-prd-canonical.json`](../../core/schema/fixtures/events/valid/01-prd-canonical.json)
- **Auth mode:** none (bootstrap — no API key required for local bench)
- **Durability mode:** SQLite in-memory (`:memory:` DSN; no disk I/O)
- **Queue backend:** none (HTTP sync — ingest returns after scoring, no queue
  handoff)

This payload represents the PRD §29 reference event: a mid-complexity coding
task with a full set of metrics, a non-trivial confidence score, and an
explainability trace. It is the canonical input for all latency and
throughput measurements.

## Current bench results

> **TBD — run `make benchmark-smoke` to populate.**

The table below will be filled in automatically once the bench-smoke workflow
has executed. Until then, all cells contain `-`.

| Metric                          | p50    | p95    | p99    | CI run |
| ------------------------------- | ------ | ------ | ------ | ------ |
| `score()` wall-clock latency    | -      | -      | -      | -      |
| Ingest HTTP ack latency         | -      | -      | -      | -      |
| Throughput (enqueues/s/node)    | -      | -      | -      | -      |

## How to run locally

### Criterion microbenchmarks (`heeczer-core`)

```bash
cargo bench -p heeczer-core
```

Results are written to `target/criterion/`. Open
`target/criterion/report/index.html` in a browser for the HTML summary.

To benchmark only the `score()` function:

```bash
cargo bench -p heeczer-core -- score
```

### CLI smoke with timing

```bash
time make cli-smoke
```

`make cli-smoke` runs `heec score` against the reference fixture and asserts
the exit code is 0. The `time` prefix captures wall-clock, user, and sys
time. Compare the wall-clock figure against the < 2 ms native-mode target.

### Full benchmark smoke (CI-equivalent)

```bash
make benchmark-smoke
```

This builds the CLI in release mode, runs `make cli-smoke`, and enforces the
30-second wall-clock budget. It mirrors the `bench-smoke.yml` CI job.

## How `bench-smoke.yml` works

1. **Build:** `cargo build --release -p heeczer-cli` — produces an optimized
   `heec` binary. Debug-mode binaries are not used for timing.
2. **Run:** `make cli-smoke` — scores the reference fixture and validates the
   output schema.
3. **Budget enforcement:** the CI job is configured with a 30-second
   wall-clock timeout. If the job exceeds this budget, CI fails and the run is
   treated as a performance regression.

The 30-second budget is intentionally conservative relative to the < 2 ms
target. It accommodates CI runner variance and cold-start overhead (Rust
runtime init, SQLite open). The actual scoring call is a small fraction of
that budget.

## Calibration (plan 0015)

Once plan 0015 ships, benchmark packs will be defined in `heec_benchmark_packs`
and run via:

```bash
heec calibrate run --pack <pack-id> --profile <profile-id>
```

The command scores each item in the pack, compares results against the
expected human-effort range defined in the pack, and outputs per-item deltas
and suggested profile adjustments.

Calibration packs and run history are stored append-only. Profile updates
create a new profile version; existing versions are never mutated (ADR-0003).

See [Plan 0015 — Calibration and benchmarks](../plan/0015-calibration-benchmarks.md)
for the full calibration design.

## References

- PRD §29 — Performance requirements
- PRD §25 — Calibration requirements
- [Plan 0015 — Calibration and benchmarks](../plan/0015-calibration-benchmarks.md)
- [Plan 0002 — Scoring core (benchmark section)](../plan/0002-scoring-core.md)
- [Scoring engine architecture](scoring-engine.md)
- Reference fixture: [`core/schema/fixtures/events/valid/01-prd-canonical.json`](../../core/schema/fixtures/events/valid/01-prd-canonical.json)

# ai-heeczer

[![CI](https://github.com/cognizhi/ai-heeczer/actions/workflows/ci.yml/badge.svg)](https://github.com/cognizhi/ai-heeczer/actions/workflows/ci.yml)
[![Release](https://github.com/cognizhi/ai-heeczer/actions/workflows/release.yml/badge.svg)](https://github.com/cognizhi/ai-heeczer/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

ai-heeczer — deterministic AI effort & cost estimation SDK.

A reproducible scoring engine that converts raw AI agent task telemetry into
**Human Equivalent Effort (HEE)** and **Financial Equivalent Cost (FEC)** —
the business-legible units that engineering leadership, finance, and platform
teams actually need. Every score is derived from a mathematically grounded,
peer-literature-anchored model; the same event, profile, and tier set always
produces the same number, on any platform, in any language.

## Research foundations

The scoring model is not heuristic — it is built on four convergent research
traditions and validated against published empirical results:

- **Classical software effort estimation.** The BCU formula inherits the
  multiplicative-multiplier structure of COCOMO (Boehm, 1981) and the
  functional-scope principles of Function Point Analysis (Albrecht, 1979).
  Token mass maps directly to Halstead's cognitive volume metric (1977).
- **Empirical AI productivity evidence.** Peng et al. (2023) measured a **55.8%
  speed-up** for AI-assisted developers; Brynjolfsson et al. (2023) found a
  **15% throughput gain** for AI-assisted support agents; Eloundou et al. (2023)
  estimated **~15% of all US worker tasks** can already be completed faster at
  equivalent quality with LLM access. These studies provide direct calibration
  anchors for the HEE tier-adjustment step.
- **Agentic AI behavioral research.** SWE-agent (Yang et al., 2024) and the
  MINT benchmark (Wang et al., 2023) show that tool-call count and workflow
  step count are independent, reliable signals of task complexity — which is
  why they appear as first-class BCU components rather than being rolled into
  duration alone.
- **Knowledge-work productivity theory.** Drucker's (1999) task-output
  framing grounds the key question: not "how long did the agent run?" but
  "how long would a human of a specified role have taken to produce an
  equivalent result?"

For the full derivation, worked numerical example, confidence model, and
calibration roadmap, see
[`docs/research/hee-fec-scoring-model.md`](docs/research/hee-fec-scoring-model.md).

This repository delivers the scoring core, language SDKs, an ingestion
service, a dashboard, and the operational tooling to ship them.

## Status

This is the **foundation slice** of the project (plans 0001–0003, 0013).
It contains:

| Crate                                      | Purpose                                     |
| ------------------------------------------ | ------------------------------------------- |
| [`heeczer-core`](core/heeczer-core)        | Pure-Rust scoring engine, schema validation |
| [`heeczer-core-c`](core/heeczer-core-c)    | C ABI shim used by every non-Rust SDK       |
| [`heeczer-storage`](core/heeczer-storage)  | SQLite/Postgres storage and migrations      |
| [`heeczer-cli`](core/heeczer-cli) (`heec`) | First-class local developer CLI (ADR-0010)  |

Plans 0004–0012, 0014, and 0015 are tracked under [`docs/plan/`](docs/plan/) and
land iteratively on top of this foundation.

## Quickstart

```bash
# score a single event right away (after installing the CLI)
heec score examples/event.json
```

For a full local setup:

```bash
# install the latest stable Rust toolchain plus local security tooling
make bootstrap

# build & test the entire workspace, including cross-language SDK parity
make test

# mirror the Rust security CI jobs, including fresh cargo-audit/cargo-deny installs
make security-ci

# install the developer CLI
make cli-install

# end-to-end smoke against shipped fixtures
make cli-smoke

# score a single canonical event
heec score core/schema/fixtures/events/valid/01-prd-canonical.json --format pretty
```

The full target catalogue is in the [`Makefile`](Makefile); run `make help`.

## Language SDKs

| Language | Package                                 | Install                                 | Status      |
| -------- | --------------------------------------- | --------------------------------------- | ----------- |
| Rust     | [`heeczer-rs`](bindings/heeczer-rs)     | `cargo add heeczer`                     | In progress |
| JS / TS  | [`heeczer-js`](bindings/heeczer-js)     | `pnpm add @cognizhi/heeczer`            | In progress |
| Python   | [`heeczer-py`](bindings/heeczer-py)     | `pip install heeczer`                   | In progress |
| Go       | [`heeczer-go`](bindings/heeczer-go)     | `go get github.com/cognizhi/heeczer-go` | In progress |
| Java     | [`heeczer-java`](bindings/heeczer-java) | Maven / Gradle — see SDK README         | In progress |

All SDKs call into `heeczer-core` (directly for Rust; via `heeczer-core-c` for
everyone else) and produce **byte-identical** scoring output for the same event,
profile, and tier set.

See [`examples/`](examples/) for per-language usage.

## Architecture at a glance

```text
                ┌──────────────────────┐
   adapters ──▶ │  ingestion (Plan 04) │ ──▶ queue ──▶ scorer ──▶ storage ──▶ dashboard
                └──────────────────────┘                  │
                                                          ▼
                                              heeczer-core (this slice)
                                              ├─ schema validator
                                              ├─ scoring orchestrator
                                              ├─ deterministic Decimal math
                                              └─ versioned profiles + tiers
```

Cross-language SDKs (JS/TS, Python, Go, Rust, Java) call into
`heeczer-core` either directly (Rust) or through `heeczer-core-c` (everyone
else) so all callers produce **byte-identical** scoring output for the same
event + profile + tier set.

Full architecture documentation is in [`docs/architecture/`](docs/architecture/),
starting with [`docs/architecture/system-overview.md`](docs/architecture/system-overview.md).

## Repository layout

| Path             | Owner                                        |
| ---------------- | -------------------------------------------- |
| `core/`          | Rust core, C ABI, storage, CLI               |
| `core/schema/`   | JSON schemas + golden fixtures               |
| `docs/research/` | Scoring model paper and calibration analysis |
| `docs/adr/`      | Architecture Decision Records                |
| `docs/plan/`     | Implementation plans 0000–0015               |
| `docs/agents/`   | Agent harness and operating rules            |
| `.github/`       | CI workflows + agent role files              |

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`docs/agents/AGENT_HARNESS.md`](docs/agents/AGENT_HARNESS.md). Every change
must keep the canonical scoring tests green and may not bump
`SCORING_VERSION` without an ADR-0003 amendment.

## Security

See [`SECURITY.md`](SECURITY.md) for the vulnerability reporting process.
GitHub private vulnerability reporting is enabled on this repository.

## License

[MIT](LICENSE).

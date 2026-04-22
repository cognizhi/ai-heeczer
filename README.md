# ai-heeczer

A reproducible scoring engine that turns raw AI agent task telemetry into
**Behavioral Currency Units (BCU)** and **Financial Equivalent Cost (FEC)** so
that engineering organizations can evaluate AI productivity with the same rigor
as human work.

This repository delivers the spec, the scoring core, language SDKs, an
ingestion service, a dashboard, and the operational tooling to ship them. See
[`docs/prd/ai-heeczer-prd.md`](docs/prd/ai-heeczer-prd.md) for the canonical product
requirements.

## Status

This is the **foundation slice** of the project (plans 0001–0003, 0013).
It contains:

| Crate                                           | Purpose                                      |
| ----------------------------------------------- | -------------------------------------------- |
| [`heeczer-core`](core/heeczer-core)             | Pure-Rust scoring engine, schema validation  |
| [`heeczer-core-c`](core/heeczer-core-c)         | C ABI shim used by every non-Rust SDK        |
| [`heeczer-storage`](core/heeczer-storage)       | SQLite/Postgres storage and migrations       |
| [`heeczer-cli`](core/heeczer-cli) (`aih`)       | First-class local developer CLI (ADR-0010)  |

Plans 0004–0012, 0014, and 0015 are tracked under [`docs/plan/`](docs/plan/) and
land iteratively on top of this foundation.

## Quickstart

```bash
# install rust toolchain (rust-toolchain.toml pins 1.88)
rustup show

# build & test the entire workspace
make test

# install the developer CLI
make cli-install

# end-to-end smoke against shipped fixtures
make cli-smoke

# score a single canonical event
aih score core/schema/fixtures/events/valid/01-prd-canonical.json --format pretty
```

The full target catalogue is in the [`Makefile`](Makefile); run `make help`.

## Architecture at a glance

```
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

## Repository layout

| Path                  | Owner                            |
| --------------------- | -------------------------------- |
| `core/`               | Rust core, C ABI, storage, CLI   |
| `core/schema/`        | JSON schemas + golden fixtures   |
| `docs/prd/`           | Product requirements             |
| `docs/adr/`           | Architecture Decision Records    |
| `docs/plan/`          | Implementation plans 0000–0015   |
| `docs/agents/`        | Agent harness and operating rules |
| `.github/`            | CI workflows + agent role files  |

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`docs/agents/AGENT_HARNESS.md`](docs/agents/AGENT_HARNESS.md). Every change
must keep the canonical scoring tests green and may not bump
`SCORING_VERSION` without an ADR-0003 amendment.

## License

[Apache-2.0](LICENSE).

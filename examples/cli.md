# `heec` CLI quickstart

The `heec` CLI ships in [`core/heeczer-cli`](../core/heeczer-cli) and is
the fastest way to score an event without writing any code.

## Install

```bash
make cli-install   # installs to ~/.cargo/bin
```

## Score the demo event

```bash
heec score examples/event.json
```

Output (excerpt):

```text
Estimated <minutes> Mid-Level Engineer-equivalent minutes (~<cost> cost) for
`drafting`; confidence high.
```

_(Actual numbers depend on the active scoring profile. Use
`heec score --format json | jq .score.final_estimated_minutes` for
machine-readable output.)_

## Validate against the canonical schema

```bash
heec schema validate examples/event.json
heec validate profile core/schema/profiles/default.v1.json
heec validate tier    core/schema/tiers/default.v1.json
```

## Compare two ScoreResults (regression diffing)

```bash
heec score examples/event.json --format json > /tmp/baseline.json
# …make a profile change…
heec score examples/event.json --format json > /tmp/candidate.json
heec diff /tmp/baseline.json /tmp/candidate.json
```

## Browse shipped fixtures

```bash
heec fixtures list
heec fixtures show valid/01-prd-canonical.json
```

## Run a benchmark and replay (Phase 2 commands per ADR-0010)

```bash
# Benchmark p95 against an event fixture (1000 iterations by default):
heec bench --fixture examples/event.json --iter 10000

# Replay re-scores a previously-persisted event from storage; it does
# NOT take a fixture file. You first need to ingest an event (CLI score
# does not persist; the ingestion service does), then replay by id:
heec replay --database-url sqlite:./heeczer.db \
            --workspace ws_default \
            --event-id 11111111-2222-4333-8444-555555555555
```

## See also

- Per-language SDK quickstarts: [`README.md`](README.md)
- ADR-0010 — local developer CLI: [`../docs/adr/0010-local-developer-cli.md`](../docs/adr/0010-local-developer-cli.md)

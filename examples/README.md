# ai-heeczer examples

Runnable per-language quickstarts that all submit the same canonical
event ([`event.json`](event.json)) to either the in-process Rust core or
the ingestion service over HTTP. Each example is self-contained and
intentionally short — the goal is to show the smallest path from "I have
an event" to "I have a `ScoreResult`".

| Language | Path | Mode | Run |
| --- | --- | --- | --- |
| Rust (in-process) | [`../bindings/heeczer-rs/examples/quickstart.rs`](../bindings/heeczer-rs/examples/quickstart.rs) | native | `cargo run -p heeczer --example quickstart` |
| Node.js (HTTP) | [`node/quickstart.mjs`](node/quickstart.mjs) | image | `node examples/node/quickstart.mjs` |
| Python (HTTP, async) | [`python/quickstart.py`](python/quickstart.py) | image | `uv run examples/python/quickstart.py` |
| Go (HTTP) | [`go/quickstart.go`](go/quickstart.go) | image | `(cd examples/go && go run .)` |
| Java (HTTP) | [`java/Quickstart.java`](java/Quickstart.java) | image | `java --class-path <heeczer-sdk-jar> examples/java/Quickstart.java` |
| `heec` CLI | [`cli.md`](cli.md) | native | `heec score examples/event.json` |

_Mode legend: **native** runs the Rust scoring core in-process (no
service required); **image** posts to a running ingestion service over
HTTP. Native parity for the JS/Python/Go/Java SDKs tracks plans 0005–0007
and 0009; until then those four SDKs are HTTP-only._

All HTTP examples assume the ingestion service is running locally on
`http://127.0.0.1:8080`. To start it:

```bash
cargo run -p heeczer-ingest
```

### Prerequisites

- **Rust toolchain** (stable, ≥ 1.85): `rustup toolchain install stable`. The
  first `cargo build` takes ~2–3 min to compile the workspace from scratch.
- **Port 8080** available on localhost (for HTTP examples).
- **`HEECZER_API_KEY`**: optional. The service is permissive by default;
  set the env-var only if you have enabled key enforcement.
- **Per-language runtimes**: Node ≥ 20, Python ≥ 3.11, Go ≥ 1.22, Java ≥ 17.

In a separate terminal, set `HEECZER_BASE_URL` if the service is not on
the default port and run any example above.

## Where the schema lives

- Canonical event schema: [`core/schema/event.v1.json`](../core/schema/event.v1.json).
- Reference valid fixtures (different categories, outcomes, frameworks):
  [`core/schema/fixtures/events/valid/`](../core/schema/fixtures/events/valid/):

  | Fixture | Category | Outcome | Risk | HIL | Framework |
  | --- | --- | --- | --- | --- | --- |
  | `01-prd-canonical.json` | code_generation | success | medium | no | langgraph |
  | `02-summarization-human-in-loop.json` | summarization | success | low | yes | langgraph |
  | `03-rca-failure-high-risk.json` | root_cause_analysis | failure | high | yes | google_adk |
  | `04-planning-architecture-partial.json` | planning_architecture | partial_success | medium | yes | langgraph |
  | `05-regulated-decision-support.json` | regulated_decision_support | success | high | yes | google_adk |
  | `06-drafting-timeout.json` | drafting | timeout | low | no | langgraph |
  | `07-ci-triage-tool-heavy.json` | code_generation | success | low | no | google_adk |

- Default scoring profile: [`core/schema/profiles/default.v1.json`](../core/schema/profiles/default.v1.json).
- Default tier set: [`core/schema/tiers/default.v1.json`](../core/schema/tiers/default.v1.json).

## Where the per-SDK reference docs live

- Rust SDK: [`bindings/heeczer-rs/README.md`](../bindings/heeczer-rs/README.md)
- JS/TS SDK: [`bindings/heeczer-js/README.md`](../bindings/heeczer-js/README.md)
- Python SDK: [`bindings/heeczer-py/README.md`](../bindings/heeczer-py/README.md)
- Go SDK: [`bindings/heeczer-go/README.md`](../bindings/heeczer-go/README.md)
- Java SDK: [`bindings/heeczer-java/README.md`](../bindings/heeczer-java/README.md)

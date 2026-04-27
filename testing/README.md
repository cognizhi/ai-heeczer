# Local SDK test stacks

Plan 0016 adds opt-in, per-SDK sandboxes for exercising a chatbot, the matching
ai-heeczer SDK surface, the ingestion service, persisted storage, and the
dashboard together. The stacks are intentionally outside `make test`; they are
heavier than the normal local gate and are meant for hands-on SDK exploration or
nightly smoke coverage.

## Quickstart

Prerequisites: Docker with Compose, Rust/Cargo for the release ingest build,
and `uv` for the smoke harness.

```bash
cp testing/compose/js/.env.example testing/compose/js/.env
make start-test-js
make smoke-test-js
```

Open the JS stack at:

| Surface | URL |
| --- | --- |
| Chat UI | `http://127.0.0.1:18000` |
| Chat API | `http://127.0.0.1:18001` |
| Ingest service | `http://127.0.0.1:18010` |
| Dashboard | `http://127.0.0.1:18020` |

Use `make stop-test-js` to stop containers while keeping the database volume.
Use `make reset-test-js CONFIRM=1` to drop that stack's persisted data.

Current stack helpers use SQLite only: `testing/compose/_shared/ingest.yml` sets
`HEECZER_DATABASE_URL=sqlite:/data/heeczer.db?mode=rwc`. A Postgres compose
fragment exists under `testing/compose/_shared/postgres.yml`, but it is reserved
for the follow-up that wires PostgreSQL pool switching into the HTTP service;
see `services/heeczer-ingest/README.md` for the current runtime limitation.

## Port Matrix

Each stack owns a 100-port band so multiple stacks can run together.

| Offset | Service | JS | Py | Go | Java | Rust | PydAI |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| +00 | chatbot-ui | 18000 | 18100 | 18200 | 18300 | 18400 | 18500 |
| +01 | chatbot-api | 18001 | 18101 | 18201 | 18301 | 18401 | 18501 |
| +10 | heeczer-ingest | 18010 | 18110 | 18210 | 18310 | 18410 | 18510 |
| +20 | dashboard | 18020 | 18120 | 18220 | 18320 | 18420 | 18520 |
| +32 | reserved postgres fragment | 18032 | 18132 | 18232 | 18332 | 18432 | 18532 |
| +79 | ollama | 18079 | 18179 | 18279 | 18379 | 18479 | 18579 |

## Make Targets

| Target | Purpose |
| --- | --- |
| `make start-test-<sdk>` | Build prerequisites and bring one stack up. |
| `make stop-test-<sdk>` | Stop one stack and keep persisted data. |
| `make reset-test-<sdk>` | Stop one stack and drop its database volume after confirmation. |
| `make logs-test-<sdk>` | Tail stack logs. |
| `make ps-test-<sdk>` | Show stack container state. |
| `make smoke-test-<sdk>` | Run the Python smoke harness for one stack. |
| `make start-test-all` | Bring up all six stacks. |
| `make stop-test-all` | Stop all six stacks. |
| `make reset-test-all` | Drop all six persisted stack volumes after confirmation. |
| `make smoke-test-stacks` | Run the smoke harness for every stack. |

Supported SDK keys are `js`, `py`, `pydanticai`, `go`, `java`, and `rs`.

## LLM Providers

Every stack reads `testing/compose/<sdk>/.env`, copied from `.env.example`.

| `LLM_PROVIDER` | Required env | Notes |
| --- | --- | --- |
| `mock` | none | Deterministic and hermetic; used by smoke tests. |
| `openrouter` | `OPENROUTER_API_KEY`, `OPENROUTER_MODEL` | Posts OpenAI-compatible chat completions to `https://openrouter.ai/api/v1/chat/completions`. |
| `gemini` | `GEMINI_API_KEY`, `GEMINI_MODEL` | Uses Gemini's OpenAI-compatible endpoint at `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions`. |
| `local` | `LOCAL_MODEL` | Uses `LOCAL_MODEL_BASE_URL`, defaulting to `http://ollama:11434`; starts the `local-model` compose profile. |

### Mock Provider Contract

The mock provider never calls a network LLM. Given:

```json
{ "skill": "compliance", "prompt": "Summarize this local SDK stack" }
```

it loads `testing/tests/fixtures/skills/compliance.json`, replays the listed
`mock_script` tool calls, builds one canonical event, submits it through the SDK
client to `heeczer-ingest`, and returns:

```json
{
  "ok": true,
  "skill": "compliance",
  "event_id": "<uuid>",
  "reply": "Mock compliance turn completed.",
  "tool_trace": [
    { "tool_name": "document_reader", "invoked_at_ms": 0, "output_size": 0.0 }
  ],
  "event": { "spec_version": "1.0" },
  "score_result": { "scoring_version": "1.0.0" }
}
```

Only metric fields backed by provider usage or tool trace data are populated.
For real providers, missing token counts are emitted as `null` so the scoring
confidence penalty stays visible.

## Built-in Skills

| Skill | Command | Category | What it exercises |
| --- | --- | --- | --- |
| `code_gen` | `/skill code-gen` | `code_generation/api_design` | Code execution, diff generation, and summary artifacts. |
| `rca` | `/skill rca` | `root_cause_analysis/debugging` | Search, failed execution, retry accounting, high risk, human review. |
| `doc_summary` | `/skill doc-summary` | `summarization/document_review` | Document-token load, summary artifact, and review-required context. |
| `compliance` | `/skill compliance` | `regulated_decision_support/compliance_briefing` | High-risk regulated decision support with review and deterministic temperature. |
| `ci_triage` | `/skill ci-triage` | `code_generation/ci_triage` | Tool-heavy but lower-risk CI diagnosis. |
| `architecture` | `/skill architecture` | `planning_architecture/architecture_review` | Multi-step planning with search, review, analysis, and summary artifacts. |

The fixture files under `testing/tests/fixtures/skills/` are the smoke-test
source of truth for expected event shape.

## Troubleshooting

- `missing testing/compose/<sdk>/.env`: copy `.env.example` and use
  `LLM_PROVIDER=mock` for smoke tests.
- Port already in use: choose a different stack or stop the process occupying the
  port shown in the matrix.
- Ingest is unreachable: run `make logs-test-<sdk>` and confirm the cargo release
  binary built successfully before compose started.
- Direct `pytest testing/tests/smoke` runs skip when stacks are down. The
  `make smoke-test-<sdk>` targets set `HEECZER_REQUIRE_STACK=1` and fail if the
  stack is unreachable; run `make ps-test-<sdk>` to verify container state.

# Plan 16 — Local per-SDK test stacks

- **Status:** Active
- **Owner:** DevEx Engineer (with SDK Engineer + Test Engineer co-owners)
- **Last reviewed:** 2026-04-27
- **PRD:** §12.13 (Make surface), §12.21 (local docker), §23 (SDKs), §24 (framework adapters), §26 (DevEx)
- **ADR:** ADR-0005 (ingestion service), ADR-0007 (monorepo tooling), ADR-0008 (dashboard),
  ADR-0010 (local CLI), ADR-0011 (C-ABI envelope)
- **Supersedes / refines:** the single `docker-compose.dev.yml` bullet in
  [plan 0013](0013-developer-experience.md) §"Local containers".

## 1. Goal

Give a contributor — or a curious user — a one-command path to a fully working,
opinionated, isolated, _language-specific_ sandbox where they can poke at a real
agent that emits real ai-heeczer events and watch them scored in a real
dashboard, against a real persisted database. Per SDK, plus a PydanticAI variant.

```bash
make start-test-js          # Node/TS chatbot stack
make start-test-py          # Python (vanilla SDK) chatbot stack
make start-test-pydanticai  # Python + PydanticAI agent stack
make start-test-go          # Go chatbot stack
make start-test-java        # Java chatbot stack
make start-test-rs          # Rust chatbot stack
```

Each stack:

- Self-contained `docker compose` project (isolated network + volumes + project name).
- Uses a **dedicated port band** so multiple stacks can run side-by-side with
  zero collisions.
- Persists its database across `down` / `up` cycles (named volume), and exposes
  a separate `make reset-test-<sdk>` to wipe it deliberately.
- Ships a basic chatbot wired to the matching ai-heeczer SDK out of the box,
  pluggable across **OpenRouter**, **Gemini**, or a **locally hosted model**
  (Ollama by default) via env-only configuration.
- Brings up the ai-heeczer ingestion service and dashboard so emitted events
  show up immediately.
- Reads secrets from a per-stack `.env` file the contributor populates from a
  committed `.env.example`. No secrets ever committed.
- Built TDD-first: every stack lands behind a smoke test (`make smoke-test-<sdk>`)
  that fails before the stack exists and passes after.

## 2. Non-goals

- Not a production deployment template. Not a load-test rig.
- Not a replacement for `make test` (the canonical fast local gate).
- Not where parity tests live (those stay in plans 0005–0009).
- No multi-tenant auth, no TLS, no horizontal scaling. Single-workspace,
  single-replica, localhost-only.
- Does not change SDK or ingestion contracts; consumes them as-is.

## 3. Reading order

1. PRD §12.13, §12.21, §23, §24, §26.
2. ADR-0005, ADR-0007, ADR-0008, ADR-0010, ADR-0011.
3. [`services/heeczer-ingest/README.md`](../../services/heeczer-ingest/README.md)
   — env vars, endpoints, body limits.
4. [`dashboard/README.md`](../../dashboard/README.md) — runtime config.
5. [`examples/README.md`](../../examples/README.md) — existing per-language
   quickstarts; the chatbot apps build directly on these.
6. [Plan 0011](0011-framework-adapters.md) — adapter contract for the
   PydanticAI stack.
7. Plan 0013 §"Local containers" — note this plan refines that bullet.

## 4. Architecture

### 4.1 Topology (per stack)

```text
                ┌────────────────────────────────────────────────────┐
                │  docker compose project: heeczer-test-<sdk>        │
                │                                                    │
   user ──► chatbot-ui ──► chatbot-api ──► heeczer-sdk-<sdk>         │
                                              │                      │
                                              ▼  HTTP /v1/events     │
                                          heeczer-ingest ──► SQLite volume│
                                              ▲                      │
                                              │ read-only            │
                                          dashboard ◄────────────────┤
                                                                     │
                       ┌──────────── optional ────────────┐          │
                       │  ollama (local model runtime)    │          │
                       └──────────────────────────────────┘          │
                └────────────────────────────────────────────────────┘
```

Components:

| Service          | Image source                             | Purpose                                                  |
| ---------------- | ---------------------------------------- | -------------------------------------------------------- |
| `chatbot-ui`     | per-SDK `Dockerfile`                     | Minimal browser UI (or curlable HTTP) for chatting.      |
| `chatbot-api`    | per-SDK `Dockerfile`                     | Calls chosen LLM provider, wraps each turn with the SDK. |
| `heeczer-ingest` | built from `services/heeczer-ingest`     | Validates + scores + persists events.                    |
| `dashboard`      | built from `dashboard/`                  | Read-only Next.js dashboard.                             |
| `postgres`       | upstream `postgres:16-alpine`            | Reserved follow-up fragment until HTTP runtime PostgreSQL pool switching lands. |
| `ollama`         | upstream `ollama/ollama` (profile-gated) | Local model host. Off by default.                        |

### 4.2 Port allocation matrix

Each SDK gets a **100-port band** starting at `18000`. Within a band, offsets are
identical across stacks so docs and scripts can use a single mental model.

| Offset | Service        |    JS |    Py |    Go |  Java |  Rust | PydAI |
| -----: | -------------- | ----: | ----: | ----: | ----: | ----: | ----: |
|    +00 | chatbot-ui     | 18000 | 18100 | 18200 | 18300 | 18400 | 18500 |
|    +01 | chatbot-api    | 18001 | 18101 | 18201 | 18301 | 18401 | 18501 |
|    +10 | heeczer-ingest | 18010 | 18110 | 18210 | 18310 | 18410 | 18510 |
|    +20 | dashboard      | 18020 | 18120 | 18220 | 18320 | 18420 | 18520 |
|    +32 | postgres       | 18032 | 18132 | 18232 | 18332 | 18432 | 18532 |
|    +79 | ollama         | 18079 | 18179 | 18279 | 18379 | 18479 | 18579 |

The ingest port `18010` deliberately differs from the dev default `8080`
(see [`services/heeczer-ingest/README.md`](../../services/heeczer-ingest/README.md))
to make it obvious which stack a contributor is hitting.

### 4.3 Filesystem layout

```text
testing/
├── README.md                       # index of all stacks, how to use, troubleshooting
├── compose/
│   ├── _shared/
│   │   ├── ingest.yml              # heeczer-ingest service fragment (compose include)
│   │   ├── dashboard.yml           # dashboard service fragment
│   │   ├── postgres.yml            # postgres service fragment + named volume
│   │   └── ollama.yml              # optional local model service fragment (profile)
│   ├── js/
│   │   ├── docker-compose.yml
│   │   ├── .env.example
│   │   └── chatbot/                # Node/TS chatbot app
│   │       ├── src/
│   │       │   ├── tools/
│   │       │   │   └── catalogue.ts        # §4.8 tool schemas + stubs
│   │       │   └── skills/
│   │       │       └── catalogue.ts        # §4.9 skill definitions + mock scripts
│   │       └── ...
│   ├── py/
│   │   ├── docker-compose.yml
│   │   ├── .env.example
│   │   └── chatbot/                # Python chatbot app (FastAPI)
│   │       ├── tools/catalogue.py
│   │       ├── skills/catalogue.py
│   │       └── ...
│   ├── pydanticai/
│   │   ├── docker-compose.yml
│   │   ├── .env.example
│   │   └── chatbot/                # PydanticAI agent app
│   │       ├── tools/catalogue.py  # declared as pydantic-ai Tool objects
│   │       ├── skills/catalogue.py
│   │       └── ...
│   ├── go/        ...              # tools/catalogue.go, skills/catalogue.go
│   ├── java/      ...              # tools/Catalogue.java, skills/SkillCatalogue.java
│   └── rs/        ...              # src/tools/catalogue.rs, src/skills/catalogue.rs
└── tests/
    ├── smoke/                      # cross-stack pytest smoke harness (drives HTTP)
    │   ├── conftest.py
    │   ├── test_js_stack.py
    │   ├── test_py_stack.py
    │   ├── test_pydanticai_stack.py
    │   ├── test_go_stack.py
    │   ├── test_java_stack.py
    │   └── test_rs_stack.py
    └── fixtures/
        ├── prompts/
        │   └── 01-summarize.json   # canonical chat turn → expected event shape
        └── skills/                 # per-skill mock scripts + expected event shapes (§4.10)
            ├── code_gen.json
            ├── rca.json
            ├── doc_summary.json
            ├── compliance.json
            ├── ci_triage.json
            └── architecture.json
```

Rationale for `testing/` (new top-level): keeps the contributor "play" surface
distinct from `examples/` (which stays minimal, copy-pastable, doc-driven) and
from `services/` (production code paths). Each chatbot app is intentionally a
**reference implementation**, not a library — vendored into `testing/compose/<sdk>/chatbot/`.

### 4.4 LLM provider abstraction

Each chatbot app must support four providers, selected by `LLM_PROVIDER`:

| `LLM_PROVIDER` | Required env                                                          | Notes                                                                     |
| -------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `mock`         | none                                                                  | Deterministic, hermetic provider used by smoke tests.                     |
| `openrouter`   | `OPENROUTER_API_KEY`, `OPENROUTER_MODEL`                              | HTTP to `https://openrouter.ai/api/v1/chat/completions`.                  |
| `gemini`       | `GEMINI_API_KEY`, `GEMINI_MODEL`                                      | Google `generativelanguage` v1beta, OpenAI-compatible endpoint preferred. |
| `local`        | `LOCAL_MODEL_BASE_URL` (default `http://ollama:11434`), `LOCAL_MODEL` | Activates the `local-model` compose profile; pulls model on first run.    |

Provider clients are thin (no SDK lock-in) so the chatbot stays a faithful
SDK-integration demo, not a framework demo. **Use `mcp_context7_get-library-docs`
during implementation to confirm the current OpenRouter and Gemini request/response
shapes** — both APIs evolve frequently and any drift is caught immediately by the
smoke harness in §6.

### 4.5 Heeczer integration contract per chatbot

Every chatbot, regardless of language, must on **each chat turn**:

1. Call the LLM provider.
2. Build a canonical event matching `core/schema/event.v1.json` from that turn
   (workspace_id, agent identity, tool call summary, latency, outcome).
3. Submit it via the matching SDK's high-level `submit(event)` API to
   `http://heeczer-ingest:8080` (in-network) or the host port (from the host).
4. Surface the returned `score_result` in the chat UI (collapsible panel).

This pins each stack to the **public SDK surface only** — if the SDK shape
breaks, the smoke test breaks. That is the whole point.

### 4.6 Persistence

- Active persistence uses one SQLite data volume per stack: `heeczer-test-<sdk>-data`.
- `docker compose down` keeps it. `down -v` (used by `make reset-test-<sdk>`) drops it.
- Current foundation stacks use the shipped SQLite HTTP runtime path, persisted
  in the `heeczer-test-<sdk>-data` volume at `/data/heeczer.db`. The Postgres
  fragment is reserved until runtime PostgreSQL pool switching is wired into the
  ingestion service; see [`services/heeczer-ingest/README.md`](../../services/heeczer-ingest/README.md).
- The reserved Postgres fragment uses `heeczer-test-<sdk>-pgdata` when enabled
  by a future runtime PostgreSQL slice.
- Migrations run automatically by `heeczer-ingest` boot — no separate migrate
  step.

### 4.7 Networking

- Each compose project gets its own bridge network, `heeczer-test-<sdk>-net`.
- Inter-service hostnames are stable (`heeczer-ingest`, `dashboard`, `postgres`,
  `ollama`) so chatbot env vars can be identical across stacks except for the
  host-side port mapping in `.env.example`.

### 4.8 Built-in tool catalogue

Every chatbot ships the same **eight function-call tools** regardless of SDK or
LLM provider. They are declared as standard function-calling schemas sent to the
LLM on every turn; in `LLM_PROVIDER=mock` mode the chatbot's mock driver
hard-codes which tools are "called" based on the active skill (§4.9).

The right column shows the `metrics.*` and `context.*` fields the chatbot **must
increment or set** when the LLM invokes that tool. These are the signals heeczer
uses for effort-cost calculation (token divisors, tool weight, step weight,
artifact weight, risk/HiL multipliers — see `scoring_profile.v1.json`).

| Tool name         | Function signature (summary)                   | `metrics.*` contribution                                                                        | `context.*` side-effect                                         |
| ----------------- | ---------------------------------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| `web_search`      | `(query: str) → SearchResult`                  | `tool_call_count +1`, `workflow_steps +1`, `tokens_prompt ~+500`                                | —                                                               |
| `code_executor`   | `(language: str, code: str) → ExecutionResult` | `tool_call_count +1`, `workflow_steps +1`, `artifact_count +1`, `output_size_proxy ~+0.5`       | —                                                               |
| `document_reader` | `(source: str) → DocumentChunks`               | `tool_call_count +1`, `workflow_steps +1`, `tokens_prompt ~+2000`                               | —                                                               |
| `data_analyst`    | `(data: str, query: str) → AnalysisResult`     | `tool_call_count +1`, `artifact_count +1`, `tokens_completion ~+800`, `output_size_proxy ~+1.0` | —                                                               |
| `plan_reviewer`   | `(plan_text: str) → ReviewNotes`               | `tool_call_count +1`, `workflow_steps +1`                                                       | `review_required = true`                                        |
| `risk_checker`    | `(action: str) → RiskAssessment`               | `tool_call_count +1`                                                                            | elevates `risk_class` to `high` when score ≥ 0.7, else `medium` |
| `summarizer`      | `(content: str, max_words: int) → Summary`     | `tool_call_count +1`, `artifact_count +1`, `output_size_proxy ~+0.8`                            | —                                                               |
| `diff_generator`  | `(before: str, after: str) → UnifiedDiff`      | `tool_call_count +1`, `artifact_count +1`, `output_size_proxy ~+0.3`                            | —                                                               |

Implementation notes:

- The tool catalogue lives in a shared module inside each chatbot app
  (`tools/catalogue.{ts,py,go,java,rs}`). Only the schema (function declaration
  JSON) and a thin stub implementation are provided; stubs return plausible
  synthetic data sufficient for demonstration.
- Real integrations (actual web search, real code sandboxes) are **out of scope**
  for the test stacks. Stubs are fine and make the stacks fast and hermetic.
- The chatbot accumulates a `ToolTrace` across the turn (list of
  `{tool_name, invoked_at_ms, output_size}` entries), then derives all
  `metrics.*` deltas from it before building the event. This trace is also shown
  in the chat UI (collapsible "Tool calls" panel alongside the score panel).
- `retries` is incremented by the chatbot if the LLM returns a malformed tool
  call or the stub throws, and the turn is retried. The mock provider triggers
  one deliberate retry for the `rca` skill (§4.9) to demonstrate the retry
  context multiplier.

### 4.9 Built-in skill catalogue

A **skill** is a pre-wired scenario the user selects from the chat UI via a
`/skill <name>` command or a dropdown. Selecting a skill:

1. Pre-fills a scenario-appropriate **system prompt** for the LLM.
2. Limits the **active tool subset** (only those function declarations are sent).
3. Sets default **`task.*`** and **`context.*`** fields on the outgoing event.
4. Controls the **mock driver** so `LLM_PROVIDER=mock` produces a deterministic
   tool call sequence — making smoke tests fully hermetic.

The six built-in skills span the full range of heeczer's scoring dimensions:

| Skill key      | UI command            | `task.category`              | `task.sub_category`   | Active tools                                                     | Default context flags                                                              | FEC signal band             |
| -------------- | --------------------- | ---------------------------- | --------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------- | --------------------------- |
| `code_gen`     | `/skill code-gen`     | `code_generation`            | `api_design`          | `code_executor`, `diff_generator`, `summarizer`                  | `risk_class=medium`                                                                | medium–high                 |
| `rca`          | `/skill rca`          | `root_cause_analysis`        | `debugging`           | `web_search`, `code_executor`, `diff_generator`                  | `risk_class=high`, `review_required=true`, `human_in_loop=true`                    | high                        |
| `doc_summary`  | `/skill doc-summary`  | `summarization`              | `document_review`     | `document_reader`, `summarizer`, `plan_reviewer`                 | `human_in_loop=true`, `review_required=true`                                       | medium                      |
| `compliance`   | `/skill compliance`   | `regulated_decision_support` | `compliance_briefing` | `document_reader`, `risk_checker`, `plan_reviewer`, `summarizer` | `risk_class=high`, `human_in_loop=true`, `review_required=true`, `temperature=0.0` | high (regulated multiplier) |
| `ci_triage`    | `/skill ci-triage`    | `code_generation`            | `ci_triage`           | `web_search`, `code_executor`, `diff_generator`, `risk_checker`  | `risk_class=low`                                                                   | medium                      |
| `architecture` | `/skill architecture` | `planning_architecture`      | `architecture_review` | `web_search`, `plan_reviewer`, `data_analyst`, `summarizer`      | `risk_class=medium`, `review_required=true`                                        | medium–high (multi-step)    |

Each skill also ships a **mock script** (`testing/tests/fixtures/skills/<skill>.json`)
that specifies the exact tool call sequence the mock driver replays, enabling
deterministic smoke tests (see §4.10 and §6).

Scoring range intent: the six skills are chosen so that, with the default scoring
profile (`scoring_profile.v1.json`), they produce meaningfully _different_
`fec` / `financial_equivalent_cost` (Financial Equivalent Cost) values in the dashboard. This lets a contributor
immediately see heeczer's scoring variation without needing a real workload.

### 4.10 Event composition contract

The chatbot builds one canonical event per completed chat turn. Every field must
be derivable from the turn's execution data without accessing heeczer internals:

```text
event_id          ← UUID generated fresh per turn
correlation_id    ← session_id + ":" + turn_counter  (stable across a session)
timestamp         ← turn end time (UTC RFC 3339)
framework_source  ← e.g. "chatbot-js", "chatbot-py", "chatbot-pydanticai"
workspace_id      ← CHATBOT_WORKSPACE_ID env (default: "local-test-<sdk>")
project_id        ← CHATBOT_PROJECT_ID env (optional)

task.name         ← active skill key + ": " + first 80 chars of user prompt
task.category     ← skill.task_category
task.sub_category ← skill.task_sub_category
task.outcome      ← "success" | "partial_success" | "failure" | "timeout"
                    (derived from whether the LLM/tools completed cleanly)

metrics.duration_ms     ← turn_end_ms − turn_start_ms
metrics.tokens_prompt   ← sum(tokens from LLM provider response.usage.prompt_tokens
                              + estimated tool input tokens from ToolTrace)
metrics.tokens_completion ← response.usage.completion_tokens
metrics.tool_call_count ← len(ToolTrace)
metrics.workflow_steps  ← sum(tool.workflow_steps_contribution per ToolTrace entry)
metrics.retries         ← turn retry count (0 for clean turns)
metrics.artifact_count  ← sum(tool.artifact_count_contribution per ToolTrace entry)
metrics.output_size_proxy ← sum(tool.output_size_proxy_contribution per ToolTrace entry)

context.human_in_loop   ← skill.default_human_in_loop (overridable by user toggle)
context.review_required ← skill.default_review_required OR plan_reviewer in ToolTrace
context.temperature     ← skill.default_temperature (if set) else provider default
context.risk_class      ← max(skill.default_risk_class, risk_checker output if invoked)
context.tags            ← ["local-stack", "<sdk>", active_skill_key]

meta.sdk_language       ← per-SDK literal ("node" | "python" | "go" | "java" | "rust")
meta.sdk_version        ← resolved from the installed SDK package at startup
meta.scoring_profile    ← CHATBOT_SCORING_PROFILE env (default: "default")
meta.extensions         ← { "chatbot.skill": skill_key,
                             "chatbot.turn": turn_counter,
                             "chatbot.tool_trace": [<tool_name>, …] }
```

The chatbot **must not** guess or fabricate metric values; if a value is
unavailable (e.g. provider did not return token counts) it must emit `null`
rather than a placeholder integer, so the scoring engine's confidence penalty
for missing signals fires correctly — demonstrating that mechanism too.

## 5. Make surface

All targets are added to the existing root `Makefile` and listed in
`make help`. Each target shells out to a small bash helper under
`testing/compose/_bin/` so the Makefile stays declarative.

```make
# Per-SDK lifecycle (NN ∈ {js, py, pydanticai, go, java, rs})
start-test-<NN>     ## bring up the <NN> SDK test stack (idempotent)
stop-test-<NN>      ## stop the <NN> SDK test stack, keep the database
reset-test-<NN>     ## stop and DROP the <NN> SDK database volume (destructive)
logs-test-<NN>      ## tail logs for the <NN> SDK test stack
ps-test-<NN>        ## show running services for the <NN> SDK test stack
smoke-test-<NN>     ## run the cross-stack smoke harness against <NN>

# Convenience aggregates
start-test-all      ## bring up every stack (CI rarely uses this; warns on RAM)
stop-test-all       ## stop every stack, keep databases
reset-test-all      ## DROP every test database (double-confirm prompt)
smoke-test-stacks   ## run smoke harness across every running stack
```

Rules:

- `reset-test-*` prompts `Type the stack name to confirm:` unless `CONFIRM=1` is
  set (CI uses `CONFIRM=1`). Aligns with operationalSafety guardrails.
- `start-test-*` is idempotent — safe to re-run; recreates only changed services.
- `start-test-*` refuses to run if the corresponding `.env` is missing required
  keys, with an actionable error message naming the missing key(s).
- No target in this plan is added to the default `make test` gate. Stacks are
  opt-in. A planned nightly CI job, tracked as a plan 0012 follow-up, will run
  `smoke-test-stacks` against a matrix of stacks with mocked LLM providers.

## 6. TDD strategy

Order of operations for every stack slice (mirrors AGENT_HARNESS §2):

1. **Contract first — skill fixtures.** Before writing any app code, author the
   six skill fixture files under `testing/tests/fixtures/skills/`. Each file has
   two sections:
    - `mock_script`: the ordered list of tool calls the mock driver will replay
      (tool name + stub output shape). This defines deterministic LLM behaviour
      for smoke tests.
    - `expected_event`: the canonical event shape that _must_ be submitted to
      heeczer when the mock script is replayed. All `metrics.*` fields that the
      tool catalogue (§4.8) contributes must be present and within expected bounds.
      `task.category`, `task.sub_category`, and relevant `context.*` fields must
      match the skill definition (§4.9). The expected `score_result` is **not**
      pinned — scoring is the engine's job — but `score_result.scoring_version`
      must match the engine.
    - Example structure:

        ```json
        {
            "skill": "compliance",
            "mock_script": [
                {
                    "tool": "document_reader",
                    "stub_output": { "chunks": 3, "tokens": 2000 }
                },
                {
                    "tool": "risk_checker",
                    "stub_output": { "risk_score": 0.85 }
                },
                {
                    "tool": "plan_reviewer",
                    "stub_output": { "notes": ["item1"] }
                },
                { "tool": "summarizer", "stub_output": { "word_count": 320 } }
            ],
            "expected_event": {
                "task": {
                    "category": "regulated_decision_support",
                    "sub_category": "compliance_briefing",
                    "outcome": "success"
                },
                "metrics": {
                    "tool_call_count": 4,
                    "workflow_steps": 3,
                    "artifact_count": 2,
                    "tokens_prompt_min": 2000
                },
                "context": {
                    "risk_class": "high",
                    "human_in_loop": true,
                    "review_required": true,
                    "temperature": 0.0
                }
            }
        }
        ```

2. **Failing smoke tests.** Add `testing/tests/smoke/test_<sdk>_stack.py`. Each
   test function maps to one skill and must fail before the chatbot app exists:
    - Skips with a clear message if `docker compose ps` shows the stack down.
    - Posts `/chat` with `{ "skill": "<key>", "prompt": "…" }` against the mock
      provider.
    - Polls `heeczer-ingest` via the exact event endpoint returned by `/chat`
      (`/v1/events/{event_id}?workspace_id=...`) until an event with
      `meta.extensions["chatbot.skill"] == skill_key` appears. The service does
      not expose a list-events endpoint in this slice.
    - Asserts the event shape against the fixture (tool_call_count, artifact_count,
      category, sub_category, context flags, and the absence of fabricated nulls
      on required fields).
    - Asserts `score_result` is present and `score_result.scoring_version` matches
      `/v1/version`.
    - For `compliance` and `rca` skills, additionally asserts that
      `score_result.fec` is strictly greater than `score_result.fec` for the
      `ci_triage` skill run in the same test session — confirming the scoring
      engine reflects effort complexity.
3. **Implement** the tool catalogue + skill router + chatbot app + compose file +
   Make targets to make all six skill tests pass.
4. **Make `smoke-test-<sdk>` green** locally with all six skills covered, then
   wire into the nightly job in a follow-up under plan 0012.
5. **Docs:** update `testing/README.md` skill table and the per-stack section.

The smoke harness lives in `testing/tests/smoke/` (Python + `pytest` + `httpx`)
because Python is already an in-tree language with `uv`, and a single harness
keeps assertions consistent across SDKs.

## 7. Security

- `.env.example` is committed; `.env` is `.gitignore`d. Pre-commit hook check
  (DevEx follow-up) refuses commits that contain known secret-key patterns
  for OpenRouter/Gemini.
- Containers run as non-root where the upstream image allows.
- Postgres is **never** exposed to `0.0.0.0`; the host port mapping binds to
  `127.0.0.1` only. Same for `ollama` and `heeczer-ingest`.
- The `chatbot-api` does not log full prompt/response bodies by default
  (privacy), only token counts and turn IDs. A `CHATBOT_DEBUG=1` env opens
  full bodies behind an explicit opt-in.
- No real LLM provider key is ever required to run smoke tests
  (`LLM_PROVIDER=mock`). Real providers are for manual exploration only.
- Version-tagged shared images are used in the foundation slice; digest pinning
  remains a supply-chain follow-up before these stacks are treated as
  production-like infrastructure.
- Chatbots send only the canonical event fields the SDK exposes — no raw
  prompts or completions enter ai-heeczer storage. The fixture-defined event
  shape is the contract.

## 8. Checklist

### Foundations (one PR)

- [x] Add `testing/` skeleton: `README.md`, `compose/_shared/{ingest,dashboard,postgres,ollama}.yml`, `compose/_bin/` helpers.
- [x] Add `.gitignore` rules for `testing/compose/*/.env`.
- [x] Add `LLM_PROVIDER=mock` mode contract to `testing/README.md` (pseudocode + expected reply).
- [x] **Author all six skill fixture files** under `testing/tests/fixtures/skills/` (§6 step 1). Fixtures are schema-validated against `core/schema/event.v1.json` as part of the PR CI check.
- [x] Add `testing/tests/smoke/conftest.py` and shared smoke harness helpers for stack readiness, mock provider turns, event polling, score polling, fixture loading, and scoring comparisons.
- [x] Add Make targets `start/stop/reset/logs/ps/smoke-test-<sdk>` for each SDK.
- [x] Add aggregates `start-test-all`, `stop-test-all`, `reset-test-all`, `smoke-test-stacks`.
- [x] Update `make help` and root README "Local stacks" section.
- [x] Update plan 0013 to point its `docker-compose.dev.yml` bullet at this plan.

### JS/TS stack (depends on plan 0005)

- [x] Failing smoke tests `testing/tests/smoke/test_js_stack.py` — one test per skill (6 total).
- [x] `testing/compose/js/chatbot/src/tools/catalogue.ts` — all eight tool schemas + stubs (§4.8).
- [x] `testing/compose/js/chatbot/src/skills/catalogue.ts` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/js/chatbot/` Node + TypeScript HTTP app using `@cognizhi/heeczer-sdk`.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `testing/compose/js/docker-compose.yml` with port band 18000–18099.
- [x] `testing/compose/js/.env.example` with all four `LLM_PROVIDER` modes documented.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-js`.
- [x] JS stack compiles locally; full container smoke is handled by `make smoke-test-js` when the stack is running.

### Python stack (depends on plan 0006)

- [x] Failing smoke tests `test_py_stack.py` — one per skill (6 total).
- [x] `testing/compose/py/chatbot/tools/catalogue.py` — all eight tool schemas + stubs (§4.8).
- [x] `testing/compose/py/chatbot/skills/catalogue.py` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/py/chatbot/` FastAPI app using `heeczer` Python SDK.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `docker-compose.yml` with port band 18100–18199.
- [x] `.env.example`.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-py`.

### PydanticAI stack (depends on plan 0011 PydanticAI adapter)

- [x] Failing smoke tests `test_pydanticai_stack.py` — one per skill (6 total).
- [x] `testing/compose/pydanticai/chatbot/tools/catalogue.py` — tools declared as `pydantic-ai` `Tool` objects (§4.8).
      **Use `mcp_context7_get-library-docs` for `pydantic-ai` to confirm current `Tool` / `Agent` hook surface before implementing.**
- [x] `testing/compose/pydanticai/chatbot/skills/catalogue.py` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/pydanticai/chatbot/` adapter-backed PydanticAI stack path that transforms the adapter-generated event before submission.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `docker-compose.yml` with port band 18500–18599.
- [x] `.env.example`.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-pydanticai`.
- [x] Adapter-shape assertion in the smoke test (`meta.extensions["chatbot.skill"]`, `meta.extensions["chatbot.adapter_event_id"]`, and `task.category` match skill definition).

### Go stack (depends on plan 0007)

- [x] Failing smoke tests `test_go_stack.py` — one per skill (6 total).
- [x] `testing/compose/go/chatbot/tools/catalogue.go` — all eight tool schemas + stubs (§4.8).
- [x] `testing/compose/go/chatbot/skills/catalogue.go` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/go/chatbot/` Go HTTP service using `heeczer-go`.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `docker-compose.yml` with port band 18200–18299.
- [x] `.env.example`.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-go`.

### Java stack (depends on plan 0009)

- [x] Failing smoke tests `test_java_stack.py` — one per skill (6 total).
- [x] `testing/compose/java/chatbot/tools/Catalogue.java` — all eight tool schemas + stubs (§4.8).
- [x] `testing/compose/java/chatbot/skills/SkillCatalogue.java` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/java/chatbot/` Java HTTP service using `heeczer-java`.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `docker-compose.yml` with port band 18300–18399.
- [x] `.env.example`.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-java`.

### Rust stack (depends on plan 0008)

- [x] Failing smoke tests `test_rs_stack.py` — one per skill (6 total).
- [x] `testing/compose/rs/chatbot/src/tools/catalogue.rs` — all eight tool schemas + stubs (§4.8).
- [x] `testing/compose/rs/chatbot/src/skills/catalogue.rs` — six skill definitions + mock driver (§4.9).
- [x] `testing/compose/rs/chatbot/` Axum app using the `heeczer` Rust SDK HTTP client.
- [x] Browser UI exposes a skill selector and JSON response containing per-turn `ToolTrace`.
- [x] `docker-compose.yml` with port band 18400–18499.
- [x] `.env.example`.
- [x] Wire `start/stop/reset/logs/ps/smoke-test-rs`.

### CI integration (handed off to plan 0012)

- [ ] Nightly job `smoke-test-stacks` matrix (sdk = {js, py, pydanticai, go, java, rs}) using `LLM_PROVIDER=mock`.
- [ ] Job timeout 20 min per matrix cell; failure annotates the failing SDK.
- [ ] Container image digests rebuilt + cached per SDK.

### Docs

- [x] `testing/README.md` — quickstart, port matrix, troubleshooting, per-stack table.
- [x] Root README "Try a local stack" section linking here.
- [x] Update `examples/README.md` with a "Want a full sandbox?" pointer.
- [x] Tech Writer subagent review pass before marking the stack phase done.

Validation notes for this implementation slice:

- [x] `make help` includes the local stack targets.
- [x] `cargo test -p heeczer-core --test schema_validation plan_0016_skill_fixtures_materialise_valid_events` passes.
- [x] Local compile/import checks pass for Go, Java, Rust, JS stack, Python stack, PydanticAI stack, and the smoke harness.
- [x] Direct smoke harness run with stacks down skips cleanly; `make smoke-test-<sdk>` sets `HEECZER_REQUIRE_STACK=1` and fails unreachable stacks.

## 9. Acceptance

A stack slice is "Done" when:

1. `make start-test-<sdk>` from a clean clone with only `.env` populated brings
   the stack up green within 3 minutes (cold image build excluded).
2. The smoke test passes locally and in the nightly matrix job — all six skill
   tests pass, covering the full tool catalogue (§4.8) and skill catalogue (§4.9).
3. `make stop-test-<sdk> && make start-test-<sdk>` preserves all events from the
   previous session (persistence verified).
4. `make reset-test-<sdk> CONFIRM=1` returns the database to empty.
5. Two stacks can run concurrently with no port collision (matrix in §4.2 enforced).
6. The dashboard shows six distinct events with visibly different `fec` values for
   the `compliance`, `rca`, `doc_summary`, `architecture`, `code_gen`, and
   `ci_triage` skills — confirming the scoring engine reflects effort complexity.
7. `testing/README.md` and the per-stack table reflect the shipped behavior,
   including the skill table and a "What each skill exercises" explanation.
8. Tech Lead, DevEx, Test Engineer, and Tech Writer subagent sign-offs recorded
   on the implementing PR.

## 10. Open questions

- **Dashboard auth.** PRD §22 may require even local dashboards to gate behind an
  API key. Decide whether the test stacks ship a fixed dev key or stay open on
  127.0.0.1. Default proposal: open on 127.0.0.1; document the trade-off.
- **Ollama model size.** Default model (e.g. `llama3.2:1b`) must fit on a
  contributor laptop. Pick after a quick benchmark; document the override.
- **PydanticAI adapter location.** Lives in `bindings/heeczer-py` (preferred) or
  a separate `heeczer-pydanticai` extra? Resolve in plan 0011 before that slice.
- **Cleanup of orphaned volumes.** Add a `make doctor-test` target to list and
  optionally prune dangling `heeczer-test-*` volumes? Tracked as follow-up.

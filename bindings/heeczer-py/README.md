# cognizhi-heeczer (Python)

Async Python client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service, with strict Pydantic v2 event models.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

> **Pre-release.** `cognizhi-heeczer` is not on PyPI yet (see plan 0012).
> Until then, install from source: `uv sync --project bindings/heeczer-py`.

```bash
uv add cognizhi-heeczer
```

Requires Python ≥ 3.11.

## Usage

```python
import asyncio
from heeczer import HeeczerClient

async def main() -> None:
    async with HeeczerClient(
        base_url="https://ingest.example.com",
        api_key="…",
        mode="image",
        retry={"attempts": 2, "backoff_ms": 100},
    ) as client:
        result = await client.ingest_event(
            workspace_id="ws_default",
            event=canonical_event,  # see core/schema/event.v1.json
        )
        print(result["score"]["final_estimated_minutes"])

asyncio.run(main())
```

## Error handling

Every client method raises :class:`heeczer.HeeczerApiError` on a non-2xx
response. The error carries the closed `kind` enum from the ingestion
service's envelope:

```python
from heeczer import HeeczerApiError

try:
    await client.ingest_event(workspace_id="ws", event=bad_event)
except HeeczerApiError as err:
    if err.kind == "schema":
        ...
```

## Configuration

| Argument    | Type                               | Default      | Description                                                      |
| ----------- | ---------------------------------- | ------------ | ---------------------------------------------------------------- |
| `base_url`  | `str`                              | required     | Base URL of the ingestion service. Trailing slash is stripped.   |
| `api_key`   | `str \| None`                      | `None`       | Sent as `x-heeczer-api-key`.                                     |
| `mode`      | `"image" \| "native"`              | `"image"`    | `native` fails fast until the pyo3/maturin binding ships.        |
| `timeout`   | `float`                            | `10.0`       | Per-request timeout in seconds.                                  |
| `retry`     | `RetryPolicy \| None`              | `2 attempts` | Retries transient transport/status failures.                     |
| `transport` | `httpx.AsyncBaseTransport \| None` | `None`       | Inject a custom transport (e.g. `httpx.MockTransport` in tests). |

## Methods

| Method                                                               | HTTP                           | Returns                                                               |
| -------------------------------------------------------------------- | ------------------------------ | --------------------------------------------------------------------- |
| `healthz()`                                                          | `GET /healthz`                 | `bool`                                                                |
| `version()`                                                          | `GET /v1/version`              | `VersionResponse`                                                     |
| `ingest_event(workspace_id, event)`                                  | `POST /v1/events`              | `IngestEventResponse`                                                 |
| `test_score_pipeline(event, profile=…, tier_set=…, tier_override=…)` | `POST /v1/test/score-pipeline` | `TestPipelineResponse` (gated by the test-orchestration feature flag) |

## Error kinds

`HeeczerApiError.kind` is a closed `Literal` mirroring the ingestion
service envelope:

| Kind                       | When                                                       |
| -------------------------- | ---------------------------------------------------------- |
| `schema`                   | Event failed canonical schema validation.                  |
| `bad_request`              | Malformed JSON or missing top-level fields.                |
| `scoring`                  | Engine rejected a normalized event (e.g. unknown tier id). |
| `storage`                  | Persistence layer error.                                   |
| `not_found`                | Read endpoint did not find the resource.                   |
| `unauthorized`             | Missing, invalid, or revoked API key.                      |
| `forbidden`                | Auth or RBAC denied the request.                           |
| `conflict`                 | Duplicate idempotency key or conflicting event payload.    |
| `payload_too_large`        | Payload exceeded service limits.                           |
| `rate_limit_exceeded`      | Per-key or workspace quota was exceeded.                   |
| `feature_disabled`         | Endpoint exists but the feature flag is off.               |
| `unsupported_spec_version` | Event `spec_version` is not accepted.                      |
| `unavailable`              | Readiness or dependency check failed.                      |
| `unknown`                  | Non-JSON 5xx body; the raw text is in `api_message`.       |

## Runnable example

See [`examples/python/quickstart.py`](../../examples/python/quickstart.py)
and the cross-language index in [`examples/README.md`](../../examples/README.md).

## Common patterns

**Validate locally before sending** with the bundled Pydantic v2 models:

```python
from heeczer import EventModel, validate_event

model = validate_event(event)
await client.ingest_event(workspace_id="ws_default", event=model)
```

**Surface schema field errors from the service:**

```python
try:
    await client.ingest_event(workspace_id="ws", event=event)
except HeeczerApiError as err:
    if err.kind == "schema":
        # err.api_message contains the field-level detail from the server envelope.
        print("schema rejection:", err.api_message)
```

**Batching note.** The ingestion service exposes `POST /v1/events:batch`;
the SDK batch helper follows the public method expansion tracked in
[plan 0006](../../docs/plan/0006-sdk-python.md). Until then, use
`asyncio.gather` for concurrent single-event sends.

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface (`ScoreResult` is a `TypedDict`
with `total=False`).

## Synchronous client

For scripts and notebooks where no event loop is running, use the
`SyncHeeczerClient` wrapper:

```python
from heeczer import SyncHeeczerClient

with SyncHeeczerClient(base_url="https://ingest.example.com", api_key="…") as client:
    result = client.ingest_event(workspace_id="ws_default", event=my_event)
    print(result["score"]["final_estimated_minutes"])
```

`SyncHeeczerClient` exposes the same methods as `HeeczerClient`
(`healthz`, `version`, `ingest_event`, `test_score_pipeline`),
all blocking. Internally it calls `asyncio.get_event_loop().run_until_complete()`.
Do **not** mix it with an already-running asyncio loop (e.g. inside
`async def`); use `HeeczerClient` directly there instead.

## Framework adapters

### LangGraph

Wrap `SyncHeeczerClient` inside a custom LangGraph node to record each
agent step's cost before passing control downstream:

```python
from langgraph.graph import StateGraph
from heeczer import SyncHeeczerClient
from typing import TypedDict

class AgentState(TypedDict):
    event: dict
    score: dict | None

def heeczer_node(state: AgentState) -> AgentState:
    with SyncHeeczerClient(base_url="http://localhost:3000") as client:
        resp = client.ingest_event(
            workspace_id="ws_default",
            event=state["event"],
        )
    return {**state, "score": resp["score"]}

builder = StateGraph(AgentState)
builder.add_node("heeczer", heeczer_node)
```

In async LangGraph graphs call the `async` client instead:

```python
async def heeczer_node(state: AgentState) -> AgentState:
    async with HeeczerClient(base_url="http://localhost:3000") as client:
        resp = await client.ingest_event(
            workspace_id="ws_default",
            event=state["event"],
        )
    return {**state, "score": resp["score"]}
```

### Google ADK

Google's Agent Development Kit (ADK) uses an async event-loop internally.
Use `HeeczerClient` directly in ADK tool callbacks:

```python
from google.adk.tools import FunctionTool
from heeczer import HeeczerClient

async def record_step(workspace_id: str, event: dict) -> dict:
    """Record an agent step with heeczer and return the score."""
    async with HeeczerClient(base_url="http://localhost:3000") as client:
        return await client.ingest_event(workspace_id=workspace_id, event=event)

heeczer_tool = FunctionTool(record_step)
```

Attach `heeczer_tool` to your ADK `Agent` via `tools=[heeczer_tool]`.
The return value of `record_step` is serialised by ADK and forwarded as
the tool response to the model.

## Development

```bash
uv sync --extra dev
uv run pytest
uv run ruff check
uv run mypy
```

## License

MIT.

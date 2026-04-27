# cognizhi-heeczer (Python)

Async Python client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service, with strict Pydantic v2 event models.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

> **Pre-release.** `cognizhi-heeczer` is not on PyPI yet (see plan 0012).
> Until then, install from source: `uv sync --project bindings/heeczer-py`.

```bash
uv sync --project bindings/heeczer-py
```

Requires Python >= 3.11.

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
            event=canonical_event,
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

| Argument    | Type                               | Default      | Description                                                    |
| ----------- | ---------------------------------- | ------------ | -------------------------------------------------------------- |
| `base_url`  | `str`                              | required     | Base URL of the ingestion service. Trailing slash is stripped. |
| `api_key`   | `str \| None`                      | `None`       | Sent as `x-heeczer-api-key`.                                   |
| `mode`      | `"image" \| "native"`              | `"image"`    | `native` fails fast until the pyo3/maturin binding ships.      |
| `timeout`   | `float`                            | `10.0`       | Per-request timeout in seconds.                                |
| `retry`     | `RetryPolicy \| None`              | `2 attempts` | Retries transient transport/status failures.                   |
| `transport` | `httpx.AsyncBaseTransport \| None` | `None`       | Inject a custom transport (e.g. `httpx.MockTransport`).        |

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
from heeczer import validate_event

model = validate_event(event)
await client.ingest_event(workspace_id="ws_default", event=model)
```

**Surface schema field errors from the service:**

```python
try:
    await client.ingest_event(workspace_id="ws", event=event)
except HeeczerApiError as err:
    if err.kind == "schema":
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
all blocking. Internally it uses `asyncio.run()` when no loop is active.
Do **not** mix it with an already-running asyncio loop (e.g. inside
`async def`); use `HeeczerClient` directly there instead.

## Framework adapters

### LangGraph

Use the duck-typed callback handler to emit one canonical event per node run:

```python
from heeczer import HeeczerClient
from heeczer.adapters.langgraph import HeeczerLangGraphCallback

client = HeeczerClient(base_url="http://localhost:8080")
callback = HeeczerLangGraphCallback(client=client, workspace_id="ws_default")

graph.invoke({"messages": [...]}, config={"callbacks": [callback]})
```

### Google ADK

Wrap the async ADK entrypoint with the provided decorator:

```python
from heeczer import HeeczerClient
from heeczer.adapters.google_adk import heeczer_adk_wrapper

client = HeeczerClient(base_url="http://localhost:8080")


@heeczer_adk_wrapper(client=client, workspace_id="ws_default", task_name="support_agent")
async def run_agent(inputs: dict) -> dict:
    ...
```

### PydanticAI

Wrap a PydanticAI agent-like object to instrument `run()` and `run_sync()`
without taking a hard dependency on `pydantic_ai` inside ai-heeczer itself.

Async usage:

```python
from pydantic_ai import Agent

from heeczer import HeeczerClient
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent


async def main() -> None:
    client = HeeczerClient(base_url="http://localhost:8080")
    agent = Agent("openai:gpt-5.2", name="support_agent")
    instrumented = instrument_pydanticai_agent(
        agent=agent,
        client=client,
        workspace_id="ws_default",
    )
    result = await instrumented.run("Summarize the ticket")
    print(result.output)
```

Sync usage:

```python
from pydantic_ai import Agent

from heeczer import SyncHeeczerClient
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent

client = SyncHeeczerClient(base_url="http://localhost:8080")
agent = Agent("openai:gpt-5.2", name="support_agent")
instrumented = instrument_pydanticai_agent(
    agent=agent,
    client=client,
    workspace_id="ws_default",
)
result = instrumented.run_sync("Summarize the ticket")
print(result.output)
```

## Development

```bash
uv sync --extra dev
uv run pytest
uv run ruff check
uv run mypy
```

## License

MIT.

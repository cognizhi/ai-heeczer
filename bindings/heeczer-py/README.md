# cognizhi-heeczer (Python)

Async Python client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

> **Pre-release.** ``cognizhi-heeczer`` is not on PyPI yet (see plan 0012).
> Until then, install from source: ``uv sync --project bindings/heeczer-py``.

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
response. The error carries the closed ``kind`` enum from the ingestion
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

| Argument | Type | Default | Description |
| --- | --- | --- | --- |
| `base_url` | `str` | required | Base URL of the ingestion service. Trailing slash is stripped. |
| `api_key` | `str \| None` | `None` | Sent as `x-heeczer-api-key`. |
| `timeout` | `float` | `10.0` | Per-request timeout in seconds. |
| `transport` | `httpx.AsyncBaseTransport \| None` | `None` | Inject a custom transport (e.g. `httpx.MockTransport` in tests). |

## Methods

| Method | HTTP | Returns |
| --- | --- | --- |
| `healthz()` | `GET /healthz` | `bool` |
| `version()` | `GET /v1/version` | `VersionResponse` |
| `ingest_event(workspace_id, event)` | `POST /v1/events` | `IngestEventResponse` |
| `test_score_pipeline(event, profile=…, tier_set=…, tier_override=…)` | `POST /v1/test/score-pipeline` | `TestPipelineResponse` (gated by the test-orchestration feature flag) |

## Error kinds

``HeeczerApiError.kind`` is a closed ``Literal`` mirroring the ingestion
service envelope:

| Kind | When |
| --- | --- |
| ``schema`` | Event failed canonical schema validation. |
| ``bad_request`` | Malformed JSON or missing top-level fields. |
| ``scoring`` | Engine rejected a normalized event (e.g. unknown tier id). |
| ``storage`` | Persistence layer error. |
| ``not_found`` | Read endpoint did not find the resource. |
| ``forbidden`` | Auth or RBAC denied the request. |
| ``feature_disabled`` | Endpoint exists but the feature flag is off. |
| ``unknown`` | Non-JSON 5xx body; the raw text is in ``api_message``. |

## Runnable example

See [`examples/python/quickstart.py`](../../examples/python/quickstart.py)
and the cross-language index in [`examples/README.md`](../../examples/README.md).

## Common patterns

**Validate locally before sending** (avoids a network round-trip on bad
events). The schema is JSON Schema Draft 2020-12; use
[`jsonschema`](https://python-jsonschema.readthedocs.io/) or
[`fastjsonschema`](https://horejsek.github.io/python-fastjsonschema/):

```python
import json, jsonschema

with open("core/schema/event.v1.json") as f:
    schema = json.load(f)

jsonschema.validate(event, schema)  # raises jsonschema.ValidationError if invalid
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

**Batching note.** ``POST /v1/events:batch`` (single-transaction,
partial-success semantics) is planned but not yet shipped — see
[plan 0004](../../docs/plan/0004-ingestion-service.md). Until then,
use ``asyncio.gather`` for concurrent sends.

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface (``ScoreResult`` is a ``TypedDict``
with ``total=False``).

## Development

```bash
uv sync --extra dev
uv run pytest
uv run ruff check
uv run mypy
```

## License

MIT.

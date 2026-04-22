# heeczer (Python)

Async Python client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

```bash
uv add heeczer
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

## Development

```bash
uv sync --extra dev
uv run pytest
uv run ruff check
uv run mypy
```

## License

MIT.

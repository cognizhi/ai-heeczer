"""Quickstart: submit an event to the ingestion service via the Python SDK.

Prereq: ingestion service running locally (``cargo run -p heeczer-ingest``).

Run::

    uv sync --project bindings/heeczer-py
    uv run --project bindings/heeczer-py python examples/python/quickstart.py
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# Prefer the installed package (after `uv sync --project bindings/heeczer-py`).
# Fall back to the local source tree only if the package is not yet installed,
# so a contributor running this from a fresh clone gets a working demo.
try:
    from heeczer import HeeczerApiError, HeeczerClient
except ModuleNotFoundError:
    sys.stderr.write(
        "warning: `heeczer` not installed; falling back to local source tree. "
        "Run `uv sync --project bindings/heeczer-py` to silence this.\n"
    )
    sys.path.insert(0, str(ROOT / "bindings" / "heeczer-py" / "src"))
    from heeczer import HeeczerApiError, HeeczerClient  # noqa: E402


async def main() -> int:
    event = json.loads((ROOT / "examples" / "event.json").read_text())
    base_url = os.environ.get("HEECZER_BASE_URL", "http://127.0.0.1:8080")
    api_key = os.environ.get("HEECZER_API_KEY")

    async with HeeczerClient(base_url=base_url, api_key=api_key) as client:
        version = await client.version()
        print(f"» service version: {version}")

        try:
            resp = await client.ingest_event(workspace_id="ws_default", event=event)
        except HeeczerApiError as err:
            print(f"SDK error: kind={err.kind} status={err.status} message={err.api_message}")
            return 1

        score = resp["score"]
        print(f"» event {resp['event_id']} ingested")
        print(f"» summary: {score['human_summary']}")
        print(f"» minutes={score['final_estimated_minutes']} band={score['confidence_band']}")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))

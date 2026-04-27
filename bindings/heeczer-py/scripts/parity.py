from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path

from heeczer import HeeczerClient


def _required_env(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise RuntimeError(f"{name} is required")
    return value


async def main() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    fixture_dir = Path(
        os.environ.get(
            "HEECZER_PARITY_FIXTURE_DIR",
            repo_root / "core" / "schema" / "fixtures" / "events" / "valid",
        )
    )
    reference_dir = Path(_required_env("HEECZER_PARITY_REFERENCE_DIR"))
    base_url = _required_env("HEECZER_PARITY_BASE_URL")

    fixtures = sorted(fixture_dir.glob("*.json"))
    if not fixtures:
        raise RuntimeError(f"no valid fixtures found in {fixture_dir}")

    failures: list[str] = []
    async with HeeczerClient(
        base_url=base_url,
        retry={"attempts": 3, "backoff_ms": 50},
    ) as client:
        for fixture_path in fixtures:
            event = json.loads(fixture_path.read_text(encoding="utf-8"))
            expected = (reference_dir / f"{fixture_path.stem}.json").read_text(
                encoding="utf-8"
            ).rstrip()
            response = await client.test_score_pipeline(event=event)
            actual = json.dumps(response["score"], separators=(",", ":"))
            if actual != expected:
                failures.append(
                    f"{fixture_path.name}: score JSON differed from Rust reference"
                )

    if failures:
        raise AssertionError("\n".join(failures))
    print(f"Python SDK parity passed for {len(fixtures)} fixture(s)")


if __name__ == "__main__":
    asyncio.run(main())

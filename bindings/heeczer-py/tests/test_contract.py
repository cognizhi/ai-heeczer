"""Contract tests for plan 0001 / ADR-0002 — Python binding.

Verifies:
1. Every valid fixture round-trips through the typed Event TypedDict
   without data loss (json.loads → cast → json.dumps → compare).
2. Extension fields under meta.extensions survive a round-trip.
3. Unknown top-level fields are NOT stripped by cast() (TypedDict is
   structural; enforcement is server-side), but are documented here.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import cast

import pytest
from pydantic import ValidationError

from heeczer import Event, EventModel, validate_event

_FIXTURE_DIR = Path(__file__).parents[3] / "core" / "schema" / "fixtures" / "events" / "valid"


def _load_valid_fixtures() -> list[tuple[str, str]]:
    """Return sorted list of (filename, body) pairs for all valid fixtures."""
    paths = sorted(_FIXTURE_DIR.glob("*.json"))
    return [(p.name, p.read_text(encoding="utf-8")) for p in paths]


# ── Round-trip tests ──────────────────────────────────────────────────────────


@pytest.mark.parametrize(
    "name,body",
    _load_valid_fixtures(),
    ids=[n for n, _ in _load_valid_fixtures()],
)
def test_valid_fixture_round_trips_losslessly(name: str, body: str) -> None:
    """Fixture → cast(Event) → json.dumps → json.loads must equal original."""
    original = json.loads(body)

    # TypedDict cast is structural — no runtime coercion; object is unchanged.
    event = cast(Event, original)
    validate_event(original)

    roundtripped = json.loads(json.dumps(event))

    assert roundtripped == original, (
        f"Fixture `{name}` round-trip produced different value:\n"
        f"  original   : {original}\n"
        f"  roundtripped: {roundtripped}"
    )


def test_at_least_one_valid_fixture_exists() -> None:
    """Sanity-check that the fixture directory is accessible."""
    fixtures = _load_valid_fixtures()
    assert len(fixtures) > 0, f"No valid fixtures found in {_FIXTURE_DIR}"


# ── Extensions round-trip ─────────────────────────────────────────────────────


def test_meta_extensions_survive_round_trip() -> None:
    """Fields in meta.extensions are preserved after a JSON round-trip."""
    raw: Event = {
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-aabbccddeeff",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_ext",
        "task": {"name": "ext_test", "outcome": "success"},
        "metrics": {"duration_ms": 100},
        "meta": {
            "sdk_language": "python",
            "sdk_version": "0.1.0",
            "extensions": {"custom_key": 42, "nested": {"x": True}},
        },
    }

    roundtripped = cast(Event, json.loads(json.dumps(raw)))

    meta = roundtripped["meta"]
    assert "extensions" in meta, "meta.extensions must survive round-trip"
    assert meta["extensions"]["custom_key"] == 42
    assert meta["extensions"]["nested"]["x"] is True


def test_absent_optional_fields_remain_absent_after_round_trip() -> None:
    """Optional fields not set must not appear in the round-tripped dict."""
    raw: Event = {
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-000000000001",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_min",
        "task": {"name": "min_task", "outcome": "success"},
        "metrics": {"duration_ms": 50},
        "meta": {"sdk_language": "python", "sdk_version": "0.1.0"},
    }

    roundtripped = cast(Event, json.loads(json.dumps(raw)))

    # Optional top-level fields must be absent.
    assert "correlation_id" not in roundtripped
    assert "identity" not in roundtripped
    assert "context" not in roundtripped
    # Optional meta fields must be absent.
    assert "extensions" not in roundtripped["meta"]


# ── Unknown field behaviour documentation ─────────────────────────────────────


def test_unknown_top_level_field_not_stripped_by_cast_but_rejected_by_pydantic() -> None:
    """TypedDict cast() does NOT strip unknown keys at runtime.

    Unknown-field enforcement is the server-side JSON Schema validator's
    responsibility (ADR-0002 / PRD §13). This test documents that behavior
    explicitly so future maintainers understand the trust boundary.
    """
    raw_with_extra = {
        "spec_version": "1.0",
        "event_id": "00000000-0000-4000-8000-000000000002",
        "timestamp": "2026-04-22T10:00:00Z",
        "framework_source": "test",
        "workspace_id": "ws_strict",
        "task": {"name": "t", "outcome": "success"},
        "metrics": {"duration_ms": 1},
        "meta": {"sdk_language": "python", "sdk_version": "0.1.0"},
        "forbidden_extra_field": "value",  # NOT in Event
    }

    # cast() is a no-op at runtime — the extra key survives.
    event = cast(Event, raw_with_extra)
    assert event.get("forbidden_extra_field") == "value"
    # NOTE: mypy --strict would flag the .get() call above with a typeddict-item
    # error — that's intentional and shows the type-system is working.
    with pytest.raises(ValidationError):
        EventModel.model_validate(raw_with_extra)

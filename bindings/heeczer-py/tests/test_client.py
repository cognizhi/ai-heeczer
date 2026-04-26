"""Tests for the heeczer Python client (plan 0006)."""

from __future__ import annotations

import json
from typing import Any

import httpx
import pytest

from heeczer import (
    EventModel,
    HeeczerApiError,
    HeeczerClient,
    HeeczerUnsupportedModeError,
    SyncHeeczerClient,
)

VALID_EVENT: dict[str, Any] = {
    "spec_version": "1.0",
    "event_id": "00000000-0000-4000-8000-000000000101",
    "timestamp": "2026-04-22T10:00:00Z",
    "framework_source": "test",
    "workspace_id": "ws_test",
    "task": {"name": "fixture", "outcome": "success"},
    "metrics": {"duration_ms": 100},
    "meta": {"sdk_language": "python", "sdk_version": "0.5.1"},
}


def _stub(handler: Any) -> httpx.AsyncBaseTransport:
    return httpx.MockTransport(handler)


@pytest.mark.asyncio
async def test_constructor_requires_base_url() -> None:
    with pytest.raises(ValueError, match="base_url is required"):
        HeeczerClient(base_url="")


@pytest.mark.asyncio
async def test_native_mode_fails_fast_until_binding_ships() -> None:
    with pytest.raises(HeeczerUnsupportedModeError, match="native mode"):
        HeeczerClient(base_url="https://api.example.com", mode="native")


@pytest.mark.asyncio
async def test_healthz_is_true_on_2xx() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/healthz"
        return httpx.Response(200, json={"ok": True, "envelope_version": "1"})

    async with HeeczerClient(base_url="https://api.example.com", transport=_stub(handler)) as c:
        assert await c.healthz() is True


@pytest.mark.asyncio
async def test_version_returns_envelope() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(
            200,
            json={
                "ok": True,
                "envelope_version": "1",
                "scoring_version": "1.0.0",
                "spec_version": "1.0",
                "service": "0.1.0",
            },
        )

    async with HeeczerClient(base_url="https://api.example.com", transport=_stub(handler)) as c:
        v = await c.version()
        assert v["scoring_version"] == "1.0.0"
        assert v["spec_version"] == "1.0"


@pytest.mark.asyncio
async def test_ingest_event_posts_canonical_body() -> None:
    captured: dict[str, Any] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured["body"] = json.loads(request.content)
        captured["headers"] = dict(request.headers)
        return httpx.Response(
            200,
            json={
                "ok": True,
                "envelope_version": "1",
                "event_id": "evt-1",
                "score": {
                    "scoring_version": "1.0.0",
                    "spec_version": "1.0",
                    "scoring_profile": "default",
                    "category": "uncategorized",
                    "final_estimated_minutes": "1",
                    "estimated_hours": "0.02",
                    "estimated_days": "0.0025",
                    "financial_equivalent_cost": "1",
                    "confidence_score": "0.5",
                    "confidence_band": "Medium",
                    "human_summary": "ok",
                },
            },
        )

    async with HeeczerClient(
        base_url="https://api.example.com",
        api_key="k_secret",
        transport=_stub(handler),
    ) as c:
        r = await c.ingest_event(
            workspace_id="ws_test", event=EventModel.model_validate(VALID_EVENT)
        )
        assert r["event_id"] == "evt-1"
        assert r["score"]["confidence_band"] == "Medium"

    assert captured["body"] == {"workspace_id": "ws_test", "event": VALID_EVENT}
    assert captured["headers"]["x-heeczer-api-key"] == "k_secret"


@pytest.mark.asyncio
async def test_retries_transient_status_codes() -> None:
    calls = 0

    def handler(request: httpx.Request) -> httpx.Response:
        nonlocal calls
        calls += 1
        if calls == 1:
            return httpx.Response(
                503,
                json={
                    "ok": False,
                    "error": {"kind": "unavailable", "message": "warming"},
                },
            )
        return httpx.Response(200, json={"ok": True, "envelope_version": "1"})

    async with HeeczerClient(
        base_url="https://api.example.com",
        retry={"attempts": 2, "backoff_ms": 0},
        transport=_stub(handler),
    ) as c:
        assert await c.healthz() is True
    assert calls == 2


@pytest.mark.asyncio
async def test_error_envelope_maps_to_typed_exception() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(
            400,
            json={
                "ok": False,
                "envelope_version": "1",
                "error": {"kind": "schema", "message": "missing field event_id"},
            },
        )

    async with HeeczerClient(base_url="https://api.example.com", transport=_stub(handler)) as c:
        with pytest.raises(HeeczerApiError) as excinfo:
            await c.ingest_event(workspace_id="ws", event={})
        assert excinfo.value.status == 400
        assert excinfo.value.kind == "schema"
        assert "missing field event_id" in excinfo.value.api_message


@pytest.mark.asyncio
async def test_non_json_error_falls_back_to_unknown() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(504, content=b"upstream timeout")

    async with HeeczerClient(base_url="https://api.example.com", transport=_stub(handler)) as c:
        with pytest.raises(HeeczerApiError) as excinfo:
            await c.version()
        assert excinfo.value.status == 504
        assert excinfo.value.kind == "unknown"


@pytest.mark.asyncio
async def test_test_score_pipeline_always_sends_tester_header() -> None:
    captured: dict[str, Any] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured["headers"] = dict(request.headers)
        return httpx.Response(
            200,
            json={
                "ok": True,
                "envelope_version": "1",
                "score": {
                    "scoring_version": "1.0.0",
                    "spec_version": "1.0",
                    "scoring_profile": "default",
                    "category": "uncategorized",
                    "final_estimated_minutes": "1",
                    "estimated_hours": "0.02",
                    "estimated_days": "0.0025",
                    "financial_equivalent_cost": "1",
                    "confidence_score": "0.5",
                    "confidence_band": "Medium",
                    "human_summary": "ok",
                },
            },
        )

    async with HeeczerClient(base_url="https://api.example.com", transport=_stub(handler)) as c:
        await c.test_score_pipeline(event={"event_id": "evt"})

    assert captured["headers"]["x-heeczer-tester"] == "1"


@pytest.mark.asyncio
async def test_base_url_trailing_slash_is_normalised() -> None:
    seen: dict[str, Any] = {}

    def handler(request: httpx.Request) -> httpx.Response:
        seen["url"] = str(request.url)
        return httpx.Response(200, json={"ok": True, "envelope_version": "1"})

    async with HeeczerClient(
        base_url="https://api.example.com/",
        transport=_stub(handler),
    ) as c:
        await c.healthz()

    assert seen["url"] == "https://api.example.com/healthz"


# ---------------------------------------------------------------------------
# SyncHeeczerClient tests
# ---------------------------------------------------------------------------


def test_sync_healthz_returns_true() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json={"ok": True, "envelope_version": "1"})

    # SyncHeeczerClient uses HeeczerClient internally — inject transport via
    # the async client attribute.
    client = SyncHeeczerClient.__new__(SyncHeeczerClient)
    client._async = HeeczerClient(base_url="https://api.example.com", transport=_stub(handler))
    assert client.healthz() is True


def test_sync_client_context_manager_closes() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json={"ok": True, "envelope_version": "1"})

    with SyncHeeczerClient.__new__(SyncHeeczerClient) as client:
        client._async = HeeczerClient(base_url="https://api.example.com", transport=_stub(handler))
        ok = client.healthz()
    assert ok is True

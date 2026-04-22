"""Tests for the heeczer Python client (plan 0006)."""

from __future__ import annotations

import json
from typing import Any

import httpx
import pytest

from heeczer import HeeczerApiError, HeeczerClient


def _stub(handler: Any) -> httpx.AsyncBaseTransport:
    return httpx.MockTransport(handler)


@pytest.mark.asyncio
async def test_constructor_requires_base_url() -> None:
    with pytest.raises(ValueError, match="base_url is required"):
        HeeczerClient(base_url="")


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
        r = await c.ingest_event(workspace_id="ws_test", event={"event_id": "evt-1"})
        assert r["event_id"] == "evt-1"
        assert r["score"]["confidence_band"] == "Medium"

    assert captured["body"] == {"workspace_id": "ws_test", "event": {"event_id": "evt-1"}}
    assert captured["headers"]["x-heeczer-api-key"] == "k_secret"


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

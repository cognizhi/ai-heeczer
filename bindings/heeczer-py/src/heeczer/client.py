"""Thin async client for the ai-heeczer ingestion service.

Speaks the envelope_version=1 contract documented in ADR-0011 (mirrored by
the ingestion service in `services/heeczer-ingest/src/error.rs`). Surfaces
typed errors via :class:`HeeczerApiError` with a closed ``kind`` enum so
callers do not pattern-match on strings.
"""

from __future__ import annotations

from typing import Any, Literal, TypedDict

import httpx

ConfidenceBand = Literal["Low", "Medium", "High"]

ApiErrorKind = Literal[
    "schema",
    "bad_request",
    "scoring",
    "storage",
    "not_found",
    "forbidden",
    "feature_disabled",
    "unknown",
]


class ScoreResult(TypedDict, total=False):
    """Subset of the engine's ScoreResult exposed as a typed surface.

    Additive fields are tolerated via ``total=False``; consumers should rely
    on the keys declared here as the contract surface.
    """

    scoring_version: str
    spec_version: str
    scoring_profile: str
    category: str
    final_estimated_minutes: str
    estimated_hours: str
    estimated_days: str
    financial_equivalent_cost: str
    confidence_score: str
    confidence_band: ConfidenceBand
    human_summary: str


class IngestEventResponse(TypedDict):
    ok: Literal[True]
    envelope_version: Literal["1"]
    event_id: str
    score: ScoreResult


class VersionResponse(TypedDict):
    ok: Literal[True]
    envelope_version: Literal["1"]
    scoring_version: str
    spec_version: str
    service: str


class TestPipelineResponse(TypedDict):
    ok: Literal[True]
    envelope_version: Literal["1"]
    score: ScoreResult


class HeeczerApiError(Exception):
    """Raised by every client method on a non-2xx response."""

    def __init__(self, status: int, kind: ApiErrorKind, message: str) -> None:
        super().__init__(f"heeczer {status} {kind}: {message}")
        self.status = status
        self.kind = kind
        self.api_message = message


class HeeczerClient:
    """Async client for the ai-heeczer ingestion service.

    Pass a custom ``transport`` (``httpx.MockTransport`` etc.) to inject a
    fake server in tests; the client otherwise uses the default
    :class:`httpx.AsyncClient` transport.
    """

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 10.0,
        transport: httpx.AsyncBaseTransport | None = None,
    ) -> None:
        if not base_url:
            raise ValueError("base_url is required")
        headers: dict[str, str] = {}
        if api_key:
            headers["x-heeczer-api-key"] = api_key
        self._client = httpx.AsyncClient(
            base_url=base_url.rstrip("/"),
            headers=headers,
            timeout=timeout,
            transport=transport,
        )

    async def __aenter__(self) -> HeeczerClient:
        return self

    async def __aexit__(self, *exc_info: object) -> None:
        await self.aclose()

    async def aclose(self) -> None:
        await self._client.aclose()

    async def healthz(self) -> bool:
        resp = await self._client.get("/healthz")
        return resp.is_success

    async def version(self) -> VersionResponse:
        return await self._get_json("/v1/version")  # type: ignore[no-any-return]

    async def ingest_event(
        self, *, workspace_id: str, event: dict[str, Any]
    ) -> IngestEventResponse:
        return await self._post_json(  # type: ignore[no-any-return]
            "/v1/events",
            {"workspace_id": workspace_id, "event": event},
        )

    async def test_score_pipeline(
        self,
        *,
        event: dict[str, Any],
        profile: dict[str, Any] | None = None,
        tier_set: dict[str, Any] | None = None,
        tier_override: str | None = None,
    ) -> TestPipelineResponse:
        body: dict[str, Any] = {"event": event}
        if profile is not None:
            body["profile"] = profile
        if tier_set is not None:
            body["tier_set"] = tier_set
        if tier_override is not None:
            body["tier_override"] = tier_override
        return await self._post_json(  # type: ignore[no-any-return]
            "/v1/test/score-pipeline",
            body,
            extra_headers={"x-heeczer-tester": "1"},
        )

    async def _get_json(self, path: str) -> Any:
        resp = await self._client.get(path)
        return self._handle(resp)

    async def _post_json(
        self,
        path: str,
        body: dict[str, Any],
        *,
        extra_headers: dict[str, str] | None = None,
    ) -> Any:
        resp = await self._client.post(path, json=body, headers=extra_headers or {})
        return self._handle(resp)

    @staticmethod
    def _handle(resp: httpx.Response) -> Any:
        if resp.is_success:
            return resp.json()
        kind: ApiErrorKind = "unknown"
        message = resp.text or resp.reason_phrase
        try:
            payload = resp.json()
            if (
                isinstance(payload, dict)
                and payload.get("ok") is False
                and isinstance(payload.get("error"), dict)
            ):
                err = payload["error"]
                if isinstance(err.get("kind"), str):
                    kind = err["kind"]
                if isinstance(err.get("message"), str):
                    message = err["message"]
        except ValueError:
            pass
        raise HeeczerApiError(resp.status_code, kind, message)

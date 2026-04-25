"""Thin async client for the ai-heeczer ingestion service.

Speaks the envelope_version=1 contract documented in ADR-0011 (mirrored by
the ingestion service in `services/heeczer-ingest/src/error.rs`). Surfaces
typed errors via :class:`HeeczerApiError` with a closed ``kind`` enum so
callers do not pattern-match on strings.
"""

from __future__ import annotations

import asyncio
from typing import Any, Literal, NotRequired, TypedDict, cast, get_args

import httpx

# ── Canonical event types (mirrored from core/schema/event.v1.json) ──────────
# Mirrors heeczer_core::event (Rust) and generated per plan 0001 / ADR-0002.

Outcome = Literal["success", "partial_success", "failure", "timeout"]
"""Outcome of a task (closed enum)."""

RiskClass = Literal["low", "medium", "high"]
"""Risk classification (closed enum)."""


class EventIdentity(TypedDict, total=False):
    """Optional identity block; all fields optional."""

    user_id: str | None
    team_id: str | None
    business_unit_id: str | None
    tier_id: str | None


class EventTask(TypedDict):
    """Task descriptor. ``name`` and ``outcome`` are required."""

    name: str
    outcome: Outcome
    category: NotRequired[str | None]
    sub_category: NotRequired[str | None]


class EventMetrics(TypedDict):
    """Telemetry metrics. ``duration_ms`` is required."""

    duration_ms: int
    tokens_prompt: NotRequired[int | None]
    tokens_completion: NotRequired[int | None]
    tool_call_count: NotRequired[int | None]
    workflow_steps: NotRequired[int | None]
    retries: NotRequired[int | None]
    artifact_count: NotRequired[int | None]
    output_size_proxy: NotRequired[float | None]


class EventContext(TypedDict, total=False):
    """Optional execution context; all fields optional."""

    human_in_loop: bool | None
    review_required: bool | None
    temperature: float | None
    risk_class: RiskClass | None
    tags: list[str] | None


class EventMeta(TypedDict):
    """SDK metadata. ``sdk_language`` and ``sdk_version`` are required.
    ``extensions`` is the sole permitted bucket for unknown fields (PRD §13)."""

    sdk_language: str
    sdk_version: str
    scoring_profile: NotRequired[str | None]
    extensions: NotRequired[Any]


class Event(TypedDict):
    """Canonical ai-heeczer telemetry event (v1).

    Mirrors ``heeczer_core::Event`` (Rust) and the JSON Schema in
    ``core/schema/event.v1.json``. Construct this type and pass it to
    :meth:`HeeczerClient.ingest_event` as the ``event`` argument.

    Example::

        event: Event = {
            "spec_version": "1.0",
            "event_id": str(uuid.uuid4()),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "framework_source": "langgraph",
            "workspace_id": "ws_default",
            "task": {"name": "summarise_pr", "category": "summarization", "outcome": "success"},
            "metrics": {"duration_ms": 3200},
            "meta": {"sdk_language": "python", "sdk_version": "0.1.0"},
        }
    """

    spec_version: Literal["1.0"]
    event_id: str
    timestamp: str
    framework_source: str
    workspace_id: str
    task: EventTask
    metrics: EventMetrics
    meta: EventMeta
    correlation_id: NotRequired[str | None]
    project_id: NotRequired[str | None]
    identity: NotRequired[EventIdentity | None]
    context: NotRequired[EventContext | None]


# ── Client types ──────────────────────────────────────────────────────────────

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
        _valid_kinds: frozenset[str] = frozenset(get_args(ApiErrorKind))
        try:
            payload = resp.json()
            if (
                isinstance(payload, dict)
                and payload.get("ok") is False
                and isinstance(payload.get("error"), dict)
            ):
                err = payload["error"]
                if isinstance(err.get("kind"), str):
                    raw_kind = err["kind"]
                    kind = cast(
                        ApiErrorKind,
                        raw_kind if raw_kind in _valid_kinds else "unknown",
                    )
                if isinstance(err.get("message"), str):
                    message = err["message"]
        except ValueError:
            pass
        raise HeeczerApiError(resp.status_code, kind, message)


def _run(coro: Any) -> Any:
    """Run a coroutine in a new event loop (for sync wrapper methods)."""
    return asyncio.get_event_loop().run_until_complete(coro)


class SyncHeeczerClient:
    """Synchronous wrapper around :class:`HeeczerClient`.

    Provides the same methods as :class:`HeeczerClient` but blocks until
    each response arrives. Useful in scripts and notebooks where an
    asyncio event loop is not running.

    .. code-block:: python

        from heeczer import SyncHeeczerClient

        client = SyncHeeczerClient(base_url="https://ingest.example.com", api_key="…")
        result = client.ingest_event(workspace_id="ws_default", event=my_event)
        print(result["score"]["final_estimated_minutes"])
        client.close()

    Or as a context manager::

        with SyncHeeczerClient(base_url="…") as client:
            print(client.version())
    """

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 10.0,
    ) -> None:
        self._async = HeeczerClient(base_url=base_url, api_key=api_key, timeout=timeout)

    def __enter__(self) -> SyncHeeczerClient:
        return self

    def __exit__(self, *exc_info: object) -> None:
        self.close()

    def close(self) -> None:
        _run(self._async.aclose())

    def healthz(self) -> bool:
        return _run(self._async.healthz())  # type: ignore[no-any-return]

    def version(self) -> VersionResponse:
        return _run(self._async.version())  # type: ignore[no-any-return]

    def ingest_event(
        self, *, workspace_id: str, event: dict[str, Any]
    ) -> IngestEventResponse:
        return _run(self._async.ingest_event(workspace_id=workspace_id, event=event))  # type: ignore[no-any-return]

    def test_score_pipeline(
        self,
        *,
        event: dict[str, Any],
        profile: dict[str, Any] | None = None,
        tier_set: dict[str, Any] | None = None,
        tier_override: str | None = None,
    ) -> TestPipelineResponse:
        return _run(  # type: ignore[no-any-return]
            self._async.test_score_pipeline(
                event=event,
                profile=profile,
                tier_set=tier_set,
                tier_override=tier_override,
            )
        )

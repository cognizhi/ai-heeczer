"""PydanticAI instrumentation wrapper for ai-heeczer.

Wraps a PydanticAI agent-like object and emits a canonical ai-heeczer event
for each ``run()`` or ``run_sync()`` invocation without taking a hard
dependency on ``pydantic_ai``.

Usage::

    from pydantic_ai import Agent
    from heeczer import HeeczerClient
    from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent

    client = HeeczerClient(base_url="http://localhost:8080")
    agent = Agent("openai:gpt-5.2", name="support_agent")
    instrumented = instrument_pydanticai_agent(
        agent=agent,
        client=client,
        workspace_id="ws_myteam",
    )

    result = await instrumented.run("summarize the ticket")
"""

from __future__ import annotations

import asyncio
import inspect
import time
import uuid
from collections.abc import Coroutine
from contextlib import suppress
from datetime import UTC, datetime
from typing import Any, Protocol, cast

from heeczer.client import HeeczerClient, SyncHeeczerClient

from .. import __version__


class _IngestClient(Protocol):
    def ingest_event(self, *, workspace_id: str, event: dict[str, Any]) -> Any:
        """Submit a canonical ai-heeczer event."""


class HeeczerPydanticAIAgent:
    """Proxy around a PydanticAI agent that emits canonical ai-heeczer events."""

    def __init__(  # noqa: PLR0913
        self,
        agent: Any,
        client: HeeczerClient | SyncHeeczerClient | _IngestClient,
        workspace_id: str,
        task_name: str | None = None,
        task_category: str | None = None,
        framework_source: str = "pydantic_ai",
        sdk_language: str = "python",
        sdk_version: str = __version__,
    ) -> None:
        if not callable(getattr(agent, "run", None)) and not callable(
            getattr(agent, "run_sync", None)
        ):
            raise TypeError("agent must define run() or run_sync()")
        self._agent = agent
        self._client = client
        self._workspace_id = workspace_id
        self._task_name = task_name or _resolve_task_name(agent)
        self._task_category = task_category
        self._framework_source = framework_source
        self._sdk_language = sdk_language
        self._sdk_version = sdk_version

    def __getattr__(self, name: str) -> Any:
        return getattr(self._agent, name)

    async def run(self, *args: Any, **kwargs: Any) -> Any:
        run_method = getattr(self._agent, "run", None)
        if not callable(run_method):
            raise AttributeError("wrapped agent does not define run()")

        started_at = datetime.now(UTC)
        start = time.monotonic()
        outcome = "success"
        error_summary: str | None = None
        result: Any = None

        try:
            result = await run_method(*args, **kwargs)
            return result
        except Exception as exc:  # noqa: BLE001
            outcome = "failure"
            error_summary = str(exc)[:256]
            raise
        finally:
            event = self._build_event(
                started_at=started_at,
                duration_ms=int((time.monotonic() - start) * 1000),
                outcome=outcome,
                result=result,
                error_summary=error_summary,
            )
            await self._submit_async(event)

    def run_sync(self, *args: Any, **kwargs: Any) -> Any:
        run_method = getattr(self._agent, "run_sync", None)
        if not callable(run_method):
            raise AttributeError("wrapped agent does not define run_sync()")

        started_at = datetime.now(UTC)
        start = time.monotonic()
        outcome = "success"
        error_summary: str | None = None
        result: Any = None

        try:
            result = run_method(*args, **kwargs)
            return result
        except Exception as exc:  # noqa: BLE001
            outcome = "failure"
            error_summary = str(exc)[:256]
            raise
        finally:
            event = self._build_event(
                started_at=started_at,
                duration_ms=int((time.monotonic() - start) * 1000),
                outcome=outcome,
                result=result,
                error_summary=error_summary,
            )
            self._submit_sync(event)

    def _build_event(
        self,
        *,
        started_at: datetime,
        duration_ms: int,
        outcome: str,
        result: Any,
        error_summary: str | None,
    ) -> dict[str, Any]:
        tokens_prompt, tokens_completion, tool_calls, retries = _extract_usage(result)
        task: dict[str, Any] = {
            "name": self._task_name,
            "outcome": outcome,
        }
        if self._task_category is not None:
            task["category"] = self._task_category
        event: dict[str, Any] = {
            "spec_version": "1.0",
            "event_id": str(uuid.uuid4()),
            "timestamp": started_at.isoformat(),
            "framework_source": self._framework_source,
            "workspace_id": self._workspace_id,
            "task": task,
            "metrics": {
                "duration_ms": duration_ms,
            },
            "meta": {
                "sdk_language": self._sdk_language,
                "sdk_version": self._sdk_version,
            },
        }
        if tokens_prompt or tokens_completion:
            event["metrics"]["tokens_prompt"] = tokens_prompt
            event["metrics"]["tokens_completion"] = tokens_completion
        if tool_calls:
            event["metrics"]["tool_call_count"] = tool_calls
        if retries:
            event["metrics"]["retries"] = retries
        if error_summary:
            event["meta"]["extensions"] = {"error_summary": error_summary}
        return event

    async def _submit_async(self, event: dict[str, Any]) -> None:
        try:
            submission = self._client.ingest_event(workspace_id=self._workspace_id, event=event)
            if inspect.isawaitable(submission):
                await cast(Coroutine[Any, Any, Any], submission)
        except Exception:  # noqa: BLE001
            pass

    def _submit_sync(self, event: dict[str, Any]) -> None:
        try:
            submission = self._client.ingest_event(workspace_id=self._workspace_id, event=event)
            if not inspect.isawaitable(submission):
                return
            try:
                loop = asyncio.get_running_loop()
            except RuntimeError:
                asyncio.run(cast(Coroutine[Any, Any, Any], submission))
            else:
                task = loop.create_task(cast(Coroutine[Any, Any, Any], submission))
                task.add_done_callback(_discard_task_exception)
        except Exception:  # noqa: BLE001
            pass


def instrument_pydanticai_agent(  # noqa: PLR0913
    agent: Any,
    client: HeeczerClient | SyncHeeczerClient | _IngestClient,
    workspace_id: str,
    task_name: str | None = None,
    task_category: str | None = None,
    framework_source: str = "pydantic_ai",
    sdk_language: str = "python",
    sdk_version: str = __version__,
) -> HeeczerPydanticAIAgent:
    """Wrap a PydanticAI agent-like object and emit one canonical event per run."""

    return HeeczerPydanticAIAgent(
        agent=agent,
        client=client,
        workspace_id=workspace_id,
        task_name=task_name,
        task_category=task_category,
        framework_source=framework_source,
        sdk_language=sdk_language,
        sdk_version=sdk_version,
    )


def _resolve_task_name(agent: Any) -> str:
    name = getattr(agent, "name", None)
    if isinstance(name, str) and name:
        return name
    class_name = getattr(agent.__class__, "__name__", None)
    if isinstance(class_name, str) and class_name:
        return class_name
    return "pydanticai_agent"


def _extract_usage(result: Any) -> tuple[int, int, int, int]:
    try:
        usage = _resolve_usage(result)
        return (
            _coerce_count(_field_value(usage, "input_tokens", "prompt_tokens")),
            _coerce_count(_field_value(usage, "output_tokens", "completion_tokens")),
            _coerce_count(_field_value(usage, "tool_calls"), _field_value(result, "tool_calls")),
            _coerce_count(_field_value(usage, "retries"), _field_value(result, "retries")),
        )
    except Exception:  # noqa: BLE001
        return (0, 0, 0, 0)


def _resolve_usage(result: Any) -> Any:
    usage_attr = getattr(result, "usage", None)
    if callable(usage_attr):
        return usage_attr()
    return usage_attr


def _field_value(source: Any, *names: str) -> Any:
    for name in names:
        if isinstance(source, dict) and name in source:
            return source[name]
        try:
            value = getattr(source, name, None)
        except Exception:  # noqa: BLE001
            continue
        if value is not None:
            return value
    return None


def _coerce_count(*values: Any) -> int:
    for value in values:
        if isinstance(value, bool):
            return int(value)
        if isinstance(value, int):
            return value
        if isinstance(value, (list, tuple, set, frozenset, dict)):
            return len(value)
    return 0


def _discard_task_exception(task: asyncio.Task[Any]) -> None:
    with suppress(Exception):
        task.exception()

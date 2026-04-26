"""Google Agent Development Kit (ADK) instrumentation hook for ai-heeczer.

Wraps an ADK ``Agent`` (or any callable that matches the ADK invocation
signature) and emits a canonical ai-heeczer event per invocation.

Usage::

    from heeczer.adapters.google_adk import heeczer_adk_wrapper
    from heeczer import HeeczerClient

    client = HeeczerClient(base_url="http://localhost:8080")

    @heeczer_adk_wrapper(client=client, workspace_id="ws_myteam", task_name="my_agent")
    async def my_agent(inputs):
        ...
"""

from __future__ import annotations

import functools
import time
import uuid
from collections.abc import Callable
from typing import Any, TypeVar

from heeczer.client import HeeczerClient

F = TypeVar("F", bound=Callable[..., Any])


def heeczer_adk_wrapper(  # noqa: PLR0913
    client: HeeczerClient,
    workspace_id: str,
    task_name: str,
    task_category: str = "agentic_task",
    framework_source: str = "google_adk",
    sdk_language: str = "python",
    sdk_version: str = "0.2.0",
) -> Callable[[F], F]:
    """Decorator that wraps an ADK agent coroutine and emits a canonical event.

    The wrapped function's return value is passed through unchanged.
    Instrumentation errors are silently swallowed so they never crash the agent.
    """

    def decorator(fn: F) -> F:
        @functools.wraps(fn)
        async def wrapper(*args: Any, **kwargs: Any) -> Any:
            start = time.monotonic()
            outcome = "success"
            error_summary: str | None = None
            tool_calls = 0
            retries = 0
            tokens_prompt = 0
            tokens_completion = 0

            try:
                result = await fn(*args, **kwargs)
                # Attempt to extract telemetry from ADK result objects
                if hasattr(result, "usage"):
                    usage = result.usage
                    tokens_prompt = getattr(usage, "prompt_tokens", 0) or 0
                    tokens_completion = getattr(usage, "completion_tokens", 0) or 0
                if hasattr(result, "tool_calls"):
                    tool_calls = len(result.tool_calls or [])
                return result
            except Exception as exc:  # noqa: BLE001
                outcome = "failure"
                error_summary = str(exc)[:256]
                raise
            finally:
                duration_ms = int((time.monotonic() - start) * 1000)
                event_id = str(uuid.uuid4())
                event: dict[str, Any] = {
                    "spec_version": "1.0",
                    "event_id": event_id,
                    "framework_source": framework_source,
                    "workspace_id": workspace_id,
                    "task": {
                        "name": task_name,
                        "category": task_category,
                        "outcome": outcome,
                    },
                    "metrics": {
                        "duration_ms": duration_ms,
                    },
                    "meta": {
                        "sdk_language": sdk_language,
                        "sdk_version": sdk_version,
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

                import asyncio  # noqa: PLC0415

                try:
                    loop = asyncio.get_event_loop()
                    if loop.is_running():
                        loop.create_task(
                            client.ingest_event(workspace_id=workspace_id, event=event)
                        )
                    else:
                        await client.ingest_event(workspace_id=workspace_id, event=event)
                except Exception:  # noqa: BLE001
                    pass

        return wrapper  # type: ignore[return-value]

    return decorator

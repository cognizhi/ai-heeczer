"""LangGraph callback adapter for ai-heeczer.

Translates LangGraph node-execution events into canonical ai-heeczer events
and submits them to the ingestion service.

Usage::

    from heeczer.adapters.langgraph import HeeczerLangGraphCallback
    from heeczer import HeeczerClient

    client = HeeczerClient(base_url="http://localhost:8080")
    callback = HeeczerLangGraphCallback(
        client=client,
        workspace_id="ws_myteam",
        framework_source="langgraph",
    )
    # Pass callback to your LangGraph graph
    graph.invoke(input, config={"callbacks": [callback]})
"""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from typing import Any

from heeczer.client import HeeczerClient


@dataclass
class NodeRunContext:
    """Tracks in-flight timing and metadata for a single node run."""

    node_name: str
    run_id: str
    start_time: float = field(default_factory=time.monotonic)
    tokens_prompt: int = 0
    tokens_completion: int = 0
    tool_call_count: int = 0
    retries: int = 0
    outcome: str = "success"
    error: str | None = None


class HeeczerLangGraphCallback:
    """LangGraph compatible callback handler that emits canonical ai-heeczer events.

    This class implements the duck-typed callback interface expected by LangGraph.
    It does not import from ``langgraph`` directly so that ai-heeczer has no
    hard dependency on the LangGraph package.
    """

    def __init__(
        self,
        client: HeeczerClient,
        workspace_id: str,
        framework_source: str = "langgraph",
        sdk_language: str = "python",
        sdk_version: str = "0.2.0",
    ) -> None:
        self._client = client
        self._workspace_id = workspace_id
        self._framework_source = framework_source
        self._sdk_language = sdk_language
        self._sdk_version = sdk_version
        self._runs: dict[str, NodeRunContext] = {}

    # ── LangGraph callback hooks (duck-typed) ─────────────────────────────────

    def on_chain_start(
        self, serialized: dict[str, Any], inputs: dict[str, Any], run_id: str, **kwargs: Any
    ) -> None:
        """Called when a node/chain starts executing."""
        node_name = serialized.get("name") or serialized.get("id", ["unknown"])[-1]
        self._runs[run_id] = NodeRunContext(node_name=str(node_name), run_id=run_id)

    def on_chain_end(self, outputs: dict[str, Any], run_id: str, **kwargs: Any) -> None:
        """Called when a node/chain ends successfully."""
        ctx = self._runs.pop(run_id, None)
        if ctx is None:
            return
        ctx.outcome = "success"
        self._submit(ctx)

    def on_chain_error(self, error: BaseException, run_id: str, **kwargs: Any) -> None:
        """Called when a node/chain raises an error."""
        ctx = self._runs.pop(run_id, None)
        if ctx is None:
            return
        ctx.outcome = "failure"
        ctx.error = str(error)
        self._submit(ctx)

    def on_llm_start(
        self, serialized: dict[str, Any], prompts: list[str], run_id: str, **kwargs: Any
    ) -> None:
        """Track prompt token count (approximate by character count if tokens unavailable)."""
        parent_run_id = kwargs.get("parent_run_id")
        ctx = self._runs.get(str(parent_run_id or ""))
        if ctx and prompts:
            # Approximate: 4 chars ≈ 1 token. Real implementations use tiktoken.
            ctx.tokens_prompt += sum(len(p) for p in prompts) // 4

    def on_llm_end(self, response: Any, run_id: str, **kwargs: Any) -> None:
        """Track completion token count from LLM response."""
        parent_run_id = kwargs.get("parent_run_id")
        ctx = self._runs.get(str(parent_run_id or ""))
        if ctx is None:
            return
        # LangChain/LangGraph LLMResult structure
        try:
            for generation in getattr(response, "generations", []):
                for gen in generation:
                    usage = getattr(gen, "generation_info", {}) or {}
                    ctx.tokens_completion += usage.get("completion_tokens", 0)
        except Exception:  # noqa: BLE001
            pass

    def on_tool_start(
        self, serialized: dict[str, Any], input_str: str, run_id: str, **kwargs: Any
    ) -> None:
        """Count tool invocations."""
        parent_run_id = kwargs.get("parent_run_id")
        ctx = self._runs.get(str(parent_run_id or ""))
        if ctx:
            ctx.tool_call_count += 1

    def on_retry(self, retry_state: Any, **kwargs: Any) -> None:
        """Count retries."""
        run_id = kwargs.get("run_id", "")
        ctx = self._runs.get(str(run_id))
        if ctx:
            ctx.retries += 1

    # ── Internal ──────────────────────────────────────────────────────────────

    def _submit(self, ctx: NodeRunContext) -> None:
        """Build and asynchronously submit a canonical event. Fire-and-forget."""
        import asyncio

        duration_ms = int((time.monotonic() - ctx.start_time) * 1000)
        event_id = str(uuid.uuid4())

        event: dict[str, Any] = {
            "spec_version": "1.0",
            "event_id": event_id,
            "framework_source": self._framework_source,
            "workspace_id": self._workspace_id,
            "task": {
                "name": ctx.node_name,
                "outcome": ctx.outcome,
            },
            "metrics": {
                "duration_ms": duration_ms,
            },
            "meta": {
                "sdk_language": self._sdk_language,
                "sdk_version": self._sdk_version,
            },
        }
        if ctx.tokens_prompt or ctx.tokens_completion:
            event["metrics"]["tokens_prompt"] = ctx.tokens_prompt
            event["metrics"]["tokens_completion"] = ctx.tokens_completion
        if ctx.tool_call_count:
            event["metrics"]["tool_call_count"] = ctx.tool_call_count
        if ctx.retries:
            event["metrics"]["retries"] = ctx.retries
        if ctx.error:
            event["meta"]["extensions"] = {"error_summary": ctx.error[:256]}

        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                loop.create_task(
                    self._client.ingest_event(
                        workspace_id=self._workspace_id, event=event
                    )
                )
            else:
                loop.run_until_complete(
                    self._client.ingest_event(
                        workspace_id=self._workspace_id, event=event
                    )
                )
        except Exception:  # noqa: BLE001
            # Never let instrumentation crash the agent.
            pass

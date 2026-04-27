"""Tests for framework adapters."""

from __future__ import annotations

from typing import Any, cast
from unittest.mock import AsyncMock, MagicMock

import pytest

from heeczer.adapters.google_adk import heeczer_adk_wrapper
from heeczer.adapters.langgraph import HeeczerLangGraphCallback
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent
from heeczer.client import HeeczerClient
from heeczer.models import validate_event


@pytest.fixture
def mock_client(monkeypatch: pytest.MonkeyPatch) -> MagicMock:
    client = MagicMock(spec=HeeczerClient)
    client.ingest_event = AsyncMock(return_value={"ok": True, "event_id": "test-id"})
    return client


class TestLangGraphCallback:
    def test_chain_start_creates_run_context(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        cb.on_chain_start({"name": "my_node"}, {}, run_id="run_001")
        assert "run_001" in cb._runs
        assert cb._runs["run_001"].node_name == "my_node"

    def test_chain_end_removes_context(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        cb.on_chain_start({"name": "my_node"}, {}, run_id="run_002")
        cb.on_chain_end({}, run_id="run_002")
        assert "run_002" not in cb._runs
        event = _captured_event(mock_client)
        validate_event(event)
        assert event["task"]["outcome"] == "success"

    def test_chain_error_sets_failure_outcome(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        cb.on_chain_start({"name": "failing_node"}, {}, run_id="run_003")
        cb.on_chain_error(ValueError("oops"), run_id="run_003")
        # Context removed after error
        assert "run_003" not in cb._runs
        event = _captured_event(mock_client)
        validate_event(event)
        assert event["task"]["outcome"] == "failure"

    def test_tool_start_increments_count(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        cb.on_chain_start({"name": "tool_node"}, {}, run_id="run_004")
        cb.on_tool_start({"name": "search"}, "", run_id="tool_001", parent_run_id="run_004")
        assert cb._runs["run_004"].tool_call_count == 1

    def test_unknown_run_id_is_noop(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        # Should not raise
        cb.on_chain_end({}, run_id="nonexistent")
        cb.on_chain_error(ValueError("x"), run_id="nonexistent")


class TestGoogleADKWrapper:
    @pytest.mark.asyncio
    async def test_success_emits_event(self, mock_client: MagicMock) -> None:
        @heeczer_adk_wrapper(mock_client, "ws_test", "test_task")
        async def my_agent(x: int) -> int:
            return x * 2

        result = await my_agent(5)
        assert result == 10
        mock_client.ingest_event.assert_called_once()
        call_kwargs = mock_client.ingest_event.call_args[1]
        assert call_kwargs["workspace_id"] == "ws_test"
        assert call_kwargs["event"]["task"]["outcome"] == "success"
        validate_event(call_kwargs["event"])

    @pytest.mark.asyncio
    async def test_failure_emits_failure_event(self, mock_client: MagicMock) -> None:
        @heeczer_adk_wrapper(mock_client, "ws_test", "failing_task")
        async def bad_agent() -> None:
            raise RuntimeError("agent crashed")

        with pytest.raises(RuntimeError):
            await bad_agent()
        mock_client.ingest_event.assert_called_once()
        call_kwargs = mock_client.ingest_event.call_args[1]
        assert call_kwargs["event"]["task"]["outcome"] == "failure"
        validate_event(call_kwargs["event"])


class _DummyUsage:
    def __init__(
        self,
        input_tokens: int = 0,
        output_tokens: int = 0,
        tool_calls: int = 0,
        retries: int = 0,
    ) -> None:
        self.input_tokens = input_tokens
        self.output_tokens = output_tokens
        self.tool_calls = tool_calls
        self.retries = retries


class _DummyRunResult:
    def __init__(self, usage: _DummyUsage) -> None:
        self.output = "done"
        self.data = {"ok": True}
        self._usage = usage

    def usage(self) -> _DummyUsage:
        return self._usage


class _DummyPydanticAgent:
    def __init__(
        self,
        *,
        async_result: object | None = None,
        async_error: BaseException | None = None,
        sync_result: object | None = None,
        sync_error: BaseException | None = None,
    ) -> None:
        self.name = "support_agent"
        self._async_result = async_result or _DummyRunResult(
            _DummyUsage(input_tokens=21, output_tokens=8, tool_calls=2)
        )
        self._async_error = async_error
        self._sync_result = sync_result or _DummyRunResult(
            _DummyUsage(input_tokens=13, output_tokens=5, tool_calls=1)
        )
        self._sync_error = sync_error

    async def run(self, *_args: object, **_kwargs: object) -> object:
        if self._async_error is not None:
            raise self._async_error
        return self._async_result

    def run_sync(self, *_args: object, **_kwargs: object) -> object:
        if self._sync_error is not None:
            raise self._sync_error
        return self._sync_result


class _BrokenUsageRunResult:
    output = "done"
    data = {"ok": True}

    def usage(self) -> _DummyUsage:
        raise RuntimeError("usage unavailable")


class TestPydanticAIAdapter:
    @pytest.mark.asyncio
    async def test_async_run_emits_event(self, mock_client: MagicMock) -> None:
        agent = instrument_pydanticai_agent(_DummyPydanticAgent(), mock_client, "ws_test")

        result = await agent.run("hello")

        assert isinstance(result, _DummyRunResult)
        mock_client.ingest_event.assert_called_once()
        call_kwargs = mock_client.ingest_event.call_args[1]
        assert call_kwargs["workspace_id"] == "ws_test"
        assert call_kwargs["event"]["framework_source"] == "pydantic_ai"
        assert call_kwargs["event"]["task"]["name"] == "support_agent"
        assert call_kwargs["event"]["task"]["outcome"] == "success"
        assert "category" not in call_kwargs["event"]["task"]
        assert call_kwargs["event"]["metrics"]["tokens_prompt"] == 21
        assert call_kwargs["event"]["metrics"]["tokens_completion"] == 8
        assert call_kwargs["event"]["metrics"]["tool_call_count"] == 2
        validate_event(call_kwargs["event"])

    def test_sync_run_failure_emits_failure_event(self, mock_client: MagicMock) -> None:
        agent = instrument_pydanticai_agent(
            _DummyPydanticAgent(sync_error=RuntimeError("sync run failed")),
            mock_client,
            "ws_test",
        )

        with pytest.raises(RuntimeError, match="sync run failed"):
            agent.run_sync("hello")

        mock_client.ingest_event.assert_called_once()
        call_kwargs = mock_client.ingest_event.call_args[1]
        assert call_kwargs["event"]["task"]["outcome"] == "failure"
        assert call_kwargs["event"]["meta"]["extensions"]["error_summary"] == "sync run failed"
        validate_event(call_kwargs["event"])

    def test_sync_run_success_emits_valid_event(self, mock_client: MagicMock) -> None:
        agent = instrument_pydanticai_agent(_DummyPydanticAgent(), mock_client, "ws_test")

        result = agent.run_sync("hello")

        assert isinstance(result, _DummyRunResult)
        event = _captured_event(mock_client)
        validate_event(event)
        assert event["task"]["outcome"] == "success"
        assert event["metrics"]["tokens_prompt"] == 13

    @pytest.mark.asyncio
    async def test_async_run_failure_emits_valid_failure_event(
        self, mock_client: MagicMock
    ) -> None:
        agent = instrument_pydanticai_agent(
            _DummyPydanticAgent(async_error=RuntimeError("async run failed")),
            mock_client,
            "ws_test",
        )

        with pytest.raises(RuntimeError, match="async run failed"):
            await agent.run("hello")

        event = _captured_event(mock_client)
        validate_event(event)
        assert event["task"]["outcome"] == "failure"

    @pytest.mark.asyncio
    async def test_usage_extraction_failure_is_swallowed(self, mock_client: MagicMock) -> None:
        agent = instrument_pydanticai_agent(
            _DummyPydanticAgent(async_result=_BrokenUsageRunResult()),
            mock_client,
            "ws_test",
        )

        result = await agent.run("hello")

        assert isinstance(result, _BrokenUsageRunResult)
        event = _captured_event(mock_client)
        validate_event(event)
        assert "tokens_prompt" not in event["metrics"]
        assert "tool_call_count" not in event["metrics"]


def _captured_event(mock_client: MagicMock) -> dict[str, Any]:
    mock_client.ingest_event.assert_called_once()
    return cast(dict[str, Any], mock_client.ingest_event.call_args[1]["event"])

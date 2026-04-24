"""Tests for framework adapters."""
from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import pytest

from heeczer.adapters.google_adk import heeczer_adk_wrapper
from heeczer.adapters.langgraph import HeeczerLangGraphCallback
from heeczer.client import HeeczerClient


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

    def test_chain_error_sets_failure_outcome(self, mock_client: MagicMock) -> None:
        cb = HeeczerLangGraphCallback(mock_client, "ws_test")
        cb.on_chain_start({"name": "failing_node"}, {}, run_id="run_003")
        cb.on_chain_error(ValueError("oops"), run_id="run_003")
        # Context removed after error
        assert "run_003" not in cb._runs

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

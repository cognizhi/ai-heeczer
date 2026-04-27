from __future__ import annotations

import json
import os
import time
from decimal import Decimal
from pathlib import Path
from typing import Any

import httpx
import pytest

ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIR = ROOT / "testing" / "tests" / "fixtures" / "skills"
PROMPT_FIXTURE = ROOT / "testing" / "tests" / "fixtures" / "prompts" / "01-summarize.json"

SKILLS = [
    "code_gen",
    "rca",
    "doc_summary",
    "compliance",
    "ci_triage",
    "architecture",
]

REQUIRE_STACK = os.environ.get("HEECZER_REQUIRE_STACK") == "1"
FORBIDDEN_CONTENT_KEYS = {
    "prompt",
    "prompt_text",
    "response",
    "response_text",
    "completion",
    "completion_text",
    "output",
    "output_text",
    "api_key",
    "apikey",
    "apikeys",
    "file_attachments",
}


def unavailable_stack(message: str) -> None:
    if REQUIRE_STACK:
        pytest.fail(message)
    pytest.skip(message)

SDK_CONFIG = {
    "js": {"api_port": 18001, "ingest_port": 18010, "workspace_id": "local-test-js"},
    "py": {"api_port": 18101, "ingest_port": 18110, "workspace_id": "local-test-py"},
    "go": {"api_port": 18201, "ingest_port": 18210, "workspace_id": "local-test-go"},
    "java": {"api_port": 18301, "ingest_port": 18310, "workspace_id": "local-test-java"},
    "rs": {"api_port": 18401, "ingest_port": 18410, "workspace_id": "local-test-rs"},
    "pydanticai": {
        "api_port": 18501,
        "ingest_port": 18510,
        "workspace_id": "local-test-pydanticai",
    },
}


def load_skill_fixture(skill: str) -> dict[str, Any]:
    with (FIXTURE_DIR / f"{skill}.json").open(encoding="utf-8") as fh:
        return json.load(fh)


def prompt_text() -> str:
    with PROMPT_FIXTURE.open(encoding="utf-8") as fh:
        return json.load(fh)["prompt"]


def require_stack_up(sdk: str) -> dict[str, Any]:
    cfg = SDK_CONFIG[sdk]
    ingest_url = f"http://127.0.0.1:{cfg['ingest_port']}/v1/ready"
    try:
        ingest_resp = httpx.get(ingest_url, timeout=0.6)
    except httpx.HTTPError as exc:
        unavailable_stack(
            f"heeczer-test-{sdk} ingest is down or unreachable at {ingest_url}; "
            f"run `make start-test-{sdk}` first ({exc.__class__.__name__})"
        )
    if ingest_resp.status_code >= 400:
        unavailable_stack(
            f"heeczer-test-{sdk} ingest readiness returned HTTP {ingest_resp.status_code}; "
            f"run `make logs-test-{sdk}`"
        )

    url = f"http://127.0.0.1:{cfg['api_port']}/healthz"
    try:
        resp = httpx.get(url, timeout=0.6)
    except httpx.HTTPError as exc:
        unavailable_stack(
            f"heeczer-test-{sdk} stack is down or unreachable at {url}; "
            f"run `make start-test-{sdk}` first ({exc.__class__.__name__})"
        )
    if resp.status_code >= 400:
        unavailable_stack(
            f"heeczer-test-{sdk} stack health check returned HTTP {resp.status_code}; "
            f"run `make ps-test-{sdk}` and `make logs-test-{sdk}`"
        )
    return cfg


def run_chat_turn(sdk: str, skill: str) -> dict[str, Any]:
    cfg = require_stack_up(sdk)
    resp = httpx.post(
        f"http://127.0.0.1:{cfg['api_port']}/chat",
        json={"skill": skill, "prompt": prompt_text(), "provider": "mock"},
        timeout=15,
    )
    resp.raise_for_status()
    body = resp.json()
    assert body["ok"] is True
    assert body["skill"] == skill
    assert body["event_id"]
    assert body["event"]["event_id"] == body["event_id"]
    assert "score_result" in body
    return body


def fetch_persisted_event(sdk: str, event_id: str) -> dict[str, Any]:
    cfg = SDK_CONFIG[sdk]
    url = (
        f"http://127.0.0.1:{cfg['ingest_port']}/v1/events/{event_id}"
        f"?workspace_id={cfg['workspace_id']}"
    )
    deadline = time.monotonic() + 5
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            resp = httpx.get(url, timeout=1)
            if resp.status_code == 200:
                return resp.json()["payload"]
            last_error = AssertionError(f"HTTP {resp.status_code}: {resp.text}")
        except httpx.HTTPError as exc:
            last_error = exc
        time.sleep(0.2)
    raise AssertionError(f"event {event_id} was not readable from ingest: {last_error}")


def fetch_persisted_scores(sdk: str, event_id: str) -> list[dict[str, Any]]:
    cfg = SDK_CONFIG[sdk]
    url = (
        f"http://127.0.0.1:{cfg['ingest_port']}/v1/events/{event_id}/scores"
        f"?workspace_id={cfg['workspace_id']}"
    )
    resp = httpx.get(url, timeout=2)
    resp.raise_for_status()
    body = resp.json()
    assert body["ok"] is True
    return body["scores"]


def fetch_scoring_version(sdk: str) -> str:
    cfg = SDK_CONFIG[sdk]
    resp = httpx.get(f"http://127.0.0.1:{cfg['ingest_port']}/v1/version", timeout=2)
    resp.raise_for_status()
    return resp.json()["scoring_version"]


def assert_event_matches_fixture(event: dict[str, Any], fixture: dict[str, Any], sdk: str) -> None:
    assert_no_raw_content(event)
    expected = fixture["expected_event"]
    assert event["spec_version"] == "1.0"
    assert event["framework_source"] == f"chatbot-{sdk}"
    assert event["workspace_id"] == SDK_CONFIG[sdk]["workspace_id"]
    assert event["task"]["category"] == expected["task"]["category"]
    assert event["task"]["sub_category"] == expected["task"]["sub_category"]
    assert event["task"]["outcome"] == expected["task"]["outcome"]

    metrics = event["metrics"]
    expected_metrics = expected["metrics"]
    for key in ["tool_call_count", "workflow_steps", "retries", "artifact_count"]:
        assert metrics[key] == expected_metrics[key]
    assert Decimal(str(metrics["output_size_proxy"])) == Decimal(str(expected_metrics["output_size_proxy"]))
    assert metrics["tokens_prompt"] >= expected_metrics["tokens_prompt_min"]
    assert metrics["tokens_completion"] >= expected_metrics["tokens_completion_min"]

    context = event["context"]
    for key, value in expected["context"].items():
        assert context[key] == value
    assert "local-stack" in context["tags"]
    assert sdk in context["tags"]
    assert fixture["skill"] in context["tags"]

    meta = event["meta"]
    expected_language = "python" if sdk == "pydanticai" else {"js": "node", "py": "python", "rs": "rust"}.get(sdk, sdk)
    assert meta["sdk_language"] == expected_language
    assert meta["scoring_profile"] == "default"
    assert meta["extensions"]["chatbot.skill"] == fixture["skill"]
    assert meta["extensions"]["chatbot.tool_trace"] == [step["tool"] for step in fixture["mock_script"]]


def assert_no_raw_content(value: Any) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            assert key.lower() not in FORBIDDEN_CONTENT_KEYS, f"raw content key leaked into event: {key}"
            assert_no_raw_content(child)
    elif isinstance(value, list):
        for child in value:
            assert_no_raw_content(child)


def run_skill_case(sdk: str, skill: str) -> dict[str, Any]:
    fixture = load_skill_fixture(skill)
    body = run_chat_turn(sdk, skill)
    persisted = fetch_persisted_event(sdk, body["event_id"])
    assert_event_matches_fixture(persisted, fixture, sdk)
    score = body["score_result"]
    assert score["scoring_version"] == fetch_scoring_version(sdk)
    scores = fetch_persisted_scores(sdk, body["event_id"])
    matching_scores = [
        persisted_score
        for persisted_score in scores
        if persisted_score["scoring_version"] == score["scoring_version"]
    ]
    assert matching_scores, "persisted score row for returned scoring_version was not found"
    persisted_score = matching_scores[0]
    raw_fec = score.get("fec") or score.get("financial_equivalent_cost")
    assert Decimal(persisted_score["final_fec"]) == Decimal(str(raw_fec))
    assert Decimal(persisted_score["final_minutes"]) == Decimal(str(score["final_estimated_minutes"]))
    return body


def score_fec(body: dict[str, Any]) -> Decimal:
    score = body["score_result"]
    raw = score.get("fec") or score.get("financial_equivalent_cost")
    assert raw is not None
    return Decimal(str(raw))


def assert_high_effort_scores_above_ci_triage(sdk: str) -> None:
    ci = run_skill_case(sdk, "ci_triage")
    compliance = run_skill_case(sdk, "compliance")
    rca = run_skill_case(sdk, "rca")
    assert score_fec(compliance) > score_fec(ci)
    assert score_fec(rca) > score_fec(ci)

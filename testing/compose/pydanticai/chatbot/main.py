from __future__ import annotations

import os
import time
import uuid
from datetime import UTC, datetime
from typing import Any

import httpx
from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse
from pydantic import BaseModel

from heeczer import HeeczerClient, __version__
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent
from skills.catalogue import active_tools, load_skill
from tools.catalogue import function_schemas, pydantic_ai_tools, trace_for_tools

app = FastAPI(title="ai-heeczer PydanticAI local stack")
workspace_id = os.environ.get("CHATBOT_WORKSPACE_ID", "local-test-pydanticai")
scoring_profile = os.environ.get("CHATBOT_SCORING_PROFILE", "default")
heeczer_client = HeeczerClient(base_url=os.environ.get("HEECZER_BASE_URL", "http://heeczer-ingest:8080"))


class ChatRequest(BaseModel):
    skill: str | None = None
    prompt: str | None = None
    provider: str | None = None


class MockRunResult:
    def __init__(self, usage: dict[str, Any]) -> None:
        self.output = usage["text"]
        self._usage = {
            "input_tokens": usage["prompt_tokens"] or 0,
            "output_tokens": usage["completion_tokens"] or 0,
            "tool_calls": usage["tool_calls"],
            "retries": usage["retries"],
        }

    def usage(self) -> dict[str, Any]:
        return self._usage


class MockPydanticAgent:
    def __init__(self, name: str, usage: dict[str, Any]) -> None:
        self.name = name
        self._usage = usage

    async def run(self, prompt: str) -> MockRunResult:
        return MockRunResult(self._usage)


class TransformingAdapterClient:
    def __init__(self, fixture_event: dict[str, Any]) -> None:
        self.fixture_event = fixture_event
        self.last_submission: dict[str, Any] | None = None
        self.submitted_event: dict[str, Any] | None = None

    async def ingest_event(self, *, workspace_id: str, event: dict[str, Any]) -> dict[str, Any]:
        adapted_event = dict(event)
        adapted_event["task"] = {**event.get("task", {}), **self.fixture_event["task"]}
        adapted_event["metrics"] = {**event.get("metrics", {}), **self.fixture_event["metrics"]}
        adapted_event["context"] = self.fixture_event["context"]
        adapted_event["meta"] = {
            **event.get("meta", {}),
            **self.fixture_event["meta"],
            "extensions": {
                **self.fixture_event["meta"]["extensions"],
                "chatbot.adapter_event_id": event["event_id"],
            },
        }
        if project_id := self.fixture_event.get("project_id"):
            adapted_event["project_id"] = project_id
        self.submitted_event = adapted_event
        self.last_submission = await heeczer_client.ingest_event(workspace_id=workspace_id, event=adapted_event)
        return self.last_submission


@app.get("/healthz")
async def healthz() -> dict[str, bool]:
    return {"ok": True}


@app.get("/", response_class=HTMLResponse)
async def root() -> str:
    return """<!doctype html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>ai-heeczer PydanticAI stack</title></head><body><main><h1>ai-heeczer PydanticAI stack</h1><form id='chat'><select name='skill'><option value='code_gen'>code_gen</option><option value='rca'>rca</option><option value='doc_summary'>doc_summary</option><option value='compliance'>compliance</option><option value='ci_triage'>ci_triage</option><option value='architecture'>architecture</option></select><input name='prompt' value='Summarize this local SDK stack'><button>Send</button></form><pre id='out'></pre></main><script>document.querySelector('#chat').addEventListener('submit',async(e)=>{e.preventDefault();const f=new FormData(e.target);const r=await fetch('/chat',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(Object.fromEntries(f))});document.querySelector('#out').textContent=JSON.stringify(await r.json(),null,2);});</script></body></html>"""


async def call_provider(fixture: dict[str, Any], prompt: str, provider: str) -> dict[str, Any]:
    if provider == "mock":
        metrics = fixture["expected_event"]["metrics"]
        return {
            "prompt_tokens": metrics["tokens_prompt_min"],
            "completion_tokens": metrics["tokens_completion_min"],
            "tool_calls": metrics["tool_call_count"],
            "retries": metrics["retries"],
            "text": f"Mock {fixture['skill']} turn completed.",
        }

    if provider in {"openrouter", "gemini"}:
        is_gemini = provider == "gemini"
        api_key = os.environ.get("GEMINI_API_KEY" if is_gemini else "OPENROUTER_API_KEY")
        model = os.environ.get("GEMINI_MODEL" if is_gemini else "OPENROUTER_MODEL")
        if not api_key or not model or "changeme" in api_key:
            raise HTTPException(status_code=400, detail=f"{provider} requires an API key and model")
        endpoint = (
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
            if is_gemini
            else "https://openrouter.ai/api/v1/chat/completions"
        )
        async with httpx.AsyncClient(timeout=30) as client:
            response = await client.post(
                endpoint,
                headers={"authorization": f"Bearer {api_key}", "content-type": "application/json"},
                json={
                    "model": model,
                    "messages": [
                        {"role": "system", "content": f"Run the {fixture['skill']} PydanticAI local-stack scenario without revealing raw prompts."},
                        {"role": "user", "content": prompt},
                    ],
                    "tools": function_schemas(active_tools(fixture)),
                    "tool_choice": "auto",
                },
            )
            response.raise_for_status()
            payload = response.json()
        usage = payload.get("usage") or {}
        return {
            "prompt_tokens": usage.get("prompt_tokens"),
            "completion_tokens": usage.get("completion_tokens"),
            "tool_calls": len(((payload.get("choices") or [{}])[0].get("message") or {}).get("tool_calls") or []),
            "retries": 0,
            "text": (((payload.get("choices") or [{}])[0].get("message") or {}).get("content")) or f"{provider} completed {fixture['skill']}.",
        }

    if provider == "local":
        base_url = os.environ.get("LOCAL_MODEL_BASE_URL", "http://ollama:11434").rstrip("/")
        model = os.environ.get("LOCAL_MODEL", "llama3.2:1b")
        async with httpx.AsyncClient(timeout=60) as client:
            response = await client.post(
                f"{base_url}/api/chat",
                json={"model": model, "stream": False, "messages": [{"role": "user", "content": prompt}]},
            )
            response.raise_for_status()
            payload = response.json()
        return {"prompt_tokens": None, "completion_tokens": None, "tool_calls": 0, "retries": 0, "text": (payload.get("message") or {}).get("content") or "Local model completed."}

    raise HTTPException(status_code=400, detail=f"unsupported provider {provider}")


def build_event(fixture: dict[str, Any], usage: dict[str, Any], started: float) -> dict[str, Any]:
    expected = fixture["expected_event"]
    metrics = expected["metrics"]
    context = dict(expected["context"])
    context["tags"] = ["local-stack", "pydanticai", fixture["skill"]]
    event = {
        "spec_version": "1.0",
        "event_id": str(uuid.uuid4()),
        "correlation_id": f"pydanticai-session:{int(time.time() * 1000)}",
        "timestamp": datetime.now(UTC).isoformat(),
        "framework_source": "chatbot-pydanticai",
        "workspace_id": workspace_id,
        "task": {"name": f"{fixture['skill']}: local stack turn", **expected["task"]},
        "metrics": {
            "duration_ms": max(1, int((time.monotonic() - started) * 1000)),
            "tokens_prompt": usage["prompt_tokens"],
            "tokens_completion": usage["completion_tokens"],
            "tool_call_count": metrics["tool_call_count"],
            "workflow_steps": metrics["workflow_steps"],
            "retries": metrics["retries"],
            "artifact_count": metrics["artifact_count"],
            "output_size_proxy": metrics["output_size_proxy"],
        },
        "context": context,
        "meta": {
            "sdk_language": "python",
            "sdk_version": __version__,
            "scoring_profile": scoring_profile,
            "extensions": {
                "chatbot.skill": fixture["skill"],
                "chatbot.turn": 1,
                "chatbot.tool_trace": [entry["tool_name"] for entry in trace_for_tools(active_tools(fixture))],
            },
        },
    }
    if project_id := os.environ.get("CHATBOT_PROJECT_ID"):
        event["project_id"] = project_id
    return event


@app.post("/chat")
async def chat(request: ChatRequest) -> dict[str, Any]:
    started = time.monotonic()
    fixture = load_skill(request.skill)
    provider = request.provider or os.environ.get("LLM_PROVIDER", "mock")
    prompt = request.prompt or "Summarize this local SDK stack."
    usage = await call_provider(fixture, prompt, provider)
    _declared_tools = pydantic_ai_tools(active_tools(fixture))
    event = build_event(fixture, usage, started)
    adapter_client = TransformingAdapterClient(event)
    agent = MockPydanticAgent(f"heeczer-{fixture['skill']}", usage)
    instrumented = instrument_pydanticai_agent(
        agent=agent,
        client=adapter_client,
        workspace_id=workspace_id,
        task_name=f"{fixture['skill']}: local stack turn",
        task_category=fixture["expected_event"]["task"]["category"],
        framework_source="chatbot-pydanticai",
        sdk_language="python",
        sdk_version=__version__,
    )
    result = await instrumented.run(prompt)
    if adapter_client.last_submission is None:
        raise HTTPException(status_code=500, detail="PydanticAI adapter did not submit an event")
    if adapter_client.submitted_event is None:
        raise HTTPException(status_code=500, detail="PydanticAI adapter event was not captured")
    return {
        "ok": True,
        "skill": fixture["skill"],
        "event_id": adapter_client.last_submission["event_id"],
        "reply": result.output,
        "tool_trace": trace_for_tools(active_tools(fixture)),
        "event": adapter_client.submitted_event,
        "score_result": adapter_client.last_submission["score"],
    }

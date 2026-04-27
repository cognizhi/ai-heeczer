# Plan 11 — Framework adapters

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-27
- **PRD:** §24
- **ADR:** ADR-0002

## Goal

Translate native telemetry from popular agentic frameworks into the canonical event schema.

## Checklist

### MVP adapters

- [x] LangGraph adapter (Python): `HeeczerLangGraphCallback` duck-typed callback handler in `bindings/heeczer-py/src/heeczer/adapters/langgraph.py`. Emits a canonical event per node execution. (session Apr-2026)
- [x] Google ADK adapter (Python): `heeczer_adk_wrapper` async decorator in `bindings/heeczer-py/src/heeczer/adapters/google_adk.py`. (session Apr-2026)
- [x] Adapter contract test: 12 tests in `bindings/heeczer-py/tests/test_adapters.py` with strict event validation for emitted adapter payloads. (session Apr-2026)

### Phase 2

- [x] PydanticAI adapter (Python): `instrument_pydanticai_agent` proxy wrapper around `Agent.run()` / `run_sync()` in `bindings/heeczer-py/src/heeczer/adapters/pydantic_ai.py`. Emits one canonical event per invocation without a hard `pydantic_ai` dependency. (session Apr-2026)

### Future

- [ ] Langfuse webhook adapter.
- [ ] OpenTelemetry bridge.
- [ ] Generic webhook adapter.
- [ ] Custom middleware template repo.

### Docs and examples

- [x] `bindings/heeczer-py/README.md` adapters section (LangGraph + Google ADK + PydanticAI). (session Apr-2026)
- [x] Example under `examples/langgraph/README.md` and `examples/google-adk/README.md`. (session Apr-2026)
- [x] Example under `examples/pydanticai/README.md`. (session Apr-2026)
- [x] Mapping reference in `docs/architecture/integrations.md` (LangGraph + Google ADK + PydanticAI). (session Apr-2026)

## Acceptance

- Adapter contract tests green for every shipped adapter.

# Plan 11 — Framework adapters

- **Status:** Active
- **Owner:** SDK Engineer
- **PRD:** §24
- **ADR:** n/a

## Goal

Translate native telemetry from popular agentic frameworks into the canonical event schema.

## Checklist

### MVP adapters

- [x] LangGraph adapter (Python): `HeeczerLangGraphCallback` duck-typed callback handler in `bindings/heeczer-py/src/heeczer/adapters/langgraph.py`. Emits a canonical event per node execution. (session Apr-2026)
- [x] Google ADK adapter (Python): `heeczer_adk_wrapper` async decorator in `bindings/heeczer-py/src/heeczer/adapters/google_adk.py`. (session Apr-2026)
- [x] Adapter contract test: 7 tests in `bindings/heeczer-py/tests/test_adapters.py` (all green). (session Apr-2026)

### Phase 2

- [ ] PydanticAI adapter.

### Future

- [ ] Langfuse webhook adapter.
- [ ] OpenTelemetry bridge.
- [ ] Generic webhook adapter.
- [ ] Custom middleware template repo.

### Docs and examples

- [x] `bindings/heeczer-py/README.md` adapters section (LangGraph + Google ADK). (session Cat-3)
- [x] Example under `examples/langgraph/README.md` and `examples/google-adk/README.md`. (session Apr-2026)
- [x] Mapping reference in `docs/architecture/integrations.md`. (session Apr-2026)

## Acceptance

- Adapter contract tests green for every shipped adapter.

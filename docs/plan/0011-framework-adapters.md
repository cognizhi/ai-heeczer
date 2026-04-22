# Plan 11 — Framework adapters

- **Status:** Active
- **Owner:** SDK Engineer
- **PRD:** §24
- **ADR:** n/a

## Goal
Translate native telemetry from popular agentic frameworks into the canonical event schema.

## Checklist

### MVP adapters
- [ ] LangGraph adapter (Python): callback handler that emits a canonical event per node execution.
- [ ] Google ADK adapter (Python): instrumentation hook covering tool calls, retries, outcomes.
- [ ] Adapter contract test: golden inputs (raw framework events) → canonical events match fixture.

### Phase 2
- [ ] PydanticAI adapter.

### Future
- [ ] Langfuse webhook adapter.
- [ ] OpenTelemetry bridge.
- [ ] Generic webhook adapter.
- [ ] Custom middleware template repo.

### Docs and examples
- [ ] `bindings/python/README.md` adapters section.
- [ ] Example under `examples/langgraph/` and `examples/google-adk/`.
- [ ] Mapping reference in `docs/architecture/integrations.md`.

## Acceptance
- Adapter contract tests green for every shipped adapter.

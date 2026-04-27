# Plan 06 — Python SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-27
- **PRD:** §23
- **ADR:** ADR-0001

## Goal

Ship `ai-heeczer` on PyPI with `pyo3` + `maturin` packaging and `abi3` wheels, idiomatic Python API, full parity with shared fixtures.

## Checklist

### Package

- [x] `bindings/heeczer-py/` package with `pyproject.toml` (uv-managed, hatchling backend, Python ≥ 3.11 since the test fixtures use modern type syntax). Path differs from the original `bindings/python/` placeholder.
- [ ] abi3 wheels for cpython 3.10+, manylinux/musllinux/macos/windows. (deferred: HTTP-first SDK ships now; pyo3/maturin in-process binding follows after the HTTP parity gate is stable)
- [x] `py.typed` marker shipped; types inline in `client.py` via `TypedDict` + `Literal` (closed `kind` enum).
- [x] Adapter module `heeczer.adapters` added with LangGraph and Google ADK adapters. (session Apr-2026)

### Public API

- [x] `HeeczerClient` async client with `healthz`, `version`, `ingest_event`, `test_score_pipeline`. (The plan's original `track`/`track_batch`/`flush`/`close` shape predates the ingestion service; HTTP-first surface is the foundation, with batching following the batch endpoint in plan 0004.)
- [x] Both sync and async (`asyncio`) methods. `SyncHeeczerClient` wrapper added in `client.py`, exported from `__init__.py`. (session Cat-3)
- [x] Mode selection: `mode="image" | "native"` is accepted by async and sync clients; image mode is implemented and native mode fails fast with an explicit pyo3/maturin binding message. Native functionality remains gated by the unchecked abi3 wheel item above. (session Apr-2026)
- [x] Pydantic v2 models for events. `EventModel` and nested models reject unknown fields outside `meta.extensions`; `validate_event()` is exported. Score/trace output remains represented by open `TypedDict` surfaces because the Rust engine owns additive result fields.

### Tests

- [x] Unit (`pytest` async; httpx.MockTransport instead of mocks per the user's "use emulation method as priority" guidance). 8/8 pass.
- [x] Contract: shared fixtures. Pytest round-trips all shared valid fixtures and validates them with the Pydantic v2 model.
- [x] Parity: byte-equal output vs Rust reference. `parity.yml` now generates Rust CLI reference `ScoreResult` JSON, starts `heeczer-ingest` with test orchestration enabled, and runs `bindings/heeczer-py/scripts/parity.py` against every shared valid fixture. (session Apr-2026)
- [x] `mypy --strict` clean (3 source files).
- [x] `ruff check` clean.
- [ ] Packaging: `maturin build --release` smoke test. (depends on pyo3 binding above)

### Docs

- [x] `bindings/heeczer-py/README.md` with quickstart, configuration, methods table, error-kind matrix, and link to runnable example.
- [x] Example app under `examples/python/quickstart.py` (cross-language index in `examples/README.md`).

## Acceptance

- Parity job green for the HTTP/image SDK surface.
- abi3 wheels publish on release via PyPI trusted publishing (ADR-0009).

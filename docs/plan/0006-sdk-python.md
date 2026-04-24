# Plan 06 — Python SDK

- **Status:** Active
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23
- **ADR:** ADR-0001

## Goal
Ship `ai-heeczer` on PyPI with `pyo3` + `maturin` packaging and `abi3` wheels, idiomatic Python API, full parity with shared fixtures.

## Checklist

### Package
- [x] `bindings/heeczer-py/` package with `pyproject.toml` (uv-managed, hatchling backend, Python ≥ 3.11 since the test fixtures use modern type syntax). Path differs from the original `bindings/python/` placeholder.
- [ ] abi3 wheels for cpython 3.10+, manylinux/musllinux/macos/windows. (deferred: HTTP-first SDK ships now; pyo3/maturin in-process binding follows once parity test rig + napi-rs sibling land)
- [x] `py.typed` marker shipped; types inline in `client.py` via `TypedDict` + `Literal` (closed `kind` enum).
- [x] Adapter module `heeczer.adapters` added with LangGraph and Google ADK adapters. (session Apr-2026)

### Public API
- [x] `HeeczerClient` async client with `healthz`, `version`, `ingest_event`, `test_score_pipeline`. (The plan's original `track`/`track_batch`/`flush`/`close` shape predates the ingestion service; HTTP-first surface is the foundation, with batching following the batch endpoint in plan 0004.)
- [ ] Both sync and async (`asyncio`) methods. (async-only today; sync wrapper deferred)
- [ ] Mode selection: `native` and `image`. (image-only today; in-process scoring depends on pyo3 binding above)
- [ ] Pydantic v2 models for events, scores, traces. (TypedDicts today to keep the wheel stdlib-only beyond httpx)

### Tests
- [x] Unit (`pytest` async; httpx.MockTransport instead of mocks per the user's "use emulation method as priority" guidance). 8/8 pass.
- [ ] Contract: shared fixtures. (pending: needs the parity fixture rig in plan 0001 §Tests)
- [ ] Parity: byte-equal output vs Rust reference.
- [x] `mypy --strict` clean (3 source files).
- [x] `ruff check` clean.
- [ ] Packaging: `maturin build --release` smoke test. (depends on pyo3 binding above)

### Docs
- [x] `bindings/heeczer-py/README.md` with quickstart, configuration, methods table, error-kind matrix, and link to runnable example.
- [x] Example app under `examples/python/quickstart.py` (cross-language index in `examples/README.md`).

## Acceptance
- Parity job green.
- abi3 wheels publish on release via PyPI trusted publishing (ADR-0009).

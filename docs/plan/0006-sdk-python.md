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
- [ ] `bindings/python/` package with `pyproject.toml` + maturin.
- [ ] abi3 wheels for cpython 3.10+, manylinux/musllinux/macos/windows.
- [ ] Type stubs (`.pyi`) generated from schema; `py.typed` marker.

### Public API
- [ ] `Client` with `track`, `track_batch`, `flush`, `close`.
- [ ] Both sync and async (`asyncio`) methods.
- [ ] Mode selection: `native` and `image`.
- [ ] Pydantic v2 models for events, scores, traces.

### Tests
- [ ] Unit (`pytest`).
- [ ] Contract: shared fixtures.
- [ ] Parity: byte-equal output vs Rust reference.
- [ ] `mypy --strict` clean.
- [ ] `ruff check` clean.
- [ ] Packaging: `maturin build --release` smoke test.

### Docs
- [ ] `bindings/python/README.md` with quickstart, API reference.
- [ ] Example app under `examples/python/`.

## Acceptance
- Parity job green.
- abi3 wheels publish on release via PyPI trusted publishing (ADR-0009).

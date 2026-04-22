# Plan 09 — Java SDK

- **Status:** Active (Phase 2 target)
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23, §31 (Excluded from MVP if not ready)
- **ADR:** ADR-0001

## Goal
Ship the Java SDK on Maven Central using the Foreign Function & Memory API (JDK 22+); JNI fallback only if the support matrix demands earlier JDKs.

## Checklist

- [x] `bindings/heeczer-java/` Maven module.
- [ ] FFM bindings to `heeczer-core-c`.
- [x] Public `HeeczerClient` HTTP mode via `java.net.http`.
- [x] Mode selection: HTTP client (stdlib, no third-party HTTP lib).
- [x] Unit (JUnit 5) + WireMock 3.x contract tests — 9/9 pass.
- [x] `mvn test` clean.
- [x] `bindings/heeczer-java/README.md`.
- [ ] Example under `examples/java/`.

## Acceptance
- Parity job green.
- Maven Central staging close clean in release dry-run.

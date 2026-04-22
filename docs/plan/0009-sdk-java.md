# Plan 09 — Java SDK

- **Status:** Active (Phase 2 target)
- **Owner:** SDK Engineer
- **Last reviewed:** 2026-04-22
- **PRD:** §23, §31 (Excluded from MVP if not ready)
- **ADR:** ADR-0001

## Goal
Ship the Java SDK on Maven Central using the Foreign Function & Memory API (JDK 22+); JNI fallback only if the support matrix demands earlier JDKs.

## Checklist

- [ ] `bindings/java/` Maven module.
- [ ] FFM bindings to `heeczer-core-c`.
- [ ] Public `Client` with sync and `CompletableFuture` async APIs.
- [ ] Mode selection: `native` and `image` (via `java.net.http`).
- [ ] Unit (JUnit 5), contract, parity tests.
- [ ] `mvn package` clean; sources + javadoc jars.
- [ ] `bindings/java/README.md`.
- [ ] Example under `examples/java/`.

## Acceptance
- Parity job green.
- Maven Central staging close clean in release dry-run.

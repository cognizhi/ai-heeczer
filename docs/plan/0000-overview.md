# Implementation plan — Overview

- **Status:** Active
- **Owner:** Tech Lead
- **Last reviewed:** 2026-04-24
- **Related:** PRD §31 MVP Definition, §32 Roadmap, §37 Engineering Milestones

This is the entry point for ai-heeczer implementation plans. Each linked document below is a concrete, checklist-driven plan covering one slice of the system. Plans cite their PRD and ADR sources so any agent or contributor can trace a checklist item back to product intent.

## Conventions

- Each plan has a status (`Active`, `Blocked`, `Done`).
- Each checklist item is `[ ]` (open) or `[x]` (complete with merged-PR link).
- An item is marked done **only** when the implementing PR is merged and CI is green (per `docs/agents/AGENT_HARNESS.md` §2.10).
- Plan-level changes (adding/removing checkboxes, restructuring) require Tech Lead review.

## Phasing (PRD §32)

- **Phase 1 — Foundation:** plans 01–05 (schema, scoring, storage, ingestion preview, JS/TS SDK).
- **Phase 1.x:** plans 06–07 (Python and Go SDKs), 10 (read-only dashboard), 11 (LangGraph + ADK adapters), 12 (CI/CD), 13 (DevEx), 14 (security baseline), 15 (calibration scaffolding).
- **Phase 2 — Production readiness:** image mode default, PostgreSQL worker, RBAC, audit, Java SDK, mature publish flows.
- **Phase 3 — Enterprise analytics:** advanced calibration, anomaly detection, benchmark packs.
- **Phase 4 — Ecosystem expansion:** OTel bridge, Langfuse, profile marketplace.

## Plans

| # | Plan | Primary PRD refs | Primary ADR refs |
| --- | --- | --- | --- |
| 01 | [Schema and contracts](0001-schema-and-contracts.md) | §13, §12.2, §12.16 | ADR-0002 |
| 02 | [Scoring core](0002-scoring-core.md) | §14, §15, §16 | ADR-0001, ADR-0003 |
| 03 | [Storage and migrations](0003-storage-and-migrations.md) | §20, §12.20 | ADR-0004 |
| 04 | [Ingestion service](0004-ingestion-service.md) | §12.4, §19, §29 | ADR-0005, ADR-0006 |
| 05 | [JS/TS SDK](0005-sdk-jsts.md) | §23 | ADR-0001 |
| 06 | [Python SDK](0006-sdk-python.md) | §23 | ADR-0001 |
| 07 | [Go SDK](0007-sdk-go.md) | §23 | ADR-0001 |
| 08 | [Rust SDK](0008-sdk-rust.md) | §23 | ADR-0001 |
| 09 | [Java SDK](0009-sdk-java.md) | §23 | ADR-0001 |
| 10 | [Dashboard](0010-dashboard.md) | §21, §28.4 | ADR-0008 |
| 11 | [Framework adapters](0011-framework-adapters.md) | §24 | — |
| 12 | [CI/CD and release](0012-cicd-release.md) | §27, §12.10–§12.11 | ADR-0009 |
| 13 | [Developer experience](0013-developer-experience.md) | §26, §12.13, §12.21 | ADR-0007, ADR-0010 |
| 14 | [Security and privacy](0014-security-and-privacy.md) | §22, §12.17–§12.19 | — |
| 15 | [Calibration and benchmarks](0015-calibration-benchmarks.md) | §25, §12.8 | — |
| 16 | [Local per-SDK test stacks](0016-local-sdk-test-stacks.md) | §12.13, §12.21, §23, §24, §26 | ADR-0005, ADR-0007, ADR-0008, ADR-0010 |

## Cross-cutting acceptance gates

A plan is "Done" only when:

1. All checklist items are `[x]` with merged-PR links.
2. All required CI jobs pass on `main`.
3. All affected docs (README, architecture, ADRs) are current.
4. The Tech Lead has signed off via the chatmode review template.

## Proposing a new plan

1. Open a draft PR adding `docs/plan/NN-<slug>.md` based on the structure of an existing plan.
2. Cite at least one PRD section and zero or more ADRs.
3. Add an entry to the table above and assign an owner.
4. Request review from the Tech Lead and Tech Writer subagents.
5. Mark the plan `Active` only after merge.

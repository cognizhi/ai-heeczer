# AGENTS.md

This file is the shared root entrypoint for agentic AI working in this repository.

Treat this file as a routing layer, not as a second policy source. The authoritative repository rules live in `docs/agents/AGENT_HARNESS.md`.

## Read first

1. `docs/agents/AGENT_HARNESS.md`
2. `docs/prd/ai-heeczer-prd.md`
3. `docs/adr/*.md` (every ADR marked Accepted)
4. The `docs/plan/*.md` plan covering the area you are touching

## Repository agent files

- `docs/agents/AGENT_HARNESS.md` — authoritative repository harness, development loop, and quality gates.
- `.github/copilot-instructions.md` — Copilot-specific shim and chatmode entrypoint.
- `.github/agents/*.md` — role definitions and tool allowlists.
- `CLAUDE.md` — Claude-compatible shim that points back to this file and the harness.

## Rules for any agent

- Follow the strict existing harness. If this file conflicts with the harness, the harness wins.
- Keep shared repository policy in `docs/agents/AGENT_HARNESS.md` or this file. Keep tool-specific entrypoints thin.
- Respect the tool allowlists declared in `.github/agents/*.md`. If a tool is missing for a role, do not assume it is available.
- Stop and ask rather than invent when PRD, ADR, plan, or security guidance conflicts.

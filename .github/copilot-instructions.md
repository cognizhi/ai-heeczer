# Copilot Instructions — ai-heeczer

`AGENTS.md` is the shared cross-agent entrypoint for this repository.

`docs/agents/AGENT_HARNESS.md` is the authoritative repository harness. This file stays intentionally thin so shared policy lives in one place.

## Read first
1. `AGENTS.md`
2. `docs/agents/AGENT_HARNESS.md`
3. `docs/prd/ai-heeczer-prd.md`
4. `docs/adr/*.md` (every ADR marked Accepted)
5. The `docs/plan/*.md` plan covering the area you are touching

## Copilot-specific guidance
- Use the chatmodes defined in `.github/agents/` when the task benefits from role specialization.
- Respect each role file's `tools:` list. If a tool is missing, use a different appropriate role or update the role file via PR.
- Keep shared repository policy out of this file. Add it to `AGENTS.md` or `docs/agents/AGENT_HARNESS.md` instead.

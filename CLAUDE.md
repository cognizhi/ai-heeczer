# CLAUDE.md

This file is the Claude entrypoint for this repository.

Follow `AGENTS.md` first. This file intentionally stays thin so shared repository policy lives in one place.

## Required reading order
1. `AGENTS.md`
2. `docs/agents/AGENT_HARNESS.md`
3. `docs/prd/ai-heeczer-prd.md`
4. `docs/adr/*.md` (every ADR marked Accepted)
5. The `docs/plan/*.md` plan covering the area you are touching

## Claude-specific notes
- Use `.github/agents/*.md` when a specialized review or implementation role is needed.
- Respect each role file's declared tool list. Do not assume extra tools are available.
- Do not duplicate repository policy in this file. Add shared guidance to `AGENTS.md` or `docs/agents/AGENT_HARNESS.md` instead.
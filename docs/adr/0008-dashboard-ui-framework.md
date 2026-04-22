# ADR-0008: Dashboard UI Framework

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §21, §28.4

## Context
The dashboard is a separately containerized web UI (PRD §21). It needs server-side rendering for fast initial paint, strong typing, mature charting, and a UI test framework capable of E2E coverage (PRD §28.4).

## Decision
- **Framework:** Next.js (App Router) with TypeScript.
- **UI primitives:** Tailwind CSS + shadcn/ui.
- **Charts:** Recharts (sufficient for the documented metrics; revisit only if a need for >100k point series emerges).
- **Data fetching:** React Server Components against the ingestion service's read API; TanStack Query for client-side mutations.
- **E2E tests:** Playwright with traces and accessibility assertions per critical flow.
- **Component tests:** Vitest + React Testing Library.

## Alternatives Considered
- **Remix** — comparable, smaller ecosystem for shadcn-style component libraries today.
- **SvelteKit** — fewer enterprise contributors familiar with it.
- **Cypress** — still strong, but Playwright wins on parallelism, browser coverage, and trace UX.

## Consequences
- Positive: large hiring pool, strong typing, fast iteration, mature E2E story.
- Negative: Next.js is a moving target; lock to LTS-equivalent versions in `package.json` and document upgrade cadence.
- Follow-ups: define dashboard data contract in `docs/architecture/data-model.md`.

## References
- PRD §21 Dashboard and Admin UX
- PRD §28.4 UI Test Framework

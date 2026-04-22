# Plan 10 — Dashboard

- **Status:** Active
- **Owner:** SDK Engineer (frontend) + Tech Lead
- **PRD:** §21, §28.4, §12.6
- **ADR:** ADR-0008

## Goal
Ship a Next.js dashboard container that renders summary metrics, trends, leaderboards, drill-downs, queue/job views, and the admin console — backed by the ingestion service read APIs.

## Checklist

### Scaffolding
- [ ] `dashboard/` Next.js (App Router) + TypeScript + Tailwind + shadcn/ui.
- [ ] Container image with non-root user; HSTS; CSP locked down.
- [ ] Auth integration (session-based; OIDC pluggable).

### User dashboard pages
- [ ] Overview: total tasks, HEE (min/h/d), FEC, confidence distribution.
- [ ] Trends: time-series with date-range filter.
- [ ] Leaderboards: by user, team, project, framework, category.
- [ ] Event drill-down with explainability trace view.
- [ ] Queue health: depth, age, throughput, retries, DLQ.

### Admin console
- [ ] Tier management.
- [ ] Scoring profile management.
- [ ] Rate management with effective-date editing.
- [ ] Audit log viewer.
- [ ] Calibration workflows scaffold.
- [ ] Workspace/project overrides.
- [ ] RBAC-gated actions.

### UX guardrails (PRD §21)
- [ ] Every financial number labeled "labor-equivalent estimate".
- [ ] Confidence badge visible on every score.

### Tests (PRD §28.4)
- [ ] Vitest + RTL unit tests for components.
- [ ] Playwright E2E for: summary render, filter+drill-down, explainability load, settings persistence, RBAC denial.
- [ ] Accessibility assertions in Playwright (axe).
- [ ] Visual regression for the overview and explainability pages.

### Docs
- [ ] `dashboard/README.md`.
- [ ] `docs/architecture/dashboard.md` with data contract.

## Acceptance
- All E2E flows green in CI.
- Container image signed and scanned.

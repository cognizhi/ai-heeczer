# Plan 10 — Dashboard

- **Status:** Active
- **Owner:** SDK Engineer (frontend) + Tech Lead
- **PRD:** §21, §28.4, §12.6
- **ADR:** ADR-0008

## Goal

Ship a Next.js dashboard container that renders summary metrics, trends, leaderboards, drill-downs, queue/job views, and the admin console — backed by the ingestion service read APIs.

## Checklist

### Scaffolding

- [x] `dashboard/` Next.js 15 (App Router) + TypeScript + Tailwind CSS.
- [x] Security headers in `next.config.ts` (HSTS, CSP, X-Frame-Options, X-Content-Type-Options).
- [ ] Container image with non-root user; HSTS; CSP locked down.
- [ ] Auth integration (session-based; OIDC pluggable).

### User dashboard pages

- [x] Overview: total tasks, HEE (min/h/d), FEC, confidence distribution (scaffold).
- [ ] Trends: time-series with date-range filter.
- [ ] Leaderboards: by user, team, project, framework, category.
- [ ] Event drill-down with explainability trace view.
- [ ] Queue health: depth, age, throughput, retries, DLQ.

### Admin console

- [x] `/test-orchestration` view: fixture browser → pipeline runner → golden diff (ADR-0012 scaffold).
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

- [x] Vitest + RTL unit tests for components — 9/9 pass (ConfidenceBadge, MetricCard, FixtureBrowser).
- [x] Playwright E2E skeleton: overview heading, test-orchestration page, Run button state.
- [ ] Full Playwright E2E for: summary render, filter+drill-down, explainability load, settings persistence, RBAC denial.
- [ ] Accessibility assertions in Playwright (axe).
- [ ] Visual regression for the overview and explainability pages.

### Docs

- [x] `dashboard/README.md`.
- [ ] `docs/architecture/dashboard.md` with data contract.

## Acceptance

- All E2E flows green in CI.
- Container image signed and scanned.

# Plan 10 — Dashboard

- **Status:** Active
- **Owner:** SDK Engineer (frontend) + Tech Lead
- **Last reviewed:** 2026-04-26
- **PRD:** §21, §28.4, §12.6
- **ADR:** ADR-0008

## Goal

Ship a Next.js dashboard container that renders summary metrics, trends, leaderboards, drill-downs, queue/job views, and the admin console — backed by the ingestion service read APIs.

## Checklist

### Scaffolding

- [x] `dashboard/` Next.js 15 (App Router) + TypeScript + Tailwind CSS.
- [x] Security headers in `next.config.ts` (HSTS, CSP, X-Frame-Options, X-Content-Type-Options).
- [x] Container image with non-root user; HSTS; CSP locked down. `dashboard/Dockerfile` builds the standalone Next.js app and runs as `nextjs`; headers remain in `next.config.ts`.
- [x] Auth integration scaffold (session-based; OIDC pluggable). `getDashboardSession()` reads local role/user env and switches provider metadata when `HEECZER_OIDC_ISSUER` is set; full production login/logout and token validation remain owned by plan 0014.

### User dashboard pages

- [x] Overview: total tasks, HEE (min/h/d), FEC, confidence distribution (scaffold).
- [x] Trends: time-series with date-range filter.
- [x] Leaderboards: by user, team, project, framework, category.
- [x] Event drill-down with explainability trace view.
- [x] Queue health: depth, age, throughput, retries, DLQ.

### Admin console

- [x] `/test-orchestration` view: fixture browser → pipeline runner → golden diff (ADR-0012 scaffold).
- [x] Tier management UI scaffold.
- [x] Scoring profile management UI scaffold.
- [x] Rate management UI scaffold with effective-date editing placeholder.
- [x] Audit log viewer UI scaffold.
- [x] Calibration workflows scaffold.
- [x] Workspace/project overrides UI scaffold.
- [x] RBAC-gated actions. Non-admin users see disabled actions and an explicit denial state; write endpoints remain backend-bound.

### UX guardrails (PRD §21)

- [x] Every financial number labeled "labor-equivalent estimate".
- [x] Confidence badge visible on every score.

### Tests (PRD §28.4)

- [x] Vitest + RTL unit tests for components — 9/9 pass (ConfidenceBadge, MetricCard, FixtureBrowser).
- [x] Playwright E2E skeleton: overview heading, test-orchestration page, Run button state.
- [x] Full Playwright E2E for: summary render, filter+drill-down, explainability load, settings persistence, RBAC denial. Also covers queue health and test-orchestration selection state.
- [x] Accessibility assertions in Playwright (axe). Overview route includes an axe smoke assertion in `dashboard/e2e/overview.spec.ts` and now passes against the production bundle.
- [x] Visual regression for the overview and explainability pages. Linux Chromium baselines live under `dashboard/e2e/overview.spec.ts-snapshots/` and are exercised in Playwright.

### Docs

- [x] `dashboard/README.md`.
- [x] `docs/architecture/dashboard.md` with data contract. (session Cat-3)

## Acceptance

- All E2E flows green in CI.
- Container image signed and scanned.

# ai-heeczer Dashboard

Next.js 15 (App Router) + TypeScript + Tailwind CSS dashboard for ai-heeczer. ADR-0008.

## Stack

| Layer        | Choice                         |
| ------------ | ------------------------------ |
| Framework    | Next.js 15, App Router         |
| Language     | TypeScript 5 strict            |
| Styles       | Tailwind CSS 3                 |
| Charts       | Recharts                       |
| Client state | TanStack Query                 |
| Unit tests   | Vitest + React Testing Library |
| E2E tests    | Playwright                     |

## Development

```bash
npm install
npm run dev        # http://localhost:3000
npm test           # vitest (unit)
npm run typecheck  # tsc --noEmit
npm run test:e2e   # playwright (needs running server)
```

## Pages

| Route                 | Description                                                   |
| --------------------- | ------------------------------------------------------------- |
| `/`                   | Overview — total tasks, HEE, FEC, confidence distribution     |
| `/trends`             | Time-series with date-range filter and event drill-down links |
| `/leaderboards`       | Rankings by user, team, project, framework, and category      |
| `/events/[eventId]`   | Score detail with explainability trace                        |
| `/queue`              | Queue depth, age, throughput, retries, and DLQ status         |
| `/admin`              | RBAC-gated admin console scaffold                             |
| `/test-orchestration` | Fixture browser → pipeline runner → golden diff (ADR-0012)    |
| `/settings`           | Local dashboard settings persistence                          |

## Environment variables

| Variable                 | Default                 | Description                                           |
| ------------------------ | ----------------------- | ----------------------------------------------------- |
| `NEXT_PUBLIC_INGEST_URL` | `http://localhost:8080` | Base URL of the ingestion service                     |
| `HEECZER_DASHBOARD_ROLE` | `viewer`                | Local/session role gate (`viewer`, `analyst`, `admin`, `owner`; legacy `tester` maps to `analyst`) |
| `HEECZER_OIDC_ISSUER`    | _(unset)_               | Marks the session provider as OIDC when configured    |

## Security

Security headers (HSTS, CSP, X-Frame-Options, etc.) are set in `next.config.ts` on every response. See plan 0010 and ADR-0008.

`Dockerfile` builds a production Next.js standalone image and runs it as a
non-root `nextjs` user. `Dockerfile.dev` is for local hot-reload work.

## PRD references

- PRD §21: Dashboard and Admin UX
- PRD §21.3: Every financial number labeled "labor-equivalent estimate"
- PRD §21.3: Confidence badge visible on every score
- ADR-0008: Next.js App Router decision
- ADR-0012: Test orchestration view

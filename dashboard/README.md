# ai-heeczer Dashboard

Next.js 15 (App Router) + TypeScript + Tailwind CSS dashboard for ai-heeczer. ADR-0008.

## Stack

| Layer | Choice |
|---|---|
| Framework | Next.js 15, App Router |
| Language | TypeScript 5 strict |
| Styles | Tailwind CSS 3 |
| Charts | Recharts |
| Client state | TanStack Query |
| Unit tests | Vitest + React Testing Library |
| E2E tests | Playwright |

## Development

```bash
npm install
npm run dev        # http://localhost:3000
npm test           # vitest (unit)
npm run typecheck  # tsc --noEmit
npm run test:e2e   # playwright (needs running server)
```

## Pages

| Route | Description |
|---|---|
| `/` | Overview — total tasks, HEE, FEC, confidence distribution |
| `/test-orchestration` | Fixture browser → pipeline runner → golden diff (ADR-0012) |

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `NEXT_PUBLIC_INGEST_URL` | `http://localhost:8080` | Base URL of the ingestion service |

## Security

Security headers (HSTS, CSP, X-Frame-Options, etc.) are set in `next.config.ts` on every response. See plan 0010 and ADR-0008.

## PRD references

- PRD §21: Dashboard and Admin UX
- PRD §21.3: Every financial number labeled "labor-equivalent estimate"
- PRD §21.3: Confidence badge visible on every score
- ADR-0008: Next.js App Router decision
- ADR-0012: Test orchestration view

# Dashboard Architecture

The ai-heeczer dashboard (plan 0010) is a Next.js 15 App Router application
that provides real-time visibility into effort-estimation scores and
workspace activity.

## Technology stack

| Layer | Technology | Notes |
|---|---|---|
| Framework | Next.js 15 App Router | React Server Components + `use client` islands |
| Styling | Tailwind CSS | `tailwind.config.ts` |
| Testing (unit) | Vitest | `vitest.config.ts`, runs in jsdom |
| Testing (E2E) | Playwright | `playwright.config.ts`, requires a running Next.js server |
| Package manager | pnpm | `pnpm-lock.yaml` |

## Data contract

The dashboard communicates with the ingestion service via the same
`envelope_version=1` JSON contract used by all language SDKs (ADR-0011).

### Reading scored events

The dashboard calls the ingestion service REST API. No separate BFF
(Backend For Frontend) is planned; Next.js Server Components act as the
data-fetching layer, keeping API keys out of the browser bundle.

```text
Browser (React)
  └── Next.js Server Component (RSC)
        └── fetch("http://ingest:8080/…", { next: { revalidate: 30 } })
              └── heeczer-ingest
                    └── SQLite / PostgreSQL
```text
### Key API endpoints consumed

| Endpoint | Usage |
|---|---|
| `GET /healthz` | Service health indicator in the header bar. |
| `GET /v1/version` | Displayed in the footer as the current engine version. |
| `POST /v1/events` (via SDK) | Future: submit ad-hoc events from the UI. |

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `NEXT_PUBLIC_INGEST_URL` | `http://localhost:8080` | Base URL of `heeczer-ingest`. Set server-side only. |
| `HEECZER_API_KEY` | _(unset)_ | Sent as `x-heeczer-api-key`. Never exposed to the browser. |

## Directory structure

```text
dashboard/
  src/
    app/          # Next.js App Router pages and layouts
    components/   # React components (both RSC and client islands)
    test/         # Shared test fixtures and helpers
  e2e/            # Playwright end-to-end tests
```text
## Running locally

```bash
# Install dependencies
pnpm install

# Development server (hot reload)
pnpm dev

# Production build
pnpm build && pnpm start

# Unit tests
pnpm test

# E2E tests (requires running dev server on :3000)
pnpm exec playwright test
```text
## Testing strategy

See [ADR-0012](../adr/0012-dashboard-test-orchestration.md) for the full
testing strategy. Summary:

- **Vitest** (jsdom): pure component logic, hooks, utilities.
- **Playwright**: full browser E2E against the built app + a live
  `heeczer-ingest` instance (SQLite, in CI).

## Integration with CI

The `integration` GitHub Actions workflow runs:
1. `cargo build -p heeczer-ingest` to produce the service binary.
2. Starts the service in the background.
3. Runs `pnpm exec playwright test` against it.

See `.github/workflows/integration.yml`.

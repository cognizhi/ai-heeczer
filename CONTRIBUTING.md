# Contributing to ai-heeczer

Thank you for your interest in ai-heeczer. This document is the canonical guide for contributing code, tests, docs, and ideas.

By participating you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Table of contents

1. [Quick start](#quick-start)
2. [Project ground rules](#project-ground-rules)
3. [Development loop](#development-loop)
4. [Project layout](#project-layout)
5. [Coding standards](#coding-standards)
6. [Testing requirements](#testing-requirements)
7. [Documentation requirements](#documentation-requirements)
8. [Commit, branch, and PR conventions](#commit-branch-and-pr-conventions)
9. [Release process](#release-process)
10. [Security disclosures](#security-disclosures)
11. [License and DCO](#license-and-dco)
12. [Getting help](#getting-help)

---

## Quick start

```bash
git clone https://github.com/<org>/ai-heeczer.git
cd ai-heeczer
make bootstrap          # install all toolchains and deps
make test               # run all required tests
make example-quickstart # run a sample event end-to-end
```

If `make bootstrap` is not green on a fresh clone, that is a P1 bug — please open an issue.

Toolchains we pin:

- Rust (see `rust-toolchain.toml`)
- Node.js (see `.nvmrc`)
- Python (see `.python-version`)
- Go (see `go.mod`)
- Java JDK 22+ (see `pom.xml` properties)
- Docker (latest stable)

---

## Project ground rules

These are not negotiable; please read them before opening a PR.

1. **PRD and ADRs are the contract.** Read `docs/prd/ai-heeczer-prd.md` and `docs/adr/*.md` before changing anything non-trivial.
2. **TDD is the default.** Write the failing test first.
3. **Scoring is deterministic and versioned.** Any change that can alter persisted decimal output bumps `scoring_version`, updates golden fixtures, and passes parity tests across every binding (ADR-0003).
4. **Privacy by default.** Never store prompt content, model output, file attachments, secrets, or access tokens.
5. **Cross-language parity.** SDKs delegate to the Rust core (ADR-0001). Do not re-implement scoring.
6. **Docs travel with code.** README, architecture docs, ADRs, and plans must update in the same PR as the behavior they describe.
7. **CI is the truth.** A change is "done" only when all required CI jobs are green and a reviewer has approved.

The full agent harness is in [`docs/agents/AGENT_HARNESS.md`](docs/agents/AGENT_HARNESS.md). Human contributors are expected to follow it too.

---

## Development loop

For every behavior change:

1. **Find the PRD section and ADR(s)** the change touches. Cite them in the PR description.
2. **Pick a plan item** under `docs/plan/` (or open one). Mark it in-progress.
3. **Update contracts first** if your change touches a schema, API, ABI, scoring math, or public SDK surface.
4. **Write the failing test** at the right layer (see [Testing requirements](#testing-requirements)).
5. **Implement** the minimum change to make it pass.
6. `make test` locally.
7. **Update docs** — README, architecture, ADR, plan checklist, CHANGELOG, release impact note (where applicable).
8. **Open a PR** with conventional-commit title.
9. **Mark the plan checklist item done** when the PR is merged.

---

## Project layout

See PRD §35 for the canonical structure. Key directories:

- `core/` — Rust scoring engine (the source of truth for all scoring).
- `bindings/` — `rust/`, `node/`, `python/`, `go/`, `java/` SDKs over the core.
- `server/` — ingestion service (Rust, axum, sqlx).
- `dashboard/` — Next.js dashboard.
- `examples/` — runnable examples per ecosystem.
- `docs/prd/` — Product Requirements Document.
- `docs/adr/` — Architectural Decision Records.
- `docs/architecture/` — system documentation.
- `docs/plan/` — actionable implementation plans with checklists.
- `docs/agents/` — agent harness and process docs.
- `.github/agents/` — Copilot subagent definitions.
- `.github/workflows/` — CI/CD pipelines.

---

## Coding standards

| Language | Formatter | Linter | Notes |
| --- | --- | --- | --- |
| Rust | `cargo fmt` | `cargo clippy --all-targets --all-features -- -D warnings` | No `unwrap`/`panic` on untrusted input |
| TypeScript | `prettier` | `eslint` | Strict mode; no `any` without justification |
| Python | `ruff format` | `ruff check`, `mypy --strict` | abi3 wheels for the SDK |
| Go | `gofmt` | `golangci-lint` | `go vet ./...` clean |
| Java | `google-java-format` | `checkstyle`, `spotbugs` | JDK 22+ |
| SQL | `sqlfluff` | — | Portable subset; dialect-specific files where divergent |
| Markdown | `prettier` | `markdownlint` | Sentence-case headings |

All formatters and linters are wired into `make format` and `make lint`. CI rejects diffs that fail either.

---

## Testing requirements

See `docs/implementation/test-strategy.md` for the full strategy. Required test layers (PRD §28.2):

1. **Unit** — deterministic logic; lives next to the code.
2. **Integration** — multi-component flows.
3. **Contract** — HTTP API shapes, SDK public APIs, schema validation.
4. **Golden fixture** — exact persisted decimal outputs from `core/schema/fixtures/`.
5. **Cross-language parity** — every binding consumes the same fixtures.
6. **UI** — Playwright E2E for dashboard critical flows.
7. **Migration** — every schema migration runs on SQLite and PostgreSQL.
8. **Benchmark smoke** — `track()` p95, ack p95, enqueue throughput.
9. **Release pipeline** — `release-please` dry-run.

A PR cannot be merged unless every layer it affects has a passing CI job.

---

## Documentation requirements

If your change affects user-visible or contributor-visible behavior, you must update:

- the root `README.md` quickstart and feature list when relevant
- the per-binding `README.md` for SDK changes
- `docs/architecture/*` for system shape changes
- a new ADR for any architectural decision
- the relevant `docs/plan/*.md` checklist
- the per-package `CHANGELOG.md` for SDK changes
- the OpenAPI / schema files when API or schema changes

The Tech Writer subagent (`.github/agents/tech-writer.md`) reviews every doc PR.

---

## Commit, branch, and PR conventions

### Branches

- `main` is protected; direct pushes are not allowed.
- Feature branches: `feat/<short-slug>` or `fix/<short-slug>`.
- Release branches are managed by `release-please` (do not hand-create).

### Commits

Conventional Commits are required:

```text
feat(scoring): add risk-class capping for high-risk telemetry
fix(server): correct retry backoff for transient PG conflicts
docs(adr): add ADR-0010 for OTel bridge
chore(deps): bump axum to 0.7.5
```

Allowed types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `revert`. Add `!` (e.g., `feat!:`) or a `BREAKING CHANGE:` footer for breaking changes — these drive a major version bump (PRD §27.3).

### PRs

PR description must include:

1. **What and why** in two or three sentences.
2. **PRD/ADR references** — cite section numbers.
3. **Plan checklist link** — the item this PR completes.
4. **Test layers added/updated.**
5. **Doc updates** — list every file.
6. **Release impact** — none / patch / minor / major / breaking.
7. **Screenshots** for UI changes.

PRs require at least one approving review and all required CI jobs green.

---

## Release process

Releases are automated by `release-please` in manifest mode (ADR-0009, PRD §27).

- Conventional-commit titles drive version computation.
- A single `release-please` PR aggregates the changelog and version bump.
- Merging the release PR triggers signed, attested publishing to npm, PyPI, crates.io, Maven Central, Go module tags, GitHub Releases, and container registries.
- All artifacts share the same product version.
- The release manifest at `.github/release-manifest.json` is the authoritative status; release completes only when every required target reports success.
- Partial failures resume the same version via the `release-resume` workflow — never bump.

The Release Engineer subagent (`.github/agents/release-engineer.md`) owns this process.

---

## Security disclosures

**Do not** open public issues for security vulnerabilities. Email security disclosures per [`SECURITY.md`](SECURITY.md). We follow a coordinated disclosure timeline.

---

## License and DCO

This project is MIT-licensed (see [`LICENSE`](LICENSE)). By contributing, you agree your contributions are licensed under MIT.

We use the Developer Certificate of Origin (DCO). Sign off every commit:

```bash
git commit -s -m "feat(...): ..."
```

Your sign-off line appears as `Signed-off-by: Your Name <you@example.com>`. CI enforces DCO.

---

## Getting help

- **Discussions:** GitHub Discussions for questions and design conversations.
- **Issues:** bug reports and feature requests.
- **Chat:** see the project README for any Slack/Discord links.
- **Tech Lead handle:** `@tech-lead` in PRs and issues.

We aim to respond to first contributions within five business days. Thank you for helping make ai-heeczer better.

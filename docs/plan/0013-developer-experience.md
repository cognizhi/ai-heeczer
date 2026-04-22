# Plan 13 — Developer experience

- **Status:** Active
- **Owner:** DevEx Engineer
- **PRD:** §26, §12.13, §12.21
- **ADR:** ADR-0007, ADR-0010

## Goal
Make the repo trivially easy to clone, bootstrap, test, and contribute to.

## Checklist

### Makefile (PRD §12.13)
- [ ] Targets: `bootstrap`, `format`, `lint`, `unit-test`, `integration-test`, `contract-test`, `parity-test`, `ui-test`, `migration-test`, `benchmark-smoke`, `test`, `build`, `release-dry-run`, `docs`, `clean`.
- [ ] Help target `make help` lists every target with one-line description.
- [ ] Idempotent `bootstrap` detects existing toolchains.

### Toolchain pinning
- [ ] `rust-toolchain.toml`.
- [ ] `.nvmrc`.
- [ ] `.python-version` + `uv.lock`.
- [ ] `go.work` + `go.mod` versions aligned.
- [ ] Maven properties pinned.

### Examples
- [ ] `examples/quickstart/` (cross-language).
- [ ] `examples/node/`, `examples/python/`, `examples/go/`, `examples/rust/`, `examples/java/`.
- [ ] `examples/langgraph/`, `examples/google-adk/`.
- [ ] Each example wired to `make example-<name>`.
- [ ] Examples smoke-tested in CI.

### README
- [ ] Comprehensive root README satisfying every bullet in PRD §12.12. Do not duplicate PRD content; link to it for normative text.
- [ ] CI badge, release badge, license badge, security badge.

### Local containers
- [ ] `docker-compose.dev.yml` brings up ingestion service + PostgreSQL + dashboard for local dev.

### Local CLI (`aih`, ADR-0010)
- [ ] `make cli-install` builds and installs `aih` into `~/.cargo/bin`.
- [ ] `make cli-smoke` runs `aih schema validate`, `aih score`, and `aih diff` against shipped fixtures and exits non-zero on any drift.
- [ ] README quickstart includes `aih score examples/event.json` as the first thing a contributor runs.

### Editor configs
- [ ] `.editorconfig`.
- [ ] `.vscode/extensions.json` recommendations.
- [ ] `.devcontainer/` for codespaces.

## Acceptance
- `make bootstrap && make test` works on a clean clone in a Linux container.
- DevEx smoke job in CI runs the same.

# Plan 13 — Developer experience

- **Status:** Active
- **Owner:** DevEx Engineer
- **PRD:** §26, §12.13, §12.21
- **ADR:** ADR-0007, ADR-0010

## Goal
Make the repo trivially easy to clone, bootstrap, test, and contribute to.

## Checklist

### Makefile (PRD §12.13)
- [x] Targets: `bootstrap`, `format`, `lint`, `unit-test`, `integration-test`, `contract-test`, `parity-test`, `ui-test`, `migration-test`, `benchmark-smoke`, `test`, `build`, `release-dry-run`, `docs`, `clean`. (PR #1)
- [x] Help target `make help` lists every target with one-line description. (PR #1)
- [x] Idempotent `bootstrap` detects existing toolchains, refreshes Rust stable, and installs `cargo-audit` / `cargo-deny` when missing.
- [x] `make security-ci` mirrors the Rust security CI jobs by fresh-installing `cargo-audit` / `cargo-deny` into a temp root before running the scans.

### Toolchain pinning
- [x] `rust-toolchain.toml` tracks the current Rust stable channel.
- [x] `.nvmrc`. (PR #1)
- [ ] `.python-version` + `uv.lock`. (partial: `.python-version` added PR #1; `uv.lock` pending Python SDK work)
- [ ] `go.work` + `go.mod` versions aligned.
- [ ] Maven properties pinned.

### Examples
- [x] `examples/` cross-language quickstarts indexed by [`examples/README.md`](../../examples/README.md). Every per-language quickstart submits the same canonical [`examples/event.json`](../../examples/event.json) for an apples-to-apples comparison. (Original bullet read `examples/quickstart/`; the shipped layout puts the index at `examples/README.md` instead of a `quickstart/` subdir, so per-language quickstarts can sit at `examples/<lang>/`.)
- [x] `examples/node/`, `examples/python/`, `examples/go/`, `examples/java/` plus the in-tree Rust example at `bindings/heeczer-rs/examples/quickstart.rs` (`cargo run -p heeczer --example quickstart`).
- [ ] `examples/langgraph/`, `examples/google-adk/`. (pending plan 0011 framework adapters)
- [ ] Each example wired to `make example-<name>`.
- [ ] Examples smoke-tested in CI.

### README
- [ ] Comprehensive root README satisfying every bullet in PRD §12.12. Do not duplicate PRD content; link to it for normative text.
- [ ] CI badge, release badge, license badge, security badge.

### Local containers
- [ ] `docker-compose.dev.yml` brings up ingestion service + PostgreSQL + dashboard for local dev.
  Per-SDK contributor sandboxes (Node/Python/PydanticAI/Go/Java/Rust chatbot
  stacks with persisted DB, dashboard, and pluggable LLM provider) are designed
  in [plan 0016](0016-local-sdk-test-stacks.md). This bullet stays scoped to the
  minimal `docker-compose.dev.yml` for everyday core development.

### Local CLI (`heec`, ADR-0010)
- [x] `make cli-install` builds and installs `heec` into `~/.cargo/bin`. (PR #1)
- [x] `make cli-smoke` runs `heec schema validate`, `heec score`, and `heec diff` against shipped fixtures and exits non-zero on any drift. (PR #1)
- [x] `heec fixtures list` + `heec fixtures show <NAME>` walk the embedded fixture tree (`include_dir!`). (commit 13d75f1)
- [ ] README quickstart includes `heec score examples/event.json` as the first thing a contributor runs.

### Editor configs
- [ ] `.editorconfig`.
- [ ] `.vscode/extensions.json` recommendations.
- [ ] `.devcontainer/` for codespaces.

## Acceptance
- `make bootstrap && make test` works on a clean clone in a Linux container.
- DevEx smoke job in CI runs the same.

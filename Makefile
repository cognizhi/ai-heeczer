SHELL := /usr/bin/env bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := help

# ai-heeczer Makefile. Single human-facing command surface (PRD §12.13, ADR-0007).
# Every target listed here is documented and CI-invokable.

# ----- meta ------------------------------------------------------------------

.PHONY: help
help: ## list every target with a one-line description
	@awk 'BEGIN{FS=":.*##"; printf "Available targets:\n\n"} /^[a-zA-Z0-9._-]+:.*##/ {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----- bootstrap -------------------------------------------------------------

.PHONY: bootstrap
bootstrap: ## install/verify language toolchains and pre-commit hooks
	@echo "» rust toolchain (rust-toolchain.toml drives rustup)"
	@command -v rustup >/dev/null || { echo "install rustup from https://rustup.rs"; exit 1; }
	@rustup show active-toolchain >/dev/null
	@echo "» node (.nvmrc)"
	@command -v node >/dev/null || { echo "install Node 22 LTS"; exit 1; }
	@command -v pnpm >/dev/null || { echo "install pnpm: corepack enable"; exit 1; }
	@echo "» python via uv"
	@command -v uv >/dev/null || { echo "install uv: https://docs.astral.sh/uv/"; exit 1; }
	@echo "» go"
	@command -v go >/dev/null || { echo "install Go 1.24+"; exit 1; }
	@echo "» pre-commit (optional but recommended)"
	@command -v pre-commit >/dev/null && pre-commit install || echo "  pre-commit not installed; skipping hook install"
	@echo "bootstrap ok"

# ----- format / lint ---------------------------------------------------------

.PHONY: format
format: ## auto-format all sources
	cargo fmt --all

.PHONY: format-check
format-check: ## verify formatting without writing
	cargo fmt --all -- --check

.PHONY: lint
lint: ## run all linters
	cargo clippy --workspace --all-targets -- -D warnings

# ----- tests -----------------------------------------------------------------

.PHONY: unit-test
unit-test: ## fast unit tests (Rust)
	cargo test --workspace --lib --bins

.PHONY: integration-test
integration-test: ## integration tests
	cargo test --workspace --tests

.PHONY: contract-test
contract-test: ## schema and CLI contract tests
	cargo test --workspace --test schema_validation --test golden_scoring

.PHONY: parity-test
parity-test: ## cross-language parity (will run SDK matrices once SDKs land)
	@echo "parity-test: SDK matrices land in plans 0005-0009"

.PHONY: migration-test
migration-test: ## storage migration tests on every supported backend
	cargo test -p heeczer-storage

.PHONY: ui-test
ui-test: ## dashboard end-to-end tests (lands with plan 0010)
	@echo "ui-test: lands with plan 0010 dashboard"

.PHONY: benchmark-smoke
benchmark-smoke: ## smoke-run criterion benchmarks (PRD §29)
	@echo "benchmark-smoke: lands with plan 0015"

.PHONY: test
test: format-check lint unit-test integration-test contract-test migration-test ## full local test gate

# ----- build / release -------------------------------------------------------

.PHONY: build
build: ## build the entire Rust workspace in release mode
	cargo build --workspace --release

.PHONY: release-dry-run
release-dry-run: ## release-please manifest dry-run (lands with plan 0012)
	@echo "release-dry-run: implemented in plan 0012 CI workflow"

# ----- CLI (ADR-0010) --------------------------------------------------------

.PHONY: cli-install
cli-install: ## install the heec CLI to ~/.cargo/bin
	cargo install --path core/heeczer-cli --locked

.PHONY: cli-smoke
cli-smoke: build ## end-to-end smoke of the heec CLI against shipped fixtures
	./target/release/heec version
	./target/release/heec schema validate core/schema/fixtures/events/valid/01-prd-canonical.json
	./target/release/heec score core/schema/fixtures/events/valid/01-prd-canonical.json --format json > /tmp/heec-score.json
	./target/release/heec diff /tmp/heec-score.json /tmp/heec-score.json
	./target/release/heec migrate up --database-url sqlite::memory:

# ----- security --------------------------------------------------------------

.PHONY: security
security: security-audit security-licenses ## run all local security scans

.PHONY: security-audit
security-audit: ## cargo-audit dependency vulnerability scan
	@command -v cargo-audit >/dev/null || cargo install cargo-audit --locked
	cargo audit

.PHONY: security-licenses
security-licenses: ## cargo-deny license + advisories
	@command -v cargo-deny >/dev/null || cargo install cargo-deny --locked
	cargo deny check

# ----- docs -----------------------------------------------------------------

.PHONY: docs
docs: ## generate rustdoc
	cargo doc --workspace --no-deps

# ----- housekeeping ---------------------------------------------------------

.PHONY: clean
clean: ## remove build outputs
	cargo clean
	rm -rf node_modules dist .pnpm-store

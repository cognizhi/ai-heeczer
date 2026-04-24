#!/usr/bin/env bash
# post-create.sh — run once after the devcontainer image is built.
set -euo pipefail

echo "==> Installing pnpm"
npm install -g pnpm@9

echo "==> Installing uv (Python package manager)"
curl -LsSf https://astral.sh/uv/install.sh | sh
# Make uv available in PATH for subsequent steps
export PATH="$HOME/.local/bin:$PATH"

echo "==> Installing govulncheck"
go install golang.org/x/vuln/cmd/govulncheck@latest

echo "==> Installing cargo tools"
cargo install cargo-deny --locked
cargo install cargo-audit --locked
cargo install cargo-nextest --locked

echo "==> Syncing Python dev dependencies"
cd bindings/heeczer-py && uv sync --all-extras && cd ../..

echo "==> Installing JS dependencies"
cd bindings/heeczer-js && pnpm install --frozen-lockfile && cd ../..

echo "==> Installing dashboard dependencies"
cd dashboard && pnpm install --frozen-lockfile && cd ..

echo "==> Done. Run 'make help' to see available targets."

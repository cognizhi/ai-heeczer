#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
reference_dir="$(mktemp -d)"
database_dir="$(mktemp -d)"
database_path="${database_dir}/heeczer.db"
port="${HEECZER_PARITY_PORT:-18080}"
base_url="http://127.0.0.1:${port}"
service_log="${reference_dir}/heeczer-ingest.log"
service_pid=""

cleanup() {
    if [[ -n "${service_pid}" ]] && kill -0 "${service_pid}" 2>/dev/null; then
        kill "${service_pid}" 2>/dev/null || true
    fi
    rm -rf "${reference_dir}" "${database_dir}"
}
trap cleanup EXIT

cd "${repo_root}"

cargo build -p heeczer-cli -p heeczer-ingest --release

for fixture_path in core/schema/fixtures/events/valid/*.json; do
    fixture_name="$(basename "${fixture_path}" .json)"
    ./target/release/heec score "${fixture_path}" --format json > "${reference_dir}/${fixture_name}.json"
done

HEECZER_LISTEN="127.0.0.1:${port}" \
HEECZER_AUTH__ENABLED=false \
HEECZER_FEATURES__TEST_ORCHESTRATION=true \
HEECZER_DATABASE_URL="sqlite:${database_path}?mode=rwc" \
    ./target/release/heeczer-ingest > "${service_log}" 2>&1 &
service_pid="$!"

for attempt in {1..30}; do
    if curl -fsS "${base_url}/v1/ready" >/dev/null; then
        break
    fi
    if [[ "${attempt}" -eq 30 ]]; then
        cat "${service_log}"
        echo "heeczer-ingest did not become ready" >&2
        exit 1
    fi
    sleep 1
done

export HEECZER_PARITY_BASE_URL="${base_url}"
export HEECZER_PARITY_REFERENCE_DIR="${reference_dir}"
export HEECZER_PARITY_FIXTURE_DIR="${repo_root}/core/schema/fixtures/events/valid"

cargo test -p heeczer --test parity

pushd bindings/heeczer-js >/dev/null
pnpm install --frozen-lockfile
pnpm run build
node scripts/parity.mjs
popd >/dev/null

pushd bindings/heeczer-py >/dev/null
uv sync --all-extras
uv run python scripts/parity.py
popd >/dev/null

pushd bindings/heeczer-go >/dev/null
go run ./cmd/parity
popd >/dev/null

pushd bindings/heeczer-java >/dev/null
mvn -q -Dtest=ParityTest test
popd >/dev/null

echo "All SDK parity checks passed"

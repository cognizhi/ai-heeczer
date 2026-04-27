#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STACKS=(js py pydanticai go java rs)

if [[ -z "$ACTION" ]]; then
  echo "usage: testing/compose/_bin/stack-all.sh <start|stop|reset|smoke>" >&2
  exit 2
fi

if [[ "$ACTION" == "reset" && "${CONFIRM:-}" != "1" ]]; then
  printf 'Type reset-all to drop every local test-stack database volume: ' >&2
  read -r answer
  if [[ "$answer" != "reset-all" ]]; then
    echo "reset-all cancelled" >&2
    exit 1
  fi
  export CONFIRM=1
fi

for sdk in "${STACKS[@]}"; do
  "$SCRIPT_DIR/stack.sh" "$ACTION" "$sdk"
done

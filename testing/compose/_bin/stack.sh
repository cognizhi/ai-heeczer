#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-}"
SDK="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
COMPOSE="docker compose"

usage() {
  cat <<'USAGE'
usage: testing/compose/_bin/stack.sh <start|stop|reset|logs|ps|smoke> <js|py|pydanticai|go|java|rs>
USAGE
}

if [[ -z "$ACTION" || -z "$SDK" ]]; then
  usage
  exit 2
fi

case "$SDK" in
  js) UI_PORT=18000; API_PORT=18001; INGEST_PORT=18010; DASHBOARD_PORT=18020; POSTGRES_PORT=18032; OLLAMA_PORT=18079 ;;
  py) UI_PORT=18100; API_PORT=18101; INGEST_PORT=18110; DASHBOARD_PORT=18120; POSTGRES_PORT=18132; OLLAMA_PORT=18179 ;;
  go) UI_PORT=18200; API_PORT=18201; INGEST_PORT=18210; DASHBOARD_PORT=18220; POSTGRES_PORT=18232; OLLAMA_PORT=18279 ;;
  java) UI_PORT=18300; API_PORT=18301; INGEST_PORT=18310; DASHBOARD_PORT=18320; POSTGRES_PORT=18332; OLLAMA_PORT=18379 ;;
  rs) UI_PORT=18400; API_PORT=18401; INGEST_PORT=18410; DASHBOARD_PORT=18420; POSTGRES_PORT=18432; OLLAMA_PORT=18479 ;;
  pydanticai) UI_PORT=18500; API_PORT=18501; INGEST_PORT=18510; DASHBOARD_PORT=18520; POSTGRES_PORT=18532; OLLAMA_PORT=18579 ;;
  *) echo "unknown SDK stack: $SDK" >&2; usage; exit 2 ;;
esac

STACK_DIR="$REPO_ROOT/testing/compose/$SDK"
ENV_FILE="$STACK_DIR/.env"
ENV_EXAMPLE="$STACK_DIR/.env.example"
if [[ ! -d "$STACK_DIR" ]]; then
  echo "stack not yet implemented: $SDK" >&2
  exit 1
fi

read_env() {
  local key="$1" file="$2"
  grep -E "^[[:space:]]*${key}=" "$file" 2>/dev/null | tail -n 1 | sed -E "s/^[[:space:]]*${key}=//; s/^['\"]//; s/['\"]$//"
}

require_env_key() {
  local key="$1" file="$2"
  local value
  value="$(read_env "$key" "$file")"
  if [[ -z "$value" || "$value" == "changeme" || "$value" == "<changeme>" ]]; then
    echo "missing required $key in $file" >&2
    return 1
  fi
}

validate_env() {
  if [[ ! -f "$ENV_FILE" ]]; then
    echo "missing $ENV_FILE" >&2
    echo "copy $ENV_EXAMPLE to .env and set LLM_PROVIDER=mock for hermetic smoke tests." >&2
    exit 1
  fi
  local provider
  provider="$(read_env LLM_PROVIDER "$ENV_FILE")"
  provider="${provider:-mock}"
  case "$provider" in
    mock) ;;
    openrouter)
      require_env_key OPENROUTER_API_KEY "$ENV_FILE"
      require_env_key OPENROUTER_MODEL "$ENV_FILE"
      ;;
    gemini)
      require_env_key GEMINI_API_KEY "$ENV_FILE"
      require_env_key GEMINI_MODEL "$ENV_FILE"
      ;;
    local)
      require_env_key LOCAL_MODEL "$ENV_FILE"
      ;;
    *) echo "unsupported LLM_PROVIDER=$provider in $ENV_FILE" >&2; exit 1 ;;
  esac
}

compose_env_file() {
  if [[ -f "$ENV_FILE" ]]; then
    printf '%s' "$ENV_FILE"
  elif [[ -f "$ENV_EXAMPLE" ]]; then
    printf '%s' "$ENV_EXAMPLE"
  else
    printf '%s' /dev/null
  fi
}

compose_cmd() {
  local env_file
  env_file="$(compose_env_file)"
  HEECZER_REPO_ROOT="$REPO_ROOT" \
  HEECZER_STACK="$SDK" \
  HEECZER_CHATBOT_UI_PORT="$UI_PORT" \
  HEECZER_CHATBOT_API_PORT="$API_PORT" \
  HEECZER_INGEST_PORT="$INGEST_PORT" \
  HEECZER_DASHBOARD_PORT="$DASHBOARD_PORT" \
  HEECZER_POSTGRES_PORT="$POSTGRES_PORT" \
  HEECZER_OLLAMA_PORT="$OLLAMA_PORT" \
  $COMPOSE --project-name "heeczer-test-$SDK" \
    --env-file "$env_file" \
    -f "$REPO_ROOT/testing/compose/_shared/ingest.yml" \
    -f "$REPO_ROOT/testing/compose/_shared/dashboard.yml" \
    -f "$REPO_ROOT/testing/compose/_shared/ollama.yml" \
    -f "$REPO_ROOT/testing/compose/$SDK/docker-compose.yml" \
    "$@"
}

confirm_reset() {
  if [[ "${CONFIRM:-}" == "1" ]]; then
    return 0
  fi
  local expected="heeczer-test-$SDK" answer
  printf 'Type the stack name to confirm reset (%s): ' "$expected" >&2
  read -r answer
  if [[ "$answer" != "$expected" ]]; then
    echo "reset cancelled" >&2
    exit 1
  fi
}

case "$ACTION" in
  start)
    validate_env
    (cd "$REPO_ROOT" && cargo build -p heeczer-ingest --release)
    if [[ "$(read_env LLM_PROVIDER "$ENV_FILE")" == "local" ]]; then
      compose_cmd --profile local-model up -d --build
    else
      compose_cmd up -d --build
    fi
    ;;
  stop)
    compose_cmd down
    ;;
  reset)
    confirm_reset
    compose_cmd down -v
    ;;
  logs)
    compose_cmd logs -f --tail=200
    ;;
  ps)
    compose_cmd ps
    ;;
  smoke)
    (cd "$REPO_ROOT" && HEECZER_REQUIRE_STACK=1 uv run --with pytest --with httpx pytest "testing/tests/smoke/test_${SDK}_stack.py")
    ;;
  *)
    usage
    exit 2
    ;;
esac

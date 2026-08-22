#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE="$ROOT/docker-compose.yml"
ENV_EXAMPLE="$ROOT/.env.example"

failures=0

fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}

pass() {
  echo "ok: $1"
}

assert_file_contains() {
  local file="$1"
  local needle="$2"
  local name="$3"
  if grep -F -q -- "$needle" "$file"; then
    pass "$name"
  else
    fail "$name: missing '$needle' in $file"
  fi
}

assert_file_contains "$COMPOSE" \
  "traefik.http.routers.timey-api.tls.certresolver=\${TRAEFIK_CERTRESOLVER:-letsencrypt}" \
  "api router binds ACME certresolver"
assert_file_contains "$COMPOSE" \
  "traefik.http.routers.timey-web.tls.certresolver=\${TRAEFIK_CERTRESOLVER:-letsencrypt}" \
  "web router binds ACME certresolver"
assert_file_contains "$ENV_EXAMPLE" \
  "TRAEFIK_CERTRESOLVER=letsencrypt" \
  ".env.example sets TRAEFIK_CERTRESOLVER"

if [ "$failures" -ne 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi

echo "all tests passed"

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/publish-images.yml"

failures=0

fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}

pass() {
  echo "ok: $1"
}

assert_file_exists() {
  local file="$1"
  local name="$2"
  if [ -f "$file" ]; then
    pass "$name"
  else
    fail "$name: missing $file"
  fi
}

assert_file_contains() {
  local file="$1"
  local needle="$2"
  local name="$3"
  if [ -f "$file" ] && grep -F -q -- "$needle" "$file"; then
    pass "$name"
  else
    fail "$name: missing '$needle' in $file"
  fi
}

assert_file_exists "$WORKFLOW" "publish-images workflow exists"
assert_file_contains "$WORKFLOW" "tags:" "workflow runs on git tags"
assert_file_contains "$WORKFLOW" "ghcr.io" "workflow pushes to GHCR"
assert_file_contains "$WORKFLOW" "linux/amd64" "workflow builds linux/amd64"
assert_file_contains "$WORKFLOW" "packages: write" "workflow can write packages"
assert_file_contains "$WORKFLOW" "org.opencontainers.image.source" "workflow sets OCI source label"
assert_file_contains "$WORKFLOW" "https://github.com/LogicException/timey" "OCI source points at the GitHub repo"

if [ "$failures" -ne 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi

echo "all tests passed"

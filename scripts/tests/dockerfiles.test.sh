#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
API_DOCKERFILE="$ROOT/api/Dockerfile"
WEB_DOCKERFILE="$ROOT/web/Dockerfile"
CARGO_TOML="$ROOT/api/Cargo.toml"

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

assert_file_contains "$API_DOCKERFILE" "FROM --platform=\$BUILDPLATFORM rust:1.97-bookworm" "api builder rustc is new enough for Cargo.lock"
assert_file_contains "$API_DOCKERFILE" "x86_64-unknown-linux-gnu" "api cross-compiles to x86_64-unknown-linux-gnu"
assert_file_contains "$API_DOCKERFILE" "target/x86_64-unknown-linux-gnu/release/timey-api" "api copies the x86_64 release binary"
assert_file_contains "$CARGO_TOML" 'features = ["bundled"]' "sqlite is bundled for cross-link"
assert_file_contains "$WEB_DOCKERFILE" "FROM --platform=\$BUILDPLATFORM" "web builder uses BUILDPLATFORM (static assets)"

if [ "$failures" -ne 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi

echo "all tests passed"

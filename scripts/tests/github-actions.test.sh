#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/ci.yml"
GITLAB_CI="$ROOT/.gitlab-ci.yml"

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

assert_file_absent() {
  local file="$1"
  local name="$2"
  if [ ! -f "$file" ]; then
    pass "$name"
  else
    fail "$name: $file should not exist"
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

assert_file_exists "$WORKFLOW" "GitHub Actions workflow exists"
assert_file_absent "$GITLAB_CI" "GitLab CI config is removed"
assert_file_contains "$WORKFLOW" "push" "workflow runs on push"
assert_file_contains "$WORKFLOW" "pull_request" "workflow runs on pull_request"
assert_file_contains "$WORKFLOW" "1.97" "api job uses Rust 1.97"
assert_file_contains "$WORKFLOW" "cargo clippy --all-targets -- -D warnings" "api job runs clippy"
assert_file_contains "$WORKFLOW" "cargo test" "api job runs cargo test"
assert_file_contains "$WORKFLOW" "22" "web job uses Node 22"
assert_file_contains "$WORKFLOW" "npm ci" "web job installs with npm ci"
assert_file_contains "$WORKFLOW" "npm test" "web job runs tests"
assert_file_contains "$WORKFLOW" "npm run check" "web job runs svelte-check"
assert_file_contains "$WORKFLOW" "npm run build" "web job builds frontend"

if [ "$failures" -ne 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi

echo "all tests passed"

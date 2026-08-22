#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LIB="$ROOT/scripts/lib/build-images.sh"
SCRIPT="$ROOT/scripts/build-images.sh"

failures=0

fail() {
  echo "FAIL: $1" >&2
  failures=$((failures + 1))
}

pass() {
  echo "ok: $1"
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local name="$3"
  if [ "$actual" = "$expected" ]; then
    pass "$name"
  else
    fail "$name: expected '$expected', got '$actual'"
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local name="$3"
  if printf '%s' "$haystack" | grep -F -q -- "$needle"; then
    pass "$name"
  else
    fail "$name: missing '$needle' in:\n$haystack"
  fi
}

# shellcheck source=../lib/build-images.sh
source "$LIB"

assert_eq "$PLATFORM" "linux/amd64" "platform is linux/amd64"
assert_eq "$(image_ref api v1.2.3)" "ghcr.io/logicexception/timey-api:v1.2.3" "default api image ref"
assert_eq "$(image_ref web v1.2.3)" "ghcr.io/logicexception/timey-web:v1.2.3" "default web image ref"

got="$(REGISTRY=example.internal image_ref web v9)"
assert_eq "$got" "example.internal/timey-web:v9" "registry override"

set +e
( require_tag "" ) >/tmp/require_tag.out 2>/tmp/require_tag.err
require_tag_status=$?
set -e
if [ "$require_tag_status" -ne 0 ]; then
  pass "require_tag rejects empty tag"
else
  fail "require_tag should fail for empty tag"
fi
assert_contains "$(cat /tmp/require_tag.err)" "usage:" "require_tag prints usage"

set +e
( assert_git_tag "does-not-exist" ) >/tmp/assert_tag.out 2>/tmp/assert_tag.err
assert_tag_status=$?
set -e
if [ "$assert_tag_status" -ne 0 ]; then
  pass "assert_git_tag rejects unknown tag"
else
  fail "assert_git_tag should fail for unknown tag"
fi
assert_contains "$(cat /tmp/assert_tag.err)" "does-not-exist" "unknown tag is named in error"

inspect_ok="$(printf 'Name: demo\nPlatform:          linux/amd64\n')"
if architecture_is_amd64 "$inspect_ok"; then
  pass "architecture_is_amd64 accepts linux/amd64"
else
  fail "architecture_is_amd64 should accept linux/amd64 inspect output"
fi

inspect_arm="$(printf 'Name: demo\nPlatform:          linux/arm64\n')"
if architecture_is_amd64 "$inspect_arm"; then
  fail "architecture_is_amd64 should reject linux/arm64-only inspect output"
else
  pass "architecture_is_amd64 rejects linux/arm64"
fi

make_repo() {
  local dir="$1"
  mkdir -p "$dir/api" "$dir/web"
  printf 'FROM scratch\n' >"$dir/api/Dockerfile"
  printf 'FROM scratch\n' >"$dir/web/Dockerfile"
  git -C "$dir" init -q
  git -C "$dir" config user.email "test@example.com"
  git -C "$dir" config user.name "Test"
  git -C "$dir" add api web
  git -C "$dir" commit -q -m "init"
  git -C "$dir" tag v0.1.0
}

make_docker_mock() {
  local bin_dir="$1"
  mkdir -p "$bin_dir"
  cat >"$bin_dir/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${DOCKER_LOG:?}"
if [ "${1:-}" = "buildx" ] && [ "${2:-}" = "version" ]; then
  echo "github.com/docker/buildx v0.0.0"
  exit 0
fi
if [ "${1:-}" = "buildx" ] && [ "${2:-}" = "imagetools" ]; then
  echo "Platform:          ${MOCK_INSPECT_PLATFORM:-linux/amd64}"
  exit 0
fi
if [ "${1:-}" = "buildx" ] && [ "${2:-}" = "build" ]; then
  exit 0
fi
exit 0
EOF
  chmod +x "$bin_dir/docker"
}

run_script() {
  local repo="$1"
  shift
  (
    cd "$repo"
    export PATH="$MOCK_BIN:$PATH"
    export DOCKER_LOG
    export MOCK_INSPECT_PLATFORM="${MOCK_INSPECT_PLATFORM:-linux/amd64}"
    "$SCRIPT" "$@"
  )
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
REPO="$TMP/repo"
MOCK_BIN="$TMP/bin"
DOCKER_LOG="$TMP/docker.log"
mkdir -p "$REPO"
make_repo "$REPO"
make_docker_mock "$MOCK_BIN"

set +e
run_script "$REPO" >/tmp/no_tag.out 2>/tmp/no_tag.err
no_tag_status=$?
set -e
if [ "$no_tag_status" -ne 0 ]; then
  pass "script requires a tag argument"
else
  fail "script should fail without a tag argument"
fi

set +e
run_script "$REPO" missing-tag >/tmp/missing.out 2>/tmp/missing.err
missing_status=$?
set -e
if [ "$missing_status" -ne 0 ]; then
  pass "script rejects missing git tag"
else
  fail "script should fail for a missing git tag"
fi

: >"$DOCKER_LOG"
if run_script "$REPO" v0.1.0 >/tmp/build.out 2>/tmp/build.err; then
  pass "script succeeds for existing tag"
else
  fail "script should succeed for tag v0.1.0: $(cat /tmp/build.err)"
fi

log="$(cat "$DOCKER_LOG")"
assert_contains "$log" "buildx build --platform linux/amd64" "build uses linux/amd64"
assert_contains "$log" "--push" "build pushes"
assert_contains "$log" "timey-api:v0.1.0" "pushes api image tagged from git tag"
assert_contains "$log" "timey-web:v0.1.0" "pushes web image tagged from git tag"

api_build="$(grep 'buildx build' "$DOCKER_LOG" | grep timey-api || true)"
web_build="$(grep 'buildx build' "$DOCKER_LOG" | grep timey-web || true)"
assert_contains "$api_build" "/api" "api build context is api/"
assert_contains "$web_build" "/web" "web build context is web/"

: >"$DOCKER_LOG"
export MOCK_INSPECT_PLATFORM="linux/arm64"
set +e
run_script "$REPO" v0.1.0 >/tmp/arm.out 2>/tmp/arm.err
arm_status=$?
set -e
unset MOCK_INSPECT_PLATFORM
if [ "$arm_status" -ne 0 ]; then
  pass "script fails when inspect is not amd64"
else
  fail "script should fail when image architecture is not amd64"
fi
assert_contains "$(cat /tmp/arm.err)" "amd64" "non-amd64 failure mentions amd64"

if [ "$failures" -ne 0 ]; then
  echo "$failures test(s) failed" >&2
  exit 1
fi

echo "all tests passed"

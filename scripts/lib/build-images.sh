#!/usr/bin/env bash
set -euo pipefail

PLATFORM="linux/amd64"
DEFAULT_REGISTRY="ghcr.io/logicexception"

die() {
  printf '%s\n' "$1" >&2
  exit 1
}

require_tag() {
  local tag="${1:-}"
  if [ -z "$tag" ]; then
    die "usage: build-images.sh <git-tag>"
  fi
}

registry() {
  printf '%s\n' "${REGISTRY:-$DEFAULT_REGISTRY}"
}

image_ref() {
  local service="$1"
  local tag="$2"
  printf '%s/timey-%s:%s\n' "$(registry)" "$service" "$tag"
}

assert_git_tag() {
  local tag="$1"
  if ! git rev-parse "refs/tags/${tag}" >/dev/null 2>&1; then
    die "git tag not found: ${tag}"
  fi
}

require_docker() {
  if ! command -v docker >/dev/null 2>&1; then
    die "docker not found"
  fi
}

require_buildx() {
  if ! docker buildx version >/dev/null 2>&1; then
    die "docker buildx not found"
  fi
}

architecture_is_amd64() {
  local inspect="$1"
  printf '%s\n' "$inspect" | grep -Eq 'Platform:[[:space:]]+linux/amd64'
}

assert_image_amd64() {
  local image="$1"
  local inspect
  inspect="$(docker buildx imagetools inspect "$image")"
  if ! architecture_is_amd64 "$inspect"; then
    die "image ${image} is not linux/amd64"
  fi
}

build_image() {
  local context="$1"
  local image="$2"
  docker buildx build --platform "$PLATFORM" -t "$image" --push "$context"
}

prepare_tag_worktree() {
  local tag="$1"
  local repo_root="$2"
  local worktree
  worktree="$(mktemp -d "${TMPDIR:-/tmp}/timey-build.XXXXXX")"
  rmdir "$worktree"
  git -C "$repo_root" worktree add --detach "$worktree" "$tag" >/dev/null
  printf '%s\n' "$worktree"
}

remove_tag_worktree() {
  local repo_root="$1"
  local worktree="$2"
  git -C "$repo_root" worktree remove --force "$worktree" >/dev/null 2>&1 || true
  rm -rf "$worktree"
}

main() {
  require_tag "${1:-}"
  local tag="$1"
  require_docker
  require_buildx
  assert_git_tag "$tag"

  local repo_root worktree
  repo_root="$(git rev-parse --show-toplevel)"
  worktree="$(prepare_tag_worktree "$tag" "$repo_root")"
  trap 'remove_tag_worktree "'"$repo_root"'" "'"$worktree"'"' EXIT

  local api_image web_image
  api_image="$(image_ref api "$tag")"
  web_image="$(image_ref web "$tag")"

  build_image "${worktree}/api" "$api_image"
  assert_image_amd64 "$api_image"
  build_image "${worktree}/web" "$web_image"
  assert_image_amd64 "$web_image"
}

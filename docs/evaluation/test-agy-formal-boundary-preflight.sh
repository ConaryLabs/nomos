#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
preflight="$repo_root/docs/evaluation/agy-formal-boundary-preflight.sh"
fake_agy="$repo_root/docs/evaluation/fixtures/fake-agy-formal-boundary"
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT

AGY_BIN="$fake_agy" "$preflight" >"$tmp_dir/pass.txt"
grep -Fx 'AGY_FORMAL_BOUNDARY PASS' "$tmp_dir/pass.txt" >/dev/null

assert_blocked() {
  local mode=$1
  local expected=$2
  if FAKE_AGY_FORMAL_MODE="$mode" AGY_BIN="$fake_agy" \
    "$preflight" >"$tmp_dir/$mode.out" 2>"$tmp_dir/$mode.err"; then
    printf 'expected formal boundary mode %s to be blocked\n' "$mode" >&2
    exit 1
  fi
  grep -F "$expected" "$tmp_dir/$mode.err" >/dev/null
  grep -Fx 'AGY_FORMAL_BOUNDARY BLOCKED' "$tmp_dir/$mode.err" >/dev/null
}

assert_blocked forbidden-tool 'effective tools are not the exact protocol allowlist'
assert_blocked missing-init 'expected exactly one init event, found 0'
assert_blocked wrong-model 'init event does not pin gemini-3.7-flash-high'
assert_blocked wrong-worktree 'init event cwd is not the target worktree'
assert_blocked missing-context 'init event does not prove an empty context-source set'
assert_blocked reused-context 'conversation is not a fresh one-turn session'
assert_blocked tool-call 'neutral preflight unexpectedly executed 1 tool events'

printf 'agy formal boundary harness: PASS\n'

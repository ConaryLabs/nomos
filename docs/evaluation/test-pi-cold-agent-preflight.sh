#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
preflight="$repo_root/docs/evaluation/pi-cold-agent-preflight.sh"
fake_pi="$repo_root/docs/evaluation/fixtures/fake-pi-cold-agent"
fake_bwrap="$repo_root/docs/evaluation/fixtures/fake-bwrap-pi-cold-agent"
tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

for lane in deepseek gemini claude; do
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" "$preflight" "$lane" >"$tmp_dir/pass-$lane.txt"
  grep -Fx 'PI_COLD_AGENT_BOUNDARY PASS' "$tmp_dir/pass-$lane.txt" >/dev/null
done

assert_blocked() {
  local mode=$1
  local expected=$2
  if FAKE_PI_MODE="$mode" PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
    NOMOS_PI_TEST_SECRET='fixture-secret-never-print' \
    "$preflight" deepseek >"$tmp_dir/$mode.out" 2>"$tmp_dir/$mode.err"; then
    printf 'expected Pi boundary mode %s to be blocked\n' "$mode" >&2
    exit 1
  fi
  grep -F "$expected" "$tmp_dir/$mode.err" >/dev/null
}

assert_blocked missing-boundary 'expected one runtime boundary record, found 0'
assert_blocked missing-session 'expected one session header, found 0'
assert_blocked wrong-provider 'runtime boundary record does not prove the required state'
assert_blocked wrong-model 'runtime boundary record does not prove the required state'
assert_blocked wrong-worktree 'runtime boundary record does not prove the required state'
assert_blocked wrong-thinking 'runtime boundary record does not prove the required state'
assert_blocked reused-session 'runtime boundary record does not prove the required state'
assert_blocked forbidden-tool 'runtime boundary record does not prove the required state'
assert_blocked enabled-resource 'runtime boundary record does not prove the required state'
assert_blocked enabled-skill 'runtime boundary record does not prove the required state'
assert_blocked unexpected-tool 'neutral probe unexpectedly executed 2 tool events'
assert_blocked absent-isolation 'runtime boundary record does not prove the required state'
assert_blocked wrong-target-commit 'runtime boundary record does not prove the required state'
assert_blocked outside-read-succeeded 'runtime boundary record does not prove the required state'
assert_blocked outside-write-succeeded 'runtime boundary record does not prove the required state'
assert_blocked network-succeeded 'runtime boundary record does not prove the required state'
assert_blocked leak-secret 'provider output leaked credential environment variable NOMOS_PI_TEST_SECRET'

printf 'pi cold-agent boundary harness: PASS\n'

#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
preflight="$repo_root/docs/evaluation/agy-print-preflight.sh"
fake_agy="$repo_root/docs/evaluation/fixtures/fake-agy"
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT

AGY_BIN="$fake_agy" "$preflight" >"$tmp_dir/pass.txt"
grep -Fx 'AGY_PREFLIGHT PASS' "$tmp_dir/pass.txt" >/dev/null
grep -F "AGY_WORKTREE $repo_root" "$tmp_dir/pass.txt" >/dev/null

if FAKE_AGY_GREETING_ONLY=1 AGY_BIN="$fake_agy" \
  "$preflight" >"$tmp_dir/fail.txt" 2>"$tmp_dir/fail.err"; then
  printf 'expected greeting-only fake agy to fail the preflight\n' >&2
  exit 1
fi
grep -F 'event stream has no init event' "$tmp_dir/fail.err" >/dev/null

printf 'agy print preflight harness: PASS\n'

#!/usr/bin/env bash
set -Eeuo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
wrapper=$script_directory/r2-complete-proof-xfs.sh
test_root=$(mktemp -d "${TMPDIR:-/tmp}/nomos-r2-xfs-shell-test.XXXXXX")
trap 'rm -rf -- "$test_root"' EXIT

run_expect_fail() {
  local output status
  set +e
  output=$("$@" 2>&1)
  status=$?
  set -e
  [[ $status -ne 0 ]] || { printf 'expected failure: %s\n' "$*" >&2; return 1; }
  printf '%s\n' "$output"
}

mkdir -- "$test_root/work"
run_expect_fail "$wrapper" >"$test_root/usage.log"
grep -Fq 'usage: r2-complete-proof-xfs.sh --source CLEAN --work EMPTY' "$test_root/usage.log"
[[ -z $(find "$test_root/work" -mindepth 1 -print -quit) ]]

# The private branch must refuse an unprivileged caller before it can inspect
# or mutate any provisioning path.  A root runner instead exercises the
# argument-shape guard, which also avoids attempting a privileged eight-GiB
# lifecycle in a unit test.
if [[ $(id -u) != 0 ]]; then
  run_expect_fail "$wrapper" --supervise a b c d 1 1 user PATH /tmp /tmp '' '' >"$test_root/private.log"
  grep -Fq 'private supervisor is root-only' "$test_root/private.log"
else
  run_expect_fail "$wrapper" --supervise a b c d 1 1 user PATH /tmp /tmp '' >"$test_root/private.log"
  grep -Fq 'private supervisor argument shape is invalid' "$test_root/private.log"
fi

run_expect_fail "$wrapper" --source "$test_root/missing-source" --work "$test_root/work" >"$test_root/source.log"
grep -Fq 'source does not exist' "$test_root/source.log"
[[ -z $(find "$test_root/work" -mindepth 1 -print -quit) ]]

source=$test_root/source
mkdir -- "$source"
git -C "$source" init --quiet
git -C "$source" config user.email test@example.invalid
git -C "$source" config user.name 'R2 XFS Test'
printf 'test\n' >"$source/file"
git -C "$source" add file
git -C "$source" commit --quiet -m test
git -C "$source" checkout --quiet --detach HEAD

symlink_work=$test_root/symlink-work
mkdir -- "$test_root/real-work"
ln -s -- "$test_root/real-work" "$symlink_work"
run_expect_fail "$wrapper" --source "$source" --work "$symlink_work" >"$test_root/symlink.log"
grep -Fq 'work is not canonical' "$test_root/symlink.log"
[[ -z $(find "$test_root/real-work" -mindepth 1 -print -quit) ]]

# Keep the overlap target under Git's private metadata so the source remains
# clean long enough for the wrapper to reach its canonical non-overlap check.
nested_work=$source/.git/nested-work
mkdir -- "$nested_work"
run_expect_fail "$wrapper" --source "$source" --work "$nested_work" >"$test_root/overlap.log"
grep -Fq 'source and work paths overlap' "$test_root/overlap.log"
[[ -z $(find "$nested_work" -mindepth 1 -print -quit) ]]

printf 'r2-complete-proof-xfs shell validation tests: PASS\n'

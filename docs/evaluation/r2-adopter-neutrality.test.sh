#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 adopter-neutrality plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
checker=$script_dir/r2-adopter-neutrality.sh
temporary=$(mktemp -d)
trap 'rm -r -- "$temporary"' EXIT

mkdir -p "$temporary/root/crates" "$temporary/root/apps" "$temporary/root/fixtures" "$temporary/root/docs/evaluation"
cp -a "$repo_root/crates/nomos-observed-scene" "$temporary/root/crates/"
cp -a "$repo_root/apps/nomos-observed-viewer" "$temporary/root/apps/"
cp -a "$repo_root/fixtures/r2" "$temporary/root/fixtures/"
cp "$repo_root/docs/evaluation/R2_SOURCE_PROVENANCE.md" "$temporary/root/docs/evaluation/"
R2_NEUTRALITY_ROOT=$temporary/root "$checker" >/dev/null || fail 'clean copied scope did not pass'

expect_failure() {
  local label=$1
  local value=$2
  local root=$temporary/$label
  cp -a "$temporary/root" "$root"
  printf '%s\n' "$value" >>"$root/apps/nomos-observed-viewer/README.md"
  if R2_NEUTRALITY_ROOT=$root "$checker" >/dev/null 2>&1; then
    fail "$label plant passed"
  fi
}

expect_failure project-name 'the-mortal-estate'
expect_failure underscore-name 'mortal_estate'
expect_failure former-name 'cairn'
expect_failure frame-digest '3ab13123836830a50227bbe3729a21ed10b89bec2617a46d74c9fc9be04e7b48'
expect_failure projection-digest 'ad8c5577c7d52715eddeac104b273866b015b45db890d29bc3d36a6d7dbadb21'

printf 'R2_ADOPTER_NEUTRALITY_PLANTS PASS\n'
printf 'planted_failures 5\n'

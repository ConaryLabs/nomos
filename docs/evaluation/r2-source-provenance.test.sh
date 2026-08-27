#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 source provenance plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
checker=$script_dir/r2-source-provenance.sh

for command in cp git ln mktemp sed tar; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT
baseline=$tmp_dir/baseline
mkdir "$baseline"
git -C "$repo_root" archive --format=tar HEAD | tar -xf - -C "$baseline"
R2_PROVENANCE_ROOT=$baseline "$checker" >/dev/null || fail 'committed baseline did not pass'

expect_failure() {
  local label=$1
  local root=$tmp_dir/$label
  cp -a "$baseline" "$root"
  shift
  "$@" "$root"
  if R2_PROVENANCE_ROOT=$root "$checker" >/dev/null 2>&1; then
    fail "$label plant passed"
  fi
}

plant_missing() {
  rm -- "$1/fixtures/r2/scenes/scene_one.json"
}

plant_extra() {
  printf '{}\n' >"$1/fixtures/r2/unregistered.json"
}

plant_symlink() {
  rm -- "$1/crates/nomos-observed-scene/src/value.rs"
  ln -s /dev/null "$1/crates/nomos-observed-scene/src/value.rs"
}

plant_digest_drift() {
  printf '\n' >>"$1/crates/nomos-observed-scene/src/value.rs"
}

plant_unknown_origin() {
  sed -i '0,/`r2_authored`/s//`unknown_origin`/' "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_unlicensed() {
  sed -i '0,/| `project_mit` |/s//| `unlicensed` |/' "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_dangling_receipt() {
  sed -i '0,/`docs\/evaluation\/runs\/r2\/2026-08-27-issue-195-author\/AUTHOR_RECEIPT.md`/s//`docs\/evaluation\/runs\/r2\/missing.md`/' \
    "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_wrong_final_receipt() {
  sed -i '\|`docs/evaluation/r2-complete-proof.sh`|s|`docs/evaluation/runs/r2/2026-08-27-issue-199-author/AUTHOR_RECEIPT.md`|`docs/evaluation/runs/r2/2026-08-27-issue-197-author/AUTHOR_RECEIPT.md`|' \
    "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

expect_failure missing plant_missing
expect_failure extra plant_extra
expect_failure symlink plant_symlink
expect_failure digest-drift plant_digest_drift
expect_failure unknown-origin plant_unknown_origin
expect_failure unlicensed plant_unlicensed
expect_failure dangling-receipt plant_dangling_receipt
expect_failure wrong-final-receipt plant_wrong_final_receipt

printf 'R2_SOURCE_PROVENANCE_PLANTS PASS\n'
printf 'planted_failures 8\n'

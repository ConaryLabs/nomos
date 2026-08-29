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

for command in awk cp find git ln mktemp mv sed sort tar; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT
baseline=$tmp_dir/baseline
mkdir "$baseline"
R2_PROVENANCE_ROOT=$repo_root "$checker" >/dev/null ||
  fail 'committed repository did not pass before fixture reduction'

# The real repository check above owns closed-inventory coverage. Plant roots
# need only the already-verified registered bytes and their producing receipts;
# copying the 150+ MiB historical evaluation archive for every mutation would
# make the persistent 50 ms filesystem sampler measure disposable fixtures.
baseline_paths=$tmp_dir/baseline-paths
{
  printf '%s\n' LICENSE docs/evaluation/R2_SOURCE_PROVENANCE.md
  awk -F '|' '/^\| `[^`]+` \| `[0-9a-f]+` \| `/ {
    path = $2
    receipt = $5
    gsub(/[` ]/, "", path)
    gsub(/[` ]/, "", receipt)
    print path
    print receipt
  }' "$repo_root/docs/evaluation/R2_SOURCE_PROVENANCE.md"
} | LC_ALL=C sort -u >"$baseline_paths"
mapfile -t baseline_members <"$baseline_paths"
[[ ${#baseline_members[@]} -gt 2 ]] || fail 'reduced fixture member list is empty'
git -C "$repo_root" archive --format=tar HEAD -- "${baseline_members[@]}" |
  tar -xf - -C "$baseline"
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
  find "$root" -depth -delete
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

plant_ancestor_symlink() {
  mv -- "$1/.github" "$1/.github-real"
  ln -s .github-real "$1/.github"
}

plant_digest_drift() {
  printf '\n' >>"$1/crates/nomos-observed-scene/src/value.rs"
}

plant_project_license_drift() {
  printf '\n' >>"$1/LICENSE"
}

plant_unknown_origin() {
  # Backticks are literal Markdown delimiters in these planted sed patterns.
  # shellcheck disable=SC2016
  sed -i '0,/`r2_authored`/s//`unknown_origin`/' "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_unlicensed() {
  # shellcheck disable=SC2016
  sed -i '0,/| `project_mit` |/s//| `unlicensed` |/' "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_dangling_receipt() {
  # shellcheck disable=SC2016
  sed -i '0,/`docs\/evaluation\/runs\/r2\/2026-08-27-issue-195-author\/AUTHOR_RECEIPT.md`/s//`docs\/evaluation\/runs\/r2\/missing.md`/' \
    "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

plant_wrong_revision_4_receipt() {
  # shellcheck disable=SC2016
  sed -i '\|`docs/evaluation/r2-complete-proof.sh`|s|`docs/evaluation/runs/r2/2026-08-29-issue-199-revision-4-author/AUTHOR_RECEIPT.md`|`docs/evaluation/runs/r2/2026-08-28-issue-199-revision-3-author/AUTHOR_RECEIPT.md`|' \
    "$1/docs/evaluation/R2_SOURCE_PROVENANCE.md"
}

expect_failure missing plant_missing
expect_failure extra plant_extra
expect_failure symlink plant_symlink
expect_failure ancestor-symlink plant_ancestor_symlink
expect_failure digest-drift plant_digest_drift
expect_failure project-license-drift plant_project_license_drift
expect_failure unknown-origin plant_unknown_origin
expect_failure unlicensed plant_unlicensed
expect_failure dangling-receipt plant_dangling_receipt
expect_failure wrong-revision-4-receipt plant_wrong_revision_4_receipt

printf 'R2_SOURCE_PROVENANCE_PLANTS PASS\n'
printf 'planted_failures 10\n'

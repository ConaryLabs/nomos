#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 source provenance: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this checker accepts no arguments'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
default_root=$(cd "$script_dir/../.." && pwd -P)
repo_root=${R2_PROVENANCE_ROOT:-$default_root}
repo_root=$(cd "$repo_root" && pwd -P)
register=$repo_root/docs/evaluation/R2_SOURCE_PROVENANCE.md

for command in awk cmp cut diff find grep mktemp readlink sha256sum sort wc; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done
[[ -f $register && ! -L $register ]] || fail 'register is absent, non-regular, or symlinked'
[[ -f $repo_root/LICENSE && ! -L $repo_root/LICENSE ]] || fail 'project license is absent or symlinked'

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

awk -F '|' '/^\| `[^`]+` \| `[0-9a-f]+` \| `/ {
  path = $2
  digest = $3
  origin = $4
  receipt = $5
  license = $6
  gsub(/[` ]/, "", path)
  gsub(/[` ]/, "", digest)
  gsub(/[` ]/, "", origin)
  gsub(/[` ]/, "", receipt)
  gsub(/[` ]/, "", license)
  printf "%s\t%s\t%s\t%s\t%s\n", path, digest, origin, receipt, license
}' "$register" >"$tmp_dir/rows"

[[ -s $tmp_dir/rows ]] || fail 'register has no inventory rows'
[[ $(cut -f 1 "$tmp_dir/rows" | sort -u | wc -l) -eq $(wc -l <"$tmp_dir/rows") ]] ||
  fail 'register repeats a path'
sort -t $'\t' -k1,1 "$tmp_dir/rows" >"$tmp_dir/sorted-rows"
cmp -s "$tmp_dir/rows" "$tmp_dir/sorted-rows" || fail 'register rows are not LC_ALL=C path-sorted'

(
  cd "$repo_root"
  find crates/nomos-observed-scene -mindepth 1 -type f -printf '%p\n'
  if [[ -d apps/nomos-observed-viewer ]]; then
    find apps/nomos-observed-viewer -path '*/dist' -prune -o -type f -printf '%p\n'
  fi
  find fixtures/r2 -mindepth 1 -type f -printf '%p\n'
  find docs/evaluation -maxdepth 1 -type f \
    \( -name 'R2_*' -o -name 'r2-*' -o -name 'generate-r2-*' -o -name 'measure-r2-*' \) \
    ! -name 'R2_SOURCE_PROVENANCE.md' ! -name 'r2-source-provenance.sh' -printf '%p\n'
  printf '%s\n' \
    apps/nomos-viewer/vendor/three/LICENSE \
    apps/nomos-viewer/vendor/three/three.core.min.js \
    apps/nomos-viewer/vendor/three/three.module.min.js
) | sort -u >"$tmp_dir/expected-paths"

cut -f 1 "$tmp_dir/rows" >"$tmp_dir/registered-paths"
diff -u "$tmp_dir/expected-paths" "$tmp_dir/registered-paths" >/dev/null ||
  fail 'register has a missing or extra R2 source path'

check_tree() {
  local root=$1
  [[ -d $repo_root/$root && ! -L $repo_root/$root ]] ||
    fail "required source tree is absent or symlinked: $root"
  if find "$repo_root/$root" -mindepth 1 ! -type d ! -type f -print -quit | grep -q .; then
    fail "source tree contains a symlink or non-regular entry: $root"
  fi
}

check_tree crates/nomos-observed-scene
check_tree fixtures/r2
if [[ -e $repo_root/apps/nomos-observed-viewer ]]; then
  [[ -d $repo_root/apps/nomos-observed-viewer && ! -L $repo_root/apps/nomos-observed-viewer ]] ||
    fail 'R2 browser source tree is non-directory or symlinked'
  if find "$repo_root/apps/nomos-observed-viewer" -path '*/dist' -prune -o \
    ! -type d ! -type f -print -quit | grep -q .; then
    fail 'R2 browser source tree contains a symlink or non-regular entry outside dist'
  fi
fi
if find "$repo_root/docs/evaluation" -maxdepth 1 \
  \( -name 'R2_*' -o -name 'r2-*' -o -name 'generate-r2-*' -o -name 'measure-r2-*' \) \
  ! -type d ! -type f -print -quit | grep -q .; then
  fail 'R2 evaluation scope contains a symlink or non-regular entry'
fi

while IFS=$'\t' read -r path digest origin receipt license; do
  [[ $path != /* && $path != *'..'* && $path != *'//' ]] || fail "unsafe registered path: $path"
  [[ $digest =~ ^[0-9a-f]{64}$ ]] || fail "invalid SHA-256 spelling for $path"
  [[ -f $repo_root/$path && ! -L $repo_root/$path ]] ||
    fail "registered path is absent, non-regular, or symlinked: $path"
  actual=$(sha256sum "$repo_root/$path" | awk '{print $1}')
  [[ $actual == "$digest" ]] || fail "digest mismatch for $path"
  [[ -f $repo_root/$receipt && ! -L $repo_root/$receipt ]] ||
    fail "producing receipt is dangling for $path: $receipt"

  case $origin in
    r2_authored)
      [[ $receipt == docs/evaluation/runs/r2/2026-08-27-issue-195-author/AUTHOR_RECEIPT.md ]] ||
        fail "unexpected R2 author receipt for $path"
      ;;
    compiler_produced)
      [[ $path == fixtures/r2/plans/scene_one.json ]] ||
        fail "unexpected compiler-produced path in R2-1: $path"
      [[ $receipt == docs/evaluation/runs/r2/2026-08-27-issue-195-author/COMPILER_RECEIPT.md ]] ||
        fail "unexpected compiler receipt for $path"
      ;;
    r1_vendor_reuse)
      [[ $path == apps/nomos-viewer/vendor/three/* ]] ||
        fail "R1 vendor origin assigned outside the exact Three.js paths: $path"
      [[ $receipt == apps/nomos-viewer/vendor/MANIFEST.json ]] ||
        fail "unexpected R1 vendor receipt for $path"
      ;;
    browser_produced)
      fail "R2-1 contains no admissible browser-produced file: $path"
      ;;
    *) fail "unknown origin class for $path: $origin" ;;
  esac

  case $license in
    project_mit)
      [[ $origin != r1_vendor_reuse ]] || fail "vendor file uses project license disposition: $path"
      ;;
    three_mit_preserved)
      [[ $origin == r1_vendor_reuse ]] || fail "non-vendor file uses Three.js license disposition: $path"
      ;;
    *) fail "unknown or absent license disposition for $path: $license" ;;
  esac
done <"$tmp_dir/rows"

[[ $(sha256sum "$repo_root/apps/nomos-viewer/vendor/three/LICENSE" | awk '{print $1}') == \
  8b378ebe60e2fe500158cb0ac71cb5e8b7d92953c2abcc63a0eb90499653b5bc ]] ||
  fail 'preserved Three.js MIT license digest moved'

printf 'R2_SOURCE_PROVENANCE PASS\n'
printf 'inventory_rows %s\n' "$(wc -l <"$tmp_dir/rows")"
printf 'register_sha256 %s\n' "$(sha256sum "$register" | awk '{print $1}')"
printf 'project_license_sha256 %s\n' "$(sha256sum "$repo_root/LICENSE" | awk '{print $1}')"

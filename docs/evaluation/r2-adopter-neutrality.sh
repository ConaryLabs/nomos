#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 adopter neutrality: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this checker accepts no arguments'
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
default_root=$(cd "$script_dir/../.." && pwd -P)
repo_root=${R2_NEUTRALITY_ROOT:-$default_root}
repo_root=$(cd "$repo_root" && pwd -P)

for command in find grep sort; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

roots=(crates/nomos-observed-scene apps/nomos-observed-viewer fixtures/r2)
for root in "${roots[@]}"; do
  [[ -d $repo_root/$root && ! -L $repo_root/$root ]] || fail "missing or symlinked scope: $root"
done

mapfile -t files < <(
  (
    cd "$repo_root"
    find crates/nomos-observed-scene -type f -print
    find apps/nomos-observed-viewer -path '*/dist' -prune -o -type f -print
    find fixtures/r2 -type f -print
    if [[ -f docs/evaluation/r2-second-scene-packet/MANIFEST.sha256 ]]; then
      printf '%s\n' docs/evaluation/r2-second-scene-packet/MANIFEST.sha256
    fi
    printf '%s\n' docs/evaluation/R2_SOURCE_PROVENANCE.md
  ) | sort -u
)

[[ ${#files[@]} -gt 0 ]] || fail 'scan scope is empty'
absolute_files=()
for path in "${files[@]}"; do
  [[ -f $repo_root/$path && ! -L $repo_root/$path ]] || fail "non-regular or symlinked path: $path"
  absolute_files+=("$repo_root/$path")
done

patterns=(
  'the-mortal-estate'
  'mortal_estate'
  'cairn'
  '3ab13123836830a50227bbe3729a21ed10b89bec2617a46d74c9fc9be04e7b48'
  'ad8c5577c7d52715eddeac104b273866b015b45db890d29bc3d36a6d7dbadb21'
)

for pattern in "${patterns[@]}"; do
  if grep -a -i -F -n -- "$pattern" "${absolute_files[@]}" >/dev/null; then
    fail "forbidden adopter token or digest found: $pattern"
  fi
done

printf 'R2_ADOPTER_NEUTRALITY PASS\n'
printf 'files_scanned %s\n' "${#files[@]}"
printf 'forbidden_patterns %s\n' "${#patterns[@]}"

#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 second-scene packet assembly: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 3 ]] || fail 'usage: assemble.sh <40-hex-commit> <release-compiler> <new-output-directory>'
commit=$1
compiler=$2
output=$3
repo_root=$(git -C "$(dirname "${BASH_SOURCE[0]}")/../../.." rev-parse --show-toplevel)

for command in awk cp find git mkdir mktemp mv sha256sum sort tar; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done
[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail 'commit must be full lowercase hexadecimal'
[[ $(git -C "$repo_root" cat-file -t "$commit" 2>/dev/null) == commit ]] || fail 'candidate commit is absent'
[[ -f $compiler && ! -L $compiler && -x $compiler ]] || fail 'release compiler is absent, symlinked, or non-executable'
[[ ! -e $output ]] || fail 'output path already exists'

tree=$(git -C "$repo_root" rev-parse "$commit^{tree}")
mkdir -p "$output"
paths=(
  R2.md
  apps/nomos-observed-viewer
  apps/nomos-viewer/vendor/MANIFEST.json
  apps/nomos-viewer/vendor/three/LICENSE
  apps/nomos-viewer/vendor/three/three.core.min.js
  apps/nomos-viewer/vendor/three/three.module.min.js
  docs/evaluation/R2_SCHEMA_OWNERSHIP.md
  docs/evaluation/r2-scene-signature.mjs
  docs/evaluation/r2-scene-signature.test.mjs
  docs/evaluation/r2-second-scene-packet/AUTHOR_TASK.md
  docs/evaluation/r2-second-scene-packet/PROOF_COMMANDS.md
  docs/evaluation/r2-second-scene-packet/audit-author-output.sh
  docs/evaluation/r2-second-scene-packet/verify.sh
  fixtures/r2/plans/scene_one.json
  fixtures/r2/scenes/scene_one.json
)
git -C "$repo_root" archive --format=tar "$commit" -- "${paths[@]}" | tar -xf - -C "$output"
mkdir -p "$output/bin"
cp "$compiler" "$output/bin/nomos-observed-scene"
chmod 755 "$output/bin/nomos-observed-scene"
printf 'commit %s\ntree %s\n' "$commit" "$tree" >"$output/.nomos-packet-baseline"

manifest=$(mktemp)
trap 'rm -f -- "$manifest"' EXIT
(
  cd "$output"
  while IFS= read -r path; do
    digest=$(sha256sum "$path" | awk '{print $1}')
    printf '%s  %s\n' "$digest" "$path"
  done < <(find . -type f ! -name MANIFEST.sha256 -printf '%P\n' | sort)
) >"$manifest"
mv "$manifest" "$output/MANIFEST.sha256"
trap - EXIT
"$output/docs/evaluation/r2-second-scene-packet/verify.sh" "$output"

printf 'R2_SECOND_SCENE_PACKET_ASSEMBLY PASS\n'
printf 'commit %s\n' "$commit"
printf 'tree %s\n' "$tree"
printf 'manifest_sha256 %s\n' "$(sha256sum "$output/MANIFEST.sha256" | awk '{print $1}')"

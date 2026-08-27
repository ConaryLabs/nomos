#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 second-scene packet verification: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail 'usage: verify.sh <packet-directory>'
packet=$(cd "$1" && pwd -P)
manifest=$packet/MANIFEST.sha256
[[ -f $manifest && ! -L $manifest ]] || fail 'manifest is absent, non-regular, or symlinked'

for command in awk cmp cut find grep mktemp sha256sum sort wc; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done
if find "$packet" -mindepth 1 ! -type d ! -type f -print -quit | grep -q .; then
  fail 'packet contains a symlink or non-regular entry'
fi

temporary=$(mktemp -d)
trap 'rm -r -- "$temporary"' EXIT
awk 'match($0, /^([0-9a-f]{64})  ([^ ][^\r\n]*)$/, row) {
  if (row[2] == "MANIFEST.sha256" || row[2] ~ /^\// || row[2] ~ /(^|\/)\.\.($|\/)/) exit 2
  print row[2]
  next
}
{ exit 3 }' "$manifest" >"$temporary/recorded" || fail 'manifest grammar or path is invalid'
[[ -s $temporary/recorded ]] || fail 'manifest inventory is empty'
sort -u "$temporary/recorded" >"$temporary/sorted"
cmp -s "$temporary/recorded" "$temporary/sorted" || fail 'manifest paths are not sorted and unique'
(
  cd "$packet"
  find . -type f ! -name MANIFEST.sha256 -printf '%P\n' | sort
) >"$temporary/actual"
cmp -s "$temporary/recorded" "$temporary/actual" || fail 'manifest is not the exact packet inventory'
(cd "$packet" && sha256sum -c MANIFEST.sha256 >/dev/null) || fail 'packet digest mismatch'

baseline=$packet/.nomos-packet-baseline
[[ $(wc -l <"$baseline") -eq 2 ]] || fail 'baseline must have exactly two lines'
grep -Eq '^commit [0-9a-f]{40}$' "$baseline" || fail 'baseline commit is malformed'
grep -Eq '^tree [0-9a-f]{40}$' "$baseline" || fail 'baseline tree is malformed'
for required in \
  R2.md \
  bin/nomos-observed-scene \
  apps/nomos-observed-viewer/SOURCE_MANIFEST \
  apps/nomos-observed-viewer/PUBLIC_FILES \
  apps/nomos-observed-viewer/src/plan.mjs \
  apps/nomos-observed-viewer/src/catalog.mjs \
  apps/nomos-observed-viewer/src/render.mjs \
  apps/nomos-observed-viewer/src/ui.mjs \
  apps/nomos-observed-viewer/build.mjs \
  apps/nomos-observed-viewer/smoke/smoke.mjs \
  docs/evaluation/R2_SCHEMA_OWNERSHIP.md \
  docs/evaluation/r2-scene-signature.mjs \
  docs/evaluation/r2-second-scene-packet/AUTHOR_TASK.md \
  docs/evaluation/r2-second-scene-packet/PROOF_COMMANDS.md \
  fixtures/r2/scenes/scene_one.json \
  fixtures/r2/plans/scene_one.json; do
  grep -Fxq "$required" "$temporary/recorded" || fail "required packet input is absent: $required"
done
[[ -x $packet/bin/nomos-observed-scene ]] || fail 'release compiler is not executable'

printf 'R2_SECOND_SCENE_PACKET_VERIFY PASS\n'
printf 'files %s\n' "$(wc -l <"$temporary/recorded")"
printf 'manifest_sha256 %s\n' "$(sha256sum "$manifest" | awk '{print $1}')"

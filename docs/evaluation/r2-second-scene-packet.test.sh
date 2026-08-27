#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 second-scene packet plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
candidate=f64d374ee001ef0e51b66e3b2b4078ad6d1d770e
tree=2cae2ec952ccd21cff9c6e9e91424922551b35e5
manifest_digest=d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948
compiler=${R2_PACKET_COMPILER:-$repo_root/target/release/nomos-observed-scene}
assembler=$script_dir/r2-second-scene-packet/assemble.sh
verifier=$script_dir/r2-second-scene-packet/verify.sh
committed=$script_dir/r2-second-scene-packet/MANIFEST.sha256

[[ -f $compiler && ! -L $compiler && -x $compiler ]] || fail 'build the release R2 compiler first'
[[ $(sha256sum "$compiler" | awk '{print $1}') == dde136c1f2abd66e68ec395ce2fcfb427eec62e17f150d1bf35776a9da41e264 ]] ||
  fail 'release compiler digest differs from the frozen packet binary'
[[ $(sha256sum "$committed" | awk '{print $1}') == "$manifest_digest" ]] || fail 'committed manifest digest drifted'

temporary=$(mktemp -d)
trap 'rm -r -- "$temporary"' EXIT
packet=$temporary/packet
"$assembler" "$candidate" "$compiler" "$packet" >/dev/null
cmp -s "$committed" "$packet/MANIFEST.sha256" || fail 'reassembled inventory differs from the committed manifest'
grep -Fxq "commit $candidate" "$packet/.nomos-packet-baseline" || fail 'packet commit differs'
grep -Fxq "tree $tree" "$packet/.nomos-packet-baseline" || fail 'packet tree differs'

cp -a "$packet" "$temporary/drift"
printf '\n' >>"$temporary/drift/R2.md"
if "$verifier" "$temporary/drift" >/dev/null 2>&1; then
  fail 'digest-drift plant passed'
fi
cp -a "$packet" "$temporary/extra"
printf 'extra\n' >"$temporary/extra/extra.txt"
if "$verifier" "$temporary/extra" >/dev/null 2>&1; then
  fail 'extra-file plant passed'
fi

printf 'R2_SECOND_SCENE_PACKET_PLANTS PASS\n'
printf 'candidate %s\n' "$candidate"
printf 'tree %s\n' "$tree"
printf 'manifest_sha256 %s\n' "$manifest_digest"
printf 'planted_failures 2\n'

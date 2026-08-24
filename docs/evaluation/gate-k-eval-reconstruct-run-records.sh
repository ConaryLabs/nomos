#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k run record reconstruction: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 2 ]] || fail 'usage: gate-k-eval-reconstruct-run-records.sh RUN OUT'
run=$1
out=$2
[[ -d $run && ! -L $run ]] || fail "run is not a real directory: $run"
run=$(realpath -e "$run")
[[ ! -e $out ]] || fail "output already exists: $out"
out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"

for path in plan.json packet-manifest.json prompt.txt transcript.ndjson commands.json \
  adjudication.json subject checker artifacts packets/subject packets/checker; do
  [[ -e $run/$path && ! -L $run/$path ]] || fail "run member is absent: $path"
done
[[ -z $(find "$run" -type l -print -quit) ]] || fail 'run contains a symlink'
[[ -z $(find "$run" ! -type f ! -type d -print -quit) ]] ||
  fail 'run contains a special entry'

stage=$(mktemp -d "$out_parent/.run-records.XXXXXX")
trap 'rm -r -- "$stage"' EXIT
mkdir "$stage/subject"
for path in plan.json packet-manifest.json prompt.txt transcript.ndjson commands.json; do
  install -m 644 "$run/$path" "$stage/subject/$path"
done
cp -R "$run/artifacts" "$stage/subject/artifacts"
cp -R "$run/subject/." "$stage/subject/"
cp -R "$run/checker" "$stage/checker"
install -m 644 "$run/adjudication.json" "$stage/adjudication.json"

mv -- "$stage" "$out"
trap - EXIT
printf 'GATE_K_RUN_RECORDS_RECONSTRUCTED output=%s\n' "$out"

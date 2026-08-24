#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
reconstruct="$repo_root/docs/evaluation/gate-k-eval-reconstruct-run-records.sh"
finalizer="$repo_root/docs/evaluation/gate-k-eval-finalize.sh"
runs="$repo_root/docs/evaluation/runs/rehearsal"
tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

for pair in \
  2026-08-24-gemini-author-deepseek-checker-r6 \
  2026-08-24-deepseek-debug-gemini-checker-r6; do
  preserved="$runs/$pair"
  records="$tmp_dir/$pair-records"
  rebuilt="$tmp_dir/$pair-run"
  "$reconstruct" "$preserved" "$records" >/dev/null
  "$finalizer" --packet-roots "$preserved/packets/subject" \
    "$preserved/packets/checker" "$records/subject" "$records/checker" \
    "$records/adjudication.json" "$rebuilt" >/dev/null
  diff -r --exclude=packets "$rebuilt" "$preserved" >/dev/null
done

[[ $(sha256sum "$runs/2026-08-24-gemini-author-deepseek-checker-r6/result.json" |
  cut -d' ' -f1) == 847a03198affb8ab896b541ff2f7f04ed9635a7877cbec960fc33bbad5627f2e ]]
[[ $(sha256sum "$runs/2026-08-24-deepseek-debug-gemini-checker-r6/result.json" |
  cut -d' ' -f1) == 6458c901ca637f6cdab796af28e89a21b3d831a6cdd29644d7438a9d7dac2ba4 ]]

printf 'gate-k revision-6 rehearsal byte-identical re-finalization: PASS\n'

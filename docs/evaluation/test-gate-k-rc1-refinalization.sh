#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
finalizer="$repo_root/docs/evaluation/gate-k-eval-finalize.sh"
runs="$repo_root/docs/evaluation/runs/gate-k"
tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

for pair in \
  2026-08-23-gemini-3.7-flash-author \
  2026-08-23-deepseek-v4-flash-vision-exp-debug; do
  frozen="$runs/$pair"
  rebuilt="$tmp_dir/$pair"
  "$finalizer" --packet-roots "$frozen/packets/subject" \
    "$frozen/packets/checker" "$frozen/subject-task" "$frozen/checker-task" \
    "$frozen/adjudication.json" "$rebuilt" >/dev/null
  diff -r "$rebuilt/subject" "$frozen/subject" >/dev/null
  diff -r "$rebuilt/checker" "$frozen/checker" >/dev/null
  cmp "$rebuilt/checker.json" "$frozen/checker.json"
  cmp "$rebuilt/result.json" "$frozen/result.json"
  cmp "$rebuilt/RUN.md" "$frozen/RUN.md"
done

[[ $(sha256sum "$runs/2026-08-23-gemini-3.7-flash-author/result.json" |
  cut -d' ' -f1) == e6990dacde903f527d1cb46784a54d938a7e130f1193e51bb830a4a2284f07dc ]]
[[ $(sha256sum "$runs/2026-08-23-deepseek-v4-flash-vision-exp-debug/result.json" |
  cut -d' ' -f1) == f09c9214329f7f8bd7d4d4b31476a0f24c825add2f5bb434b7bf780f64d8089c ]]

printf 'gate-k rc1 byte-identical re-finalization: PASS\n'

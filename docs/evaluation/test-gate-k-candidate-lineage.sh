#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
proof="$repo_root/docs/evaluation/gate-k-candidate-lineage.py"
base=gate-k-rc1
candidate=$(git -C "$repo_root" rev-parse HEAD)
tmp_dir=$(mktemp -d)
cleanup() {
  rm -r -- "$tmp_dir"
}
trap cleanup EXIT

"$proof" "$base" "$candidate" >"$tmp_dir/receipt.json"
jq -e --arg candidate "$candidate" '
  .schema == "nomos.gate_k.candidate_lineage@1" and
  .status == "pass" and
  .base.tag == "gate-k-rc1" and
  .candidate.commit == $candidate and
  .protected.status == "byte-identical" and
  .kernelContract.contractRevision == 7 and
  .kernelContract.status == "exact-disposition-only-delta" and
  .roundOne.disposition == "failed" and
  .roundOne.criteria17And18 == "failed" and
  .roundOne.preservation == "exact-hash-match" and
  (.roundOne.frozenFiles | length) == 8 and
  .changedPaths.count > 0
' "$tmp_dir/receipt.json" >/dev/null

synthetic_commit() {
  local name=$1
  local path=$2
  local content=$3
  local index="$tmp_dir/$name.index"
  local blob tree

  GIT_INDEX_FILE="$index" git -C "$repo_root" read-tree "$candidate"
  blob=$(printf '%s' "$content" | git -C "$repo_root" hash-object -w --stdin)
  GIT_INDEX_FILE="$index" git -C "$repo_root" update-index --add \
    --cacheinfo "100644,$blob,$path"
  tree=$(GIT_INDEX_FILE="$index" git -C "$repo_root" write-tree)
  printf 'candidate lineage negative fixture: %s\n' "$name" |
    git -C "$repo_root" commit-tree "$tree" -p "$candidate"
}

assert_blocked() {
  local expected=$1
  local name=$2
  local commit=$3
  if "$proof" "$base" "$commit" >"$tmp_dir/$name.out" 2>"$tmp_dir/$name.err"; then
    printf 'candidate-lineage fixture unexpectedly passed: %s\n' "$name" >&2
    exit 1
  fi
  grep -F "$expected" "$tmp_dir/$name.err" >/dev/null || {
    printf 'candidate-lineage fixture failed for the wrong reason: %s\n' "$name" >&2
    cat "$tmp_dir/$name.err" >&2
    exit 1
  }
}

protected_commit=$(synthetic_commit protected Cargo.toml 'tampered cargo input')
assert_blocked 'protected file changed since gate-k-rc1: Cargo.toml' \
  protected "$protected_commit"

kernel_commit=$(synthetic_commit kernel KERNEL.md 'tampered contract')
assert_blocked 'KERNEL.md differs by more than the exact decision-0013 disposition edit' \
  kernel "$kernel_commit"

frozen_commit=$(synthetic_commit frozen \
  docs/decisions/0013-gate-k-disposition.md 'tampered round-one disposition')
assert_blocked 'frozen round-one evidence changed' frozen "$frozen_commit"

unclassified_commit=$(synthetic_commit unclassified UNCLASSIFIED.txt 'unexpected path')
assert_blocked 'unclassified path changed since gate-k-rc1: UNCLASSIFIED.txt' \
  unclassified "$unclassified_commit"

printf 'gate-k candidate lineage harness: PASS\n'

#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
validator="$script_dir/gate-k-eval-attempt-ledger.py"
ledger="$script_dir/gate-k-formal-attempt-ledger.jsonl"
tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

assert_blocked() {
  local expected=$1
  shift
  set +e
  "$@" >"$tmp_dir/blocked.stdout" 2>"$tmp_dir/blocked.stderr"
  status=$?
  set -e
  [[ $status -ne 0 ]] || { printf 'expected command to be blocked\n' >&2; exit 1; }
  grep -F "$expected" "$tmp_dir/blocked.stderr" >/dev/null
}

python3 "$validator" validate "$ledger"
cp "$ledger" "$tmp_dir/ledger.jsonl"

candidate=1111111111111111111111111111111111111111
manifest=2222222222222222222222222222222222222222222222222222222222222222
prompt=3333333333333333333333333333333333333333333333333333333333333333
nonce=4444444444444444444444444444444444444444444444444444444444444444
receipt=5555555555555555555555555555555555555555555555555555555555555555

python3 "$validator" next-reservation "$tmp_dir/ledger.jsonl" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" "$nonce" \
  >>"$tmp_dir/ledger.jsonl"
assert_blocked 'formal attempt remains open' python3 "$validator" validate "$tmp_dir/ledger.jsonl"
python3 "$validator" verify-reservation "$tmp_dir/ledger.jsonl" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" >/dev/null
assert_blocked 'earlier formal attempt remains open' python3 "$validator" next-reservation \
  "$tmp_dir/ledger.jsonl" hidden-retry "$candidate" author antigravity gemini-future high \
  "$manifest" "$prompt" "$nonce"

python3 "$validator" next-close "$tmp_dir/ledger.jsonl" future-author "$receipt" inconclusive \
  >>"$tmp_dir/ledger.jsonl"
python3 "$validator" validate "$tmp_dir/ledger.jsonl"
sed '2s/"previousEventSha256":"[0-9a-f]*"/"previousEventSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"/' \
  "$tmp_dir/ledger.jsonl" >"$tmp_dir/tampered.jsonl"
assert_blocked 'breaks the hash chain' python3 "$validator" validate "$tmp_dir/tampered.jsonl"

printf 'gate-k formal-attempt ledger harness: PASS\n'

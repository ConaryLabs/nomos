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

python3 "$validator" next-reservation "$tmp_dir/ledger.jsonl" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" "$nonce" \
  >>"$tmp_dir/ledger.jsonl"
assert_blocked 'formal attempt remains open' python3 "$validator" validate "$tmp_dir/ledger.jsonl"
python3 "$validator" verify-reservation "$tmp_dir/ledger.jsonl" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" >/dev/null
assert_blocked 'earlier formal attempt remains open' python3 "$validator" next-reservation \
  "$tmp_dir/ledger.jsonl" hidden-retry "$candidate" author antigravity gemini-future high \
  "$manifest" "$prompt" "$nonce"

ledger_sha=$(sha256sum "$tmp_dir/ledger.jsonl" | cut -d' ' -f1)
ledger_commit=6666666666666666666666666666666666666666
jq -S -c -n --arg candidate "$candidate" --arg manifest "$manifest" \
  --arg ledger_sha "$ledger_sha" --arg ledger_commit "$ledger_commit" '
  {schema:"nomos.gate_k.task_receipt@1",shape:"author",classification:"formal",
   formalAttempt:true,candidateCommit:$candidate,
   identity:{provider:"antigravity",model:"gemini-future",thinking:"high",
     sessionId:"11111111-2222-4333-8444-555555555555",
     sessionStartedAt:"2026-08-23T12:00:00Z",client:"Pi",clientVersion:"0.84.2",
     mode:"json",freshEphemeralSession:true},environment:{hostOs:"Linux fixture"},
   disclosures:{persistedSession:false,projectMemory:false,personalContext:false,
     contextFiles:[],connectors:[],webAccess:false,toolNetworkAccess:false,
     activeTools:["bash"],repositoryMounted:false},operatorIntervention:"none",
   operatorRetries:0,attemptReservation:{attemptId:"future-author",ledgerSha256:$ledger_sha,
     ledgerCommit:$ledger_commit},accounting:{assistantTurns:1,providerReportedTokens:2,toolCalls:1},
   outcome:"inconclusive",outcomeReason:"fixture transport failed",
   digests:{packetManifestSha256:$manifest,transcriptSha256:("7"*64),commandsSha256:("8"*64),
     artifactsTreeSha256:("9"*64),boundarySha256:("a"*64),qualificationSha256:("b"*64)}}' \
  >"$tmp_dir/task-receipt.json"
cat >"$tmp_dir/launcher.txt" <<EOF
PI_TASK_STATUS 1
PI_TASK_MODEL antigravity	gemini-future	Gemini Future	high
PI_TASK_COMMIT $candidate
PI_TASK_PACKET_MANIFEST_SHA256 $manifest
PI_TASK_ATTEMPT_ID future-author
PI_TASK_ATTEMPT_LEDGER_SHA256 $ledger_sha
PI_TASK_ATTEMPT_LEDGER_COMMIT $ledger_commit
PI_COLD_AGENT_TASK RECORDED
EOF
assert_blocked 'task receipt is not a regular file' python3 "$validator" next-close \
  "$tmp_dir/ledger.jsonl" future-author "$tmp_dir/absent.json" "$tmp_dir/launcher.txt" inconclusive
cp "$tmp_dir/launcher.txt" "$tmp_dir/incomplete-launcher.txt"
sed -i '/PI_COLD_AGENT_TASK RECORDED/d' "$tmp_dir/incomplete-launcher.txt"
assert_blocked 'completed provider launch' python3 "$validator" next-close \
  "$tmp_dir/ledger.jsonl" future-author "$tmp_dir/task-receipt.json" \
  "$tmp_dir/incomplete-launcher.txt" inconclusive
assert_blocked 'usage:' python3 "$validator" next-close "$tmp_dir/ledger.jsonl" \
  future-author 5555555555555555555555555555555555555555555555555555555555555555 inconclusive
python3 "$validator" next-close "$tmp_dir/ledger.jsonl" future-author \
  "$tmp_dir/task-receipt.json" "$tmp_dir/launcher.txt" inconclusive \
  >>"$tmp_dir/ledger.jsonl"
python3 "$validator" validate "$tmp_dir/ledger.jsonl"
cancel_nonce=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
python3 "$validator" next-reservation "$tmp_dir/ledger.jsonl" cancelled-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" "$cancel_nonce" \
  >>"$tmp_dir/ledger.jsonl"
python3 "$validator" next-cancel "$tmp_dir/ledger.jsonl" cancelled-author \
  'operator cancelled before provider launch' >>"$tmp_dir/ledger.jsonl"
python3 "$validator" validate "$tmp_dir/ledger.jsonl"
assert_blocked 'does not name the one open formal attempt' python3 "$validator" next-cancel \
  "$tmp_dir/ledger.jsonl" cancelled-author 'second hidden cancellation'
sed '2s/"previousEventSha256":"[0-9a-f]*"/"previousEventSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"/' \
  "$tmp_dir/ledger.jsonl" >"$tmp_dir/tampered.jsonl"
assert_blocked 'breaks the hash chain' python3 "$validator" validate "$tmp_dir/tampered.jsonl"

printf 'gate-k formal-attempt ledger harness: PASS\n'

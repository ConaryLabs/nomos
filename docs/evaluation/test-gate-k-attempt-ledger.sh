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
# The live append-only ledger grows when a prospectively reserved formal
# attempt closes. Preserve the separate frozen-inventory assertion against the
# exact four imported round-one events rather than requiring the live ledger to
# remain forever equal to its initial prefix.
head -n 4 "$ledger" >"$tmp_dir/frozen-inventory.jsonl"
python3 "$validator" validate-frozen-inventory "$tmp_dir/frozen-inventory.jsonl"
cp "$ledger" "$tmp_dir/forged-import-ledger.jsonl"
sed -i '1s/732af459/832af459/' "$tmp_dir/forged-import-ledger.jsonl"
assert_blocked 'not an exact frozen Gate K import' python3 "$validator" validate \
  "$tmp_dir/forged-import-ledger.jsonl"
mkdir -p "$tmp_dir/repo/docs/evaluation"
cp "$ledger" "$tmp_dir/repo/docs/evaluation/gate-k-formal-attempt-ledger.jsonl"
ledger_copy="$tmp_dir/repo/docs/evaluation/gate-k-formal-attempt-ledger.jsonl"
git -C "$tmp_dir/repo" init -q
git -C "$tmp_dir/repo" config user.name 'Gate K fixture'
git -C "$tmp_dir/repo" config user.email gate-k-fixture@example.invalid
git -C "$tmp_dir/repo" add docs/evaluation/gate-k-formal-attempt-ledger.jsonl
git -C "$tmp_dir/repo" commit -qm 'Import attempt ledger'

candidate=1111111111111111111111111111111111111111
manifest=2222222222222222222222222222222222222222222222222222222222222222
prompt=3333333333333333333333333333333333333333333333333333333333333333
nonce=4444444444444444444444444444444444444444444444444444444444444444

python3 "$validator" next-reservation "$ledger_copy" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" "$nonce" \
  >>"$ledger_copy"
git -C "$tmp_dir/repo" add docs/evaluation/gate-k-formal-attempt-ledger.jsonl
git -C "$tmp_dir/repo" commit -qm 'Reserve future author attempt'
assert_blocked 'formal attempt remains open' python3 "$validator" validate "$ledger_copy"
python3 "$validator" verify-reservation "$ledger_copy" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" >/dev/null
assert_blocked 'earlier formal attempt remains open' python3 "$validator" next-reservation \
  "$ledger_copy" hidden-retry "$candidate" author antigravity gemini-future high \
  "$manifest" "$prompt" "$nonce"

ledger_sha=$(sha256sum "$ledger_copy" | cut -d' ' -f1)
ledger_commit=$(git -C "$tmp_dir/repo" rev-parse HEAD)
record="$tmp_dir/task-record"
mkdir -p "$record/artifacts"
printf '# Fixture task receipt\n' >"$record/TASK.md"
printf 'fixture prompt\n' >"$record/prompt.txt"
printf '%s\n' '{"assistantTurns":1,"providerReportedTokens":2,"toolCalls":1}' \
  >"$record/accounting.json"
prompt_sha=$(sha256sum "$record/prompt.txt" | cut -d' ' -f1)
jq -S -c -n --arg candidate "$candidate" --arg prompt_sha "$prompt_sha" '
  {schema:"nomos.gate_k.eval_plan@1",
   task:{shape:"author",classification:"formal",formalAttempt:true},
   candidate:{commit:$candidate,binaryPath:"bin/nomos",binarySha256:("5"*64)},
   packet:{briefPath:"brief.txt",briefSha256:("6"*64),promptPath:"prompt.txt",
     promptSha256:$prompt_sha,writablePaths:["workspace"],repositoryMounted:false,
     gitMetadataPresent:false,networkPermitted:false,activeTools:["bash"]},
   budgets:{freshSessions:1,operatorRetriesMaximum:0,operatorSubstantiveHintsMaximum:0},
   rubric:["identity","evidence","disposition"],
   recording:{eventStream:"complete-ndjson",
     removedProviderFields:["textSignature","thinkingSignature"],
     commandOrderPreserved:true,
     transcriptLossLimit:"only-the-two-declared-provider-signature-fields"},
   operatorIntervention:"none",verdicts:["pass","fail","assisted","inconclusive"]}
' >"$record/plan.json"
plan_sha=$(sha256sum "$record/plan.json" | cut -d' ' -f1)
plan_bytes=$(stat -c %s "$record/plan.json")
prompt_bytes=$(stat -c %s "$record/prompt.txt")
jq -S -c -n --arg candidate "$candidate" --arg plan_sha "$plan_sha" \
  --arg prompt_sha "$prompt_sha" --argjson plan_bytes "$plan_bytes" \
  --argjson prompt_bytes "$prompt_bytes" '
  {schema:"nomos.gate_k.packet_manifest@1",candidateCommit:$candidate,shape:"author",
   manifestExcludesSelf:true,writablePaths:["workspace"],files:[
     {path:"plan.json",bytes:$plan_bytes,mode:"644",sha256:$plan_sha,
      schemaIdentity:"nomos.gate_k.eval_plan@1"},
     {path:"prompt.txt",bytes:$prompt_bytes,mode:"644",sha256:$prompt_sha,
      schemaIdentity:null}]}
' >"$record/packet-manifest.json"
printf 'transcript\n' >"$record/transcript.ndjson"
printf 'commands\n' >"$record/commands.json"
printf 'boundary\n' >"$record/boundary.json"
printf 'qualification\n' >"$record/pi-qualification.txt"
printf 'stderr\n' >"$record/pi-stderr.txt"
printf 'artifact\n' >"$record/artifacts/result.txt"
manifest=$(sha256sum "$record/packet-manifest.json" | cut -d' ' -f1)
transcript=$(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1)
commands=$(sha256sum "$record/commands.json" | cut -d' ' -f1)
boundary=$(sha256sum "$record/boundary.json" | cut -d' ' -f1)
qualification=$(sha256sum "$record/pi-qualification.txt" | cut -d' ' -f1)
stderr_sha=$(sha256sum "$record/pi-stderr.txt" | cut -d' ' -f1)
artifacts=$(cd "$record/artifacts" && sha256sum result.txt | sha256sum | cut -d' ' -f1)
raw=7777777777777777777777777777777777777777777777777777777777777777

# Rebuild the committed reservation with the actual packet identity used below.
git -C "$tmp_dir/repo" reset -q --soft HEAD^
git -C "$tmp_dir/repo" reset -q
git -C "$tmp_dir/repo" checkout -q -- docs/evaluation/gate-k-formal-attempt-ledger.jsonl
python3 "$validator" next-reservation "$ledger_copy" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt_sha" "$nonce" >>"$ledger_copy"
git -C "$tmp_dir/repo" add docs/evaluation/gate-k-formal-attempt-ledger.jsonl
git -C "$tmp_dir/repo" commit -qm 'Reserve future author attempt'
python3 "$validator" next-launch "$ledger_copy" future-author \
  --committed-repo "$tmp_dir/repo" >>"$ledger_copy"
git -C "$tmp_dir/repo" add docs/evaluation/gate-k-formal-attempt-ledger.jsonl
git -C "$tmp_dir/repo" commit -qm 'Authenticate future author launch'
python3 "$validator" verify-launch "$ledger_copy" future-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt_sha" \
  --committed-repo "$tmp_dir/repo" >/dev/null
git clone -q "$tmp_dir/repo" "$tmp_dir/squashed-repo"
git -C "$tmp_dir/squashed-repo" config user.name 'Gate K fixture'
git -C "$tmp_dir/squashed-repo" config user.email gate-k-fixture@example.invalid
git -C "$tmp_dir/squashed-repo" reset -q --soft HEAD~2
git -C "$tmp_dir/squashed-repo" commit -qm 'Illegally combine reservation and launch'
assert_blocked 'launch marker was not committed after its reservation' python3 "$validator" \
  verify-launch "$tmp_dir/squashed-repo/docs/evaluation/gate-k-formal-attempt-ledger.jsonl" \
  future-author "$candidate" author antigravity gemini-future high "$manifest" \
  "$prompt_sha" --committed-repo "$tmp_dir/squashed-repo"
assert_blocked 'already has an authenticated launch marker' python3 "$validator" \
  verify-reservation "$ledger_copy" future-author "$candidate" author antigravity \
  gemini-future high "$manifest" "$prompt_sha"
ledger_sha=$(sha256sum "$ledger_copy" | cut -d' ' -f1)
ledger_commit=$(git -C "$tmp_dir/repo" rev-parse HEAD)

jq -S -c -n --arg candidate "$candidate" --arg manifest "$manifest" \
  --arg ledger_sha "$ledger_sha" --arg ledger_commit "$ledger_commit" \
  --arg transcript "$transcript" --arg commands "$commands" --arg artifacts "$artifacts" \
  --arg boundary "$boundary" --arg qualification "$qualification" '
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
   outcome:"inconclusive",outcomeReason:"Pi transport exited 1",
   execution:{pi:{path:"/fixture/pi",sha256:("1"*64)},providerExtension:null,
     bubblewrap:{path:"/fixture/bwrap",sha256:("2"*64)}},
   digests:{packetManifestSha256:$manifest,rawTranscriptSha256:("7"*64),
     transcriptSha256:$transcript,commandsSha256:$commands,
     artifactsTreeSha256:$artifacts,boundarySha256:$boundary,
     qualificationSha256:$qualification}}' >"$record/task-receipt.json"
cat >"$record/launcher.txt" <<EOF
PI_TASK_STATUS 1
PI_TASK_MODEL antigravity	gemini-future	Gemini Future	high
PI_TASK_SESSION 11111111-2222-4333-8444-555555555555 ephemeral
PI_TASK_COMMIT $candidate
PI_TASK_PACKET_MANIFEST_SHA256 $manifest
PI_TASK_RAW_EVENTS_SHA256 $raw
PI_TASK_EVENTS_SHA256 $transcript
PI_TASK_STDERR_SHA256 $stderr_sha
PI_TASK_QUALIFICATION_SHA256 $qualification
PI_TASK_ATTEMPT_ID future-author
PI_TASK_ATTEMPT_LEDGER_SHA256 $ledger_sha
PI_TASK_ATTEMPT_LEDGER_COMMIT $ledger_commit
PI_COLD_AGENT_TASK RECORDED
EOF
assert_blocked 'task record is not a regular directory' python3 "$validator" next-close \
  "$ledger_copy" future-author "$tmp_dir/absent" inconclusive --committed-repo "$tmp_dir/repo"
cp -R "$record" "$tmp_dir/incomplete-record"
sed -i '/PI_COLD_AGENT_TASK RECORDED/d' "$tmp_dir/incomplete-record/launcher.txt"
assert_blocked 'exact record schema' python3 "$validator" next-close \
  "$ledger_copy" future-author "$tmp_dir/incomplete-record" inconclusive \
  --committed-repo "$tmp_dir/repo"
cp -R "$record" "$tmp_dir/forged-stderr-record"
printf 'forged\n' >"$tmp_dir/forged-stderr-record/pi-stderr.txt"
assert_blocked 'stderr evidence' python3 "$validator" next-close \
  "$ledger_copy" future-author "$tmp_dir/forged-stderr-record" inconclusive \
  --committed-repo "$tmp_dir/repo"
cp -R "$record" "$tmp_dir/status-record"
sed -i 's/^PI_TASK_STATUS 1$/PI_TASK_STATUS 0/' "$tmp_dir/status-record/launcher.txt"
assert_blocked 'status differs from the task outcome' python3 "$validator" next-close \
  "$ledger_copy" future-author "$tmp_dir/status-record" inconclusive \
  --committed-repo "$tmp_dir/repo"
cp -R "$record" "$tmp_dir/forged-head-record"
forged_head=6666666666666666666666666666666666666666
sed -i "s/^PI_TASK_ATTEMPT_LEDGER_COMMIT .*/PI_TASK_ATTEMPT_LEDGER_COMMIT $forged_head/" \
  "$tmp_dir/forged-head-record/launcher.txt"
jq -S -c --arg commit "$forged_head" '.attemptReservation.ledgerCommit = $commit' \
  "$tmp_dir/forged-head-record/task-receipt.json" >"$tmp_dir/forged-head-record/task-receipt.update"
mv -- "$tmp_dir/forged-head-record/task-receipt.update" \
  "$tmp_dir/forged-head-record/task-receipt.json"
assert_blocked 'committed ledger HEAD' python3 "$validator" next-close \
  "$ledger_copy" future-author "$tmp_dir/forged-head-record" inconclusive \
  --committed-repo "$tmp_dir/repo"
assert_blocked 'gate-k eval finalizer: FAIL' python3 "$validator" next-close \
  "$ledger_copy" future-author "$record" inconclusive --committed-repo "$tmp_dir/repo"
assert_blocked 'cannot be cancelled after provider launch' python3 "$validator" next-cancel \
  "$ledger_copy" future-author 'fixture record deliberately fails semantic validation'
assert_blocked 'formal attempt remains open' python3 "$validator" validate "$ledger_copy"
assert_blocked 'formal attempt remains open' \
  python3 "$validator" validate-frozen-inventory "$ledger_copy"
cancel_ledger="$tmp_dir/cancellation-ledger.jsonl"
cp "$ledger" "$cancel_ledger"
cancel_nonce=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
python3 "$validator" next-reservation "$cancel_ledger" cancelled-author "$candidate" \
  author antigravity gemini-future high "$manifest" "$prompt" "$cancel_nonce" \
  >>"$cancel_ledger"
assert_blocked 'formal close lacks its authenticated launch marker' python3 "$validator" \
  next-close "$cancel_ledger" cancelled-author "$record" inconclusive \
  --committed-repo "$tmp_dir/repo"
python3 "$validator" next-cancel "$cancel_ledger" cancelled-author \
  'operator cancelled before provider launch' >>"$cancel_ledger"
python3 "$validator" validate "$cancel_ledger"
assert_blocked 'does not name the one open formal attempt' python3 "$validator" next-cancel \
  "$cancel_ledger" cancelled-author 'second hidden cancellation'
sed '2s/"previousEventSha256":"[0-9a-f]*"/"previousEventSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"/' \
  "$cancel_ledger" >"$tmp_dir/tampered.jsonl"
assert_blocked 'breaks the hash chain' python3 "$validator" validate "$tmp_dir/tampered.jsonl"
sed '1s/"sequence":1/"sequence":1.0/' "$cancel_ledger" >"$tmp_dir/float-sequence.jsonl"
assert_blocked 'invalid schema or sequence' python3 "$validator" validate \
  "$tmp_dir/float-sequence.jsonl"

printf 'gate-k formal-attempt ledger harness: PASS\n'

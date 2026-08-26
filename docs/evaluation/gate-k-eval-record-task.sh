#!/usr/bin/env bash

set -euo pipefail

# Canonical artifact ordering and tree digests are byte-defined.
export LC_ALL=C

fail() {
  printf 'gate-k eval task recorder: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 7 ]] || fail \
  'usage: gate-k-eval-record-task.sh PACKET EVENTS STDERR QUALIFICATION LAUNCHER COMMIT OUT'
packet=$1
events=$2
stderr_record=$3
qualification=$4
launcher=$5
commit=$6
out=$7
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
command_deriver="$script_dir/gate-k-eval-derive-commands.sh"
transcript_validator="$script_dir/gate-k-eval-validate-transcript.py"
json_validator="$script_dir/gate-k-eval-validate-json.py"
document_validator="$script_dir/gate-k-eval-validate-documents.py"
attempt_validator="$script_dir/gate-k-eval-attempt-ledger.py"

[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail 'commit is not a full lowercase SHA-1'
for path in "$packet/plan.json" "$packet/packet-manifest.json" "$packet/prompt.txt" \
  "$events" "$stderr_record" "$qualification" "$launcher"; do
  [[ -f $path && ! -L $path ]] || fail "required regular input is absent: $path"
done
python3 "$document_validator" plan "$packet/plan.json" || fail 'plan schema is invalid'
python3 "$document_validator" manifest "$packet/packet-manifest.json" ||
  fail 'packet manifest schema is invalid'
[[ ! -e $out ]] || fail "output already exists: $out"
out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"

for name in jq sha256sum find sort install realpath grep sed; do
  command -v "$name" >/dev/null 2>&1 || fail "required executable not found: $name"
done

grep -Fx 'PI_COLD_AGENT_BOUNDARY PASS' "$qualification" >/dev/null ||
  fail 'qualification receipt is not a pass'
grep -Fx 'PI_COLD_AGENT_TASK RECORDED' "$launcher" >/dev/null ||
  fail 'launcher did not record a complete task transport'
launcher_status=$(sed -n 's/^PI_TASK_STATUS //p' "$launcher")
[[ $launcher_status =~ ^[0-9]+$ ]] || fail 'launcher status is absent or invalid'

boundary_count=$(grep -Fc 'NOMOS_PI_BOUNDARY ' "$stderr_record" || true)
accounting_count=$(grep -Fc 'NOMOS_PI_ACCOUNTING ' "$stderr_record" || true)
[[ $boundary_count -eq 1 ]] || fail "expected one boundary record, found $boundary_count"
[[ $accounting_count -eq 1 ]] || fail "expected one accounting record, found $accounting_count"
boundary=$(sed -n 's/^NOMOS_PI_BOUNDARY //p' "$stderr_record")
accounting=$(sed -n 's/^NOMOS_PI_ACCOUNTING //p' "$stderr_record")
printf '%s\n' "$boundary" | python3 "$json_validator" - ||
  fail 'boundary record contains invalid or duplicate-key JSON'
printf '%s\n' "$accounting" | python3 "$json_validator" - ||
  fail 'accounting record contains invalid or duplicate-key JSON'
printf '%s\n' "$boundary" | jq -e . >/dev/null || fail 'boundary record is not JSON'
printf '%s\n' "$accounting" | jq -e . >/dev/null || fail 'accounting record is not JSON'

shape=$(jq -r '.task.shape' "$packet/plan.json")
classification=$(jq -r '.task.classification' "$packet/plan.json")
formal=$(jq -r '.task.formalAttempt' "$packet/plan.json")
writable=$(jq -r '.packet.writablePaths[0]' "$packet/plan.json")
[[ -d $packet/$writable && ! -L $packet/$writable ]] || fail 'subject artifact directory is absent'

session_count=$(jq -s '[.[] | select(.type == "session")] | length' "$events")
[[ $session_count -eq 1 ]] || fail "expected one session event, found $session_count"
assistant_messages=$(jq -s '[.[] | select(.type == "message_end" and .message.role == "assistant")]' "$events")
[[ $(printf '%s\n' "$assistant_messages" | jq 'length') -ge 1 ]] ||
  fail 'assistant identity/result events are absent'
provider=$(printf '%s\n' "$boundary" | jq -r '.provider // empty')
model=$(printf '%s\n' "$boundary" | jq -r '.model // empty')
thinking=$(printf '%s\n' "$boundary" | jq -r '.thinking // empty')
session_id=$(printf '%s\n' "$boundary" | jq -r '.sessionId // empty')
session_timestamp=$(jq -s -c '[.[] | select(.type == "session")][0].timestamp' "$events")
pi_version=$(sed -n 's/^PI_VERSION //p' "$qualification")
host_os=$(sed -n 's/^PI_HOST_OS //p' "$qualification")
[[ -n $provider && -n $model && -n $thinking && $session_id =~ ^[0-9a-f-]{36}$ ]] ||
  fail 'boundary result identity is incomplete'
[[ -n $pi_version && -n $host_os && $session_timestamp != null ]] ||
  fail 'client, environment, or session-date identity is incomplete'
session_timestamp_text=$(jq -r . <<<"$session_timestamp")
prompt=$(<"$packet/prompt.txt")
derived_accounting=$(python3 "$transcript_validator" "$events" \
  --prompt "$prompt" --provider "$provider" --model "$model" \
  --session "$session_id" --started "$session_timestamp_text" \
  --workspace "$(printf '%s\n' "$boundary" | jq -r '.hostWorkspace')") ||
  fail 'task transcript is not a complete ordered Pi lifecycle'
printf '%s\n' "$assistant_messages" | jq -e \
  --arg provider "$provider" --arg model "$model" '
  all(.[]; .message.provider == $provider and .message.model == $model)
  ' >/dev/null || fail 'assistant result identity differs from the boundary'

tmp_dir=$(mktemp -d)
stage=$(mktemp -d "$out_parent/.gate-k-eval-task.XXXXXX")
cleanup() {
  rm -r -- "$tmp_dir" "$stage"
}
trap cleanup EXIT

"$command_deriver" "$events" >"$tmp_dir/commands.json"
command_count=$(jq '.commands | length' "$tmp_dir/commands.json")
[[ $command_count -ge 1 ]] || fail 'completed task has no command record'
jq -e '
  all(.commands[];
    .tool == "bash" and .completed == true and
    (.arguments.command | type) == "string" and (.arguments.command | length) > 0 and
    (.isError | type) == "boolean")
  ' "$tmp_dir/commands.json" >/dev/null || fail 'command record is incomplete or contains an unexpected tool'

[[ $(jq -S -c . <<<"$accounting") == "$(jq -S -c . <<<"$derived_accounting")" ]] ||
  fail 'protocol accounting differs from the complete event stream'

if [[ $launcher_status -ne 0 ]]; then
  outcome=inconclusive
  outcome_reason="Pi transport exited $launcher_status"
elif [[ $shape == *-checker ]]; then
  outcome=completed-checker
  outcome_reason='checker transport and protocol accounting complete; checker artifact requires final assembly'
else
  outcome=eligible-for-checker
  outcome_reason='subject transport and protocol accounting complete; task merit requires checker adjudication'
fi

install -d -m 755 "$stage/artifacts"
install -m 644 "$packet/plan.json" "$stage/plan.json"
install -m 644 "$packet/packet-manifest.json" "$stage/packet-manifest.json"
install -m 644 "$packet/prompt.txt" "$stage/prompt.txt"
install -m 644 "$events" "$stage/transcript.ndjson"
install -m 644 "$tmp_dir/commands.json" "$stage/commands.json"
install -m 644 "$qualification" "$stage/pi-qualification.txt"
install -m 644 "$launcher" "$stage/launcher.txt"
install -m 644 "$stderr_record" "$stage/pi-stderr.txt"
printf '%s\n' "$boundary" | jq -S -c . >"$stage/boundary.json"
printf '%s\n' "$accounting" | jq -S -c . >"$stage/accounting.json"

[[ -z $(find "$packet/$writable" -type l -print -quit) ]] || fail 'subject artifacts contain a symlink'
[[ -z $(find "$packet/$writable" ! -type f ! -type d -print -quit) ]] ||
  fail 'subject artifacts contain a special entry'
empty_artifact_directory=$(find "$packet/$writable" -mindepth 1 -type d -empty -print -quit)
[[ -z $empty_artifact_directory ]] ||
  fail "subject artifacts contain an unbound empty directory: ${empty_artifact_directory#"$packet/$writable"/}"
while IFS= read -r -d '' relative; do
  relative=${relative#./}
  if [[ -d $packet/$writable/$relative ]]; then
    install -d -m 755 "$stage/artifacts/$relative"
  else
    install -d -m 755 "$(dirname "$stage/artifacts/$relative")"
    install -m 644 "$packet/$writable/$relative" "$stage/artifacts/$relative"
  fi
done < <(cd "$packet/$writable" && find . -mindepth 1 -print0 | sort -z)

manifest_sha=$(sha256sum "$stage/packet-manifest.json" | cut -d' ' -f1)
transcript_sha=$(sha256sum "$stage/transcript.ndjson" | cut -d' ' -f1)
raw_transcript_sha=$(sed -n 's/^PI_TASK_RAW_EVENTS_SHA256 //p' "$launcher")
[[ $raw_transcript_sha =~ ^[0-9a-f]{64}$ ]] ||
  fail 'launcher does not bind the raw provider event stream'
commands_sha=$(sha256sum "$stage/commands.json" | cut -d' ' -f1)
artifacts_sha=$(find "$stage/artifacts" -type f -printf '%P\0' | sort -z |
  while IFS= read -r -d '' relative; do
    sha256sum "$stage/artifacts/$relative" | sed "s#  $stage/artifacts/#  #"
  done | sha256sum | cut -d' ' -f1)
boundary_sha=$(sha256sum "$stage/boundary.json" | cut -d' ' -f1)
qualification_sha=$(sha256sum "$stage/pi-qualification.txt" | cut -d' ' -f1)
execution=$(jq -c '.runtimeIdentity // null' "$stage/boundary.json")
[[ $execution != null ]] || fail 'task boundary lacks the authenticated runtime identity'

attempt_reservation=null
if [[ $formal == true ]]; then
  attempt_id=$(sed -n 's/^PI_TASK_ATTEMPT_ID //p' "$launcher")
  attempt_ledger_sha=$(sed -n 's/^PI_TASK_ATTEMPT_LEDGER_SHA256 //p' "$launcher")
  attempt_ledger_commit=$(sed -n 's/^PI_TASK_ATTEMPT_LEDGER_COMMIT //p' "$launcher")
  [[ -n $attempt_id && $attempt_ledger_sha =~ ^[0-9a-f]{64}$ &&
    $attempt_ledger_commit =~ ^[0-9a-f]{40}$ ]] ||
    fail 'formal launcher lacks its committed attempt-ledger binding'
  attempt_reservation=$(jq -S -c -n --arg id "$attempt_id" --arg sha "$attempt_ledger_sha" \
    --arg commit "$attempt_ledger_commit" \
    '{attemptId:$id,ledgerSha256:$sha,ledgerCommit:$commit}')
else
  [[ -z $(sed -n '/^PI_TASK_ATTEMPT_/p' "$launcher") ]] ||
    fail 'rehearsal launcher unexpectedly claims a formal-attempt reservation'
fi

result=$(jq -S -c -n \
  --arg shape "$shape" \
  --arg classification "$classification" \
  --argjson formal "$formal" \
  --arg commit "$commit" \
  --arg provider "$provider" \
  --arg model "$model" \
  --arg thinking "$thinking" \
  --arg session "$session_id" \
  --argjson session_timestamp "$session_timestamp" \
  --arg pi_version "$pi_version" \
  --arg host_os "$host_os" \
  --arg outcome "$outcome" \
  --arg reason "$outcome_reason" \
  --arg manifest_sha "$manifest_sha" \
  --arg transcript_sha "$transcript_sha" \
  --arg raw_transcript_sha "$raw_transcript_sha" \
  --arg commands_sha "$commands_sha" \
  --arg artifacts_sha "$artifacts_sha" \
  --arg boundary_sha "$boundary_sha" \
  --arg qualification_sha "$qualification_sha" \
  --argjson attempt_reservation "$attempt_reservation" \
  --argjson execution "$execution" \
  --argjson accounting "$accounting" '
  {
    schema: "nomos.gate_k.task_receipt@2",
    protocolRevision: 6,
    shape: $shape,
    classification: $classification,
    formalAttempt: $formal,
    candidateCommit: $commit,
    identity: {
      provider: $provider,
      model: $model,
      thinking: $thinking,
      sessionId: $session,
      sessionStartedAt: $session_timestamp,
      client: "Pi",
      clientVersion: $pi_version,
      mode: "json",
      freshEphemeralSession: true
    },
    environment: {hostOs: $host_os},
    disclosures: {
      persistedSession: false,
      projectMemory: false,
      personalContext: false,
      contextFiles: [],
      connectors: [],
      webAccess: false,
      toolNetworkAccess: false,
      activeTools: ["bash"],
      repositoryMounted: false
    },
    operatorIntervention: "none",
    operatorRetries: 0,
    attemptReservation: $attempt_reservation,
    execution: $execution,
    accounting: $accounting,
    outcome: $outcome,
    outcomeReason: $reason,
    digests: {
      packetManifestSha256: $manifest_sha,
      rawTranscriptSha256: $raw_transcript_sha,
      transcriptSha256: $transcript_sha,
      commandsSha256: $commands_sha,
      artifactsTreeSha256: $artifacts_sha,
      boundarySha256: $boundary_sha,
      qualificationSha256: $qualification_sha
    }
  }
')
printf '%s\n' "$result" >"$stage/task-receipt.json"
python3 "$document_validator" task-receipt "$stage/task-receipt.json" ||
  fail 'constructed task receipt schema is invalid'

printf '%s\n' \
  "# Gate K $shape task receipt" \
  '' \
  "- Candidate: \`$commit\`" \
  "- Classification: \`$classification\`; formal attempt: \`$formal\`" \
  "- Identity: \`$provider/$model\`, thinking \`$thinking\`, fresh session \`$session_id\`" \
  '- Operator intervention: `none`; retries: `0`' \
  "- Outcome: \`$outcome\`" \
  "- Reason: $outcome_reason" \
  "- Packet manifest SHA-256: \`$manifest_sha\`" \
  "- Transcript SHA-256: \`$transcript_sha\`" \
  "- Commands SHA-256: \`$commands_sha\`" \
  "- Artifacts tree SHA-256: \`$artifacts_sha\`" \
  '' \
  'This task receipt is not a complete cold-run verdict. An eligible subject still' \
  'requires a fresh independent checker and owner/adjudicator disposition.' \
  >"$stage/TASK.md"

mv -- "$stage" "$out"
trap - EXIT
rm -r -- "$tmp_dir"
if [[ $formal == true ]]; then
  repo_root=$(git -C "$script_dir" rev-parse --show-toplevel) ||
    fail 'formal task record cannot resolve the committed attempt-ledger repository'
  close_event=$(python3 "$attempt_validator" next-close \
    "$script_dir/gate-k-formal-attempt-ledger.jsonl" "$attempt_id" \
    "$out" "$outcome" --committed-repo "$repo_root") ||
    fail 'formal task record cannot produce an authenticated close event'
  printf 'PI_TASK_ATTEMPT_CLOSE %s\n' "$close_event"
fi
printf 'GATE_K_TASK_RECORDED outcome=%s output=%s\n' "$outcome" "$out"

#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
"$repo_root/docs/evaluation/test-gate-k-attempt-ledger.sh"
"$repo_root/docs/evaluation/test-gate-k-eval-strictness.sh"
"$repo_root/docs/evaluation/test-gate-k-rc1-refinalization.sh"
"$repo_root/docs/evaluation/test-gate-k-revision6-rehearsal-refinalization.sh"
packet_builder="$repo_root/docs/evaluation/gate-k-eval-packet.sh"
packet_verifier="$repo_root/docs/evaluation/gate-k-eval-verify-packet.sh"
seed_builder="$repo_root/docs/evaluation/gate-k-eval-seed-rehearsal.sh"
task_launcher="$repo_root/docs/evaluation/pi-cold-agent-task.sh"
task_recorder="$repo_root/docs/evaluation/gate-k-eval-record-task.sh"
finalizer="$repo_root/docs/evaluation/gate-k-eval-finalize.sh"
command_deriver="$repo_root/docs/evaluation/gate-k-eval-derive-commands.sh"
transcript_validator="$repo_root/docs/evaluation/gate-k-eval-validate-transcript.py"
adjudication_validator="$repo_root/docs/evaluation/gate-k-eval-validate-adjudication.py"
fake_pi="$repo_root/docs/evaluation/fixtures/fake-pi-cold-agent"
fake_bwrap="$repo_root/docs/evaluation/fixtures/fake-bwrap-pi-cold-agent"
rehearsals="$repo_root/docs/evaluation/rehearsals"

tmp_dir=$(mktemp -d)
candidate="$tmp_dir/candidate"
cleanup() {
  git -C "$repo_root" worktree remove --force "$candidate" >/dev/null 2>&1 || true
  rm -r -- "$tmp_dir"
}
trap cleanup EXIT

commit=$(git -C "$repo_root" rev-parse HEAD)
git -C "$repo_root" worktree add --detach "$candidate" "$commit" >/dev/null

source "$repo_root/docs/evaluation/test-gate-k-eval-tooling-lib.sh"

"$seed_builder" "$candidate" "$commit" "$tmp_dir/debug-seed" >/dev/null

build_author() {
  local out=$1
  "$packet_builder" author \
    --candidate "$candidate" --commit "$commit" \
    --brief "$rehearsals/author-brief.txt" \
    --prompt "$rehearsals/author-prompt.txt" \
    --fixture "$candidate/fixtures/gaol.nomos" \
    --out "$out" >/dev/null
}

build_debug() {
  local out=$1
  "$packet_builder" debug \
    --candidate "$candidate" --commit "$commit" \
    --brief "$rehearsals/debug-brief.txt" \
    --prompt "$rehearsals/debug-prompt.txt" \
    --world "$tmp_dir/debug-seed/gaol.world" \
    --failure-input "$tmp_dir/debug-seed/failing.commands" \
    --run-artifacts "$tmp_dir/debug-seed/failing.run" \
    --forensics "$tmp_dir/debug-seed/forensics" \
    --out "$out" >/dev/null
}

build_author "$tmp_dir/author-1"
build_author "$tmp_dir/author-2"
build_debug "$tmp_dir/debug-1"
build_debug "$tmp_dir/debug-2"
grep -F 'the exact device `/dev/null` is allowed only as a non-information-bearing' \
  "$tmp_dir/author-1/prompt.txt" >/dev/null
grep -F 'Any other attempted outside access makes operational compliance fail' \
  "$tmp_dir/debug-1/prompt.txt" >/dev/null
diff -r "$tmp_dir/author-1" "$tmp_dir/author-2" >/dev/null
diff -r "$tmp_dir/debug-1" "$tmp_dir/debug-2" >/dev/null
"$packet_verifier" "$tmp_dir/author-1" "$commit" >/dev/null
"$packet_verifier" "$tmp_dir/debug-1" "$commit" >/dev/null

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-history"
mkdir -m 755 "$tmp_dir/forensics-with-history/.git"
printf '%s\n' '[core]' >"$tmp_dir/forensics-with-history/.git/config"
assert_blocked 'excluded repository metadata' debug-history-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-history" \
  --out "$tmp_dir/debug-history-packet"

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-credentials"
printf '%s\n' 'not-a-real-secret' >"$tmp_dir/forensics-with-credentials/credentials.txt"
assert_blocked 'credential-like file' debug-credential-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-credentials" \
  --out "$tmp_dir/debug-credential-packet"

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-empty-answer"
mkdir -m 755 "$tmp_dir/forensics-with-empty-answer/expected-answer-is-duplicate-unlock"
assert_blocked 'unbound empty directory' debug-empty-directory-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-empty-answer" \
  --out "$tmp_dir/debug-empty-directory-packet"

cp -R "$tmp_dir/debug-2" "$tmp_dir/declared-history-packet"
mkdir -m 755 "$tmp_dir/declared-history-packet/input/forensics/.git"
printf '%s\n' '[core]' >"$tmp_dir/declared-history-packet/input/forensics/.git/config"
declare_packet_file "$tmp_dir/declared-history-packet" input/forensics/.git/config
assert_blocked 'excluded repository metadata' declared-history-packet \
  "$packet_verifier" "$tmp_dir/declared-history-packet" "$commit"

cp -R "$tmp_dir/debug-2" "$tmp_dir/declared-unrelated-packet"
printf '%s\n' 'unrelated' >"$tmp_dir/declared-unrelated-packet/unrelated.md"
declare_packet_file "$tmp_dir/declared-unrelated-packet" unrelated.md
assert_blocked 'outside the shape allowlist' declared-unrelated-packet \
  "$packet_verifier" "$tmp_dir/declared-unrelated-packet" "$commit"

cp -R "$tmp_dir/debug-2" "$tmp_dir/undeclared-empty-directory-packet"
mkdir -m 755 "$tmp_dir/undeclared-empty-directory-packet/input/forensics/expected-answer-is-duplicate-unlock"
assert_blocked 'undeclared empty directory' declared-empty-directory-packet \
  "$packet_verifier" "$tmp_dir/undeclared-empty-directory-packet" "$commit"

cp -R "$tmp_dir/author-2" "$tmp_dir/tampered-packet"
printf 'tampered\n' >>"$tmp_dir/tampered-packet/prompt.txt"
assert_blocked 'mismatch: prompt.txt' tampered-packet \
  "$packet_verifier" "$tmp_dir/tampered-packet" "$commit"
assert_blocked 'wrong candidate' wrong-candidate \
  "$packet_builder" author --candidate "$candidate" \
  --commit 0000000000000000000000000000000000000000 \
  --brief "$rehearsals/author-brief.txt" --prompt "$rehearsals/author-prompt.txt" \
  --fixture "$candidate/fixtures/gaol.nomos" --out "$tmp_dir/wrong-candidate-packet"
touch "$candidate/untracked-boundary-fixture"
assert_blocked 'candidate worktree is dirty' dirty-candidate \
  "$packet_builder" author --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-brief.txt" --prompt "$rehearsals/author-prompt.txt" \
  --fixture "$candidate/fixtures/gaol.nomos" --out "$tmp_dir/dirty-candidate-packet"
rm -- "$candidate/untracked-boundary-fixture"

launch_task() {
  local shape_name=$1
  local packet=$2
  local raw="$tmp_dir/$shape_name-raw"
  mkdir -m 755 "$raw"
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
    "$task_launcher" claude "$candidate" "$commit" "$packet" \
    "$raw/transcript.ndjson" "$raw/stderr.txt" "$raw/qualification.txt" \
    >"$raw/launcher.txt"
}

record_task() {
  local shape_name=$1
  local packet=$2
  local raw="$tmp_dir/$shape_name-raw"
  "$task_recorder" "$packet" "$raw/transcript.ndjson" "$raw/stderr.txt" \
    "$raw/qualification.txt" "$raw/launcher.txt" "$commit" \
    "$tmp_dir/$shape_name-record" >/dev/null
}

launch_task author-subject "$tmp_dir/author-1"
mkdir -m 755 "$tmp_dir/author-1/workspace/expected-answer-is-watch-lamp"
assert_blocked 'subject artifacts contain an unbound empty directory' \
  recorder-empty-artifact-directory \
  "$task_recorder" "$tmp_dir/author-1" \
  "$tmp_dir/author-subject-raw/transcript.ndjson" \
  "$tmp_dir/author-subject-raw/stderr.txt" \
  "$tmp_dir/author-subject-raw/qualification.txt" \
  "$tmp_dir/author-subject-raw/launcher.txt" "$commit" \
  "$tmp_dir/empty-directory-subject-record"
rmdir "$tmp_dir/author-1/workspace/expected-answer-is-watch-lamp"
record_task author-subject "$tmp_dir/author-1"
jq -e '.outcome == "eligible-for-checker" and .formalAttempt == false' \
  "$tmp_dir/author-subject-record/task-receipt.json" >/dev/null

"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/author-subject-record" \
  --out "$tmp_dir/author-checker-1" >/dev/null
"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/author-subject-record" \
  --out "$tmp_dir/author-checker-2" >/dev/null
diff -r "$tmp_dir/author-checker-1" "$tmp_dir/author-checker-2" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@2' \
  "$tmp_dir/author-checker-1/prompt.txt" >/dev/null
grep -F 'even when the sandbox denied it' \
  "$tmp_dir/author-checker-1/prompt.txt" >/dev/null

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/injected-subject-record"
mkdir -m 755 "$tmp_dir/injected-subject-record/artifacts/source"
printf '%s\n' 'not an answer' >"$tmp_dir/injected-subject-record/artifacts/expected-answer.txt"
printf '%s\n' '{}' >"$tmp_dir/injected-subject-record/artifacts/hidden-mutation.json"
printf '%s\n' 'not source' >"$tmp_dir/injected-subject-record/artifacts/source/kernel.c"
printf '%s\n' 'unrelated' >"$tmp_dir/injected-subject-record/artifacts/unrelated.md"
assert_blocked 'subject artifacts differ from its task receipt' checker-injected-artifacts \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/injected-subject-record" \
  --out "$tmp_dir/injected-checker-packet"

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/empty-directory-subject-record"
mkdir -m 755 "$tmp_dir/empty-directory-subject-record/artifacts/expected-answer-is-watch-lamp"
assert_blocked 'unbound empty directory' checker-empty-directory-artifacts \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/empty-directory-subject-record" \
  --out "$tmp_dir/empty-directory-checker-packet"

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/substituted-command-record"
printf '%s\n' 'SECRET_EXPECTED_ANSWER_AND_UNRECORDED_COMMANDS' \
  >"$tmp_dir/substituted-command-record/commands.json"
assert_blocked 'subject commands.json contains invalid or duplicate-key JSON' checker-substituted-commands \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/substituted-command-record" \
  --out "$tmp_dir/substituted-command-packet"

cp -R "$tmp_dir/author-checker-2" "$tmp_dir/declared-injected-checker-packet"
printf '%s\n' 'not an answer' \
  >"$tmp_dir/declared-injected-checker-packet/subject/artifacts/expected-answer.txt"
declare_packet_file "$tmp_dir/declared-injected-checker-packet" \
  subject/artifacts/expected-answer.txt
assert_blocked 'checker subject artifacts differ from the task receipt' \
  declared-injected-checker \
  "$packet_verifier" "$tmp_dir/declared-injected-checker-packet" "$commit"

cp -R "$tmp_dir/author-checker-2" "$tmp_dir/declared-substituted-command-packet"
printf '%s\n' 'SECRET_EXPECTED_ANSWER_AND_UNRECORDED_COMMANDS' \
  >"$tmp_dir/declared-substituted-command-packet/subject/commands.json"
refresh_packet_file "$tmp_dir/declared-substituted-command-packet" subject/commands.json
assert_blocked 'checker subject commands are invalid' declared-substituted-command \
  "$packet_verifier" "$tmp_dir/declared-substituted-command-packet" "$commit"

launch_task author-checker "$tmp_dir/author-checker-1"
record_task author-checker "$tmp_dir/author-checker-1"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/author-adjudication.json"
mkdir -m 755 "$tmp_dir/author-subject-record/artifacts/unbound-empty-subject-directory"
assert_blocked 'artifact tree contains an unbound empty directory' \
  finalizer-empty-subject-directory \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/empty-subject-directory-run"
rmdir "$tmp_dir/author-subject-record/artifacts/unbound-empty-subject-directory"
mkdir -m 755 "$tmp_dir/author-checker-record/artifacts/unbound-empty-checker-directory"
assert_blocked 'artifact tree contains an unbound empty directory' \
  finalizer-empty-checker-directory \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/empty-checker-directory-run"
rmdir "$tmp_dir/author-checker-record/artifacts/unbound-empty-checker-directory"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/author-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "author"' \
  "$tmp_dir/author-run/result.json" >/dev/null
[[ -f $tmp_dir/author-run/checker/artifacts/reproduction.txt ]]
[[ $(tree_sha "$tmp_dir/author-run/checker/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' \
      "$tmp_dir/author-run/checker/task-receipt.json") ]]

assert_blocked 'output must be outside both immutable packets' finalizer-packet-output \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/author-checker-1/nested-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/retry-checker-record"
jq -c 'if .type == "agent_end" then .willRetry = true else . end' \
  "$tmp_dir/retry-checker-record/transcript.ndjson" \
  >"$tmp_dir/retry-checker-record/transcript.update"
mv -- "$tmp_dir/retry-checker-record/transcript.update" \
  "$tmp_dir/retry-checker-record/transcript.ndjson"
refresh_record_transcript_evidence "$tmp_dir/retry-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/retry-checker-record" "$tmp_dir/retry-adjudication.json"
assert_blocked 'transcript lifecycle or terminal identity is incomplete' finalizer-retry \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/retry-checker-record" \
  "$tmp_dir/retry-adjudication.json" "$tmp_dir/retry-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/accounting-checker-record"
jq -S -c '.assistantTurns = 999 | .providerReportedTokens = 0' \
  "$tmp_dir/accounting-checker-record/accounting.json" \
  >"$tmp_dir/accounting-checker-record/accounting.update"
mv -- "$tmp_dir/accounting-checker-record/accounting.update" \
  "$tmp_dir/accounting-checker-record/accounting.json"
jq -S -c '.accounting.assistantTurns = 999 | .accounting.providerReportedTokens = 0' \
  "$tmp_dir/accounting-checker-record/task-receipt.json" \
  >"$tmp_dir/accounting-checker-record/task-receipt.update"
mv -- "$tmp_dir/accounting-checker-record/task-receipt.update" \
  "$tmp_dir/accounting-checker-record/task-receipt.json"
refresh_record_runtime_evidence "$tmp_dir/accounting-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/accounting-checker-record" "$tmp_dir/accounting-adjudication.json"
assert_blocked 'accounting does not derive from the transcript' finalizer-accounting \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/accounting-checker-record" \
  "$tmp_dir/accounting-adjudication.json" "$tmp_dir/accounting-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/truncated-qualification-record"
sed '/^PI_VERSION /d' "$tmp_dir/truncated-qualification-record/pi-qualification.txt" \
  >"$tmp_dir/truncated-qualification-record/pi-qualification.update"
mv -- "$tmp_dir/truncated-qualification-record/pi-qualification.update" \
  "$tmp_dir/truncated-qualification-record/pi-qualification.txt"
refresh_record_receipt_digests "$tmp_dir/truncated-qualification-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/truncated-qualification-record" "$tmp_dir/truncated-qualification.json"
assert_blocked 'qualification client version differs from task receipt' \
  finalizer-truncated-qualification \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/truncated-qualification-record" \
  "$tmp_dir/truncated-qualification.json" "$tmp_dir/truncated-qualification-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/truncated-launcher-record"
sed '/^PI_TASK_EVENTS_SHA256 /d' "$tmp_dir/truncated-launcher-record/launcher.txt" \
  >"$tmp_dir/truncated-launcher-record/launcher.update"
mv -- "$tmp_dir/truncated-launcher-record/launcher.update" \
  "$tmp_dir/truncated-launcher-record/launcher.txt"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/truncated-launcher-record" "$tmp_dir/truncated-launcher.json"
assert_blocked 'launcher does not bind the transcript' finalizer-truncated-launcher \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/truncated-launcher-record" \
  "$tmp_dir/truncated-launcher.json" "$tmp_dir/truncated-launcher-run"

"$finalizer" --validate-task-record "$tmp_dir/author-subject-record" >/dev/null
"$finalizer" --validate-task-record "$tmp_dir/author-checker-record" >/dev/null

source "$repo_root/docs/evaluation/test-gate-k-eval-finalizer-adversarial.sh"

jq -c -s '
  to_entries as $events |
  ($events | map(select(.value.type == "tool_execution_start"))[0]) as $start |
  ($events | map(select(
    .value.type == "tool_execution_end" and
    .value.toolCallId == $start.value.toolCallId))[0]) as $end |
  [$end.value] + [$events[] | select(.key != $end.key) | .value] | .[]
  ' "$tmp_dir/author-checker-record/transcript.ndjson" \
  >"$tmp_dir/end-before-start-transcript.ndjson"
assert_blocked 'tool end before its matching start' transcript-end-before-start \
  "$command_deriver" "$tmp_dir/end-before-start-transcript.ndjson"

jq -S -c '.candidateCommit = "0000000000000000000000000000000000000000"' \
  "$tmp_dir/author-adjudication.json" >"$tmp_dir/wrong-candidate-adjudication.json"
assert_blocked 'candidateCommit differs from the subject task receipt' \
  wrong-candidate-adjudication \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/wrong-candidate-adjudication.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/unrecorded-command-checker-record"
jq -S -c '
  .commands += [(.commands[0] |
    .ordinal = 1 |
    .toolCallId = (.toolCallId + "_not_in_transcript"))]
  ' "$tmp_dir/unrecorded-command-checker-record/commands.json" \
  >"$tmp_dir/unrecorded-command-checker-record/commands.update"
mv -- "$tmp_dir/unrecorded-command-checker-record/commands.update" \
  "$tmp_dir/unrecorded-command-checker-record/commands.json"
refresh_record_receipt_digests "$tmp_dir/unrecorded-command-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/unrecorded-command-checker-record" "$tmp_dir/unrecorded-command-adjudication.json"
assert_blocked 'commands do not derive exactly from transcript' \
  finalizer-unrecorded-transcript-command \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/unrecorded-command-checker-record" "$tmp_dir/unrecorded-command-adjudication.json" \
  "$tmp_dir/unrecorded-command-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/malformed-command-checker-record"
jq -S -c '.commands[0].ordinal = true' \
  "$tmp_dir/malformed-command-checker-record/commands.json" \
  >"$tmp_dir/malformed-command-checker-record/commands.update"
mv -- "$tmp_dir/malformed-command-checker-record/commands.update" \
  "$tmp_dir/malformed-command-checker-record/commands.json"
refresh_record_receipt_digests "$tmp_dir/malformed-command-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/malformed-command-checker-record" "$tmp_dir/malformed-command-adjudication.json"
assert_blocked 'commands are not contiguous at ordinal 0' malformed-command-adjudication \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/malformed-command-checker-record" "$tmp_dir/malformed-command-adjudication.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/float-command-checker-record"
sed '0,/"ordinal":0/s//"ordinal":0.0/' \
  "$tmp_dir/float-command-checker-record/commands.json" \
  >"$tmp_dir/float-command-checker-record/commands.update"
mv -- "$tmp_dir/float-command-checker-record/commands.update" \
  "$tmp_dir/float-command-checker-record/commands.json"
refresh_record_receipt_digests "$tmp_dir/float-command-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/float-command-checker-record" "$tmp_dir/float-command-adjudication.json"
assert_blocked 'commands are not contiguous at ordinal 0' float-command-adjudication \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/float-command-checker-record" "$tmp_dir/float-command-adjudication.json"

sed -E 's/"subject":([0-9]+)/"subject":\1.0/' \
  "$tmp_dir/author-adjudication.json" >"$tmp_dir/float-count-adjudication.json"
assert_blocked 'positive integer subject/checker counts' float-count-adjudication \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/float-count-adjudication.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/malformed-result-checker-record"
jq -S -c '.commands = [null]' \
  "$tmp_dir/malformed-result-checker-record/artifacts/checker.json" \
  >"$tmp_dir/malformed-result-checker-record/artifacts/checker.update"
mv -- "$tmp_dir/malformed-result-checker-record/artifacts/checker.update" \
  "$tmp_dir/malformed-result-checker-record/artifacts/checker.json"
install -m 644 "$tmp_dir/malformed-result-checker-record/artifacts/checker.json" \
  "$tmp_dir/author-checker-1/output/checker.json"
refresh_record_receipt_digests "$tmp_dir/malformed-result-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/malformed-result-checker-record" "$tmp_dir/malformed-result-adjudication.json"
assert_blocked 'checker result does not satisfy a declared exact schema' \
  record-only-malformed-checker-result "$finalizer" --validate-task-record \
  "$tmp_dir/malformed-result-checker-record"
assert_blocked 'checker command 0 is not an object' finalizer-malformed-checker-result \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/malformed-result-checker-record" "$tmp_dir/malformed-result-adjudication.json" \
  "$tmp_dir/malformed-result-run"
install -m 644 "$tmp_dir/author-checker-record/artifacts/checker.json" \
  "$tmp_dir/author-checker-1/output/checker.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/duplicate-result-checker-record"
sed '0,/"verdict":"pass"/s//"verdict":"reject","verdict":"pass"/' \
  "$tmp_dir/duplicate-result-checker-record/artifacts/checker.json" \
  >"$tmp_dir/duplicate-result-checker-record/artifacts/checker.update"
mv -- "$tmp_dir/duplicate-result-checker-record/artifacts/checker.update" \
  "$tmp_dir/duplicate-result-checker-record/artifacts/checker.json"
install -m 644 "$tmp_dir/duplicate-result-checker-record/artifacts/checker.json" \
  "$tmp_dir/author-checker-1/output/checker.json"
refresh_record_receipt_digests "$tmp_dir/duplicate-result-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/duplicate-result-checker-record" "$tmp_dir/duplicate-result-adjudication.json"
assert_blocked 'duplicate JSON key: verdict' finalizer-duplicate-checker-result \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/duplicate-result-checker-record" "$tmp_dir/duplicate-result-adjudication.json" \
  "$tmp_dir/duplicate-result-run"
install -m 644 "$tmp_dir/author-checker-record/artifacts/checker.json" \
  "$tmp_dir/author-checker-1/output/checker.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/changed-marker-checker-record"
jq -S -c '
  .files |= map(
    if .path == ".nomos-candidate-commit"
    then .sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
    else .
    end)
  ' "$tmp_dir/changed-marker-checker-record/packet-manifest.json" \
  >"$tmp_dir/changed-marker-checker-record/packet-manifest.update"
mv -- "$tmp_dir/changed-marker-checker-record/packet-manifest.update" \
  "$tmp_dir/changed-marker-checker-record/packet-manifest.json"
changed_manifest_sha=$(sha256sum \
  "$tmp_dir/changed-marker-checker-record/packet-manifest.json" | cut -d' ' -f1)
jq -S -c --arg digest "$changed_manifest_sha" '.packetManifestSha256 = $digest' \
  "$tmp_dir/changed-marker-checker-record/boundary.json" \
  >"$tmp_dir/changed-marker-checker-record/boundary.update"
mv -- "$tmp_dir/changed-marker-checker-record/boundary.update" \
  "$tmp_dir/changed-marker-checker-record/boundary.json"
refresh_record_runtime_evidence "$tmp_dir/changed-marker-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/changed-marker-checker-record" "$tmp_dir/changed-marker-adjudication.json"
assert_blocked 'packet candidate marker differs from task receipt' finalizer-changed-marker \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/changed-marker-checker-record" "$tmp_dir/changed-marker-adjudication.json" \
  "$tmp_dir/changed-marker-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/writable-path-checker-record"
jq -S -c '.packet.writablePaths = ["input"]' \
  "$tmp_dir/writable-path-checker-record/plan.json" \
  >"$tmp_dir/writable-path-checker-record/plan.update"
mv -- "$tmp_dir/writable-path-checker-record/plan.update" \
  "$tmp_dir/writable-path-checker-record/plan.json"
refresh_packet_file "$tmp_dir/writable-path-checker-record" plan.json
jq -S -c '.writablePaths = ["input"]' \
  "$tmp_dir/writable-path-checker-record/packet-manifest.json" \
  >"$tmp_dir/writable-path-checker-record/packet-manifest.update"
mv -- "$tmp_dir/writable-path-checker-record/packet-manifest.update" \
  "$tmp_dir/writable-path-checker-record/packet-manifest.json"
writable_manifest_sha=$(sha256sum \
  "$tmp_dir/writable-path-checker-record/packet-manifest.json" | cut -d' ' -f1)
jq -S -c --arg digest "$writable_manifest_sha" '
  .packetManifestSha256 = $digest |
  .writablePaths = ["input"] |
  .sandbox.checks.declaredWritablePaths = ["input"]
  ' "$tmp_dir/writable-path-checker-record/boundary.json" \
  >"$tmp_dir/writable-path-checker-record/boundary.update"
mv -- "$tmp_dir/writable-path-checker-record/boundary.update" \
  "$tmp_dir/writable-path-checker-record/boundary.json"
refresh_record_runtime_evidence "$tmp_dir/writable-path-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/writable-path-checker-record" "$tmp_dir/writable-path-adjudication.json"
assert_blocked 'plan writable path differs' finalizer-writable-path \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/writable-path-checker-record" "$tmp_dir/writable-path-adjudication.json" \
  "$tmp_dir/writable-path-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/relabeled-checker-record"
relabeled_commit=$(printf '%s\n' 'fixture alternate candidate' | env \
  GIT_AUTHOR_NAME='Gate K fixture' GIT_AUTHOR_EMAIL='fixture@invalid' \
  GIT_AUTHOR_DATE='1970-01-01T00:00:00Z' \
  GIT_COMMITTER_NAME='Gate K fixture' GIT_COMMITTER_EMAIL='fixture@invalid' \
  GIT_COMMITTER_DATE='1970-01-01T00:00:00Z' \
  git -C "$repo_root" commit-tree "$(git -C "$repo_root" rev-parse HEAD^{tree})")
relabeled_marker_sha=$(printf '%s\n' "$relabeled_commit" | sha256sum | cut -d' ' -f1)
jq -S -c --arg commit "$relabeled_commit" '.candidateCommit = $commit' \
  "$tmp_dir/relabeled-checker-record/task-receipt.json" \
  >"$tmp_dir/relabeled-checker-record/task-receipt.update"
mv -- "$tmp_dir/relabeled-checker-record/task-receipt.update" \
  "$tmp_dir/relabeled-checker-record/task-receipt.json"
jq -S -c --arg commit "$relabeled_commit" '.candidate.commit = $commit' \
  "$tmp_dir/relabeled-checker-record/plan.json" \
  >"$tmp_dir/relabeled-checker-record/plan.update"
mv -- "$tmp_dir/relabeled-checker-record/plan.update" \
  "$tmp_dir/relabeled-checker-record/plan.json"
refresh_packet_file "$tmp_dir/relabeled-checker-record" plan.json
jq -S -c --arg commit "$relabeled_commit" --arg marker "$relabeled_marker_sha" '
  .candidateCommit = $commit |
  .files |= map(if .path == ".nomos-candidate-commit" then .sha256 = $marker else . end)
  ' "$tmp_dir/relabeled-checker-record/packet-manifest.json" \
  >"$tmp_dir/relabeled-checker-record/packet-manifest.update"
mv -- "$tmp_dir/relabeled-checker-record/packet-manifest.update" \
  "$tmp_dir/relabeled-checker-record/packet-manifest.json"
relabeled_manifest_sha=$(sha256sum \
  "$tmp_dir/relabeled-checker-record/packet-manifest.json" | cut -d' ' -f1)
jq -S -c --arg commit "$relabeled_commit" --arg manifest "$relabeled_manifest_sha" '
  .targetCommit = $commit | .packetManifestSha256 = $manifest
  ' "$tmp_dir/relabeled-checker-record/boundary.json" \
  >"$tmp_dir/relabeled-checker-record/boundary.update"
mv -- "$tmp_dir/relabeled-checker-record/boundary.update" \
  "$tmp_dir/relabeled-checker-record/boundary.json"
sed "s/^PI_TARGET_COMMIT .*/PI_TARGET_COMMIT $relabeled_commit/" \
  "$tmp_dir/relabeled-checker-record/pi-qualification.txt" \
  >"$tmp_dir/relabeled-checker-record/pi-qualification.update"
mv -- "$tmp_dir/relabeled-checker-record/pi-qualification.update" \
  "$tmp_dir/relabeled-checker-record/pi-qualification.txt"
refresh_record_runtime_evidence "$tmp_dir/relabeled-checker-record"
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/relabeled-checker-record" "$tmp_dir/relabeled-adjudication.json"
assert_blocked 'qualification receipt is incomplete' \
  finalizer-relabeled-candidate \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/relabeled-checker-record" "$tmp_dir/relabeled-adjudication.json" \
  "$tmp_dir/relabeled-run"

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/substitute-final-subject-record"
rebind_recorded_commands "$tmp_dir/substitute-final-subject-record" \
  'cat /workspace/prompt.txt'
write_pass_adjudication "$tmp_dir/substitute-final-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/substitute-final-subject-adjudication.json"
assert_blocked 'checker packet does not bind the supplied subject task receipt' \
  finalizer-substituted-subject \
  "$finalizer" "$tmp_dir/substitute-final-subject-record" \
  "$tmp_dir/author-checker-record" "$tmp_dir/substitute-final-subject-adjudication.json" \
  "$tmp_dir/substitute-final-subject-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/phantom-candidate-checker-record"
jq -S -c '.candidateCommit = "ffffffffffffffffffffffffffffffffffffffff"' \
  "$tmp_dir/phantom-candidate-checker-record/task-receipt.json" \
  >"$tmp_dir/phantom-candidate-checker-record/task-receipt.update"
mv -- "$tmp_dir/phantom-candidate-checker-record/task-receipt.update" \
  "$tmp_dir/phantom-candidate-checker-record/task-receipt.json"
assert_blocked 'qualification candidate differs from task receipt' finalizer-phantom-candidate \
  "$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/phantom-candidate-checker-record" "$tmp_dir/author-adjudication.json" \
  "$tmp_dir/phantom-candidate-run"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/outside-path-checker-record"
rebind_recorded_commands "$tmp_dir/outside-path-checker-record" \
  'cat /workspace/brief.txt 2>/dev/null'
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/outside-path-checker-record" "$tmp_dir/outside-path-pass-adjudication.json"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/outside-path-checker-record" \
  "$tmp_dir/outside-path-pass-adjudication.json" "$tmp_dir/outside-path-pass-run" >/dev/null
jq -e '
  .schema == "nomos.gate_k.run_result@2" and
  .protocolRevision == 6 and .verdict == "pass" and
  .adjudication.findings == [] and
  .records.checker.dimensions.operational_compliance.verdict == "pass"
  ' "$tmp_dir/outside-path-pass-run/result.json" >/dev/null

write_outside_path_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/outside-path-checker-record" checker 0 /dev/null \
  "$tmp_dir/dev-null-invalid-adjudication.json"
assert_blocked 'declared /dev/null exception as forbidden' dev-null-finding \
  python3 "$adjudication_validator" "$tmp_dir/author-subject-record" \
  "$tmp_dir/outside-path-checker-record" "$tmp_dir/dev-null-invalid-adjudication.json"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/forbidden-device-checker-record"
rebind_recorded_commands "$tmp_dir/forbidden-device-checker-record" \
  'cat /workspace/brief.txt 2>/dev/zero'
write_outside_path_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/forbidden-device-checker-record" checker 0 /dev/zero \
  "$tmp_dir/forbidden-device-adjudication.json"
"$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/forbidden-device-checker-record" \
  "$tmp_dir/forbidden-device-adjudication.json" \
  "$tmp_dir/forbidden-device-run" >/dev/null
jq -e '
  .verdict == "fail" and
  .adjudication.verdict == "fail" and
  .adjudication.findings[0].pathToken == "/dev/zero" and
  .records.checker.dimensions.operational_compliance.verdict == "fail" and
  .records.checker.dimensions.independence_integrity.verdict == "pass"
  ' "$tmp_dir/forbidden-device-run/result.json" >/dev/null

cp -R "$tmp_dir/forbidden-device-checker-record" "$tmp_dir/inconclusive-outside-path-record"
jq -S -c '.outcome = "inconclusive" | .outcomeReason = "fixture transport failure"' \
  "$tmp_dir/inconclusive-outside-path-record/task-receipt.json" \
  >"$tmp_dir/inconclusive-outside-path-record/task-receipt.update"
mv -- "$tmp_dir/inconclusive-outside-path-record/task-receipt.update" \
  "$tmp_dir/inconclusive-outside-path-record/task-receipt.json"
sed 's/^PI_TASK_STATUS 0$/PI_TASK_STATUS 1/' \
  "$tmp_dir/inconclusive-outside-path-record/launcher.txt" \
  >"$tmp_dir/inconclusive-outside-path-record/launcher.update"
mv -- "$tmp_dir/inconclusive-outside-path-record/launcher.update" \
  "$tmp_dir/inconclusive-outside-path-record/launcher.txt"
write_outside_path_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/inconclusive-outside-path-record" checker 0 /dev/zero \
  "$tmp_dir/inconclusive-outside-path-adjudication.json"
"$finalizer" "$tmp_dir/author-subject-record" \
  "$tmp_dir/inconclusive-outside-path-record" \
  "$tmp_dir/inconclusive-outside-path-adjudication.json" \
  "$tmp_dir/inconclusive-outside-path-run" >/dev/null
jq -e '.verdict == "fail" and .adjudication.verdict == "fail"' \
  "$tmp_dir/inconclusive-outside-path-run/result.json" >/dev/null

assert_blocked 'output must be outside both immutable task records' finalizer-nested-output \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/forbidden-device-checker-record" \
  "$tmp_dir/forbidden-device-adjudication.json" \
  "$tmp_dir/author-subject-record/final-output"

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/quoted-path-checker-record"
rebind_recorded_commands "$tmp_dir/quoted-path-checker-record" \
  'grep -E '\''/(tmp|dev|home|etc)(/|[[:space:]])'\'' commands.json; sed -n '\''/## Reproduction commands/,/^```$/p'\'' report.md; python3 -c "print('\''scan data: /dev/null'\'')"'
write_pass_adjudication "$tmp_dir/author-subject-record" \
  "$tmp_dir/quoted-path-checker-record" "$tmp_dir/quoted-path-adjudication.json"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/quoted-path-checker-record" \
  "$tmp_dir/quoted-path-adjudication.json" "$tmp_dir/quoted-path-pass-run" >/dev/null
jq -e '
  .verdict == "pass" and
  .adjudication.verdict == "pass" and
  .adjudication.findings == []
  ' "$tmp_dir/quoted-path-pass-run/result.json" >/dev/null

source "$repo_root/docs/evaluation/test-gate-k-eval-revision6-adjudication.sh"

printf '%s\n' 'tampered immutable brief' >>"$tmp_dir/author-checker-1/brief.txt"
assert_blocked 'recorded immutable packet does not satisfy its complete shape' finalizer-packet-tamper \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/packet-tamper-run"
install -m 644 "$tmp_dir/author-checker-2/brief.txt" "$tmp_dir/author-checker-1/brief.txt"
printf '\n' >>"$tmp_dir/author-checker-1/.nomos-candidate-commit"
assert_blocked 'recorded immutable packet does not satisfy its complete shape' finalizer-marker-bytes \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  "$tmp_dir/author-adjudication.json" "$tmp_dir/marker-bytes-run"
install -m 644 "$tmp_dir/author-checker-2/.nomos-candidate-commit" \
  "$tmp_dir/author-checker-1/.nomos-candidate-commit"

launch_task debug-subject "$tmp_dir/debug-1"
record_task debug-subject "$tmp_dir/debug-1"
"$packet_builder" debug-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-checker-brief.txt" \
  --prompt "$rehearsals/debug-checker-prompt.txt" \
  --subject-record "$tmp_dir/debug-subject-record" \
  --debug-evidence "$tmp_dir/debug-2/input" \
  --hidden-mutation "$tmp_dir/debug-seed/hidden-mutation.json" \
  --out "$tmp_dir/debug-checker" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@2' \
  "$tmp_dir/debug-checker/prompt.txt" >/dev/null
grep -F 'even when the sandbox denied it' \
  "$tmp_dir/debug-checker/prompt.txt" >/dev/null
launch_task debug-checker "$tmp_dir/debug-checker"
record_task debug-checker "$tmp_dir/debug-checker"
write_pass_adjudication "$tmp_dir/debug-subject-record" \
  "$tmp_dir/debug-checker-record" "$tmp_dir/debug-adjudication.json"
"$finalizer" "$tmp_dir/debug-subject-record" "$tmp_dir/debug-checker-record" \
  "$tmp_dir/debug-adjudication.json" "$tmp_dir/debug-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "debug"' \
  "$tmp_dir/debug-run/result.json" >/dev/null
[[ -f $tmp_dir/debug-run/checker/artifacts/reproduction.txt ]]
[[ $(tree_sha "$tmp_dir/debug-run/checker/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' \
      "$tmp_dir/debug-run/checker/task-receipt.json") ]]

negative_task() {
  local mode=$1
  local expected=$2
  cp -R "$tmp_dir/author-2" "$tmp_dir/negative-$mode-packet"
  mkdir -m 755 "$tmp_dir/negative-$mode-raw"
  assert_blocked "$expected" "task-$mode" env \
    FAKE_PI_TASK_MODE="$mode" PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
    "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/negative-$mode-packet" \
    "$tmp_dir/negative-$mode-raw/transcript.ndjson" \
    "$tmp_dir/negative-$mode-raw/stderr.txt" \
    "$tmp_dir/negative-$mode-raw/qualification.txt"
}

negative_task missing-identity 'event stream has no matching terminal assistant identity'
negative_task missing-accounting 'expected one accounting record, found 0'
negative_task packet-hash-mismatch 'task boundary record does not prove the declared packet isolation'
negative_task harness-mutation 'immutable packet file changed: brief.txt'
negative_task wrong-provider 'task boundary record does not prove the declared packet isolation'
negative_task wrong-model 'task boundary record does not prove the declared packet isolation'
negative_task wrong-thinking 'task boundary record does not prove the declared packet isolation'
negative_task reused-session 'task boundary record does not prove the declared packet isolation'
negative_task forbidden-tool 'task boundary record does not prove the declared packet isolation'
negative_task outside-read-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task outside-write-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task temporary-write-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task extra-device-exposed 'task boundary record does not prove the declared packet isolation'
negative_task null-read-denied 'task boundary record does not prove the declared packet isolation'
negative_task null-write-denied 'task boundary record does not prove the declared packet isolation'
negative_task missing-session 'raw task stream has invalid JSON or misplaced provider signatures'

cp -R "$tmp_dir/author-2" "$tmp_dir/negative-leak-packet"
mkdir -m 755 "$tmp_dir/negative-leak-raw"
assert_blocked 'provider output leaked credential environment variable NOMOS_PI_TEST_SECRET' \
  task-leak env FAKE_PI_TASK_MODE=leak-secret NOMOS_PI_TEST_SECRET=fixture-secret-never-print \
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/negative-leak-packet" \
  "$tmp_dir/negative-leak-raw/transcript.ndjson" "$tmp_dir/negative-leak-raw/stderr.txt" \
  "$tmp_dir/negative-leak-raw/qualification.txt"

cp -R "$tmp_dir/author-2" "$tmp_dir/absent-command-packet"
FAKE_PI_TASK_MODE=absent-command PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/absent-command-packet" \
  "$tmp_dir/absent-command-events" "$tmp_dir/absent-command-stderr" \
  "$tmp_dir/absent-command-qualification" >"$tmp_dir/absent-command-launcher"
assert_blocked 'transcript has no tool starts' absent-command-record \
  "$task_recorder" "$tmp_dir/absent-command-packet" "$tmp_dir/absent-command-events" \
  "$tmp_dir/absent-command-stderr" "$tmp_dir/absent-command-qualification" \
  "$tmp_dir/absent-command-launcher" "$commit" "$tmp_dir/absent-command-record"

printf 'gate-k evaluation tooling offline harness: PASS\n'

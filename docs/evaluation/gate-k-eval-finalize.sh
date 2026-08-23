#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k eval finalizer: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 6 ]] || fail \
  'usage: gate-k-eval-finalize.sh SUBJECT_RECORD CHECKER_RECORD VERDICT ADJUDICATOR OWNER OUT'
subject=$1
checker=$2
verdict=$3
adjudicator=$4
owner=$5
out=$6
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
command_boundary_audit="$script_dir/gate-k-eval-command-boundary.py"

case $verdict in
  pass | fail | assisted | inconclusive) ;;
  *) fail "invalid verdict: $verdict" ;;
esac
[[ -n $adjudicator && -n $owner ]] || fail 'adjudicator and owner must be recorded'
for record in "$subject" "$checker"; do
  [[ -d $record && ! -L $record ]] || fail "task record is absent: $record"
  for file in task-receipt.json plan.json packet-manifest.json prompt.txt transcript.ndjson \
    commands.json accounting.json boundary.json TASK.md pi-qualification.txt launcher.txt pi-stderr.txt; do
    [[ -f $record/$file && ! -L $record/$file ]] || fail "task record file is absent: $record/$file"
  done
  [[ -d $record/artifacts && ! -L $record/artifacts ]] || fail "artifact tree is absent: $record"
  [[ -z $(find "$record" -type l -print -quit) ]] || fail "task record contains a symlink: $record"
  [[ -z $(find "$record" ! -type f ! -type d -print -quit) ]] ||
    fail "task record contains a special entry: $record"
  actual_top=$(find "$record" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort)
  expected_top=$(printf '%s\n' TASK.md accounting.json artifacts boundary.json commands.json \
    launcher.txt packet-manifest.json pi-qualification.txt pi-stderr.txt plan.json prompt.txt \
    task-receipt.json transcript.ndjson | sort)
  [[ $actual_top == "$expected_top" ]] || fail "task record top-level allowlist mismatch: $record"
  empty_artifact_directory=$(find "$record/artifacts" -mindepth 1 -type d -empty -print -quit)
  [[ -z $empty_artifact_directory ]] ||
    fail "artifact tree contains an unbound empty directory: ${empty_artifact_directory#"$record/artifacts"/}"
done
[[ ! -e $out ]] || fail "output already exists: $out"
out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"

tree_sha() {
  local root=$1
  find "$root" -type f -printf '%P\0' | sort -z |
    while IFS= read -r -d '' relative; do
      sha256sum "$root/$relative" | sed "s#  $root/#  #"
    done | sha256sum | cut -d' ' -f1
}

for record in "$subject" "$checker"; do
  jq -e '
    .schema == "nomos.gate_k.task_receipt@1" and
    .identity.freshEphemeralSession == true and
    .identity.client == "Pi" and
    (.identity.clientVersion | type) == "string" and
    .identity.mode == "json" and
    .operatorRetries == 0 and
    .disclosures.persistedSession == false and
    .disclosures.projectMemory == false and
    .disclosures.personalContext == false and
    .disclosures.contextFiles == [] and
    .disclosures.connectors == [] and
    .disclosures.webAccess == false and
    .disclosures.toolNetworkAccess == false and
    .disclosures.activeTools == ["bash"] and
    .disclosures.repositoryMounted == false
    ' "$record/task-receipt.json" >/dev/null || fail "task receipt identity is incomplete: $record"
  [[ $(sha256sum "$record/packet-manifest.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.packetManifestSha256' "$record/task-receipt.json") ]] ||
    fail "packet-manifest digest differs from task receipt: $record"
  [[ $(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1) == \
      $(jq -r '.digests.transcriptSha256' "$record/task-receipt.json") ]] ||
    fail "transcript digest differs from task receipt: $record"
  [[ $(sha256sum "$record/commands.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.commandsSha256' "$record/task-receipt.json") ]] ||
    fail "commands digest differs from task receipt: $record"
  [[ $(tree_sha "$record/artifacts") == \
      $(jq -r '.digests.artifactsTreeSha256' "$record/task-receipt.json") ]] ||
    fail "artifact-tree digest differs from task receipt: $record"
  [[ $(sha256sum "$record/boundary.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.boundarySha256' "$record/task-receipt.json") ]] ||
    fail "boundary digest differs from task receipt: $record"
  [[ $(sha256sum "$record/pi-qualification.txt" | cut -d' ' -f1) == \
      $(jq -r '.digests.qualificationSha256' "$record/task-receipt.json") ]] ||
    fail "qualification digest differs from task receipt: $record"
done

subject_shape=$(jq -r '.shape' "$subject/task-receipt.json")
checker_shape=$(jq -r '.shape' "$checker/task-receipt.json")
case "$subject_shape:$checker_shape" in
  author:author-checker | debug:debug-checker) ;;
  *) fail "subject/checker shapes do not pair: $subject_shape / $checker_shape" ;;
esac
subject_commit=$(jq -r '.candidateCommit' "$subject/task-receipt.json")
checker_commit=$(jq -r '.candidateCommit' "$checker/task-receipt.json")
[[ $subject_commit =~ ^[0-9a-f]{40}$ && $subject_commit == "$checker_commit" ]] ||
  fail 'subject and checker candidate commits differ'
subject_class=$(jq -r '.classification' "$subject/task-receipt.json")
checker_class=$(jq -r '.classification' "$checker/task-receipt.json")
subject_formal=$(jq -r '.formalAttempt' "$subject/task-receipt.json")
checker_formal=$(jq -r '.formalAttempt' "$checker/task-receipt.json")
[[ $subject_class == "$checker_class" && $subject_formal == "$checker_formal" ]] ||
  fail 'subject and checker attempt classifications differ'
subject_session=$(jq -r '.identity.sessionId' "$subject/task-receipt.json")
checker_session=$(jq -r '.identity.sessionId' "$checker/task-receipt.json")
[[ $subject_session != "$checker_session" ]] || fail 'checker reused the subject session'

subject_outcome=$(jq -r '.outcome' "$subject/task-receipt.json")
checker_outcome=$(jq -r '.outcome' "$checker/task-receipt.json")
checker_result="$checker/artifacts/checker.json"
[[ -f $checker_result && ! -L $checker_result ]] || fail 'checker did not publish artifacts/checker.json'
jq -e '
  .schema == "nomos.gate_k.checker_result@1" and
  (.verdict == "pass" or .verdict == "reject") and
  (.commands | type) == "array" and (.commands | length) > 0 and
  (.reasons | type) == "array" and (.reasons | length) > 0
  ' "$checker_result" >/dev/null || fail 'checker result is incomplete'
checker_verdict=$(jq -r '.verdict' "$checker_result")
subject_boundary_json=$(python3 "$command_boundary_audit" "$subject/commands.json" subject) ||
  fail 'subject command boundary audit failed'
checker_boundary_json=$(python3 "$command_boundary_audit" "$checker/commands.json" checker) ||
  fail 'checker command boundary audit failed'
subject_boundary_verdict=$(jq -r '.verdict' <<<"$subject_boundary_json")
checker_boundary_verdict=$(jq -r '.verdict' <<<"$checker_boundary_json")

logical_verdict=pass
logical_reason='subject completed within protocol and the independent checker passed'
if [[ $subject_outcome == inconclusive || $checker_outcome == inconclusive ]]; then
  logical_verdict=inconclusive
  logical_reason='a subject or checker transport/harness failure prevented fair completion'
elif [[ $(jq -r '.operatorIntervention' "$subject/task-receipt.json") != none ||
        $(jq -r '.operatorIntervention' "$checker/task-receipt.json") != none ]]; then
  logical_verdict=assisted
  logical_reason='substantive operator intervention was recorded'
elif [[ $subject_boundary_verdict != pass || $checker_boundary_verdict != pass ]]; then
  logical_verdict=fail
  logical_reason='recorded subject or checker commands requested a path outside the declared workspace'
elif [[ $subject_outcome != eligible-for-checker || $checker_outcome != completed-checker ||
        $checker_verdict != pass ]]; then
  logical_verdict=fail
  logical_reason='the subject, checker transport, protocol, or checker result failed'
fi
[[ $verdict == "$logical_verdict" ]] ||
  fail "requested verdict $verdict contradicts mechanically derived verdict $logical_verdict"

if [[ $subject_class == rehearsal ]]; then
  [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
      "$subject/task-receipt.json") == anthropic/claude-opus-5/high ]] ||
    fail 'rehearsal subject is not the supplemental Claude Opus 5 high route'
  [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
      "$checker/task-receipt.json") == anthropic/claude-opus-5/high ]] ||
    fail 'rehearsal checker is not the supplemental Claude Opus 5 high route'
  [[ $subject_formal == false && $checker_formal == false ]] ||
    fail 'rehearsal was marked as a formal attempt'
else
  case $subject_shape in
    author)
      [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
          "$subject/task-receipt.json") == antigravity/gemini-3.7-flash/high ]] ||
        fail 'formal author identity differs from the roster'
      [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
          "$checker/task-receipt.json") == deepseek/deepseek-v4-flash-vision-exp/max ]] ||
        fail 'formal author checker identity differs from the roster'
      ;;
    debug)
      [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
          "$subject/task-receipt.json") == deepseek/deepseek-v4-flash-vision-exp/max ]] ||
        fail 'formal debugger identity differs from the roster'
      [[ $(jq -r '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
          "$checker/task-receipt.json") == antigravity/gemini-3.7-flash/high ]] ||
        fail 'formal debug checker identity differs from the roster'
      ;;
  esac
fi

stage=$(mktemp -d "$out_parent/.gate-k-eval-run.XXXXXX")
cleanup() {
  rm -r -- "$stage"
}
trap cleanup EXIT
install -d -m 755 "$stage/artifacts" "$stage/subject" "$stage/checker"
for file in plan.json packet-manifest.json prompt.txt transcript.ndjson commands.json; do
  install -m 644 "$subject/$file" "$stage/$file"
done
while IFS= read -r -d '' relative; do
  relative=${relative#./}
  if [[ -d $subject/artifacts/$relative ]]; then
    install -d -m 755 "$stage/artifacts/$relative"
  else
    install -d -m 755 "$(dirname "$stage/artifacts/$relative")"
    install -m 644 "$subject/artifacts/$relative" "$stage/artifacts/$relative"
  fi
done < <(cd "$subject/artifacts" && find . -mindepth 1 -print0 | sort -z)
for file in TASK.md task-receipt.json accounting.json boundary.json pi-qualification.txt \
  launcher.txt pi-stderr.txt; do
  install -m 644 "$subject/$file" "$stage/subject/$file"
done
for file in TASK.md task-receipt.json plan.json packet-manifest.json prompt.txt \
  transcript.ndjson commands.json accounting.json boundary.json pi-qualification.txt \
  launcher.txt pi-stderr.txt; do
  install -d -m 755 "$(dirname "$stage/checker/$file")"
  install -m 644 "$checker/$file" "$stage/checker/$file"
done
while IFS= read -r -d '' relative; do
  relative=${relative#./}
  if [[ -d $checker/artifacts/$relative ]]; then
    install -d -m 755 "$stage/checker/artifacts/$relative"
  else
    install -d -m 755 "$(dirname "$stage/checker/artifacts/$relative")"
    install -m 644 "$checker/artifacts/$relative" "$stage/checker/artifacts/$relative"
  fi
done < <(cd "$checker/artifacts" && find . -mindepth 1 -print0 | sort -z)

subject_receipt_sha=$(sha256sum "$subject/task-receipt.json" | cut -d' ' -f1)
checker_receipt_sha=$(sha256sum "$checker/task-receipt.json" | cut -d' ' -f1)
checker_result_sha=$(sha256sum "$checker_result" | cut -d' ' -f1)
checker_json=$(jq -S -c -n \
  --arg verdict "$checker_verdict" \
  --arg subject_receipt_sha "$subject_receipt_sha" \
  --arg checker_receipt_sha "$checker_receipt_sha" \
  --arg checker_result_sha "$checker_result_sha" \
  --slurpfile receipt "$checker/task-receipt.json" \
  --slurpfile result "$checker_result" '
  {
    schema: "nomos.gate_k.checker_receipt@1",
    verdict: $verdict,
    identity: $receipt[0].identity,
    operatorIntervention: $receipt[0].operatorIntervention,
    accounting: $receipt[0].accounting,
    subjectTaskReceiptSha256: $subject_receipt_sha,
    checkerTaskReceiptSha256: $checker_receipt_sha,
    checkerResultSha256: $checker_result_sha,
    result: $result[0]
  }
')
printf '%s\n' "$checker_json" >"$stage/checker.json"

result=$(jq -S -c -n \
  --arg verdict "$verdict" \
  --arg reason "$logical_reason" \
  --arg adjudicator "$adjudicator" \
  --arg owner "$owner" \
  --arg commit "$subject_commit" \
  --arg shape "$subject_shape" \
  --arg classification "$subject_class" \
  --argjson formal "$subject_formal" \
  --argjson subject_boundary "$subject_boundary_json" \
  --argjson checker_boundary "$checker_boundary_json" \
  --slurpfile subject "$subject/task-receipt.json" \
  --slurpfile checker "$checker/task-receipt.json" '
  {
    schema: "nomos.gate_k.run_result@1",
    verdict: $verdict,
    reason: $reason,
    adjudicator: $adjudicator,
    ownerDisposition: $owner,
    candidateCommit: $commit,
    shape: $shape,
    classification: $classification,
    formalAttempt: $formal,
    commandBoundaryAudits: {
      subject: $subject_boundary,
      checker: $checker_boundary
    },
    subject: $subject[0],
    checker: $checker[0]
  }
')
printf '%s\n' "$result" >"$stage/result.json"

[[ $(tree_sha "$stage/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' "$stage/subject/task-receipt.json") ]] ||
  fail 'final subject artifact tree differs from its task receipt'
[[ $(tree_sha "$stage/checker/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' "$stage/checker/task-receipt.json") ]] ||
  fail 'final checker artifact tree differs from its task receipt'

printf '%s\n' \
  "# Gate K $subject_shape $subject_class run" \
  '' \
  "- Verdict: \`$verdict\`" \
  "- Reason: $logical_reason" \
  "- Candidate: \`$subject_commit\`" \
  "- Formal attempt: \`$subject_formal\`" \
  "- Subject: \`$(jq -r '.identity.provider + "/" + .identity.model' "$subject/task-receipt.json")\`, session \`$subject_session\`" \
  "- Checker: \`$(jq -r '.identity.provider + "/" + .identity.model' "$checker/task-receipt.json")\`, session \`$checker_session\`" \
  '- Operator interventions: `none` for subject and checker' \
  '- Operator retries: `0` for subject and checker' \
  "- Adjudicator: $adjudicator" \
  "- Owner disposition: $owner" \
  "- Subject command boundary: \`$subject_boundary_verdict\`" \
  "- Checker command boundary: \`$checker_boundary_verdict\`" \
  '' \
  'The complete subject transcript, ordered commands, artifacts, independent checker' \
  'record, packet identities, model identities, and accounting are stored beside this file.' \
  >"$stage/RUN.md"

mv -- "$stage" "$out"
trap - EXIT
printf 'GATE_K_RUN_FINALIZED verdict=%s formal_attempt=%s output=%s\n' \
  "$verdict" "$subject_formal" "$out"

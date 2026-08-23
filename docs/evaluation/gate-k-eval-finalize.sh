#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k eval finalizer: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 4 ]] || fail \
  'usage: gate-k-eval-finalize.sh SUBJECT_RECORD CHECKER_RECORD ADJUDICATION_JSON OUT'
subject=$1
checker=$2
adjudication=$3
out=$4
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(realpath -e "$script_dir/../..")
adjudication_validator="$script_dir/gate-k-eval-validate-adjudication.py"
command_deriver="$script_dir/gate-k-eval-derive-commands.sh"
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
subject=$(realpath -e "$subject")
checker=$(realpath -e "$checker")
[[ $subject != "$checker" ]] || fail 'subject and checker task records are the same directory'
case "$subject/" in "$checker/"*) fail 'subject task record is nested under checker' ;; esac
case "$checker/" in "$subject/"*) fail 'checker task record is nested under subject' ;; esac
[[ -f $adjudication && ! -L $adjudication ]] || fail "adjudication is absent: $adjudication"
adjudication=$(realpath -e "$adjudication")
[[ ! -e $out ]] || fail "output already exists: $out"
out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"
for record in "$subject" "$checker"; do
  case "$out/" in "$record/"*) fail 'output must be outside both immutable task records' ;; esac
done

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
  derived_commands_sha=$("$command_deriver" "$record/transcript.ndjson" | sha256sum | cut -d' ' -f1)
  [[ $(sha256sum "$record/commands.json" | cut -d' ' -f1) == "$derived_commands_sha" ]] ||
    fail "commands do not derive exactly from transcript: $record"
  [[ $(tree_sha "$record/artifacts") == \
      $(jq -r '.digests.artifactsTreeSha256' "$record/task-receipt.json") ]] ||
    fail "artifact-tree digest differs from task receipt: $record"
  [[ $(sha256sum "$record/boundary.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.boundarySha256' "$record/task-receipt.json") ]] ||
    fail "boundary digest differs from task receipt: $record"
  [[ $(sha256sum "$record/pi-qualification.txt" | cut -d' ' -f1) == \
      $(jq -r '.digests.qualificationSha256' "$record/task-receipt.json") ]] ||
    fail "qualification digest differs from task receipt: $record"
  [[ $(jq -S -c . "$record/accounting.json") == \
      $(jq -S -c '.accounting' "$record/task-receipt.json") ]] ||
    fail "accounting file differs from task receipt: $record"

  for packet_file in plan.json prompt.txt; do
    packet_file_sha=$(jq -r --arg path "$packet_file" \
      '.files[] | select(.path == $path) | .sha256' "$record/packet-manifest.json")
    [[ $(sha256sum "$record/$packet_file" | cut -d' ' -f1) == "$packet_file_sha" ]] ||
      fail "$packet_file differs from packet manifest: $record"
  done
  packet_manifest_sha=$(sha256sum "$record/packet-manifest.json" | cut -d' ' -f1)
  [[ $(jq -r '.packetManifestSha256' "$record/boundary.json") == "$packet_manifest_sha" ]] ||
    fail "boundary packet identity differs from packet manifest: $record"
  prompt_sha=$(sha256sum "$record/prompt.txt" | cut -d' ' -f1)
  [[ $(jq -r '.taskPromptSha256' "$record/boundary.json") == "$prompt_sha" ]] ||
    fail "boundary prompt identity differs from packet: $record"

  receipt_commit=$(jq -r '.candidateCommit' "$record/task-receipt.json")
  [[ $receipt_commit =~ ^[0-9a-f]{40}$ ]] || fail "task receipt candidate is invalid: $record"
  jq -e '
    .schema == "nomos.gate_k.packet_manifest@1" and
    .manifestExcludesSelf == true and
    (.shape == "author" or .shape == "debug" or
     .shape == "author-checker" or .shape == "debug-checker") and
    (.writablePaths | type) == "array" and (.writablePaths | length) == 1 and
    (.files | type) == "array" and (.files | length) > 0 and
    ([.files[].path] | length) == ([.files[].path] | unique | length) and
    ([.files[].path] == ([.files[].path] | sort)) and
    all(.files[];
      (.path | type) == "string" and
      (.bytes | type) == "number" and .bytes >= 0 and
      (.mode == "644" or .mode == "755") and
      (.sha256 | type) == "string" and (.sha256 | test("^[0-9a-f]{64}$")) and
      (.schemaIdentity == null or (.schemaIdentity | type) == "string"))
    ' "$record/packet-manifest.json" >/dev/null ||
    fail "packet manifest structure is invalid: $record"
  [[ $(jq -r '.candidate.commit' "$record/plan.json") == "$receipt_commit" ]] ||
    fail "plan candidate differs from task receipt: $record"
  [[ $(jq -r '.candidateCommit' "$record/packet-manifest.json") == "$receipt_commit" ]] ||
    fail "packet-manifest candidate differs from task receipt: $record"
  [[ $(jq -r '.targetCommit' "$record/boundary.json") == "$receipt_commit" ]] ||
    fail "boundary candidate differs from task receipt: $record"
  marker_sha=$(printf '%s\n' "$receipt_commit" | sha256sum | cut -d' ' -f1)
  [[ $(jq -r --arg path .nomos-candidate-commit \
      '.files[] | select(.path == $path) | [.bytes, .mode, .sha256] | @tsv' \
      "$record/packet-manifest.json") == $'41\t644\t'"$marker_sha" ]] ||
    fail "packet candidate marker differs from task receipt: $record"
  receipt_shape=$(jq -r '.shape' "$record/task-receipt.json")
  [[ $(jq -r '.task.shape' "$record/plan.json") == "$receipt_shape" ]] ||
    fail "plan shape differs from task receipt: $record"
  [[ $(jq -r '.shape' "$record/packet-manifest.json") == "$receipt_shape" ]] ||
    fail "packet-manifest shape differs from task receipt: $record"
  [[ $(jq -r '.taskShape' "$record/boundary.json") == "$receipt_shape" ]] ||
    fail "boundary shape differs from task receipt: $record"
  [[ $(jq -r '.task.classification' "$record/plan.json") == \
      $(jq -r '.classification' "$record/task-receipt.json") ]] ||
    fail "plan classification differs from task receipt: $record"
  [[ $(jq -r '.task.formalAttempt' "$record/plan.json") == \
      $(jq -r '.formalAttempt' "$record/task-receipt.json") ]] ||
    fail "plan formal-attempt status differs from task receipt: $record"
  [[ $(jq -r '.operatorIntervention' "$record/plan.json") == \
      $(jq -r '.operatorIntervention' "$record/task-receipt.json") ]] ||
    fail "plan intervention differs from task receipt: $record"
  plan_binary_sha=$(jq -r '.candidate.binarySha256' "$record/plan.json")
  [[ $plan_binary_sha == $(jq -r '.binarySha256' "$record/boundary.json") ]] ||
    fail "boundary binary differs from plan: $record"
  [[ $(jq -r '.sandbox.checks.candidateBinaryMatched' "$record/boundary.json") == true ]] ||
    fail "sandbox did not independently match the candidate binary: $record"
  [[ $plan_binary_sha == $(jq -r --arg path bin/nomos \
      '.files[] | select(.path == $path) | .sha256' "$record/packet-manifest.json") ]] ||
    fail "manifest binary differs from plan: $record"
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
git -C "$repo_root" cat-file -e "$subject_commit^{commit}" 2>/dev/null ||
  fail 'candidate commit is absent from repository history'
subject_class=$(jq -r '.classification' "$subject/task-receipt.json")
checker_class=$(jq -r '.classification' "$checker/task-receipt.json")
subject_formal=$(jq -r '.formalAttempt' "$subject/task-receipt.json")
checker_formal=$(jq -r '.formalAttempt' "$checker/task-receipt.json")
[[ $subject_class == "$checker_class" && $subject_formal == "$checker_formal" ]] ||
  fail 'subject and checker attempt classifications differ'
subject_session=$(jq -r '.identity.sessionId' "$subject/task-receipt.json")
checker_session=$(jq -r '.identity.sessionId' "$checker/task-receipt.json")
[[ $subject_session != "$checker_session" ]] || fail 'checker reused the subject session'

subject_receipt_sha=$(sha256sum "$subject/task-receipt.json" | cut -d' ' -f1)
subject_commands_sha=$(sha256sum "$subject/commands.json" | cut -d' ' -f1)
checker_manifest="$checker/packet-manifest.json"
[[ $(jq -r --arg path subject/task-receipt.json \
    '.files[] | select(.path == $path) | .sha256' "$checker_manifest") == "$subject_receipt_sha" ]] ||
  fail 'checker packet does not bind the supplied subject task receipt'
[[ $(jq -r --arg path subject/commands.json \
    '.files[] | select(.path == $path) | .sha256' "$checker_manifest") == "$subject_commands_sha" ]] ||
  fail 'checker packet does not bind the supplied subject commands'
actual_subject_artifacts=$(find "$subject/artifacts" -type f -printf '%P\0' | sort -z |
  while IFS= read -r -d '' relative; do
    size=$(stat -c %s "$subject/artifacts/$relative")
    digest=$(sha256sum "$subject/artifacts/$relative" | cut -d' ' -f1)
    printf 'subject/artifacts/%s\t%s\t%s\n' "$relative" "$size" "$digest"
  done)
manifest_subject_artifacts=$(jq -r '
  [.files[] | select(.path | startswith("subject/artifacts/")) |
    [.path, (.bytes | tostring), .sha256] | @tsv] | sort[]
  ' "$checker_manifest")
[[ $actual_subject_artifacts == "$manifest_subject_artifacts" ]] ||
  fail 'checker packet does not bind the supplied subject artifact tree'

subject_outcome=$(jq -r '.outcome' "$subject/task-receipt.json")
checker_outcome=$(jq -r '.outcome' "$checker/task-receipt.json")
checker_result="$checker/artifacts/checker.json"
[[ -f $checker_result && ! -L $checker_result ]] || fail 'checker did not publish artifacts/checker.json'
jq -e '
  .schema == "nomos.gate_k.checker_result@1" and
  (.verdict == "pass" or .verdict == "reject") and
  (.commands | type) == "array" and (.commands | length) > 0 and
  all(.commands[];
    (type == "string" and length > 0) or
    (type == "object" and
      (.command | type) == "string" and (.command | length) > 0)) and
  (.reasons | type) == "array" and (.reasons | length) > 0 and
  all(.reasons[]; type == "string" and length > 0)
  ' "$checker_result" >/dev/null || fail 'checker result is incomplete'
checker_verdict=$(jq -r '.verdict' "$checker_result")
adjudication_json=$(python3 "$adjudication_validator" "$subject" "$checker" "$adjudication") ||
  fail 'command adjudication validation failed'
[[ $(jq -r '.candidateCommit' <<<"$adjudication_json") == "$subject_commit" ]] ||
  fail 'command adjudication candidate differs from task records'
command_adjudication_verdict=$(jq -r '.verdict' <<<"$adjudication_json")
verdict=$(jq -r '.verdict' <<<"$adjudication_json")
adjudicator=$(jq -r '.adjudicator' <<<"$adjudication_json")
owner=$(jq -r '.ownerDisposition' <<<"$adjudication_json")

logical_verdict=pass
logical_reason='subject completed within protocol and the independent checker passed'
if [[ $command_adjudication_verdict == fail ]]; then
  logical_verdict=fail
  logical_reason='independent review found an outside-workspace path request in recorded commands'
elif [[ $subject_outcome == inconclusive || $checker_outcome == inconclusive ]]; then
  logical_verdict=inconclusive
  logical_reason='a subject or checker transport/harness failure prevented fair completion'
elif [[ $(jq -r '.operatorIntervention' "$subject/task-receipt.json") != none ||
        $(jq -r '.operatorIntervention' "$checker/task-receipt.json") != none ]]; then
  logical_verdict=assisted
  logical_reason='substantive operator intervention was recorded'
elif [[ $subject_outcome != eligible-for-checker || $checker_outcome != completed-checker ||
        $checker_verdict != pass ]]; then
  logical_verdict=fail
  logical_reason='the subject, checker transport, protocol, or checker result failed'
fi
if [[ $command_adjudication_verdict == pass ]]; then
  verdict=$logical_verdict
else
  [[ $logical_verdict == fail ]] || fail 'command finding did not derive fail'
fi

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
printf '%s\n' "$adjudication_json" >"$stage/adjudication.json"
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
  --argjson adjudication "$adjudication_json" \
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
    commandAdjudication: $adjudication,
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

subject_intervention=$(jq -r '.operatorIntervention' "$subject/task-receipt.json")
checker_intervention=$(jq -r '.operatorIntervention' "$checker/task-receipt.json")
printf '%s\n' \
  "# Gate K $subject_shape $subject_class run" \
  '' \
  "- Verdict: \`$verdict\`" \
  "- Reason: $logical_reason" \
  "- Candidate: \`$subject_commit\`" \
  "- Formal attempt: \`$subject_formal\`" \
  "- Subject: \`$(jq -r '.identity.provider + "/" + .identity.model' "$subject/task-receipt.json")\`, session \`$subject_session\`" \
  "- Checker: \`$(jq -r '.identity.provider + "/" + .identity.model' "$checker/task-receipt.json")\`, session \`$checker_session\`" \
  "- Subject operator intervention: \`$subject_intervention\`" \
  "- Checker operator intervention: \`$checker_intervention\`" \
  '- Operator retries: `0` for subject and checker' \
  "- Adjudicator: $adjudicator" \
  "- Owner disposition: $owner" \
  "- Command adjudication: \`$command_adjudication_verdict\`" \
  '' \
  'The complete subject transcript, ordered commands, artifacts, independent checker' \
  'record, packet identities, model identities, and accounting are stored beside this file.' \
  >"$stage/RUN.md"

mv -- "$stage" "$out"
trap - EXIT
printf 'GATE_K_RUN_FINALIZED verdict=%s formal_attempt=%s output=%s\n' \
  "$verdict" "$subject_formal" "$out"

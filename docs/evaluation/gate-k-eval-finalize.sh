#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k eval finalizer: FAIL: %s\n' "$*" >&2
  exit 1
}

real_directory() {
  local supplied=$1
  local label=$2
  local stripped=$supplied
  local parent name resolved
  while [[ $stripped != / && $stripped == */ ]]; do
    stripped=${stripped%/}
  done
  [[ -d $stripped && ! -L $stripped ]] || fail "$label: $supplied"
  parent=$(realpath -e "$(dirname -- "$stripped")")
  name=$(basename -- "$stripped")
  [[ $name != . && $name != .. ]] || fail "$label: $supplied"
  resolved=$(realpath -e "$stripped")
  [[ $resolved == "$parent/$name" ]] || fail "$label: $supplied"
  printf '%s\n' "$resolved"
}

record_only=false
if [[ $# -eq 2 && $1 == --validate-task-record ]]; then
  record_only=true
  subject=$2
  checker=
  adjudication=
  out=
  records=("$subject")
elif [[ $# -eq 4 ]]; then
  subject=$1
  checker=$2
  adjudication=$3
  out=$4
  records=("$subject" "$checker")
else
  fail 'usage: gate-k-eval-finalize.sh SUBJECT_RECORD CHECKER_RECORD ADJUDICATION_JSON OUT | --validate-task-record RECORD'
fi
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(realpath -e "$script_dir/../..")
repo_head=$(git -C "$repo_root" rev-parse HEAD)
gate_k_rc1_commit=d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9
gate_k_rc1_binary_sha=4af70accf3d1680f6b0e78f860be5ac62c5ab11b470026a83f01eb5b95051fd1
adjudication_validator="$script_dir/gate-k-eval-validate-adjudication.py"
command_deriver="$script_dir/gate-k-eval-derive-commands.sh"
transcript_validator="$script_dir/gate-k-eval-validate-transcript.py"
qualification_validator="$script_dir/gate-k-eval-validate-qualification.py"
json_validator="$script_dir/gate-k-eval-validate-json.py"
document_validator="$script_dir/gate-k-eval-validate-documents.py"
packet_verifier="$script_dir/gate-k-eval-verify-packet.sh"
attempt_validator="$script_dir/gate-k-eval-attempt-ledger.py"
attempt_ledger="$script_dir/gate-k-formal-attempt-ledger.jsonl"
if [[ $record_only == false ]]; then
  python3 "$attempt_validator" validate-frozen-inventory "$attempt_ledger" ||
    fail 'formal-attempt ledger differs from the exact frozen Gate K inventory'
fi
for record in "${records[@]}"; do
  record=$(real_directory "$record" 'task record is absent')
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
  for json_file in task-receipt.json plan.json packet-manifest.json commands.json \
    accounting.json boundary.json; do
    python3 "$json_validator" "$record/$json_file" ||
      fail "task record contains invalid or duplicate-key JSON: $record/$json_file"
  done
  python3 "$document_validator" task-receipt "$record/task-receipt.json" ||
    fail "task receipt does not satisfy its exact schema: $record"
  python3 "$document_validator" plan "$record/plan.json" ||
    fail "plan does not satisfy its exact schema: $record"
  python3 "$document_validator" manifest "$record/packet-manifest.json" ||
    fail "packet manifest does not satisfy its exact schema: $record"
done
subject=$(realpath -e "$subject")
records=("$subject")
if [[ $record_only == false ]]; then
  checker=$(realpath -e "$checker")
  records+=("$checker")
  [[ $subject != "$checker" ]] || fail 'subject and checker task records are the same directory'
  case "$subject/" in "$checker/"*) fail 'subject task record is nested under checker' ;; esac
  case "$checker/" in "$subject/"*) fail 'checker task record is nested under subject' ;; esac
  [[ -f $adjudication && ! -L $adjudication ]] || fail "adjudication is absent: $adjudication"
  adjudication=$(realpath -e "$adjudication")
  python3 "$json_validator" "$adjudication" || fail 'adjudication contains invalid or duplicate-key JSON'
  [[ ! -e $out ]] || fail "output already exists: $out"
  out_parent=$(realpath -e "$(dirname "$out")")
  out="$out_parent/$(basename "$out")"
  for record in "${records[@]}"; do
    case "$out/" in "$record/"*) fail 'output must be outside both immutable task records' ;; esac
  done
fi

tree_sha() {
  local root=$1
  find "$root" -type f -printf '%P\0' | sort -z |
    while IFS= read -r -d '' relative; do
      sha256sum "$root/$relative" | sed "s#  $root/#  #"
    done | sha256sum | cut -d' ' -f1
}

for record in "${records[@]}"; do
  jq -e --arg plan_schema "$(jq -r '.schema' "$record/plan.json")" '
    ((.schema == "nomos.gate_k.task_receipt@1" and
      $plan_schema == "nomos.gate_k.eval_plan@1") or
     (.schema == "nomos.gate_k.task_receipt@2" and .protocolRevision == 6 and
      $plan_schema == "nomos.gate_k.eval_plan@2")) and
    .identity.freshEphemeralSession == true and
    .identity.client == "Pi" and
    (.identity.clientVersion | type) == "string" and (.identity.clientVersion | length) > 0 and
    (.identity.provider | type) == "string" and (.identity.provider | length) > 0 and
    (.identity.model | type) == "string" and (.identity.model | length) > 0 and
    (.identity.thinking | type) == "string" and (.identity.thinking | length) > 0 and
    (.identity.sessionId | type) == "string" and
      (.identity.sessionId | test("^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$")) and
    (.identity.sessionStartedAt | type) == "string" and
      (.identity.sessionStartedAt | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T")) and
    .identity.mode == "json" and
    (.operatorRetries | type) == "number" and .operatorRetries >= 0 and
    .operatorRetries == (.operatorRetries | floor) and
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
  receipt_provider=$(jq -r '.identity.provider' "$record/task-receipt.json")
  receipt_model=$(jq -r '.identity.model' "$record/task-receipt.json")
  receipt_thinking=$(jq -r '.identity.thinking' "$record/task-receipt.json")
  receipt_session=$(jq -r '.identity.sessionId' "$record/task-receipt.json")
  receipt_started=$(jq -c '.identity.sessionStartedAt' "$record/task-receipt.json")
  receipt_client_version=$(jq -r '.identity.clientVersion' "$record/task-receipt.json")
  receipt_host_os=$(jq -r '.environment.hostOs' "$record/task-receipt.json")
  receipt_class=$(jq -r '.classification' "$record/task-receipt.json")
  receipt_formal=$(jq -r '.formalAttempt' "$record/task-receipt.json")
  receipt_shape=$(jq -r '.shape' "$record/task-receipt.json")
  receipt_sha=$(sha256sum "$record/task-receipt.json" | cut -d' ' -f1)
  if [[ $record_only == false && $receipt_class == formal && $receipt_formal == true ]]; then
    case $receipt_shape in
      author) frozen_receipt_sha=732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8 ;;
      author-checker) frozen_receipt_sha=2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3 ;;
      debug) frozen_receipt_sha=2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390 ;;
      debug-checker) frozen_receipt_sha=0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37 ;;
      *) fail "formal task shape is invalid: $record" ;;
    esac
    [[ $(sha256sum "$record/task-receipt.json" | cut -d' ' -f1) == "$frozen_receipt_sha" ]] ||
      fail "formal task receipt is not one of the four frozen gate-k-rc1 records: $record"
    ledger_matches=$(jq -s --arg commit "$(jq -r .candidateCommit "$record/task-receipt.json")" \
      --arg shape "$receipt_shape" --arg provider "$(jq -r .identity.provider "$record/task-receipt.json")" \
      --arg model "$(jq -r .identity.model "$record/task-receipt.json")" \
      --arg thinking "$(jq -r .identity.thinking "$record/task-receipt.json")" \
      --arg manifest "$(jq -r .digests.packetManifestSha256 "$record/task-receipt.json")" \
      --arg receipt "$receipt_sha" --arg outcome "$(jq -r .outcome "$record/task-receipt.json")" '
      [.[] | select(.event == "import-close" and .candidateCommit == $commit and
        .shape == $shape and .provider == $provider and .model == $model and
        .thinking == $thinking and .packetManifestSha256 == $manifest and
        .taskReceiptSha256 == $receipt and .outcome == $outcome)] | length
      ' "$attempt_ledger")
    [[ $ledger_matches -eq 1 ]] ||
      fail "formal task receipt is absent from the append-only attempt ledger: $record"
  fi
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
  task_prompt=$(<"$record/prompt.txt")
  boundary_workspace=$(jq -r '.hostWorkspace' "$record/boundary.json")
  transcript_accounting=$(python3 "$transcript_validator" "$record/transcript.ndjson" \
    --prompt "$task_prompt" --provider "$receipt_provider" --model "$receipt_model" \
    --session "$receipt_session" --started "$(jq -r . <<<"$receipt_started")" \
    --workspace "$boundary_workspace") ||
    fail "transcript lifecycle or terminal identity is incomplete: $record"
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
  [[ $(jq -S -c . "$record/accounting.json") == "$(jq -S -c . <<<"$transcript_accounting")" ]] ||
    fail "accounting does not derive from the transcript: $record"

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

  [[ $(grep -Fxc 'PI_COLD_AGENT_BOUNDARY PASS' "$record/pi-qualification.txt") -eq 1 ]] ||
    fail "qualification does not contain exactly one pass disposition: $record"
  mapfile -t qualification_commits < <(
    awk '$1 == "PI_TARGET_COMMIT" && NF == 2 {print $2}' "$record/pi-qualification.txt"
  )
  [[ ${#qualification_commits[@]} -eq 1 &&
      ${qualification_commits[0]} == "$receipt_commit" ]] ||
    fail "qualification candidate differs from task receipt: $record"
  mapfile -t qualification_versions < <(sed -n 's/^PI_VERSION //p' "$record/pi-qualification.txt")
  [[ ${#qualification_versions[@]} -eq 1 &&
      ${qualification_versions[0]} == "$receipt_client_version" ]] ||
    fail "qualification client version differs from task receipt: $record"
  mapfile -t qualification_hosts < <(sed -n 's/^PI_HOST_OS //p' "$record/pi-qualification.txt")
  [[ ${#qualification_hosts[@]} -eq 1 && ${qualification_hosts[0]} == "$receipt_host_os" ]] ||
    fail "qualification host identity differs from task receipt: $record"
  mapfile -t qualification_models < <(sed -n 's/^PI_MODEL //p' "$record/pi-qualification.txt")
  [[ ${#qualification_models[@]} -eq 1 ]] ||
    fail "qualification does not contain exactly one model identity: $record"
  IFS=$'\t' read -r qualified_provider qualified_model qualified_label \
    qualified_thinking qualified_extra <<<"${qualification_models[0]}"
  [[ -n $qualified_label && -z ${qualified_extra:-} &&
      $qualified_provider == "$receipt_provider" &&
      $qualified_model == "$receipt_model" &&
      $qualified_thinking == "$receipt_thinking" ]] ||
    fail "qualification model identity differs from task receipt: $record"
  mapfile -t qualification_lanes < <(sed -n 's/^PI_LANE //p' "$record/pi-qualification.txt")
  [[ ${#qualification_lanes[@]} -eq 1 ]] ||
    fail "qualification does not contain exactly one provider lane: $record"
  case $receipt_provider:${qualification_lanes[0]} in
    anthropic:claude | antigravity:gemini | deepseek:deepseek) ;;
    *) fail "qualification lane differs from task identity: $record" ;;
  esac
  mapfile -t qualification_worktree < <(
    sed -n 's/^PI_WORKTREE_STATUS //p' "$record/pi-qualification.txt"
  )
  [[ ${#qualification_worktree[@]} -eq 1 ]] ||
    fail "qualification does not contain exactly one worktree status: $record"
  case $receipt_class:${qualification_worktree[0]} in
    formal:clean | rehearsal:clean | rehearsal:fixture-may-be-dirty) ;;
    *) fail "qualification worktree status is ineligible: $record" ;;
  esac
  python3 "$qualification_validator" "$record/pi-qualification.txt" \
    --commit "$receipt_commit" --version "$receipt_client_version" --host "$receipt_host_os" \
    --provider "$receipt_provider" --model "$receipt_model" --thinking "$receipt_thinking" \
    --lane "${qualification_lanes[0]}" --worktree "${qualification_worktree[0]}" \
    --task-receipt "$record/task-receipt.json" ||
    fail "qualification receipt is incomplete: $record"
  mapfile -t launcher_commits < <(
    awk '$1 == "PI_TASK_COMMIT" && NF == 2 {print $2}' "$record/launcher.txt"
  )
  [[ ${#launcher_commits[@]} -eq 1 && ${launcher_commits[0]} == "$receipt_commit" ]] ||
    fail "launcher candidate differs from task receipt: $record"
  mapfile -t launcher_manifests < <(
    awk '$1 == "PI_TASK_PACKET_MANIFEST_SHA256" && NF == 2 {print $2}' \
      "$record/launcher.txt"
  )
  [[ ${#launcher_manifests[@]} -eq 1 &&
      ${launcher_manifests[0]} == "$packet_manifest_sha" ]] ||
    fail "launcher packet identity differs from task record: $record"
  mapfile -t launcher_stderr < <(
    awk '$1 == "PI_TASK_STDERR_SHA256" && NF == 2 {print $2}' "$record/launcher.txt"
  )
  [[ ${#launcher_stderr[@]} -eq 1 &&
      ${launcher_stderr[0]} == "$(sha256sum "$record/pi-stderr.txt" | cut -d' ' -f1)" ]] ||
    fail "launcher does not bind the Pi stderr evidence: $record"
  mapfile -t launcher_events < <(
    awk '$1 == "PI_TASK_EVENTS_SHA256" && NF == 2 {print $2}' "$record/launcher.txt"
  )
  [[ ${#launcher_events[@]} -eq 1 &&
      ${launcher_events[0]} == "$(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1)" ]] ||
    fail "launcher does not bind the transcript: $record"
  if jq -e '.digests | has("rawTranscriptSha256")' "$record/task-receipt.json" >/dev/null; then
    mapfile -t launcher_raw_events < <(
      awk '$1 == "PI_TASK_RAW_EVENTS_SHA256" && NF == 2 {print $2}' "$record/launcher.txt"
    )
    [[ ${#launcher_raw_events[@]} -eq 1 &&
        ${launcher_raw_events[0]} == "$(jq -r .digests.rawTranscriptSha256 "$record/task-receipt.json")" ]] ||
      fail "launcher does not bind the raw provider stream digest: $record"
  fi
  mapfile -t launcher_qualification < <(
    awk '$1 == "PI_TASK_QUALIFICATION_SHA256" && NF == 2 {print $2}' "$record/launcher.txt"
  )
  legacy_receipt=false
  case $receipt_sha in
    732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8 | \
    2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3 | \
    2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390 | \
    0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37)
      legacy_receipt=true
      ;;
  esac
  if [[ $legacy_receipt == false ]]; then
    [[ ${#launcher_qualification[@]} -eq 1 &&
        ${launcher_qualification[0]} == "$(sha256sum "$record/pi-qualification.txt" | cut -d' ' -f1)" ]] ||
      fail "launcher does not bind the qualification receipt: $record"
  else
    [[ ${#launcher_qualification[@]} -eq 0 ]] ||
      fail "frozen formal launcher contains an unrecorded qualification binding: $record"
  fi
  mapfile -t launcher_sessions < <(sed -n 's/^PI_TASK_SESSION //p' "$record/launcher.txt")
  [[ ${#launcher_sessions[@]} -eq 1 &&
      ${launcher_sessions[0]} == "$receipt_session ephemeral" ]] ||
    fail "launcher session differs from task receipt: $record"
  mapfile -t launcher_models < <(sed -n 's/^PI_TASK_MODEL //p' "$record/launcher.txt")
  [[ ${#launcher_models[@]} -eq 1 &&
      ${launcher_models[0]} == "${qualification_models[0]}" ]] ||
    fail "launcher model differs from qualification: $record"
  mapfile -t launcher_statuses < <(
    awk '$1 == "PI_TASK_STATUS" && NF == 2 {print $2}' "$record/launcher.txt"
  )
  [[ ${#launcher_statuses[@]} -eq 1 && ${launcher_statuses[0]} =~ ^[0-9]+$ ]] ||
    fail "launcher status is absent or invalid: $record"
  [[ $(grep -Fxc 'PI_COLD_AGENT_TASK RECORDED' "$record/launcher.txt") -eq 1 ]] ||
    fail "launcher does not contain exactly one recorded disposition: $record"
  receipt_outcome=$(jq -r '.outcome' "$record/task-receipt.json")
  receipt_shape=$(jq -r '.shape' "$record/task-receipt.json")
  if [[ ${launcher_statuses[0]} -eq 0 ]]; then
    case $receipt_shape:$receipt_outcome in
      author:eligible-for-checker | debug:eligible-for-checker | \
        author-checker:completed-checker | debug-checker:completed-checker) ;;
      *) fail "successful launcher status differs from task outcome: $record" ;;
    esac
  else
    [[ $receipt_outcome == inconclusive ]] ||
      fail "failed launcher status differs from task outcome: $record"
  fi

  mapfile -t stderr_boundaries < <(sed -n 's/^NOMOS_PI_BOUNDARY //p' "$record/pi-stderr.txt")
  [[ ${#stderr_boundaries[@]} -eq 1 ]] ||
    fail "Pi stderr does not contain exactly one packet boundary: $record"
  printf '%s\n' "${stderr_boundaries[0]}" | python3 "$json_validator" - ||
    fail "Pi stderr boundary contains invalid or duplicate-key JSON: $record"
  stderr_boundary_json=$(jq -S -c . <<<"${stderr_boundaries[0]}")
  [[ $stderr_boundary_json == "$(jq -S -c . "$record/boundary.json")" ]] ||
    fail "Pi stderr boundary differs from task record: $record"
  mapfile -t stderr_accounting < <(sed -n 's/^NOMOS_PI_ACCOUNTING //p' "$record/pi-stderr.txt")
  [[ ${#stderr_accounting[@]} -eq 1 ]] ||
    fail "Pi stderr does not contain exactly one accounting record: $record"
  printf '%s\n' "${stderr_accounting[0]}" | python3 "$json_validator" - ||
    fail "Pi stderr accounting contains invalid or duplicate-key JSON: $record"
  stderr_accounting_json=$(jq -S -c . <<<"${stderr_accounting[0]}")
  [[ $stderr_accounting_json == "$(jq -S -c . "$record/accounting.json")" ]] ||
    fail "Pi stderr accounting differs from task record: $record"

  plan_schema=$(jq -r '.schema' "$record/plan.json")
  jq -e --arg plan_schema "$plan_schema" '
    ((.schema == "nomos.gate_k.packet_manifest@1" and
      (has("protocolRevision") | not) and
      $plan_schema == "nomos.gate_k.eval_plan@1") or
     (.schema == "nomos.gate_k.packet_manifest@2" and
      .protocolRevision == 6 and
      $plan_schema == "nomos.gate_k.eval_plan@2")) and
    .manifestExcludesSelf == true and
    (.shape == "author" or .shape == "debug" or
     .shape == "author-checker" or .shape == "debug-checker") and
    (.writablePaths | type) == "array" and (.writablePaths | length) == 1 and
    (.files | type) == "array" and (.files | length) > 0 and
    ([.files[].path] | length) == ([.files[].path] | unique | length) and
    ([.files[].path] == ([.files[].path] | sort)) and
    all(.files[];
      (.path | type) == "string" and
      (.path | test("^[A-Za-z0-9.][A-Za-z0-9._/-]*$")) and
      (.path | startswith("/") | not) and
      (.path | contains("..") | not) and
      (.path | contains("//") | not) and
      (.bytes | type) == "number" and .bytes >= 0 and .bytes == (.bytes | floor) and
      (.mode == "644" or .mode == "755") and
      (.sha256 | type) == "string" and (.sha256 | test("^[0-9a-f]{64}$")) and
      (.schemaIdentity == null or (.schemaIdentity | type) == "string"))
    ' "$record/packet-manifest.json" >/dev/null ||
    fail "packet manifest generation or structure is invalid: $record"
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
  [[ $(jq -r '.task.shape' "$record/plan.json") == "$receipt_shape" ]] ||
    fail "plan shape differs from task receipt: $record"
  [[ $(jq -r '.shape' "$record/packet-manifest.json") == "$receipt_shape" ]] ||
    fail "packet-manifest shape differs from task receipt: $record"
  [[ $(jq -r '.taskShape' "$record/boundary.json") == "$receipt_shape" ]] ||
    fail "boundary shape differs from task receipt: $record"
  case $receipt_shape in
    author) expected_writable=workspace ;;
    debug | author-checker | debug-checker) expected_writable=output ;;
    *) fail "task receipt shape is invalid: $record" ;;
  esac
  expected_writable_json=$(jq -n -c --arg path "$expected_writable" '[$path]')
  [[ $(jq -c '.packet.writablePaths' "$record/plan.json") == "$expected_writable_json" ]] ||
    fail "plan writable paths differ from task shape: $record"
  [[ $(jq -c '.writablePaths' "$record/packet-manifest.json") == "$expected_writable_json" ]] ||
    fail "manifest writable paths differ from task shape: $record"
  [[ $(jq -c '.writablePaths' "$record/boundary.json") == "$expected_writable_json" ]] ||
    fail "boundary writable paths differ from task shape: $record"
  [[ $(jq -c '.sandbox.checks.declaredWritablePaths' "$record/boundary.json") == \
      "$expected_writable_json" ]] ||
    fail "sandbox writable paths differ from task shape: $record"

  packet_root_claim=$(jq -r '.hostWorkspace' "$record/boundary.json")
  plan_binary_sha=$(jq -r '.candidate.binarySha256' "$record/plan.json")
  qualified_extension_line=$(sed -n 's/^PI_EXTENSION //p' "$record/pi-qualification.txt")
  qualified_extension=${qualified_extension_line% *}
  qualified_boundary=$(sed -n 's/^PI_BOUNDARY //p' "$record/pi-qualification.txt")
  qualified_bwrap=$(jq -r '.sandbox.binary' <<<"$qualified_boundary")
  qualified_final_system_prompt=$(jq -r '.finalSystemPromptSha256' <<<"$qualified_boundary")
  receipt_execution=$(jq -c '.execution // null' "$record/task-receipt.json")
  qualified_execution=$(jq -S -c '.runtimeIdentity // null' <<<"$qualified_boundary")
  receipt_sha=$(sha256sum "$record/task-receipt.json" | cut -d' ' -f1)
  legacy_runtime=false
  case $receipt_sha in
    732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8 | \
    2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3 | \
    2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390 | \
    0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37)
      legacy_runtime=true
      ;;
  esac
  if [[ $receipt_execution != null && $receipt_execution != "$qualified_execution" ]]; then
    fail "task runtime executables differ from the authenticated qualification: $record"
  fi
  jq -e \
    --arg commit "$receipt_commit" \
    --arg packet "$packet_root_claim" \
    --arg provider "$receipt_provider" \
    --arg model "$receipt_model" \
    --arg thinking "$receipt_thinking" \
    --arg session "$receipt_session" \
    --arg extension "$qualified_extension" \
    --arg bwrap "$qualified_bwrap" \
    --arg qualified_final_system_prompt "$qualified_final_system_prompt" \
    --arg manifest "$packet_manifest_sha" \
    --arg binary "$plan_binary_sha" \
    --arg prompt "$prompt_sha" \
    --arg shape "$receipt_shape" \
    --arg writable "$expected_writable" \
    --argjson legacy_runtime "$legacy_runtime" \
    --argjson execution "$receipt_execution" '
    (keys == (["schema", "boundaryKind", "mode", "targetCommit", "hostWorkspace",
      "guestWorkspace", "provider", "model", "thinking", "sessionId", "sessionFile",
      "projectTrusted", "entryTypesBeforeRun", "activeTools", "configuredTools",
      "contextFiles", "skills", "systemPromptSha256", "finalSystemPromptSha256",
      "packetManifestSha256", "binarySha256", "taskPromptSha256", "taskShape",
      "writablePaths", "budgets", "sandbox"] | sort) or
     keys == (["schema", "boundaryKind", "mode", "targetCommit", "hostWorkspace",
      "guestWorkspace", "provider", "model", "thinking", "sessionId", "sessionFile",
      "projectTrusted", "entryTypesBeforeRun", "activeTools", "configuredTools",
      "contextFiles", "skills", "systemPromptSha256", "finalSystemPromptSha256",
      "packetManifestSha256", "binarySha256", "taskPromptSha256", "taskShape",
      "writablePaths", "budgets", "runtimeIdentity", "sandbox"] | sort)) and
    ((.schema == "nomos.pi_cold_agent_boundary@2" and $legacy_runtime and $execution == null and
      (has("runtimeIdentity") | not)) or
     ((.schema == "nomos.pi_cold_agent_boundary@3" or
       .schema == "nomos.pi_cold_agent_boundary@4") and
      .runtimeIdentity == $execution)) and
    .boundaryKind == "packet-run" and
    .mode == "json" and .targetCommit == $commit and .hostWorkspace == $packet and
    .guestWorkspace == "/workspace" and .provider == $provider and .model == $model and
    .thinking == $thinking and .sessionId == $session and .sessionFile == null and
    .projectTrusted == false and
    .entryTypesBeforeRun == ["model_change", "thinking_level_change"] and
    .activeTools == ["bash"] and
    .configuredTools == [{"name":"bash","source":{"path":$extension,"source":"cli",
      "scope":"temporary","origin":"top-level"}}] and
    .contextFiles == [] and .skills == [] and
    .systemPromptSha256 ==
      (if .schema == "nomos.pi_cold_agent_boundary@4"
       then "c1c41bf11dd3fc42f47c174b9d431e36dd87afb60aa04d08062dd6e11963c333"
       else "2cec3aeebce2f8359cde337d3b1b2ec1601913711f282ab0289ab276b02dee79" end) and
    .finalSystemPromptSha256 ==
      (if .schema == "nomos.pi_cold_agent_boundary@4"
       then $qualified_final_system_prompt
       else "a78cae9025d8b63562a13c111e79e9f27c32ab20e726a53d2d9d8c094712e2b7" end) and
    .packetManifestSha256 == $manifest and .binarySha256 == $binary and
    .taskPromptSha256 == $prompt and .taskShape == $shape and
    .writablePaths == [$writable] and .budgets == null and
    (.sandbox | keys) == (["backend", "binary", "root", "workspace", "network",
      "environment", "checks", "selfTest"] | sort) and
    .sandbox.backend == "bubblewrap" and .sandbox.binary == $bwrap and
    .sandbox.root == "read-only" and
    .sandbox.workspace == "read-only-packet-with-declared-writable-paths" and
    .sandbox.network == "unshared" and .sandbox.environment == "cleared-and-allowlisted" and
    .sandbox.selfTest == "pass" and
    ((.schema == "nomos.pi_cold_agent_boundary@2" or
      .schema == "nomos.pi_cold_agent_boundary@3") and
     (.sandbox.checks | keys) == (["targetCommitResolved", "workspaceRead",
       "packetManifestMatched", "candidateBinaryMatched", "packetRootReadOnly",
       "temporaryStorageReadOnly", "deviceFilesystemEmpty", "processFilesystemReadOnly",
       "declaredWritablePaths", "gitMetadataAbsent", "outsideReadDenied",
       "outsideWriteDenied", "credentialEnvironmentAbsent", "networkDenied"] | sort) or
     .schema == "nomos.pi_cold_agent_boundary@4" and
     (.sandbox.checks | keys) == (["targetCommitResolved", "workspaceRead",
       "packetManifestMatched", "candidateBinaryMatched", "packetRootReadOnly",
       "temporaryStorageReadOnly", "deviceFilesystemExact", "deviceNullReadable",
       "deviceNullWritable", "processFilesystemReadOnly", "declaredWritablePaths",
       "gitMetadataAbsent", "outsideReadDenied", "outsideWriteDenied",
       "credentialEnvironmentAbsent", "networkDenied"] | sort)) and
    .sandbox.checks.declaredWritablePaths == [$writable] and
    all(.sandbox.checks | to_entries[] | select(.key != "declaredWritablePaths");
      .value == true)
    ' "$record/boundary.json" >/dev/null ||
    fail "boundary does not prove the authenticated packet isolation: $record"
  [[ $packet_root_claim == /* ]] ||
    fail "recorded immutable packet root is unavailable: $record"
  packet_root=$(real_directory "$packet_root_claim" \
    "recorded immutable packet root is unavailable: $record")
  if [[ $record_only == false ]]; then
    case "$out/" in "$packet_root/"*) fail 'output must be outside both immutable packets' ;; esac
  fi
  [[ -z $(find "$packet_root" -type l -print -quit) ]] ||
    fail "recorded immutable packet contains a symlink: $record"
  [[ -z $(find "$packet_root" ! -type f ! -type d -print -quit) ]] ||
    fail "recorded immutable packet contains a special entry: $record"
  [[ -d $packet_root/$expected_writable && ! -L $packet_root/$expected_writable ]] ||
    fail "recorded immutable packet writable root is unavailable: $record"
  [[ $(tree_sha "$packet_root/$expected_writable") == \
      $(jq -r '.digests.artifactsTreeSha256' "$record/task-receipt.json") ]] ||
    fail "recorded packet writable tree differs from task artifacts: $record"
  [[ $(sha256sum "$packet_root/packet-manifest.json" | cut -d' ' -f1) == \
      "$packet_manifest_sha" ]] ||
    fail "recorded immutable packet manifest differs from task record: $record"
  "$packet_verifier" --post-run "$packet_root" "$receipt_commit" >/dev/null ||
    fail "recorded immutable packet does not satisfy its complete shape: $record"
  actual_immutable_files=$(find "$packet_root" -type f \
    ! -path "$packet_root/$expected_writable/*" -printf '%P\n' | sort)
  expected_immutable_files=$({
    jq -r --arg writable "$expected_writable" '
      .files[] |
      select(.path != $writable and (.path | startswith($writable + "/") | not)) |
      .path
      ' "$record/packet-manifest.json"
    printf '%s\n' packet-manifest.json
  } | sort)
  [[ $actual_immutable_files == "$expected_immutable_files" ]] ||
    fail "recorded immutable packet file set differs from its manifest: $record"
  immutable_rows=$(jq -r --arg writable "$expected_writable" '
    .files[] |
    select(.path != $writable and (.path | startswith($writable + "/") | not)) |
    [.path, (.bytes | tostring), .mode, .sha256] | @tsv
    ' "$record/packet-manifest.json")
  while IFS=$'\t' read -r relative expected_bytes expected_mode expected_sha; do
    packet_path="$packet_root/$relative"
    [[ -f $packet_path && ! -L $packet_path ]] ||
      fail "recorded immutable packet member is unavailable: $record/$relative"
    [[ $(stat -c %s "$packet_path") == "$expected_bytes" ]] ||
      fail "recorded immutable packet member size differs: $record/$relative"
    [[ $(stat -c %a "$packet_path") == "$expected_mode" ]] ||
      fail "recorded immutable packet member mode differs: $record/$relative"
    [[ $(sha256sum "$packet_path" | cut -d' ' -f1) == "$expected_sha" ]] ||
      fail "recorded immutable packet member digest differs: $record/$relative"
  done <<<"$immutable_rows"
  empty_packet_directory=$(find "$packet_root" -mindepth 1 -type d -empty \
    ! -path "$packet_root/$expected_writable" -print -quit)
  [[ -z $empty_packet_directory ]] ||
    fail "recorded immutable packet contains an unbound empty directory: $record"
  [[ ! -e $packet_root/.git ]] || fail "recorded immutable packet contains Git metadata: $record"
  [[ $(jq -r '.task.classification' "$record/plan.json") == \
      $(jq -r '.classification' "$record/task-receipt.json") ]] ||
    fail "plan classification differs from task receipt: $record"
  [[ $(jq -r '.task.formalAttempt' "$record/plan.json") == \
      $(jq -r '.formalAttempt' "$record/task-receipt.json") ]] ||
    fail "plan formal-attempt status differs from task receipt: $record"
  case $receipt_class:$receipt_formal in
    formal:true)
      if [[ $record_only == false ]]; then
        [[ $receipt_commit == "$gate_k_rc1_commit" ]] ||
          fail "formal attempt candidate differs from frozen gate-k-rc1: $record"
      fi
      ;;
    rehearsal:false)
      [[ $receipt_commit == "$repo_head" ]] ||
        fail "rehearsal candidate differs from the finalizer checkout: $record"
      ;;
    *) fail "classification and formal-attempt status are inconsistent: $record" ;;
  esac
  if [[ $(jq -r '.schema' "$record/task-receipt.json") == \
        nomos.gate_k.task_receipt@1 ]]; then
    [[ $(jq -r '.operatorIntervention' "$record/plan.json") == \
        $(jq -r '.operatorIntervention' "$record/task-receipt.json") ]] ||
      fail "plan intervention differs from task receipt: $record"
  fi
  [[ $plan_binary_sha == $(jq -r '.binarySha256' "$record/boundary.json") ]] ||
    fail "boundary binary differs from plan: $record"
  [[ $(jq -r '.sandbox.checks.candidateBinaryMatched' "$record/boundary.json") == true ]] ||
    fail "sandbox did not independently match the candidate binary: $record"
  [[ $plan_binary_sha == $(jq -r --arg path bin/nomos \
      '.files[] | select(.path == $path) | .sha256' "$record/packet-manifest.json") ]] ||
    fail "manifest binary differs from plan: $record"
  [[ $(sha256sum "$packet_root/bin/nomos" | cut -d' ' -f1) == "$plan_binary_sha" ]] ||
    fail "recorded immutable packet binary differs from plan: $record"
  if [[ $record_only == false && $receipt_class == formal &&
    $plan_binary_sha != "$gate_k_rc1_binary_sha" ]]; then
    fail "formal attempt binary differs from frozen gate-k-rc1: $record"
  fi
  git -C "$repo_root" cat-file -e "$receipt_commit^{commit}" 2>/dev/null ||
    fail "candidate commit is absent from repository history: $record"
  case $receipt_shape in
    author-checker | debug-checker)
      checker_result="$record/artifacts/checker.json"
      [[ -f $checker_result && ! -L $checker_result ]] ||
        fail "checker did not publish artifacts/checker.json: $record"
      python3 "$json_validator" "$checker_result" ||
        fail "checker result contains invalid or duplicate-key JSON: $record"
      python3 "$document_validator" checker-result "$checker_result" ||
        fail "checker result does not satisfy a declared exact schema: $record"
      ;;
  esac
done

if [[ $record_only == true ]]; then
  printf 'GATE_K_TASK_RECORD_VALIDATED record=%s\n' "$subject"
  exit 0
fi

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
python3 "$json_validator" "$checker_result" ||
  fail 'checker result contains invalid or duplicate-key JSON'
python3 "$document_validator" checker-result "$checker_result" ||
  fail 'checker result does not satisfy a declared exact schema'
checker_receipt_schema=$(jq -r '.schema' "$checker/task-receipt.json")
jq -e --arg receipt_schema "$checker_receipt_schema" '
  ((.schema == "nomos.gate_k.checker_result@1" and
    (has("protocolRevision") | not) and
    $receipt_schema == "nomos.gate_k.task_receipt@1") or
   (.schema == "nomos.gate_k.checker_result@2" and .protocolRevision == 6 and
    $receipt_schema == "nomos.gate_k.task_receipt@2")) and
  (.verdict == "pass" or .verdict == "reject") and
  (.commands | type) == "array" and (.commands | length) > 0 and
  all(.commands[];
    (type == "string" and length > 0) or
    (type == "object" and
      (.command | type) == "string" and (.command | length) > 0)) and
  (.reasons | type) == "array" and (.reasons | length) > 0 and
  all(.reasons[]; type == "string" and length > 0)
  ' "$checker_result" >/dev/null || fail 'checker result generation or content is invalid'
checker_verdict=$(jq -r '.verdict' "$checker_result")
adjudication_json=$(python3 "$adjudication_validator" "$subject" "$checker" "$adjudication") ||
  fail 'command adjudication validation failed'
[[ $(jq -r '.candidateCommit' <<<"$adjudication_json") == "$subject_commit" ]] ||
  fail 'command adjudication candidate differs from task records'
command_adjudication_verdict=$(jq -r '.verdict' <<<"$adjudication_json")
adjudication_schema=$(jq -r '.schema' <<<"$adjudication_json")
verdict=$(jq -r '.verdict' <<<"$adjudication_json")
adjudicator=$(jq -r '.adjudicator' <<<"$adjudication_json")
owner=$(jq -r '.ownerDisposition' <<<"$adjudication_json")

if [[ $adjudication_schema == nomos.gate_k.command_adjudication@2 ]]; then
  [[ $(jq -r '.schema' "$subject/task-receipt.json") == nomos.gate_k.task_receipt@2 &&
    $(jq -r '.schema' "$checker/task-receipt.json") == nomos.gate_k.task_receipt@2 ]] ||
    fail 'revision-6 adjudication requires revision-6 task receipts'
  logical_verdict=$verdict
  logical_reason=$(jq -r '.reason' <<<"$adjudication_json")
else
  [[ $(jq -r '.schema' "$subject/task-receipt.json") == nomos.gate_k.task_receipt@1 &&
    $(jq -r '.schema' "$checker/task-receipt.json") == nomos.gate_k.task_receipt@1 ]] ||
    fail 'legacy adjudication requires legacy task receipts'
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
fi

if [[ $subject_class == rehearsal ]]; then
  subject_route=$(jq -r \
    '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
    "$subject/task-receipt.json")
  checker_route=$(jq -r \
    '.identity.provider + "/" + .identity.model + "/" + .identity.thinking' \
    "$checker/task-receipt.json")
  case "$subject_shape:$subject_route:$checker_route" in
    author:anthropic/claude-opus-5/high:anthropic/claude-opus-5/high | \
      debug:anthropic/claude-opus-5/high:anthropic/claude-opus-5/high | \
      author:antigravity/gemini-3.7-flash/high:deepseek/deepseek-v4-flash-vision-exp/max | \
      debug:deepseek/deepseek-v4-flash-vision-exp/max:antigravity/gemini-3.7-flash/high) ;;
    *) fail 'rehearsal subject/checker identities differ from the approved routes' ;;
  esac
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
checker_receipt_schema=nomos.gate_k.checker_receipt@1
run_result_schema=nomos.gate_k.run_result@1
if [[ $adjudication_schema == nomos.gate_k.command_adjudication@2 ]]; then
  checker_receipt_schema=nomos.gate_k.checker_receipt@2
  run_result_schema=nomos.gate_k.run_result@2
fi
checker_json=$(jq -S -c -n \
  --arg schema "$checker_receipt_schema" \
  --arg verdict "$checker_verdict" \
  --arg subject_receipt_sha "$subject_receipt_sha" \
  --arg checker_receipt_sha "$checker_receipt_sha" \
  --arg checker_result_sha "$checker_result_sha" \
  --slurpfile receipt "$checker/task-receipt.json" \
  --slurpfile result "$checker_result" '
  ({
    schema: $schema,
    verdict: $verdict,
    identity: $receipt[0].identity,
    operatorIntervention: $receipt[0].operatorIntervention,
    accounting: $receipt[0].accounting,
    subjectTaskReceiptSha256: $subject_receipt_sha,
    checkerTaskReceiptSha256: $checker_receipt_sha,
    checkerResultSha256: $checker_result_sha,
    result: $result[0]
  } + if $schema == "nomos.gate_k.checker_receipt@2"
       then {protocolRevision: 6} else {} end)
')
printf '%s\n' "$checker_json" >"$stage/checker.json"

result=$(jq -S -c -n \
  --arg schema "$run_result_schema" \
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
  ({
    schema: $schema,
    verdict: $verdict,
    reason: $reason,
    adjudicator: $adjudicator,
    ownerDisposition: $owner,
    candidateCommit: $commit,
    shape: $shape,
    classification: $classification,
    formalAttempt: $formal,
    subject: $subject[0],
    checker: $checker[0]
  } + if $schema == "nomos.gate_k.run_result@2"
       then {protocolRevision: 6, adjudication: $adjudication,
             records: $adjudication.records}
       else {commandAdjudication: $adjudication} end)
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
subject_retries=$(jq -r '.operatorRetries' "$subject/task-receipt.json")
checker_retries=$(jq -r '.operatorRetries' "$checker/task-receipt.json")
if [[ $subject_retries -eq 0 && $checker_retries -eq 0 ]]; then
  retry_summary='- Operator retries: `0` for subject and checker'
else
  retry_summary="- Operator retries: \`$subject_retries\` subject, \`$checker_retries\` checker"
fi
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
  "$retry_summary" \
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

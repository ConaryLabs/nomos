#!/usr/bin/env bash

# Shared mutation and adjudication helpers for the offline Gate K tooling suites.
# The caller owns strict shell mode and defines tmp_dir.

assert_blocked() {
  local expected=$1
  shift
  local name=$1
  shift
  if "$@" >"$tmp_dir/$name.out" 2>"$tmp_dir/$name.err"; then
    printf 'expected %s to be blocked\n' "$name" >&2
    exit 1
  fi
  if ! grep -F "$expected" "$tmp_dir/$name.err" >/dev/null; then
    printf 'expected %s failure containing: %s\n' "$name" "$expected" >&2
    sed -n '1,120p' "$tmp_dir/$name.err" >&2
    return 1
  fi
}

declare_packet_file() {
  local packet=$1
  local relative=$2
  local path="$packet/$relative"
  local size digest update
  size=$(stat -c %s "$path")
  digest=$(sha256sum "$path" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/manifest-update.XXXXXX")
  jq -S -c --arg path "$relative" --argjson size "$size" --arg digest "$digest" '
    .files += [{
      path: $path,
      bytes: $size,
      mode: "644",
      sha256: $digest,
      schemaIdentity: null
    }]
    | .files |= sort_by(.path)
    ' "$packet/packet-manifest.json" >"$update"
  mv -- "$update" "$packet/packet-manifest.json"
}

refresh_packet_file() {
  local packet=$1
  local relative=$2
  local path="$packet/$relative"
  local size digest update
  size=$(stat -c %s "$path")
  digest=$(sha256sum "$path" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/manifest-refresh.XXXXXX")
  jq -S -c --arg path "$relative" --argjson size "$size" --arg digest "$digest" '
    .files |= map(if .path == $path then .bytes = $size | .sha256 = $digest else . end)
    ' "$packet/packet-manifest.json" >"$update"
  mv -- "$update" "$packet/packet-manifest.json"
}

tree_sha() {
  local root=$1
  find "$root" -type f -printf '%P\0' | sort -z |
    while IFS= read -r -d '' relative; do
      sha256sum "$root/$relative" | sed "s#  $root/#  #"
    done | sha256sum | cut -d' ' -f1
}

rebind_recorded_commands() {
  local record=$1
  local command=$2
  local commands_update transcript_update receipt_update command_digest transcript_digest tool_call_id
  tool_call_id=$(jq -r '.commands[0].toolCallId' "$record/commands.json")
  commands_update=$(mktemp "$tmp_dir/commands-update.XXXXXX")
  jq -S -c --arg command "$command" \
    '.commands[0].arguments.command = $command' \
    "$record/commands.json" >"$commands_update"
  mv -- "$commands_update" "$record/commands.json"
  transcript_update=$(mktemp "$tmp_dir/transcript-update.XXXXXX")
  jq -c --arg tool_call_id "$tool_call_id" --arg command "$command" '
    if (.type == "message_update" and
        .assistantMessageEvent.type == "toolcall_delta")
    then .assistantMessageEvent.delta = ({command: $command} | tojson)
    elif (.type == "message_update" and
          .assistantMessageEvent.type == "toolcall_end" and
          .assistantMessageEvent.toolCall.id == $tool_call_id)
    then .assistantMessageEvent.toolCall.arguments.command = $command
    elif (.type == "message_end" and .message.role == "assistant") or
         (.type == "turn_end")
    then .message.content |= map(
      if .type == "toolCall" and .id == $tool_call_id
      then .arguments.command = $command else . end)
    elif .type == "agent_end"
    then .messages |= map(
      if .role == "assistant" then
        .content |= map(if .type == "toolCall" and .id == $tool_call_id
          then .arguments.command = $command else . end)
      else . end)
    elif ((.type == "tool_execution_start" or .type == "tool_execution_update") and
          .toolCallId == $tool_call_id)
    then .args.command = $command
    else .
    end
    ' "$record/transcript.ndjson" >"$transcript_update"
  mv -- "$transcript_update" "$record/transcript.ndjson"
  command_digest=$(sha256sum "$record/commands.json" | cut -d' ' -f1)
  transcript_digest=$(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1)
  receipt_update=$(mktemp "$tmp_dir/receipt-update.XXXXXX")
  jq -S -c --arg command_digest "$command_digest" --arg transcript_digest "$transcript_digest" '
    .digests.commandsSha256 = $command_digest |
    .digests.transcriptSha256 = $transcript_digest
    ' \
    "$record/task-receipt.json" >"$receipt_update"
  mv -- "$receipt_update" "$record/task-receipt.json"
  receipt_update=$(mktemp "$tmp_dir/launcher-events-update.XXXXXX")
  awk -v transcript_digest="$transcript_digest" '
    $1 == "PI_TASK_EVENTS_SHA256" {
      print "PI_TASK_EVENTS_SHA256 " transcript_digest; next
    }
    {print}
    ' "$record/launcher.txt" >"$receipt_update"
  mv -- "$receipt_update" "$record/launcher.txt"
}

refresh_record_receipt_digests() {
  local record=$1
  local update packet_manifest transcript commands artifacts boundary qualification
  packet_manifest=$(sha256sum "$record/packet-manifest.json" | cut -d' ' -f1)
  transcript=$(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1)
  commands=$(sha256sum "$record/commands.json" | cut -d' ' -f1)
  artifacts=$(tree_sha "$record/artifacts")
  boundary=$(sha256sum "$record/boundary.json" | cut -d' ' -f1)
  qualification=$(sha256sum "$record/pi-qualification.txt" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/receipt-digests.XXXXXX")
  jq -S -c \
    --arg packet_manifest "$packet_manifest" \
    --arg transcript "$transcript" \
    --arg commands "$commands" \
    --arg artifacts "$artifacts" \
    --arg boundary "$boundary" \
    --arg qualification "$qualification" '
    .digests.packetManifestSha256 = $packet_manifest |
    .digests.transcriptSha256 = $transcript |
    .digests.commandsSha256 = $commands |
    .digests.artifactsTreeSha256 = $artifacts |
    .digests.boundarySha256 = $boundary |
    .digests.qualificationSha256 = $qualification
    ' "$record/task-receipt.json" >"$update"
  mv -- "$update" "$record/task-receipt.json"
  update=$(mktemp "$tmp_dir/launcher-qualification.XXXXXX")
  awk -v qualification="$qualification" '
    $1 == "PI_TASK_QUALIFICATION_SHA256" {
      print "PI_TASK_QUALIFICATION_SHA256 " qualification; next
    }
    {print}
    ' "$record/launcher.txt" >"$update"
  mv -- "$update" "$record/launcher.txt"
}

refresh_record_runtime_evidence() {
  local record=$1
  local boundary accounting update manifest commit stderr_digest
  boundary=$(jq -c . "$record/boundary.json")
  accounting=$(jq -c . "$record/accounting.json")
  update=$(mktemp "$tmp_dir/pi-stderr-refresh.XXXXXX")
  awk -v boundary="$boundary" -v accounting="$accounting" '
    /^NOMOS_PI_BOUNDARY / {print "NOMOS_PI_BOUNDARY " boundary; next}
    /^NOMOS_PI_ACCOUNTING / {print "NOMOS_PI_ACCOUNTING " accounting; next}
    {print}
    ' "$record/pi-stderr.txt" >"$update"
  mv -- "$update" "$record/pi-stderr.txt"
  manifest=$(sha256sum "$record/packet-manifest.json" | cut -d' ' -f1)
  commit=$(jq -r '.candidateCommit' "$record/task-receipt.json")
  stderr_digest=$(sha256sum "$record/pi-stderr.txt" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/launcher-refresh.XXXXXX")
  awk -v commit="$commit" -v manifest="$manifest" -v stderr_digest="$stderr_digest" '
    $1 == "PI_TASK_COMMIT" {print "PI_TASK_COMMIT " commit; next}
    $1 == "PI_TASK_PACKET_MANIFEST_SHA256" {
      print "PI_TASK_PACKET_MANIFEST_SHA256 " manifest; next
    }
    $1 == "PI_TASK_STDERR_SHA256" {print "PI_TASK_STDERR_SHA256 " stderr_digest; next}
    {print}
    ' "$record/launcher.txt" >"$update"
  mv -- "$update" "$record/launcher.txt"
  refresh_record_receipt_digests "$record"
}

refresh_record_transcript_evidence() {
  local record=$1
  local transcript_digest update
  transcript_digest=$(sha256sum "$record/transcript.ndjson" | cut -d' ' -f1)
  refresh_record_receipt_digests "$record"
  update=$(mktemp "$tmp_dir/launcher-transcript-refresh.XXXXXX")
  awk -v transcript_digest="$transcript_digest" '
    $1 == "PI_TASK_EVENTS_SHA256" {
      print "PI_TASK_EVENTS_SHA256 " transcript_digest; next
    }
    {print}
    ' "$record/launcher.txt" >"$update"
  mv -- "$update" "$record/launcher.txt"
}

write_adjudication() {
  local subject_record=$1
  local checker_record=$2
  local verdict=$3
  local findings=$4
  local out=$5
  local candidate subject_receipt checker_receipt subject_commands checker_commands
  local subject_count checker_count subject_semantic checker_semantic
  local subject_independence checker_independence subject_operational checker_operational
  candidate=$(jq -r '.candidateCommit' "$subject_record/task-receipt.json")
  subject_receipt=$(sha256sum "$subject_record/task-receipt.json" | cut -d' ' -f1)
  checker_receipt=$(sha256sum "$checker_record/task-receipt.json" | cut -d' ' -f1)
  subject_commands=$(sha256sum "$subject_record/commands.json" | cut -d' ' -f1)
  checker_commands=$(sha256sum "$checker_record/commands.json" | cut -d' ' -f1)
  subject_count=$(jq '.commands | length' "$subject_record/commands.json")
  checker_count=$(jq '.commands | length' "$checker_record/commands.json")
  subject_semantic=$(sha256sum "$subject_record/artifacts/subject.txt" | cut -d' ' -f1)
  checker_semantic=$(sha256sum "$checker_record/artifacts/checker.json" | cut -d' ' -f1)
  subject_independence=$(sha256sum "$subject_record/packet-manifest.json" | cut -d' ' -f1)
  checker_independence=$(sha256sum "$checker_record/packet-manifest.json" | cut -d' ' -f1)
  subject_operational=$subject_commands
  checker_operational=$checker_commands
  jq -S -c -n \
    --arg candidate "$candidate" \
    --arg subject_receipt "$subject_receipt" \
    --arg checker_receipt "$checker_receipt" \
    --arg subject_commands "$subject_commands" \
    --arg checker_commands "$checker_commands" \
    --arg verdict "$verdict" \
    --argjson findings "$findings" \
    --argjson subject_count "$subject_count" \
    --argjson checker_count "$checker_count" \
    --arg subject_semantic "$subject_semantic" \
    --arg checker_semantic "$checker_semantic" \
    --arg subject_independence "$subject_independence" \
    --arg checker_independence "$checker_independence" \
    --arg subject_operational "$subject_operational" \
    --arg checker_operational "$checker_operational" '
    def dimension($path; $sha): {
      verdict: "pass",
      reason: "fixture evidence supports this dimension",
      evidence: [{path: $path, sha256: $sha}]
    };
    def record($semantic_path; $semantic_sha; $independence_sha; $operational_sha): {
      dimensions: {
        semantic_merit: dimension($semantic_path; $semantic_sha),
        independence_integrity: dimension("packet-manifest.json"; $independence_sha),
        operational_compliance: dimension("commands.json"; $operational_sha)
      },
      verdict: "pass",
      reason: "fixture dimensions derive pass"
    };
    {
      schema: "nomos.gate_k.command_adjudication@2",
      protocolRevision: 6,
      candidateCommit: $candidate,
      subjectTaskReceiptSha256: $subject_receipt,
      checkerTaskReceiptSha256: $checker_receipt,
      subjectCommandsSha256: $subject_commands,
      checkerCommandsSha256: $checker_commands,
      reviewedAllCommands: true,
      reviewedCommandCounts: {subject: $subject_count, checker: $checker_count},
      findings: $findings,
      verdict: $verdict,
      reason: "fixture independent review of every recorded command",
      adjudicator: "fixture-adjudicator",
      ownerDisposition: "fixture-owner",
      records: {
        subject: record("artifacts/subject.txt"; $subject_semantic;
          $subject_independence; $subject_operational),
        checker: record("artifacts/checker.json"; $checker_semantic;
          $checker_independence; $checker_operational)
      }
    }
    | reduce $findings[] as $finding (.;
        .records[$finding.record].dimensions.operational_compliance.verdict = "fail" |
        .records[$finding.record].dimensions.operational_compliance.reason = $finding.reason |
        if $finding.kind == "undeclared_information_ingress" then
          .records[$finding.record].dimensions.independence_integrity.verdict = "fail" |
          .records[$finding.record].dimensions.independence_integrity.reason = $finding.reason
        else . end)
    | .records |= with_entries(
        .value.verdict = (if ([.value.dimensions[].verdict] | index("fail"))
          then "fail"
          elif ([.value.dimensions[].verdict] | index("inconclusive"))
          then "inconclusive"
          else "pass" end) |
        .value.reason = "fixture dimension verdicts mechanically derive this record verdict")
    ' >"$out"
}

write_pass_adjudication() {
  write_adjudication "$1" "$2" pass '[]' "$3"
}

write_outside_path_adjudication() {
  local subject_record=$1
  local checker_record=$2
  local record=$3
  local ordinal=$4
  local path_token=$5
  local out=$6
  local record_dir command_sha findings
  if [[ $record == subject ]]; then
    record_dir=$subject_record
  else
    record_dir=$checker_record
  fi
  command_sha=$(jq -j --argjson ordinal "$ordinal" \
    '.commands[$ordinal].arguments.command' "$record_dir/commands.json" | sha256sum | cut -d' ' -f1)
  findings=$(jq -c -n \
    --arg record "$record" --argjson ordinal "$ordinal" --arg command_sha "$command_sha" \
    --arg path_token "$path_token" '[{
      record: $record,
      commandOrdinal: $ordinal,
      commandSha256: $command_sha,
      kind: "outside_workspace_path",
      pathToken: $path_token,
      reason: "the recorded shell command requested a path outside /workspace"
    }]')
  write_adjudication "$subject_record" "$checker_record" fail "$findings" "$out"
}

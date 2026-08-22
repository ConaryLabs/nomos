#!/usr/bin/env bash

set -euo pipefail

agy_bin=${AGY_BIN:-agy}
model=${AGY_MODEL:-gemini-3.7-flash-high}
model_label=${AGY_MODEL_LABEL:-Gemini 3.7 Flash (High)}
agent=${AGY_AGENT:-nomos-cold-subject}
print_timeout=${AGY_PRINT_TIMEOUT:-2m}
expected_agent_sha=d9b5004681113503f5411d5c28358d8710086e93796ed79426fd9356c4c17337

fail() {
  printf 'agy formal boundary preflight: FAIL: %s\n' "$*" >&2
  exit 1
}

record_failure() {
  printf 'agy formal boundary preflight: BLOCKED: %s\n' "$*" >&2
  blocked=1
}

command -v "$agy_bin" >/dev/null 2>&1 || fail "agy executable not found: $agy_bin"
command -v git >/dev/null 2>&1 || fail "git executable not found"
command -v grep >/dev/null 2>&1 || fail "grep executable not found"
command -v sed >/dev/null 2>&1 || fail "sed executable not found"
command -v sha256sum >/dev/null 2>&1 || fail "sha256sum executable not found"

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) ||
  fail "run the preflight inside the target Git worktree"
repo_root=$(cd "$repo_root" && pwd -P)
case "$repo_root" in
  *'"'* | *'\'* | *$'\t'* | *$'\n'* | *$'\r'*)
    fail "worktree path cannot be checked safely in JSON output: $repo_root"
    ;;
esac

agent_file="$repo_root/.agents/agents/$agent/agent.md"
[[ -f $agent_file ]] || fail "missing custom agent definition: $agent_file"
agent_sha=$(sha256sum "$agent_file")
agent_sha=${agent_sha%% *}
[[ $agent_sha == "$expected_agent_sha" ]] ||
  fail "custom agent definition digest mismatch: $agent_sha"

tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT

"$agy_bin" --version >"$tmp_dir/version.txt" || fail "agy --version failed"
"$agy_bin" models >"$tmp_dir/models.txt" || fail "agy models failed"
[[ $(wc -l <"$tmp_dir/version.txt") -eq 1 ]] ||
  fail "agy version output must be one line"
grep -Fx "${model}"$'\t'"${model_label}" "$tmp_dir/models.txt" >/dev/null ||
  fail "model catalog does not resolve $model to $model_label"

prompt='Reply with exactly: formal boundary preflight. Do not call tools.'
set +e
AGY_CLI_HIDE_ACCOUNT_INFO=true "$agy_bin" -p "$prompt" \
  --agent "$agent" \
  --model "$model" \
  --effort high \
  --new-project \
  --sandbox \
  --disable-slash-commands \
  --output-format stream-json \
  --print-timeout "$print_timeout" \
  --log-file "$tmp_dir/agy.log" \
  >"$tmp_dir/events.ndjson" 2>"$tmp_dir/stderr.txt"
agy_status=$?
set -e

if [[ $agy_status -ne 0 ]]; then
  cat "$tmp_dir/stderr.txt" >&2
  fail "agy exited $agy_status"
fi

project_line=$(grep -E 'project: created project .*\(id=[0-9a-f-]{36}\) at .*/projects/[0-9a-f-]{36}\.json' \
  "$tmp_dir/agy.log" | tail -n 1 || true)
[[ -n $project_line ]] || fail "agy did not record creation of a new project"
project_id=$(printf '%s\n' "$project_line" |
  sed -n 's/.*(id=\([0-9a-f-]\{36\}\)) at .*/\1/p')
project_record=$(printf '%s\n' "$project_line" | sed -n 's/.* at \(.*\)$/\1/p')
[[ -n $project_id ]] || fail "could not parse the new project ID"
[[ -f $project_record ]] || fail "new project record is missing: $project_record"
compact_project=$(tr -d '[:space:]' <"$project_record")
[[ $compact_project == *"\"id\":\"$project_id\""* ]] ||
  fail "new project record does not contain its reported ID"
[[ $compact_project == *"\"folderUri\":\"file://$repo_root\""* ]] ||
  fail "new project record does not contain only the target worktree"
[[ $(grep -Foc '"folderUri"' "$project_record") -eq 1 ]] ||
  fail "new project record contains more than one workspace folder"
if [[ $compact_project == *'"permissions":'* &&
  $compact_project != *'"permissions":null'* ]]; then
  fail "new project record carries persisted project permissions"
fi

blocked=0
init_count=$(grep -Fc '"event":"init"' "$tmp_dir/events.ndjson" || true)
[[ $init_count -eq 1 ]] || record_failure "expected exactly one init event, found $init_count"
init_line=$(grep -F '"event":"init"' "$tmp_dir/events.ndjson" | head -n 1 || true)

if [[ -n $init_line ]]; then
  [[ $init_line == *"\"model\":\"$model\""* ]] ||
    record_failure "init event does not pin $model"
  [[ $init_line == *"\"cwd\":\"$repo_root\""* ]] ||
    record_failure "init event cwd is not the target worktree"
  [[ $init_line == *"\"agent\":\"$agent\""* ]] ||
    record_failure "init event does not pin custom agent $agent"
  [[ $init_line == *"\"project_id\":\"$project_id\""* ]] ||
    record_failure "init event does not disclose the newly created project ID"
  [[ $init_line == *'"context_sources":[]'* ]] ||
    record_failure "init event does not prove an empty context-source set"
  [[ $init_line == *'"memory_enabled":false'* ]] ||
    record_failure "init event does not prove persisted memory is disabled"
  [[ $init_line == *'"permission_mode":"request-review"'* ]] ||
    record_failure "init event is not in request-review permission mode"

  tools=$(printf '%s\n' "$init_line" |
    sed -n 's/.*"tools":\[\([^]]*\)\].*/\1/p')
  if [[ -z $tools ]]; then
    record_failure "init event does not disclose its tool set"
  else
    normalized_tools=$(printf '%s\n' "$tools" | tr ',' '\n' | tr -d '"' |
      LC_ALL=C sort | tr '\n' ',' | sed 's/,$//')
    expected_tools='replace_file_content,run_command,view_file'
    [[ $normalized_tools == "$expected_tools" ]] ||
      record_failure "effective tools are not the exact protocol allowlist: $normalized_tools"
  fi
fi

result_count=$(grep -Fc '"event":"result"' "$tmp_dir/events.ndjson" || true)
[[ $result_count -eq 1 ]] ||
  record_failure "expected exactly one terminal result, found $result_count"
tool_step_count=$(grep -Fc '"step_type":"tool"' "$tmp_dir/events.ndjson" || true)
[[ $tool_step_count -eq 0 ]] ||
  record_failure "neutral preflight unexpectedly executed $tool_step_count tool events"
result_line=$(grep -F '"event":"result"' "$tmp_dir/events.ndjson" | tail -n 1 || true)
if [[ -n $result_line ]]; then
  [[ $result_line == *'"status":"SUCCESS"'* ]] ||
    record_failure "terminal result is not SUCCESS"
  [[ $result_line == *'"num_turns":1'* ]] ||
    record_failure "conversation is not a fresh one-turn session"
  [[ $result_line == *'"cache_read_tokens":0'* ]] ||
    record_failure "conversation reports reused cached context"
fi

conversation_id=$(printf '%s\n' "$init_line" |
  sed -n 's/.*"conversation_id":"\([^"]*\)".*/\1/p')
[[ -n $conversation_id ]] || record_failure "init event has no conversation ID"

printf 'AGY_VERSION %s\n' "$(<"$tmp_dir/version.txt")"
printf 'AGY_MODEL %s\t%s\n' "$model" "$model_label"
printf 'AGY_AGENT %s %s\n' "$agent" "$agent_sha"
printf 'AGY_WORKTREE %s\n' "$repo_root"
printf 'AGY_PROJECT %s\n' "$project_id"
printf 'AGY_PROJECT_RECORD %s\n' "$project_record"
printf 'AGY_CONVERSATION %s\n' "${conversation_id:-unavailable}"
printf 'AGY_EVENTS_BEGIN\n'
cat "$tmp_dir/events.ndjson"
printf 'AGY_EVENTS_END\n'

if [[ $blocked -ne 0 ]]; then
  printf 'AGY_FORMAL_BOUNDARY BLOCKED\n' >&2
  exit 1
fi

printf 'AGY_FORMAL_BOUNDARY PASS\n'

#!/usr/bin/env bash

set -euo pipefail

agy_bin=${AGY_BIN:-agy}
model=${AGY_MODEL:-gemini-3.7-flash-high}
model_label=${AGY_MODEL_LABEL:-Gemini 3.7 Flash (High)}
print_timeout=${AGY_PRINT_TIMEOUT:-2m}

fail() {
  printf 'agy print preflight: FAIL: %s\n' "$*" >&2
  exit 1
}

command -v "$agy_bin" >/dev/null 2>&1 || fail "agy executable not found: $agy_bin"
command -v git >/dev/null 2>&1 || fail "git executable not found"
command -v grep >/dev/null 2>&1 || fail "grep executable not found"

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) ||
  fail "run the preflight inside the target Git worktree"
repo_root=$(cd "$repo_root" && pwd -P)
case "$repo_root" in
  *'"'* | *'\'* | *$'\t'* | *$'\n'* | *$'\r'*)
    fail "worktree path cannot be checked safely in JSON output: $repo_root"
    ;;
esac

tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT

"$agy_bin" --version >"$tmp_dir/version.txt" || fail "agy --version failed"
"$agy_bin" models >"$tmp_dir/models.txt" || fail "agy models failed"
[[ $(wc -l <"$tmp_dir/version.txt") -eq 1 ]] ||
  fail "agy version output must be one line"
grep -Fx "${model}"$'\t'"${model_label}" "$tmp_dir/models.txt" >/dev/null ||
  fail "model catalog does not resolve $model to $model_label"

prompt="Use the terminal tool to run pwd exactly once with its working directory set to $repo_root. Report the exact output. Do not infer it and do not answer with your model name."

set +e
"$agy_bin" -p "$prompt" \
  --model "$model" \
  --effort high \
  --add-dir "$repo_root" \
  --dangerously-skip-permissions \
  --disable-slash-commands \
  --output-format stream-json \
  --print-timeout "$print_timeout" \
  >"$tmp_dir/events.ndjson" 2>"$tmp_dir/stderr.txt"
agy_status=$?
set -e

if [[ $agy_status -ne 0 ]]; then
  cat "$tmp_dir/stderr.txt" >&2
  fail "agy exited $agy_status"
fi

init_line=$(grep -F '"event":"init"' "$tmp_dir/events.ndjson" | head -n 1 || true)
[[ -n $init_line ]] || fail "event stream has no init event"
[[ $init_line == *"\"model\":\"$model\""* ]] ||
  fail "init event does not pin $model"
[[ $init_line == *"\"cwd\":\"$repo_root\""* ]] ||
  fail "init event cwd is not the target worktree"
[[ $init_line == *'"permission_mode":"always-proceed"'* ]] ||
  fail "terminal permission was not pre-approved"

tool_line=$(grep -F '"step_type":"tool"' "$tmp_dir/events.ndjson" |
  grep -F '"state":"DONE"' |
  grep -F '"tool_name":"run_command"' |
  grep -F '"CommandLine":"pwd"' |
  grep -F "\"output\":\"${repo_root}\\r\\n\"" |
  head -n 1 || true)
[[ -n $tool_line ]] ||
  fail "no completed run_command pwd tool event returned the target worktree"

result_line=$(grep -F '"event":"result"' "$tmp_dir/events.ndjson" | tail -n 1 || true)
[[ -n $result_line ]] || fail "event stream has no terminal result"
[[ $result_line == *'"status":"SUCCESS"'* ]] ||
  fail "terminal result is not SUCCESS"

printf 'AGY_VERSION %s\n' "$(<"$tmp_dir/version.txt")"
printf 'AGY_MODEL %s\t%s\n' "$model" "$model_label"
printf 'AGY_WORKTREE %s\n' "$repo_root"
printf 'AGY_INVOCATION prompt-first -p, pinned model, high effort, slash commands disabled\n'
printf 'AGY_EVENTS_BEGIN\n'
cat "$tmp_dir/events.ndjson"
printf 'AGY_EVENTS_END\n'
printf 'AGY_PREFLIGHT PASS\n'

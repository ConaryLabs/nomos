#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'pi cold-agent task: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 7 ]] || fail \
  'usage: pi-cold-agent-task.sh LANE CANDIDATE COMMIT PACKET EVENTS STDERR QUALIFICATION'
lane=$1
candidate=$2
commit=$3
packet=$4
events_out=$5
stderr_out=$6
qualification_out=$7

case $lane in
  deepseek | gemini | claude) ;;
  *) fail "unknown lane: $lane" ;;
esac
[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail 'commit is not a full lowercase SHA-1'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
preflight="$script_dir/pi-cold-agent-preflight.sh"
verify_packet="$script_dir/gate-k-eval-verify-packet.sh"
[[ -x $preflight && -x $verify_packet ]] || fail 'required repository tooling is absent'

for name in jq sha256sum timeout realpath git mktemp cp chmod grep sed; do
  command -v "$name" >/dev/null 2>&1 || fail "required executable not found: $name"
done

candidate=$(realpath -e "$candidate")
packet=$(realpath -e "$packet")
[[ $(git -C "$candidate" rev-parse --show-toplevel) == "$candidate" ]] ||
  fail 'candidate is not a worktree root'
[[ $(git -C "$candidate" rev-parse HEAD) == "$commit" ]] || fail 'candidate HEAD mismatch'
[[ -z $(git -C "$candidate" status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'candidate worktree is dirty'

for output in "$events_out" "$stderr_out" "$qualification_out"; do
  [[ ! -e $output ]] || fail "output already exists: $output"
  parent=$(realpath -e "$(dirname "$output")")
  case "$parent/$(basename "$output")" in
    "$packet"/*) fail 'launcher records must remain outside the subject packet' ;;
  esac
done

"$verify_packet" "$packet" "$commit" >/dev/null
shape=$(jq -r '.task.shape' "$packet/plan.json")
writable=$(jq -r '.packet.writablePaths[0]' "$packet/plan.json")
manifest_sha=$(sha256sum "$packet/packet-manifest.json")
manifest_sha=${manifest_sha%% *}
binary_sha=$(jq -r '.candidate.binarySha256' "$packet/plan.json")
prompt_sha=$(jq -r '.packet.promptSha256' "$packet/plan.json")
prompt=$(<"$packet/prompt.txt")
[[ $(printf '%s' "$prompt" | sha256sum | cut -d' ' -f1) == "$prompt_sha" ]] ||
  fail 'prompt bytes changed while preparing launch'

# This is the mandatory authenticated/offline-fixture neutral probe. Task launch
# is impossible unless the existing repository qualification succeeds first.
"$preflight" "$lane" "$candidate" >"$qualification_out"
grep -Fx 'PI_COLD_AGENT_BOUNDARY PASS' "$qualification_out" >/dev/null ||
  fail 'neutral Pi boundary preflight did not pass'

pi_bin=${PI_BIN:-pi}
pi_path=$(command -v "$pi_bin")
pi_path=$(readlink -f "$pi_path")
extension_line=$(grep -F 'PI_EXTENSION ' "$qualification_out")
extension=${extension_line#PI_EXTENSION }
extension=${extension% *}
extension_sha=${extension_line##* }
[[ $(sha256sum "$extension" | cut -d' ' -f1) == "$extension_sha" ]] ||
  fail 'qualified boundary extension changed before task launch'
provider_extension=$(sed -n 's/^PI_PROVIDER_EXTENSION //p' "$qualification_out")
model_line=$(sed -n 's/^PI_MODEL //p' "$qualification_out")
IFS=$'\t' read -r provider model model_label thinking <<<"$model_line"
[[ -n $provider && -n $model && -n $model_label && -n $thinking ]] ||
  fail 'could not parse the qualified model identity'
system_prompt_line=$(sed -n 's/^PI_SYSTEM_PROMPT //p' "$qualification_out")
system_prompt_file=${system_prompt_line% *}
system_prompt_sha=${system_prompt_line##* }
system_prompt=$(<"$system_prompt_file")
[[ $(printf '%s' "$system_prompt" | sha256sum | cut -d' ' -f1) == "$system_prompt_sha" ]] ||
  fail 'qualified system prompt changed before task launch'
bwrap=$(sed -n 's/^PI_BOUNDARY //p' "$qualification_out" | jq -r '.sandbox.binary')
[[ -x $bwrap ]] || fail "qualified Bubblewrap binary is absent: $bwrap"

tmp_dir=$(mktemp -d)
cleanup() {
  rm -r -- "$tmp_dir"
}
trap cleanup EXIT
config_dir="$tmp_dir/config"
mkdir -m 700 "$config_dir"
source_auth=${PI_AUTH_FILE:-${HOME:?}/.pi/agent/auth.json}
if [[ -f $source_auth ]]; then
  cp "$source_auth" "$config_dir/auth.json"
  chmod 600 "$config_dir/auth.json"
fi
if [[ $lane == deepseek ]]; then
  model_catalog_line=$(sed -n 's/^PI_MODEL_CATALOG //p' "$qualification_out")
  model_catalog=${model_catalog_line% *}
  model_catalog_sha=${model_catalog_line##* }
  [[ $(sha256sum "$model_catalog" | cut -d' ' -f1) == "$model_catalog_sha" ]] ||
    fail 'qualified DeepSeek catalog changed before task launch'
  cp "$model_catalog" "$config_dir/models.json"
  chmod 600 "$config_dir/models.json"
fi

provider_extension_args=()
if [[ $provider_extension != none ]]; then
  [[ -f $provider_extension ]] || fail "qualified provider extension is absent: $provider_extension"
  provider_extension_args=(-e "$provider_extension")
fi

for suffix in BASE_URL PROJECT_ID USER_AGENT RUNTIME_MODEL CLIENT_ID CLIENT_SECRET \
  CALLBACK_HOST NO_KEEPALIVE HTTP2 DEBUG_DUMP; do
  unset "ANTIGRAVITY_$suffix" "NOAGY_$suffix"
done
export ANTIGRAVITY_NO_PREWARM=1
unset NOAGY_NO_PREWARM

raw_events="$tmp_dir/raw-events.ndjson"
raw_stderr="$tmp_dir/raw-stderr.txt"
task_timeout=${PI_TASK_TIMEOUT:-45m}
set +e
(
  cd "$packet"
  PI_CODING_AGENT_DIR="$config_dir" \
  PI_TELEMETRY=0 \
  NOMOS_PI_HOST_WORKSPACE="$packet" \
  NOMOS_PI_BWRAP="$bwrap" \
  NOMOS_PI_BOUNDARY_KIND=packet-run \
  NOMOS_PI_EXPECTED_PROVIDER="$provider" \
  NOMOS_PI_EXPECTED_MODEL="$model" \
  NOMOS_PI_EXPECTED_THINKING="$thinking" \
  NOMOS_PI_SYSTEM_PROMPT_SHA256="$system_prompt_sha" \
  NOMOS_PI_TARGET_COMMIT="$commit" \
  NOMOS_PI_PACKET_MANIFEST_SHA256="$manifest_sha" \
  NOMOS_PI_BINARY_SHA256="$binary_sha" \
  NOMOS_PI_TASK_PROMPT_SHA256="$prompt_sha" \
  NOMOS_PI_TASK_SHAPE="$shape" \
  NOMOS_PI_WRITABLE_PATHS="$writable" \
    timeout "$task_timeout" "$pi_bin" \
    --provider "$provider" \
    --model "$model" \
    --thinking "$thinking" \
    --mode json \
    --no-session \
    --no-approve \
    --offline \
    --no-extensions \
    "${provider_extension_args[@]}" \
    -e "$extension" \
    --no-skills \
    --no-prompt-templates \
    --no-themes \
    --no-context-files \
    --no-builtin-tools \
    --tools bash \
    --system-prompt "$system_prompt" \
    "$prompt"
) >"$raw_events" 2>"$raw_stderr"
pi_status=$?
set -e

for name in $(compgen -e); do
  case $name in
    *API_KEY | *AUTH_TOKEN | *OAUTH_TOKEN | *ACCESS_TOKEN | *REFRESH_TOKEN | *PASSWORD | *SECRET)
      value=${!name}
      if [[ ${#value} -ge 8 ]] && grep -Fq -- "$value" "$raw_events" "$raw_stderr"; then
        fail "provider output leaked credential environment variable $name"
      fi
      ;;
  esac
done
if [[ -f $config_dir/auth.json ]]; then
  while IFS= read -r value; do
    if [[ ${#value} -ge 16 ]] && grep -Fq -- "$value" "$raw_events" "$raw_stderr"; then
      fail 'provider output leaked a stored credential value'
    fi
  done < <(jq -r '.. | strings' "$config_dir/auth.json")
fi

while IFS= read -r line; do
  printf '%s\n' "$line" | jq -e . >/dev/null || fail 'task event stream contains a non-JSON line'
done <"$raw_events"
jq -c 'walk(if type == "object" then del(.textSignature, .thinkingSignature) else . end)' \
  "$raw_events" >"$events_out"
cp "$raw_stderr" "$stderr_out"

boundary_count=$(grep -Fc 'NOMOS_PI_BOUNDARY ' "$stderr_out" || true)
[[ $boundary_count -eq 1 ]] || fail "expected one task boundary record, found $boundary_count"
boundary_json=$(sed -n 's/^NOMOS_PI_BOUNDARY //p' "$stderr_out")
printf '%s\n' "$boundary_json" | jq -e \
  --arg commit "$commit" \
  --arg packet "$packet" \
  --arg provider "$provider" \
  --arg model "$model" \
  --arg thinking "$thinking" \
  --arg manifest_sha "$manifest_sha" \
  --arg binary_sha "$binary_sha" \
  --arg prompt_sha "$prompt_sha" \
  --arg shape "$shape" \
  --arg writable "$writable" '
  .schema == "nomos.pi_cold_agent_boundary@2" and
  .boundaryKind == "packet-run" and
  .targetCommit == $commit and
  .hostWorkspace == $packet and
  .guestWorkspace == "/workspace" and
  .provider == $provider and .model == $model and .thinking == $thinking and
  .sessionFile == null and .projectTrusted == false and
  .entryTypesBeforeRun == ["model_change", "thinking_level_change"] and
  .activeTools == ["bash"] and .contextFiles == [] and .skills == [] and
  .packetManifestSha256 == $manifest_sha and
  .binarySha256 == $binary_sha and
  .taskPromptSha256 == $prompt_sha and
  .taskShape == $shape and
  .writablePaths == [$writable] and
  .sandbox.root == "read-only" and
  .sandbox.workspace == "read-only-packet-with-declared-writable-paths" and
  .sandbox.network == "unshared" and
  .sandbox.environment == "cleared-and-allowlisted" and
  .sandbox.selfTest == "pass" and
  .sandbox.checks.targetCommitResolved == true and
  .sandbox.checks.workspaceRead == true and
  .sandbox.checks.packetManifestMatched == true and
  .sandbox.checks.candidateBinaryMatched == true and
  .sandbox.checks.packetRootReadOnly == true and
  .sandbox.checks.temporaryStorageReadOnly == true and
  .sandbox.checks.deviceFilesystemEmpty == true and
  .sandbox.checks.processFilesystemReadOnly == true and
  .sandbox.checks.declaredWritablePaths == [$writable] and
  .sandbox.checks.gitMetadataAbsent == true and
  .sandbox.checks.outsideReadDenied == true and
  .sandbox.checks.outsideWriteDenied == true and
  .sandbox.checks.credentialEnvironmentAbsent == true and
  .sandbox.checks.networkDenied == true
  ' >/dev/null || fail 'task boundary record does not prove the declared packet isolation'

session_count=$(jq -s '[.[] | select(.type == "session")] | length' "$events_out")
[[ $session_count -eq 1 ]] || fail "expected one task session header, found $session_count"
session_id=$(jq -sr 'map(select(.type == "session"))[0].id // empty' "$events_out")
session_cwd=$(jq -sr 'map(select(.type == "session"))[0].cwd // empty' "$events_out")
[[ $session_id == $(printf '%s\n' "$boundary_json" | jq -r '.sessionId') ]] ||
  fail 'task session and boundary IDs differ'
[[ $session_cwd == "$packet" ]] || fail 'task session cwd differs from the packet'

for event in agent_start agent_end agent_settled; do
  count=$(jq -s --arg event "$event" '[.[] | select(.type == $event)] | length' "$events_out")
  [[ $count -eq 1 ]] || fail "expected one $event event, found $count"
done
agent_end_clean=$(jq -s '
  [.[] | select(.type == "agent_end")] | length == 1 and
  ([.[] | select(.type == "agent_end")][0].willRetry == false)
  ' "$events_out")
[[ $agent_end_clean == true ]] || fail 'agent ended with an unrecorded retry state'
user_prompt_count=$(jq -s --arg prompt "$prompt" '
  [.[] | select(.type == "message_end" and .message.role == "user" and
    .message.content[0].text == $prompt)] | length
  ' "$events_out")
[[ $user_prompt_count -eq 1 ]] || fail 'event stream does not preserve the exact task prompt'
assistant_count=$(jq -s --arg provider "$provider" --arg model "$model" '
  [.[] | select(.type == "message_end" and .message.role == "assistant" and
    .message.provider == $provider and .message.model == $model)] | length
  ' "$events_out")
[[ $assistant_count -ge 1 ]] || fail 'event stream has no matching terminal assistant identity'
accounting_count=$(grep -Fc 'NOMOS_PI_ACCOUNTING ' "$stderr_out" || true)
[[ $accounting_count -eq 1 ]] || fail "expected one accounting record, found $accounting_count"
terminal_assistant=$(jq -s --arg provider "$provider" --arg model "$model" \
  '
  ([.[] | select(.type == "message_end" and .message.role == "assistant")] | last) as $last |
  $last.message.provider == $provider and
  $last.message.model == $model and
  (
    $last.message.stopReason == "stop" and
    ([$last.message.content[] | select(.type == "text") | .text] | join("") | length) > 0
  )
  ' "$events_out")
[[ $terminal_assistant == true ]] || fail 'terminal assistant result identity or completion is missing'

# The writable subtree is expected to change. Everything else remains exactly
# bound to the prelaunch manifest, and no new undeclared path may appear.
jq -r --arg writable "$writable" \
  '.files[] | select(.path != $writable and (.path | startswith($writable + "/") | not)) |
   [.path, .sha256] | @tsv' "$packet/packet-manifest.json" >"$tmp_dir/immutable-rows"
while IFS=$'\t' read -r relative expected_sha; do
  [[ -f $packet/$relative && ! -L $packet/$relative ]] ||
    fail "immutable packet file disappeared: $relative"
  actual_sha=$(sha256sum "$packet/$relative")
  actual_sha=${actual_sha%% *}
  [[ $actual_sha == "$expected_sha" ]] || fail "immutable packet file changed: $relative"
done <"$tmp_dir/immutable-rows"
while IFS= read -r relative; do
  [[ $relative == packet-manifest.json || $relative == "$writable"/* ]] ||
    jq -e --arg path "$relative" 'any(.files[]; .path == $path)' \
      "$packet/packet-manifest.json" >/dev/null || fail "new file escaped writable path: $relative"
done < <(find "$packet" -type f -printf '%P\n' | sort)

printf 'PI_TASK_STATUS %s\n' "$pi_status"
printf 'PI_TASK_MODEL %s\t%s\t%s\t%s\n' "$provider" "$model" "$model_label" "$thinking"
printf 'PI_TASK_SESSION %s ephemeral\n' "$session_id"
printf 'PI_TASK_COMMIT %s\n' "$commit"
printf 'PI_TASK_PACKET_MANIFEST_SHA256 %s\n' "$manifest_sha"
printf 'PI_TASK_EVENTS_SHA256 %s\n' "$(sha256sum "$events_out" | cut -d' ' -f1)"
printf 'PI_TASK_STDERR_SHA256 %s\n' "$(sha256sum "$stderr_out" | cut -d' ' -f1)"
printf 'PI_COLD_AGENT_TASK RECORDED\n'

#!/usr/bin/env bash

set -euo pipefail

pi_bin=${PI_BIN:-pi}
bwrap_bin=${BWRAP_BIN:-bwrap}
run_timeout=${PI_RUN_TIMEOUT:-2m}
expected_pi_version=0.84.2
expected_pi_integrity='sha512-l4E+B7hgXKWddRo8bC/eSue2aWZjEgJ9xIpf5p0Og+lq8a2TArCwJ0HCoCPCgaBP/tN4zbYH/wOwvx9pJpeLCA=='
expected_pi_tree_sha=63a9dd14b0ae82cee2db30c56822682af19145d145febb58b613d5de4dbb27af
expected_antigravity_version=0.4.0
expected_antigravity_integrity='sha512-Trl0lWZRDM6TUhw8UjZ+si4Tx2IxCtLLdEwQ10gOS3BUJfgv/C32HY3m/v9PcLNZWYzo+LEfmamiB5+f0jciCg=='
expected_antigravity_tree_sha=7980e6825a23f18a9d298953c0efc9f13c1231ce4c814394803b9da9bfb565ce
expected_extension_sha=2e1b18887996660e37b36d2a096a985ac05041fbfb03518207dc5cb5d63869fb
expected_fake_sha=cb3ee55c2127137bc7530729f93b97c348d1cca09e864ec9a67c752343d262ff
expected_fake_antigravity_sha=944ab25260d0efee3c682f0d79f84beae674e7fe8a36a585f7615944bcec4417
expected_deepseek_catalog_sha=7954fb3ef750bed773619c9fe259a8eb923b6f4f8455442a33cf8e1fe2fa3773

fail() {
  printf 'pi cold-agent preflight: FAIL: %s\n' "$*" >&2
  exit 1
}

lane=${1:-}
case $lane in
  deepseek)
    provider=deepseek
    model=deepseek-v4-flash-vision-exp
    model_label='DeepSeek V4 Flash Vision Exp'
    thinking=max
    expected_auth_type=api_key
    ;;
  gemini)
    provider=antigravity
    model=gemini-3.7-flash
    model_label='Gemini 3.7 Flash'
    thinking=high
    expected_auth_type=oauth
    ;;
  claude)
    provider=anthropic
    model=claude-opus-5
    model_label='Claude Opus 5'
    thinking=high
    expected_auth_type=oauth
    ;;
  *)
    fail 'usage: pi-cold-agent-preflight.sh deepseek|gemini|claude [workspace]'
    ;;
esac

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
workspace_arg=${2:-$repo_root}
[[ -d $workspace_arg ]] || fail "workspace does not exist: $workspace_arg"
workspace=$(cd "$workspace_arg" && pwd -P)
[[ -f $workspace/README.md ]] || fail "workspace has no README.md: $workspace"
git_root=$(git -C "$workspace" rev-parse --show-toplevel 2>/dev/null) ||
  fail "workspace is not a Git worktree: $workspace"
git_root=$(cd "$git_root" && pwd -P)
[[ $git_root == "$workspace" ]] || fail "workspace is not the worktree root: $workspace"
target_commit=$(git -C "$workspace" rev-parse --verify HEAD)
[[ $target_commit =~ ^[0-9a-f]{40}$ ]] || fail "could not resolve target commit: $target_commit"
case "$workspace" in
  *'"'* | *'\'* | *$'\t'* | *$'\n'* | *$'\r'*)
    fail "workspace path cannot be checked safely in JSON: $workspace"
    ;;
esac

extension="$script_dir/pi-cold-agent-extension.ts"
system_prompt_file="$script_dir/pi-cold-agent-system-prompt.txt"
fake_pi="$script_dir/fixtures/fake-pi-cold-agent"
fake_antigravity="$script_dir/fixtures/fake-pi-antigravity-provider.ts"
deepseek_catalog="$script_dir/pi-deepseek-models.json"
[[ -f $extension ]] || fail "missing Pi boundary extension: $extension"
[[ -f $system_prompt_file ]] || fail "missing Pi system prompt: $system_prompt_file"
[[ -x $fake_pi ]] || fail "missing executable Pi fixture: $fake_pi"
[[ -f $fake_antigravity ]] || fail "missing Pi Antigravity provider fixture: $fake_antigravity"
[[ -f $deepseek_catalog ]] || fail "missing Pi DeepSeek model catalog: $deepseek_catalog"

command -v "$pi_bin" >/dev/null 2>&1 || fail "Pi executable not found: $pi_bin"
command -v "$bwrap_bin" >/dev/null 2>&1 || fail "Bubblewrap executable not found: $bwrap_bin"
command -v git >/dev/null 2>&1 || fail 'git executable not found'
command -v jq >/dev/null 2>&1 || fail 'jq executable not found'
command -v node >/dev/null 2>&1 || fail 'node executable not found'
command -v npm >/dev/null 2>&1 || fail 'npm executable not found'
command -v rustup >/dev/null 2>&1 || fail 'rustup executable not found'
command -v sha256sum >/dev/null 2>&1 || fail 'sha256sum executable not found'
command -v tar >/dev/null 2>&1 || fail 'tar executable not found'
command -v timeout >/dev/null 2>&1 || fail 'timeout executable not found'

extension_sha=$(sha256sum "$extension")
extension_sha=${extension_sha%% *}
[[ $extension_sha == "$expected_extension_sha" ]] ||
  fail "Pi boundary extension digest mismatch: $extension_sha"
fake_sha=$(sha256sum "$fake_pi")
fake_sha=${fake_sha%% *}
[[ $fake_sha == "$expected_fake_sha" ]] || fail "Pi fixture digest mismatch: $fake_sha"
fake_antigravity_sha=$(sha256sum "$fake_antigravity")
fake_antigravity_sha=${fake_antigravity_sha%% *}
[[ $fake_antigravity_sha == "$expected_fake_antigravity_sha" ]] ||
  fail "Pi Antigravity fixture digest mismatch: $fake_antigravity_sha"
deepseek_catalog_sha=$(sha256sum "$deepseek_catalog")
deepseek_catalog_sha=${deepseek_catalog_sha%% *}
[[ $deepseek_catalog_sha == "$expected_deepseek_catalog_sha" ]] ||
  fail "Pi DeepSeek model catalog digest mismatch: $deepseek_catalog_sha"
jq -e '
  .providers.deepseek.models == [{
    "id": "deepseek-v4-flash-vision-exp",
    "name": "DeepSeek V4 Flash Vision Exp",
    "reasoning": true,
    "input": ["text", "image"],
    "cost": {"input": 0.14, "output": 0.28, "cacheRead": 0.0028, "cacheWrite": 0},
    "contextWindow": 1000000,
    "maxTokens": 384000,
    "compat": {
      "supportsStore": false,
      "supportsDeveloperRole": false,
      "maxTokensField": "max_tokens",
      "requiresReasoningContentOnAssistantMessages": true,
      "thinkingFormat": "deepseek"
    },
    "thinkingLevelMap": {
      "minimal": null,
      "low": "low",
      "medium": null,
      "high": "high",
      "xhigh": null,
      "max": "max"
    }
  }]
' "$deepseek_catalog" >/dev/null || fail 'Pi DeepSeek model catalog has unexpected content'

pi_path=$(command -v "$pi_bin")
pi_path=$(readlink -f "$pi_path")
fake_path=$(readlink -f "$fake_pi")
pi_version=$($pi_bin --version) || fail 'pi --version failed'
[[ $pi_version == "$expected_pi_version" ]] ||
  fail "expected Pi $expected_pi_version, got $pi_version"

if [[ $pi_path == "$fake_path" ]]; then
  pi_tree_sha=fixture
  worktree_status=fixture-may-be-dirty
else
  [[ $pi_path == */dist/cli.js ]] || fail "cannot locate installed Pi package from $pi_path"
  pi_root=$(cd "$(dirname "$pi_path")/.." && pwd -P)
  [[ -f $pi_root/package.json ]] || fail "installed Pi package has no package.json: $pi_root"
  package_version=$(jq -r '.version // empty' "$pi_root/package.json")
  [[ $package_version == "$expected_pi_version" ]] ||
    fail "installed Pi package.json reports $package_version"
  pi_tree_sha=$(tar --sort=name --mtime='UTC 1970-01-01' --owner=0 --group=0 \
    --numeric-owner --pax-option=delete=atime,delete=ctime \
    -C "$(dirname "$pi_root")" -cf - "$(basename "$pi_root")" | sha256sum)
  pi_tree_sha=${pi_tree_sha%% *}
  [[ $pi_tree_sha == "$expected_pi_tree_sha" ]] ||
    fail "installed Pi package tree digest mismatch: $pi_tree_sha"
  [[ -z $(git -C "$workspace" status --porcelain=v1 --untracked-files=all) ]] ||
    fail 'authenticated qualification requires a clean target worktree'
  worktree_status=clean
fi

provider_extension=none
provider_extension_args=()
provider_package=none
provider_install=none
if [[ $lane == gemini ]]; then
  if [[ $pi_path == "$fake_path" ]]; then
    provider_extension=${PI_ANTIGRAVITY_EXTENSION:-$fake_antigravity}
    [[ $(readlink -f "$provider_extension") == $(readlink -f "$fake_antigravity") ]] ||
      fail "offline Gemini fixture used an unexpected provider extension: $provider_extension"
    provider_package='pi-antigravity fixture'
    provider_install='fixture'
  else
    npm_root=$(npm root -g)
    antigravity_root="$npm_root/pi-antigravity"
    provider_extension=${PI_ANTIGRAVITY_EXTENSION:-$antigravity_root/src/index.ts}
    [[ $(readlink -f "$provider_extension") == $(readlink -f "$antigravity_root/src/index.ts") ]] ||
      fail "Gemini provider extension is not the pinned package entry point: $provider_extension"
    antigravity_version=$(jq -r '.version // empty' "$antigravity_root/package.json")
    [[ $antigravity_version == "$expected_antigravity_version" ]] ||
      fail "pi-antigravity reports version $antigravity_version"
    antigravity_tree_sha=$(tar --sort=name --mtime='UTC 1970-01-01' --owner=0 --group=0 \
      --numeric-owner --pax-option=delete=atime,delete=ctime \
      -C "$npm_root" -cf - pi-antigravity | sha256sum)
    antigravity_tree_sha=${antigravity_tree_sha%% *}
    [[ $antigravity_tree_sha == "$expected_antigravity_tree_sha" ]] ||
      fail "pi-antigravity package tree digest mismatch: $antigravity_tree_sha"
    provider_package="pi-antigravity@$expected_antigravity_version $expected_antigravity_integrity $antigravity_tree_sha"
    provider_install="npm install -g --ignore-scripts --legacy-peer-deps pi-antigravity@$expected_antigravity_version"
  fi
  provider_extension_args=(-e "$provider_extension")
fi

node_version=$(node --version)
npm_version=$(npm --version)
bwrap_version=$($bwrap_bin --version | head -n 1) || fail 'bwrap --version failed'
bwrap_path=$(readlink -f "$(command -v "$bwrap_bin")")
bwrap_sha=$(sha256sum "$bwrap_path")
bwrap_sha=${bwrap_sha%% *}
rustup_home=$(rustup show home)
rustup_home=$(cd "$rustup_home" && pwd -P)
rust_toolchain=$(rustup show active-toolchain | awk '{print $1}')
[[ -n $rust_toolchain ]] || fail 'could not resolve the active Rust toolchain'

system_prompt=$(<"$system_prompt_file")
system_prompt_sha=$(printf '%s' "$system_prompt" | sha256sum)
system_prompt_sha=${system_prompt_sha%% *}
prompt='Output exactly this line and nothing else: pi boundary preflight'

for suffix in BASE_URL PROJECT_ID USER_AGENT RUNTIME_MODEL CLIENT_ID CLIENT_SECRET \
  CALLBACK_HOST NO_KEEPALIVE HTTP2 DEBUG_DUMP; do
  unset "ANTIGRAVITY_$suffix" "NOAGY_$suffix"
done
export ANTIGRAVITY_NO_PREWARM=1
unset NOAGY_NO_PREWARM

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT
config_dir="$tmp_dir/config"
mkdir -m 700 "$config_dir"
config_profile=ephemeral-auth-only
source_auth=${PI_AUTH_FILE:-${HOME:?}/.pi/agent/auth.json}
if [[ -f $source_auth ]]; then
  cp "$source_auth" "$config_dir/auth.json"
  chmod 600 "$config_dir/auth.json"
fi
if [[ $lane == deepseek ]]; then
  cp "$deepseek_catalog" "$config_dir/models.json"
  chmod 600 "$config_dir/models.json"
  config_profile=ephemeral-auth-plus-pinned-model-catalog
fi

if [[ $lane == gemini && $pi_path != "$fake_path" ]]; then
  jq -e '.antigravity.type == "oauth"' "$config_dir/auth.json" >/dev/null ||
    fail 'Pi Antigravity OAuth credentials are not ready'
  resolved_auth_type=oauth
else
  set +e
  PI_CODING_AGENT_DIR="$config_dir" PI_TELEMETRY=0 \
    timeout 30s "$pi_bin" auth check --provider "$provider" --model "$model" \
    --json >"$tmp_dir/auth.json" 2>"$tmp_dir/auth.stderr"
  auth_status=$?
  set -e
  if [[ $auth_status -ne 0 ]]; then
    cat "$tmp_dir/auth.stderr" >&2
    fail "Pi credentials are not ready for $provider/$model"
  fi
  jq -e --arg provider "$provider" --arg auth_type "$expected_auth_type" \
    '.status == "ready" and .provider == $provider and .authType == $auth_type and
     (has("credential") | not)' \
    "$tmp_dir/auth.json" >/dev/null || fail 'Pi auth check did not return a sanitized ready result'
  resolved_auth_type=$(jq -r '.authType' "$tmp_dir/auth.json")
fi

set +e
PI_CODING_AGENT_DIR="$config_dir" \
PI_TELEMETRY=0 \
NOMOS_PI_HOST_WORKSPACE="$workspace" \
NOMOS_PI_RUSTUP_HOME="$rustup_home" \
NOMOS_PI_RUST_TOOLCHAIN="$rust_toolchain" \
NOMOS_PI_BWRAP="$bwrap_path" \
NOMOS_PI_BOUNDARY_KIND=source-preflight \
NOMOS_PI_EXPECTED_PROVIDER="$provider" \
NOMOS_PI_EXPECTED_MODEL="$model" \
NOMOS_PI_EXPECTED_THINKING="$thinking" \
NOMOS_PI_SYSTEM_PROMPT_SHA256="$system_prompt_sha" \
NOMOS_PI_TARGET_COMMIT="$target_commit" \
  timeout "$run_timeout" "$pi_bin" \
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
  "$prompt" \
  >"$tmp_dir/events.ndjson" 2>"$tmp_dir/stderr.txt"
pi_status=$?
set -e
[[ $pi_status -eq 0 ]] || {
  cat "$tmp_dir/stderr.txt" >&2
  fail "Pi exited $pi_status"
}

for name in $(compgen -e); do
  case $name in
    *API_KEY | *AUTH_TOKEN | *OAUTH_TOKEN | *ACCESS_TOKEN | *REFRESH_TOKEN | *PASSWORD | *SECRET)
      value=${!name}
      if [[ ${#value} -ge 8 ]] && grep -Fq -- "$value" "$tmp_dir/events.ndjson" "$tmp_dir/stderr.txt"; then
        fail "provider output leaked credential environment variable $name"
      fi
      ;;
  esac
done
if [[ -f $config_dir/auth.json ]]; then
  while IFS= read -r value; do
    if [[ ${#value} -ge 16 ]] && grep -Fq -- "$value" "$tmp_dir/events.ndjson" "$tmp_dir/stderr.txt"; then
      fail 'provider output leaked a stored credential value'
    fi
  done < <(jq -r '.. | strings' "$config_dir/auth.json")
fi

if grep -F 'NOMOS_PI_BOUNDARY_BLOCKED ' "$tmp_dir/stderr.txt" >/dev/null; then
  cat "$tmp_dir/stderr.txt" >&2
  fail 'the runtime boundary blocked provider launch'
fi
boundary_count=$(grep -Fc 'NOMOS_PI_BOUNDARY ' "$tmp_dir/stderr.txt" || true)
[[ $boundary_count -eq 1 ]] || fail "expected one runtime boundary record, found $boundary_count"
boundary_json=$(sed -n 's/^NOMOS_PI_BOUNDARY //p' "$tmp_dir/stderr.txt")
printf '%s\n' "$boundary_json" | jq -e \
  --arg workspace "$workspace" \
  --arg target_commit "$target_commit" \
  --arg provider "$provider" \
  --arg model "$model" \
  --arg thinking "$thinking" \
  --arg extension "$extension" \
  --arg bwrap "$bwrap_path" \
  --arg system_prompt_sha "$system_prompt_sha" '
    .schema == "nomos.pi_cold_agent_boundary@2" and
    .boundaryKind == "source-preflight" and
    .mode == "json" and
    .targetCommit == $target_commit and
    .hostWorkspace == $workspace and
    .guestWorkspace == "/workspace" and
    .provider == $provider and
    .model == $model and
    .thinking == $thinking and
    (.sessionId | test("^[0-9a-f-]{36}$")) and
    .sessionFile == null and
    .projectTrusted == false and
    .entryTypesBeforeRun == ["model_change", "thinking_level_change"] and
    .activeTools == ["bash"] and
    (.configuredTools | length) == 1 and
    .configuredTools[0].name == "bash" and
    .configuredTools[0].source.path == $extension and
    .contextFiles == [] and
    .skills == [] and
    .systemPromptSha256 == $system_prompt_sha and
    .packetManifestSha256 == null and
    .binarySha256 == null and
    .taskPromptSha256 == null and
    .taskShape == null and
    .writablePaths == [] and
    .budgets == null and
    .sandbox.backend == "bubblewrap" and
    .sandbox.binary == $bwrap and
    .sandbox.root == "read-only" and
    .sandbox.workspace == "read-write-only-host-mount" and
    .sandbox.network == "unshared" and
    .sandbox.environment == "cleared-and-allowlisted" and
    .sandbox.checks == {
      "targetCommitResolved": true,
      "workspaceRead": true,
      "workspaceWrite": true,
      "outsideReadDenied": true,
      "outsideWriteDenied": true,
      "credentialEnvironmentAbsent": true,
      "networkDenied": true,
      "cargoAvailable": true
    } and
    .sandbox.selfTest == "pass"
  ' >/dev/null || fail 'runtime boundary record does not prove the required state'

while IFS= read -r line; do
  printf '%s\n' "$line" | jq -e . >/dev/null || fail 'Pi JSON stream contains a non-JSON line'
done <"$tmp_dir/events.ndjson"
jq -c 'walk(if type == "object" then del(.textSignature, .thinkingSignature) else . end)' \
  "$tmp_dir/events.ndjson" >"$tmp_dir/sanitized.ndjson"

session_count=$(jq -s '[.[] | select(.type == "session")] | length' "$tmp_dir/events.ndjson")
[[ $session_count -eq 1 ]] || fail "expected one session header, found $session_count"
session_id=$(jq -sr 'map(select(.type == "session"))[0].id // empty' "$tmp_dir/events.ndjson")
session_cwd=$(jq -sr 'map(select(.type == "session"))[0].cwd // empty' "$tmp_dir/events.ndjson")
boundary_session_id=$(printf '%s\n' "$boundary_json" | jq -r '.sessionId')
[[ $session_id == "$boundary_session_id" ]] || fail 'session and boundary IDs differ'
[[ $session_cwd == "$workspace" ]] || fail "session cwd is not the target workspace: $session_cwd"

for event in agent_start turn_start turn_end agent_end agent_settled; do
  count=$(jq -s --arg event "$event" '[.[] | select(.type == $event)] | length' "$tmp_dir/events.ndjson")
  [[ $count -eq 1 ]] || fail "expected one $event event, found $count"
done
tool_events=$(jq -s '[.[] | select(.type | startswith("tool_execution_"))] | length' "$tmp_dir/events.ndjson")
[[ $tool_events -eq 0 ]] || fail "neutral probe unexpectedly executed $tool_events tool events"

if ! jq -se --arg provider "$provider" --arg model "$model" --arg prompt "$prompt" '
  ([.[] | select(.type == "message_end" and .message.role == "user")] | length) == 1 and
  ([.[] | select(.type == "message_end" and .message.role == "user")][0].message.content[0].text) == $prompt and
  ([.[] | select(.type == "message_end" and .message.role == "assistant")] | length) == 1 and
  ([.[] | select(.type == "message_end" and .message.role == "assistant")][0].message.provider) == $provider and
  ([.[] | select(.type == "message_end" and .message.role == "assistant")][0].message.model) == $model and
  ([.[] | select(.type == "message_end" and .message.role == "assistant")][0].message.stopReason) == "stop" and
  ([.[] | select(.type == "message_end" and .message.role == "assistant")][0].message.content | map(select(.type == "text") | .text) | join("")) == "pi boundary preflight"
' "$tmp_dir/events.ndjson" >/dev/null; then
  cat "$tmp_dir/stderr.txt" >&2
  cat "$tmp_dir/sanitized.ndjson" >&2
  fail 'Pi did not return the exact successful neutral response'
fi

printf 'PI_VERSION %s\n' "$pi_version"
printf 'PI_INSTALL npm install -g --ignore-scripts @earendil-works/pi-coding-agent@%s\n' "$expected_pi_version"
printf 'PI_NPM_INTEGRITY %s\n' "$expected_pi_integrity"
printf 'PI_PACKAGE_TREE_SHA256 %s\n' "$pi_tree_sha"
printf 'PI_NODE %s\n' "$node_version"
printf 'PI_NPM %s\n' "$npm_version"
printf 'PI_BWRAP %s\n' "$bwrap_version"
printf 'PI_BWRAP_SHA256 %s\n' "$bwrap_sha"
printf 'PI_RUST_TOOLCHAIN %s\n' "$rust_toolchain"
printf 'PI_HOST_OS %s\n' "$(uname -srm)"
printf 'PI_TARGET_COMMIT %s\n' "$target_commit"
printf 'PI_LANE %s\n' "$lane"
printf 'PI_MODEL %s\t%s\t%s\t%s\n' "$provider" "$model" "$model_label" "$thinking"
printf 'PI_AUTH_TYPE %s\n' "$resolved_auth_type"
printf 'PI_EXTENSION %s %s\n' "$extension" "$extension_sha"
printf 'PI_PROVIDER_EXTENSION %s\n' "$provider_extension"
printf 'PI_PROVIDER_PACKAGE %s\n' "$provider_package"
printf 'PI_PROVIDER_INSTALL %s\n' "$provider_install"
printf 'PI_PROVIDER_ENV overrides-cleared prewarm-disabled\n'
if [[ $lane == deepseek ]]; then
  printf 'PI_MODEL_CATALOG %s %s\n' "$deepseek_catalog" "$deepseek_catalog_sha"
else
  printf 'PI_MODEL_CATALOG none\n'
fi
printf 'PI_SYSTEM_PROMPT %s %s\n' "$system_prompt_file" "$system_prompt_sha"
printf 'PI_WORKSPACE %s\n' "$workspace"
printf 'PI_WORKTREE_STATUS %s\n' "$worktree_status"
printf 'PI_CONFIG_ROOT %s %s\n' "$config_dir" "$config_profile"
printf 'PI_SESSION %s ephemeral\n' "$session_id"
if [[ $provider_extension == none ]]; then
  provider_invocation=''
else
  provider_invocation=" -e $provider_extension"
fi
printf 'PI_INVOCATION pi --provider %s --model %s --thinking %s --mode json --no-session --no-approve --offline --no-extensions%s -e %s --no-skills --no-prompt-templates --no-themes --no-context-files --no-builtin-tools --tools bash --system-prompt <sha256:%s> <neutral-prompt>\n' \
  "$provider" "$model" "$thinking" "$provider_invocation" "$extension" "$system_prompt_sha"
printf 'PI_BOUNDARY %s\n' "$boundary_json"
printf 'PI_EVENTS_BEGIN\n'
cat "$tmp_dir/sanitized.ndjson"
printf 'PI_EVENTS_END\n'
printf 'PI_COLD_AGENT_BOUNDARY PASS\n'

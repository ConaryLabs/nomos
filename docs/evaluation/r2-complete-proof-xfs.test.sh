#!/usr/bin/env bash
# shellcheck disable=SC2016,SC2034,SC2154 # Static plants and sourced dynamic-scope helpers are intentional.
set -Eeuo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_script_directory=$script_directory
wrapper=$script_directory/r2-complete-proof-xfs.sh
workdir_helper=$script_directory/r2-complete-proof-xfs-workdir.sh
receipt_helper=$script_directory/r2-complete-proof-xfs-receipt.mjs
inner=$script_directory/r2-complete-proof.sh
workflow=$script_directory/../../.github/workflows/nomos-viewer.yml
[[ -f $workdir_helper && ! -L $workdir_helper ]]
fail() {
  printf 'r2-complete-proof-xfs shell validation tests: FAIL: %s\n' "$*" >&2
  exit 1
}
node_bin=$(type -P node) || fail 'node executable is missing from the validation PATH'
node_bin=$(/usr/bin/realpath -e -- "$node_bin") || fail 'node executable is not canonicalizable'
[[ $node_bin == /* && -f $node_bin && -x $node_bin && ! -L $node_bin ]] ||
  fail 'node executable is not one canonical regular file'
[[ ${NOMOS_R2_PROOF_INNER:-} != 1 ]] ||
  fail 'XFS shell validation must run in the unprivileged outer proof before formal Bubblewrap'
# shellcheck source=docs/evaluation/r2-complete-proof-xfs-workdir.sh
source "$workdir_helper"
outer=$script_directory/r2-complete-proof-outer.sh
# shellcheck source=docs/evaluation/r2-complete-proof-outer.sh
source "$outer"
grep -Fq 'r2_pin_work_directory' "$wrapper"
grep -Fq 'work_fd_path=/proc/self/fd/' "$workdir_helper"
grep -Fq 'r2_public_open_work_directory' "$workdir_helper"
grep -Fq 'r2_public_pinned_exec' "$workdir_helper"
if grep -Fq '/usr/bin/node' "$wrapper"; then
  fail 'production wrapper retains the obsolete system Node assumption'
fi
grep -Fq 'caller_node=$(type -P node)' "$wrapper"
grep -Fq 'record_wrapper_user_tool "$tool_register" node "$caller_node"' "$wrapper"
grep -Fq 'run_as_user "$caller_node"' "$wrapper"
[[ $(grep -Fc 'r2_public_pinned_exec "$caller_node"' "$wrapper") -eq 2 ]] ||
  fail 'public receipt commands do not use the exact caller Node handoff'
inner_viewer_test_text=$(/usr/bin/awk '/^r2_viewer_tests\(\) \{/{inside=1} inside{print} inside && /^\}/{exit}' "$inner")
grep -Fq 'docs/evaluation/r2-complete-proof.test.sh' <<<"$inner_viewer_test_text"
if grep -Fq 'docs/evaluation/r2-complete-proof-xfs.test.sh' <<<"$inner_viewer_test_text"; then
  fail 'host XFS shell validation is nested inside the formal inner proof'
fi
grep -Fq '/usr/bin/bash "$repo_root/docs/evaluation/r2-complete-proof-xfs.test.sh"' "$outer" ||
  fail 'formal outer proof does not execute the XFS shell-validation suite'
grep -Fq 'docs/evaluation/r2-complete-proof-xfs.test.sh' "$workflow" ||
  fail 'hosted preflight does not execute the XFS shell-validation suite'
observed_workflow_text=$(/usr/bin/awk '/^  r2-observed-viewer:/{inside=1} /^  r2-complete-proof:/{if (inside) exit} inside{print}' "$workflow")
detached_workflow_text=$(/usr/bin/awk '/^  r2-complete-proof:/{inside=1} inside{print}' "$workflow")
mapfile -t hosted_detached_work_paths < <(/usr/bin/sed -n \
  's/^[[:space:]]*work=\(\/tmp\/[A-Za-z0-9._-]*\)$/\1/p' <<<"$detached_workflow_text")
[[ ${#hosted_detached_work_paths[@]} -eq 1 ]] ||
  fail 'hosted detached proof does not declare one short literal /tmp work path'
hosted_detached_work=${hosted_detached_work_paths[0]}
hosted_singleton_socket=$hosted_detached_work/fs/checkout/target/r2-complete-proof/host/tmp/com.google.Chrome.XXXXXX/SingletonSocket
[[ ${#hosted_singleton_socket} -le 107 ]] ||
  fail 'hosted detached proof exceeds the Linux Chrome ProcessSingleton socket-path limit'
grep -Fq 'test ! -e "$work"' <<<"$detached_workflow_text" ||
  fail 'hosted detached proof does not refuse an existing work path'
grep -Fq 'mkdir -m 0700 -- "$work"' <<<"$detached_workflow_text" ||
  fail 'hosted detached proof does not atomically create its private work directory'
for hosted_artifact_path in \
  "$hosted_detached_work" \
  "!$hosted_detached_work/filesystem.xfs" \
  "!$hosted_detached_work/checkout.tar" \
  "!$hosted_detached_work/fs" \
  "!$hosted_detached_work/user-env"; do
  grep -Fxq "            $hosted_artifact_path" <<<"$detached_workflow_text" ||
    fail "hosted detached evidence upload lost its work-root path: $hosted_artifact_path"
done
grep -Fxq "          path: $hosted_detached_work/filesystem.xfs" <<<"$detached_workflow_text" ||
  fail 'hosted detached red-image upload differs from the proof work root'
if grep -Fq '${{ runner.temp }}/nomos-r2-xfs' <<<"$detached_workflow_text"; then
  fail 'hosted detached proof retains the overlong runner-temporary work path'
fi
grep -Fq 'CHROME_BIN=$(realpath -e -- "$(command -v google-chrome)")' <<<"$detached_workflow_text" ||
  fail 'hosted detached proof does not pass a canonical Chrome executable'
assert_hosted_sandbox_lane() {
  local lane_text=$1 proof_step=$2 label=$3 prepare_line proof_line restore_line output_line clear_line
  local expected_restore_output="original='\${{ steps.r2_sandbox.outputs.original }}'"
  [[ $(grep -Fc 'name: Prepare the unprivileged R2 sandbox' <<<"$lane_text") -eq 1 &&
     $(grep -Fc 'name: Restore the host AppArmor user-namespace gate' <<<"$lane_text") -eq 1 ]] ||
    fail "$label does not configure and restore exactly one hosted sandbox gate"
  grep -Fq 'gate=kernel.apparmor_restrict_unprivileged_userns' <<<"$lane_text"
  grep -Fq 'case "$original" in 0|1) ;; *) exit 1 ;; esac' <<<"$lane_text"
  grep -Fq 'sudo /usr/sbin/sysctl -w "$gate=0"' <<<"$lane_text"
  grep -Fq 'test "$(/usr/sbin/sysctl -n "$gate")" = 0' <<<"$lane_text"
  grep -Fq "$expected_restore_output" <<<"$lane_text"
  grep -Fq 'sudo /usr/sbin/sysctl -w "$gate=$original"' <<<"$lane_text"
  grep -Fq 'test "$(/usr/sbin/sysctl -n "$gate")" = "$original"' <<<"$lane_text"
  grep -Fq $'name: Restore the host AppArmor user-namespace gate\n        if: always()' <<<"$lane_text"
  grep -Fq '/usr/bin/setpriv --no-new-privs /usr/bin/bwrap' <<<"$lane_text"
  grep -Fq -- '--die-with-parent --new-session --unshare-net --unshare-pid' <<<"$lane_text"
  grep -Fq 'for field in CapInh CapPrm CapEff CapBnd CapAmb' <<<"$lane_text"
  grep -Fq 'NoNewPrivs:' <<<"$lane_text"
  grep -Fq 'ip -j address show' <<<"$lane_text"
  grep -Fq 'route show table all' <<<"$lane_text"
  grep -Fq 'findmnt --uniq --json -T / -o TARGET,OPTIONS' <<<"$lane_text"
  prepare_line=$(grep -nF 'name: Prepare the unprivileged R2 sandbox' <<<"$lane_text" | cut -d: -f1)
  output_line=$(grep -nF "printf 'original=%s\\n'" <<<"$lane_text" | cut -d: -f1)
  clear_line=$(grep -nF 'sudo /usr/sbin/sysctl -w "$gate=0"' <<<"$lane_text" | cut -d: -f1)
  proof_line=$(grep -nF "name: $proof_step" <<<"$lane_text" | cut -d: -f1)
  restore_line=$(grep -nF 'name: Restore the host AppArmor user-namespace gate' <<<"$lane_text" | cut -d: -f1)
  [[ $prepare_line =~ ^[0-9]+$ && $output_line =~ ^[0-9]+$ && $clear_line =~ ^[0-9]+$ &&
     $proof_line =~ ^[0-9]+$ && $restore_line =~ ^[0-9]+$ &&
     $prepare_line -lt $output_line && $output_line -lt $clear_line &&
     $clear_line -lt $proof_line && $proof_line -lt $restore_line ]] ||
    fail "$label sandbox probe or restoration is ordered incorrectly"
}
assert_hosted_sandbox_lane "$observed_workflow_text" \
  'R2 registers, plants, signatures, and Node tests' 'hosted portable lane'
assert_hosted_sandbox_lane "$detached_workflow_text" \
  'Run the detached complete proof' 'hosted detached lane'
if grep -Eq -- '--share-net|sudo[[:space:]].*bwrap|sudo[[:space:]].*r2-complete-proof' "$workflow"; then
  fail 'hosted repair weakens or privileges the candidate isolation boundary'
fi
grep -Fq '/usr/bin/bash "$pinned_supervisor_path" --pinned-supervise' "$wrapper"
grep -Fq -- '--config core.hooksPath=/dev/null "$source_fd_path" "$checkout"' "$wrapper"
[[ $(grep -Fc '"/usr/sbin/mkfs.xfs","-f","-K","-l","internal",($loop_path|nz)' "$wrapper") -eq 1 ]] ||
  fail 'supervisor facts do not bind the no-discard XFS format command exactly once'
[[ $(grep -Fc '/usr/sbin/mkfs.xfs -f -K -l internal "$loop_device"' "$wrapper") -eq 1 ]] ||
  fail 'supervisor does not execute the no-discard XFS format command exactly once'
[[ $(grep -Fc 'capture_image_stat "$image"' "$wrapper") -eq 2 ]] ||
  fail 'supervisor does not capture exactly the pre-format and post-teardown image checkpoints'
grep -Fq 'image_pre_format_stat=$work/image-pre-format.stat' "$workdir_helper" ||
  fail 'pinned work helper does not reset the pre-format stat below the descriptor'
grep -Fq 'image_post_teardown_stat=$work/image-post-teardown.stat' "$workdir_helper" ||
  fail 'pinned work helper does not reset the post-teardown stat below the descriptor'
if grep -Fq 'image_stat=$work/image.stat' "$workdir_helper"; then
  fail 'pinned work helper retains the stale single-checkpoint stat path'
fi
image_sync_line=$(grep -nF '[[ $image_sync_status -eq 0 ]]' "$wrapper" | cut -d: -f1)
pre_format_stat_line=$(grep -nF 'capture_image_stat "$image" "$image_pre_format_stat"' "$wrapper" | cut -d: -f1)
mkfs_line=$(grep -nF '/usr/sbin/mkfs.xfs -f -K -l internal "$loop_device"' "$wrapper" | cut -d: -f1)
[[ $image_sync_line =~ ^[0-9]+$ && $pre_format_stat_line =~ ^[0-9]+$ && $mkfs_line =~ ^[0-9]+$ &&
   $pre_format_stat_line -gt $image_sync_line && $pre_format_stat_line -lt $mkfs_line ]] ||
  fail 'pre-format image stat is not captured after sync and before formatting'
image_unattached_line=$(grep -nF '[[ $image_unattached == true ]]' "$wrapper" | cut -d: -f1)
post_teardown_stat_line=$(grep -nF 'capture_image_stat "$image" "$image_post_teardown_stat"' "$wrapper" | cut -d: -f1)
[[ $image_unattached_line =~ ^[0-9]+$ && $post_teardown_stat_line =~ ^[0-9]+$ &&
   $post_teardown_stat_line -gt $image_unattached_line ]] ||
  fail 'post-teardown image stat is not captured after attachment absence is proved'
if grep -Fq '/usr/bin/mount --bind' "$workdir_helper"; then
  printf 'descriptor boundary must not rely on a canonical-path bind mount\n' >&2
  exit 1
fi
r2_validate_private_parent_targets /tmp/source /tmp/work /data/dev/helpers
if r2_validate_private_parent_targets /work /tmp/source; then
  fail 'receipt sandbox accepted a direct child of the filesystem root'
fi
grep -Fq 'r2_validate_precreated_work_inventory' "$workdir_helper"
grep -Fq "! -L \$entry" "$workdir_helper"
grep -Fq "'%h'" "$workdir_helper"
grep -Fq 'outer_preflight_log=$work/outer-preflight.json' "$wrapper"
grep -Fq 'NOMOS_R2_OUTER_PREFLIGHT_LOG="$outer_preflight_log"' "$wrapper"
grep -Fq '/usr/bin/env NOMOS_R2_XFS_WRAPPER=1' "$wrapper"
grep -Fq 'display_mount_path=$(r2_display_work_path "$fs")' "$wrapper"
grep -Fq 'mount_target == "$display_mount_path"' "$wrapper"
if grep -Fq 'mount_target == "$fs"' "$wrapper"; then
  fail 'mounted XFS identity still compares canonical evidence with a descriptor spelling'
fi
if grep -Fq 'chmod 0733' "$wrapper"; then
  printf 'wrapper temporarily reopens caller top-level write authority\n' >&2
  exit 1
fi
# The formal outer proof deliberately spells TMPDIR through a retained
# descriptor. Recreate that topology on every standalone run, then canonicalize
# the fixture root once so canonical-path guards and receipt expectations do not
# accidentally compare a /proc spelling with the helper's realpath result.
test_tmp_parent=${TMPDIR:-/tmp}
exec {test_tmp_fd}<"$test_tmp_parent" || fail 'could not retain the shell-test temporary parent'
test_root_spelling=$(/usr/bin/mktemp -d "/proc/self/fd/$test_tmp_fd/nomos-r2-xfs-shell-test.XXXXXX") || {
  exec {test_tmp_fd}<&-
  fail 'could not create the shell-test fixture through its retained descriptor'
}
test_root=$(/usr/bin/realpath -e -- "$test_root_spelling") || {
  /usr/bin/rm -rf -- "$test_root_spelling"
  exec {test_tmp_fd}<&-
  fail 'could not canonicalize the descriptor-spelled shell-test fixture'
}
[[ -d $test_root && ! -L $test_root && $test_root != "$test_root_spelling" ]] || {
  /usr/bin/rm -rf -- "$test_root_spelling"
  exec {test_tmp_fd}<&-
  fail 'descriptor-spelled shell-test fixture did not resolve to one real directory'
}
exec {test_tmp_fd}<&-
trap 'rm -rf -- "$test_root"' EXIT

# The outer preflight is the union of every tool the inner proof can execute
# and every tool the wrapper itself executes. Keep that declaration in exact
# parity with the inner harness and both complete-proof workflow lanes.
inner_tool_text=$(/usr/bin/awk '/^host_tools=\(/ {inside=1; next} inside && /^\)/ {exit} inside {print}' "$inner" | /usr/bin/tr '\n' ' ')
read -r -a declared_inner_tools <<<"$inner_tool_text"
[[ ${declared_inner_tools[*]} == "${R2_INNER_REQUIRED_TOOLS[*]}" ]] ||
  fail 'outer preflight tool set differs from the inner proof'
expected_workflow_tools=$(printf '%s\n' "${R2_INNER_REQUIRED_TOOLS[@]}" "${R2_WRAPPER_REQUIRED_TOOLS[@]}" |
  /usr/bin/sed 's#.*/##' | /usr/bin/sort -u)
mapfile -t workflow_tool_lists < <(/usr/bin/awk '
  /^[[:space:]]*for tool in \\/ {inside=1; list=""; next}
  inside {
    line=$0
    sub(/; do.*/, "", line)
    gsub(/\\/, "", line)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
    list=list " " line
    if ($0 ~ /; do/) {sub(/^ /, "", list); print list; inside=0}
  }
' "$workflow")
[[ ${#workflow_tool_lists[@]} -eq 2 ]] || fail 'complete-proof workflow tool loops differ in count'
for workflow_tool_list in "${workflow_tool_lists[@]}"; do
  read -r -a workflow_tools <<<"$workflow_tool_list"
  actual_workflow_tools=$(printf '%s\n' "${workflow_tools[@]}" | /usr/bin/sed 's#.*/##' | /usr/bin/sort -u)
  [[ $actual_workflow_tools == "$expected_workflow_tools" ]] ||
    fail 'complete-proof workflow preflight differs from the executable tool union'
done
wrapper_tool_tsv=$test_root/wrapper-tools.tsv
wrapper_tool_json=$test_root/wrapper-tools.json
toolcache_node_directory=$test_root/toolcache/node/22.0.0/x64/bin
mkdir -p -- "$toolcache_node_directory"
cp -- "$node_bin" "$toolcache_node_directory/node"
chmod 0755 -- "$toolcache_node_directory/node"
toolcache_node=$(/usr/bin/realpath -e -- "$toolcache_node_directory/node")
toolcache_caller_path=$toolcache_node_directory:$PATH
(
  run_as_user() { /usr/bin/env PATH="$toolcache_caller_path" "$@"; }
  record_wrapper_tools "$wrapper_tool_tsv"
  record_wrapper_user_tool "$wrapper_tool_tsv" node "$toolcache_node"
  for wrapper_user_tool in rustup cargo rustc; do
    record_wrapper_user_tool "$wrapper_tool_tsv" "$wrapper_user_tool"
  done
  record_wrapper_tool_json "$wrapper_tool_tsv" "$wrapper_tool_json"
)
pwd_tool_path=$(/usr/bin/awk -F '\t' '$1 == "pwd" {print $2}' "$wrapper_tool_tsv")
[[ $pwd_tool_path == /* && -x $pwd_tool_path ]] || fail 'wrapper tool recorder did not resolve external pwd'
recorded_node_path=$(/usr/bin/awk -F '\t' '$1 == "node" {print $2}' "$wrapper_tool_tsv")
[[ $recorded_node_path == "$toolcache_node" ]] || fail 'wrapper tool recorder did not bind non-system Node exactly'
base_validation_path=$PATH
r2_validate_caller_path "$toolcache_node_directory:$base_validation_path" \
  "$base_validation_path" "$toolcache_node"
shadow_directory=$test_root/shadowed-node-bin
mkdir -- "$shadow_directory"
cp -- "$node_bin" "$shadow_directory/node"
cp -- /usr/bin/false "$shadow_directory/jq"
chmod 0755 -- "$shadow_directory/node" "$shadow_directory/jq"
set +e
(r2_validate_caller_path "$shadow_directory:$base_validation_path" \
  "$base_validation_path" "$shadow_directory/node") >"$test_root/shadow.stdout" 2>"$test_root/shadow.stderr"
shadow_status=$?
set -e
[[ $shadow_status -ne 0 ]] || fail 'node directory was allowed to shadow a required proof tool'
grep -Fq 'node directory shadows a required caller tool: jq' "$test_root/shadow.stderr"

# findmnt reports the canonical mount-table target even when its lookup path
# is a retained descriptor. A Bubblewrap namespace can stack its writable
# /proc over the read-only /proc inherited from the root bind, so collapse
# duplicate target rows without hiding distinct targets. The production
# comparison must account for descriptor normalization without changing the
# descriptor-derived mount operation.
exec {findmnt_fd}</proc
findmnt_descriptor=/proc/self/fd/$findmnt_fd
findmnt_record=$(/usr/bin/findmnt --uniq --json -T "$findmnt_descriptor" -o TARGET) ||
  fail 'findmnt could not inspect the descriptor target'
findmnt_target=$(/usr/bin/jq -er 'if (.filesystems | length) == 1 then .filesystems[0].target else empty end' <<<"$findmnt_record") ||
  fail 'findmnt descriptor targets are absent or conflicting'
findmnt_canonical=$(/usr/bin/realpath -e -- "$findmnt_descriptor")
[[ $findmnt_target == "$findmnt_canonical" && $findmnt_target != "$findmnt_descriptor" ]] ||
  fail 'findmnt descriptor-target normalization differs'
exec {findmnt_fd}<&-

# Every jq variable in the facts program must be introduced by its jq argv.
# This catches a typo before cleanup can turn it into an empty evidence file.
write_facts_text=$(/usr/bin/awk '/^  write_facts\(\) \{/{inside=1} inside{print} inside && /^  \}$/{exit}' "$wrapper")
write_facts_arguments=$(printf '%s\n' "$write_facts_text" |
  /usr/bin/grep -oE -- '--arg(json)?[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' |
  /usr/bin/awk '{print $2}' | /usr/bin/sort -u)
write_facts_program=$(printf '%s\n' "$write_facts_text" |
  /usr/bin/awk '/def nz: if/{inside=1} inside{print} inside && /tool_register:\$wrapper_tools}/{exit}')
write_facts_references=$(printf '%s\n' "$write_facts_program" |
  /usr/bin/grep -oE '\$[A-Za-z_][A-Za-z0-9_]*' | /usr/bin/tr -d '$' |
  /usr/bin/grep -vx tmp | /usr/bin/sort -u)
write_facts_missing=$(/usr/bin/awk 'NR == FNR {present[$0]=1; next} !($0 in present)' \
  <(printf '%s\n' "$write_facts_arguments") <(printf '%s\n' "$write_facts_references"))
[[ -z $write_facts_missing ]] || fail "facts jq has unbound variables: $write_facts_missing"

# Node canonicalizes an ES-module entrypoint reached through /proc/self/fd.
# The cloned export helper must still recognize that descriptor-spelled entry
# as its own CLI rather than silently importing and exiting zero.
grep -Fq 'let invokedAsMain = import.meta.main;' "$receipt_helper" ||
  fail 'receipt helper does not use loader-provided main-module identity'
set +e
receipt_canonical_output=$("$node_bin" "$receipt_helper" unknown-command 2>&1)
receipt_canonical_status=$?
set -e
[[ $receipt_canonical_status -eq 2 && $receipt_canonical_output == usage:* ]] ||
  fail "canonical receipt helper did not enter its CLI: status=$receipt_canonical_status output=$receipt_canonical_output"
exec {receipt_entry_fd}<"$script_directory"
set +e
receipt_entry_output=$("$node_bin" "/proc/self/fd/$receipt_entry_fd/r2-complete-proof-xfs-receipt.mjs" unknown-command 2>&1)
receipt_entry_status=$?
set -e
[[ $receipt_entry_status -eq 2 && $receipt_entry_output == usage:* ]] ||
  fail "descriptor-spelled receipt helper did not enter its CLI: status=$receipt_entry_status output=$receipt_entry_output"
receipt_cli_root=$test_root/receipt-cli
mkdir -p -- "$receipt_cli_root/source" "$receipt_cli_root/export/target"
printf 'descriptor CLI\n' >"$receipt_cli_root/source/value"
receipt_cli_canonical=$(/usr/bin/realpath -e -- "$receipt_cli_root") ||
  fail 'could not canonicalize the descriptor CLI fixture'
exec {receipt_cli_fd}<"$receipt_cli_root"
receipt_cli_descriptor=/proc/self/fd/$receipt_cli_fd
"$node_bin" "/proc/self/fd/$receipt_entry_fd/r2-complete-proof-xfs-receipt.mjs" copy \
  --source "$receipt_cli_descriptor/source" \
  --destination "$receipt_cli_descriptor/export/target/r2-complete-proof" \
  --output "$receipt_cli_descriptor/inventory.json" >"$receipt_cli_root/copy.stdout"
/usr/bin/jq -e --arg source "$receipt_cli_canonical/source" \
  --arg destination "$receipt_cli_canonical/export/target/r2-complete-proof" \
  '.source == $source and .destination == $destination and .equal == true' \
  "$receipt_cli_root/inventory.json" >/dev/null ||
  fail 'descriptor CLI inventory did not publish canonical paths'
/usr/bin/cmp "$receipt_cli_root/source/value" \
  "$receipt_cli_root/export/target/r2-complete-proof/value"
exec {receipt_cli_fd}<&-
exec {receipt_entry_fd}<&-

# GNU find does not descend through a command-line descriptor symlink under
# its default -P policy. The production inventory must explicitly follow that
# one root link while retaining -P behavior for every entry beneath it.
[[ $(/usr/bin/grep -Fc 'find -H -- "$fs" -mindepth 1 -maxdepth 1 -printf' "$wrapper") -eq 2 ]] ||
  fail 'both XFS sole-entry checks must follow only their descriptor root'
precreated_text=$(/usr/bin/awk '/^PRECREATED_WORK_FILES=\(/ {inside=1; next} inside && /^\)/ {exit} inside {print}' "$wrapper" | /usr/bin/tr '\n' ' ')
read -r -a PRECREATED_WORK_FILES <<<"$precreated_text"
inventory_root=$test_root/descriptor-inventory
mkdir -- "$inventory_root"
for inventory_name in "${PRECREATED_WORK_FILES[@]}"; do
  : >"$inventory_root/$inventory_name"
  chmod 0644 -- "$inventory_root/$inventory_name"
done
exec {inventory_fd}<"$inventory_root"
r2_validate_precreated_work_inventory "/proc/self/fd/$inventory_fd" "$(id -u)" "$(id -g)"
exec {inventory_fd}<&-

# Cleanup runs after the root supervisor has restored the work directory to
# the caller. If a failed facts writer left an invalid, unwritable file, the
# caller-side fallback must replace it atomically with parseable red evidence.
(
  fallback_work=$test_root/fallback-work
  mkdir -- "$fallback_work"
  work_real=$fallback_work
  work_identity=$(/usr/bin/stat -c '%d:%i' -- "$work_real")
  caller_uid=$(/usr/bin/id -u)
  caller_gid=$(/usr/bin/id -g)
  r2_public_open_work_directory
  fallback_facts=$public_work_fd_path/supervisor-facts.json
  : >"$fallback_facts"
  /usr/bin/chmod 0444 -- "$fallback_facts"
  SOURCE_HEAD=1111111111111111111111111111111111111111
  SOURCE_TREE=2222222222222222222222222222222222222222
  write_fallback_facts /candidate/source "$public_work_fd_path" 37
  /usr/bin/jq -e '
    .setup_failed == true and .inner_pass == false and
    .candidate.source == "/candidate/source" and
    .candidate.commit == "1111111111111111111111111111111111111111" and
    .candidate.tree == "2222222222222222222222222222222222222222" and
    .candidate.source_status == 37
  ' "$fallback_facts" >/dev/null
  fallback_digest=$(/usr/bin/sha256sum "$fallback_facts" | /usr/bin/awk '{print $1}')
  write_fallback_facts /different/source "$public_work_fd_path" 99
  [[ $(/usr/bin/sha256sum "$fallback_facts" | /usr/bin/awk '{print $1}') == "$fallback_digest" ]] ||
    fail 'fallback facts replaced an already valid supervisor record'
  [[ -z $(/usr/bin/find -H -- "$public_work_fd_path" -maxdepth 1 -name '.supervisor-facts.fallback.*' -print -quit) ]] ||
    fail 'fallback facts left a temporary file'
  r2_public_close_work_directory
)

# Adversarial rename plant: a canonical work spelling is replaced after the
# supervisor-side descriptor is opened. A descriptor-derived privileged write
# must stay on the original inode and must not land in the replacement.
rename_parent=$test_root/rename-parent
rename_original=$rename_parent/work
rename_moved=$rename_parent/work-moved
mkdir -- "$rename_parent" "$rename_original"
work_real=$rename_original
work_identity=$(stat -c '%d:%i' -- "$work_real")
caller_uid=$(id -u)
caller_gid=$(id -g)
r2_public_open_work_directory
work=$public_work_fd_path
mv -- "$rename_original" "$rename_moved"
mkdir -- "$rename_original"
printf 'descriptor-original\n' >"$work/descriptor-proof"
[[ -f $rename_moved/descriptor-proof && ! -e $rename_original/descriptor-proof ]]
if r2_public_work_identity_ok; then
  fail 'production work identity accepted a canonical replacement'
fi
r2_public_close_work_directory

# The root handoff reopens the public source descriptor, so a rename cannot
# redirect repository reads to a replacement canonical dentry. The retained
# descriptor still reads the original while its canonical identity check reds.
source_parent=$test_root/source-rename-parent
source_original=$source_parent/source
source_moved=$source_parent/source-moved
mkdir -- "$source_parent" "$source_original"
printf 'source-original\n' >"$source_original/value"
source=$source_original
source_identity=$(/usr/bin/stat -c '%d:%i' -- "$source")
exec {public_source_fd}<"$source"
source_launcher=$BASHPID
source_handoff=/proc/$source_launcher/fd/$public_source_fd
exec {source_wrapper_fd}<"$wrapper"
script_source=/proc/$source_launcher/fd/$source_wrapper_fd
r2_pin_source_directory "$source_handoff"
mv -- "$source_original" "$source_moved"
mkdir -- "$source_original"
printf 'source-replacement\n' >"$source_original/value"
[[ $(<"$source_fd_path/value") == source-original ]] ||
  fail 'supervisor source descriptor was redirected to a canonical replacement'
if r2_source_path_identity_ok; then
  fail 'supervisor source identity accepted a canonical replacement'
fi
exec {source_fd}<&-
exec {public_source_fd}<&-
exec {source_wrapper_fd}<&-
source_fd='' public_source_fd=''
script_source=$wrapper

# Positive-control streams retain their descriptor spelling. A host rename
# must not redirect the comparison back to a replacement canonical dentry.
positive_original=$test_root/positive-original
positive_moved=$test_root/positive-moved
mkdir -- "$positive_original"
printf 'connected\n' >"$positive_original/stdout"
: >"$positive_original/stderr"
cp -- "$positive_original/stdout" "$test_root/positive-staged.stdout"
cp -- "$positive_original/stderr" "$test_root/positive-staged.stderr"
cp -- "$positive_original/stdout" "$test_root/positive-output.stdout"
cp -- "$positive_original/stderr" "$test_root/positive-output.stderr"
exec {positive_fd}<"$positive_original"
mv -- "$positive_original" "$positive_moved"
mkdir -- "$positive_original"
printf 'replacement\n' >"$positive_original/stdout"
: >"$positive_original/stderr"
r2_compare_outer_positive "/proc/self/fd/$positive_fd/stdout" "/proc/self/fd/$positive_fd/stderr" \
  "$test_root/positive-staged.stdout" "$test_root/positive-staged.stderr" \
  "$test_root/positive-output.stdout" "$test_root/positive-output.stderr" ||
  fail 'outer positive control was redirected after a host rename'
exec {positive_fd}<&-

# Receipt-side canonical paths are resolved only after Bubblewrap mounts the
# retained descriptor over that spelling.  Rename the host spelling after the
# child has entered its namespace: it must read the original inode, and the
# wrapper helper must still return red when its post-run identity check sees
# the replacement.
pinned_parent=$test_root/pinned-parent
pinned_original=$pinned_parent/work
pinned_moved=$pinned_parent/work-moved
mkdir -- "$pinned_parent" "$pinned_original"
printf 'pinned-original\n' >"$pinned_original/value"
module_source=$test_root/module-source
module_directory=$module_source/docs/evaluation
mkdir -p -- "$module_directory"
for module in \
  r2-complete-proof-xfs-receipt.mjs \
  r2-complete-proof-xfs-evidence.mjs \
  r2-complete-proof-xfs-ledger.mjs \
  r2-filesystem-accounting.mjs; do
  cp -- "$repository_script_directory/$module" "$module_directory/$module"
done
git -C "$module_source" init --quiet
git -C "$module_source" config user.email test@example.invalid
git -C "$module_source" config user.name 'R2 Module Test'
git -C "$module_source" add docs/evaluation
git -C "$module_source" commit --quiet -m modules
git -C "$module_source" checkout --quiet --detach HEAD
source=$module_source
script_directory=$module_directory
assert_source_head=$(git -C "$source" rev-parse --verify 'HEAD^{commit}')
assert_source_tree=$(git -C "$source" rev-parse --verify 'HEAD^{tree}')
r2_public_open_source_directory
r2_public_open_receipt_modules
exec {public_helper_fd}<"$script_directory"
public_helper_fd_path=/proc/self/fd/$public_helper_fd
public_helper_identity=$(stat -Lc '%d:%i' -- "$public_helper_fd_path")
work_real=$pinned_original
work_identity=$(stat -c '%d:%i' -- "$work_real")
r2_public_open_work_directory
work=$public_work_fd_path
NODE_OPTIONS=--require=/definitely/missing r2_public_pinned_exec \
  "$node_bin" -e 'process.stdout.write("clean-env\n")' >"$test_root/clean-env.stdout"
[[ $(<"$test_root/clean-env.stdout") == clean-env ]] || fail 'receipt sandbox inherited caller Node options'
set +e
r2_public_pinned_exec "$node_bin" "$script_directory/r2-complete-proof-xfs-receipt.mjs" \
  >"$test_root/module-entry.stdout" 2>"$test_root/module-entry.stderr"
module_entry_status=$?
set -e
[[ $module_entry_status -eq 2 ]] || fail 'receipt module snapshot did not load its complete ESM graph'
grep -Fq 'usage: r2-complete-proof-xfs-receipt.mjs' "$test_root/module-entry.stderr"
set +e
r2_public_pinned_exec /usr/bin/bash -ceu '
  printf "ready\n" >"$1/ready"
  /usr/bin/sleep 1
  head -n 1 "$1/value" >"$1/observed"
' _ "$work_real" >"$test_root/pinned-public.stdout" 2>"$test_root/pinned-public.stderr" &
pinned_pid=$!
set -e
for _ in $(seq 1 500); do
  [[ -e $work/ready ]] && break
  kill -0 "$pinned_pid" 2>/dev/null || break
  sleep 0.01
done
[[ -e $work/ready ]] || fail 'pinned public child did not enter its mount namespace'
mv -- "$pinned_original" "$pinned_moved"
mkdir -- "$pinned_original"
printf 'replacement\n' >"$pinned_original/value"
set +e
wait "$pinned_pid"
pinned_status=$?
set -e
[[ $pinned_status -eq 125 ]] || fail 'pinned public helper accepted a renamed canonical work path'
[[ -f $pinned_moved/observed && ! -e $pinned_original/observed &&
   $(<"$pinned_moved/observed") == pinned-original ]] ||
  fail "pinned public helper did not retain the original inode: $(<"$test_root/pinned-public.stderr")"
r2_public_close_work_directory

# Receipt code is copied into immutable per-invocation files and checked
# against the asserted candidate tree before Node starts. An in-place write to
# the host inode after Bubblewrap has entered must neither change the imported
# bytes nor be accepted by the post-run identity check.
mutation_work=$test_root/module-mutation-work
mkdir -- "$mutation_work"
r2_public_open_source_directory
r2_public_open_receipt_modules
exec {public_helper_fd}<"$script_directory"
public_helper_fd_path=/proc/self/fd/$public_helper_fd
public_helper_identity=$(stat -Lc '%d:%i' -- "$public_helper_fd_path")
work_real=$mutation_work
work_identity=$(stat -c '%d:%i' -- "$work_real")
r2_public_open_work_directory
work=$public_work_fd_path
set +e
r2_public_pinned_exec "$node_bin" --input-type=module -e '
  const { writeFileSync } = await import("node:fs");
  writeFileSync(`${process.argv[1]}/ready`, "ready\n");
  await new Promise((resolve) => setTimeout(resolve, 1000));
  await import(`file://${process.argv[2]}`);
  writeFileSync(`${process.argv[1]}/original-imported`, "original\n");
' "$work_real" "$script_directory/r2-complete-proof-xfs-receipt.mjs" \
  >"$test_root/module-mutation.stdout" 2>"$test_root/module-mutation.stderr" &
mutation_pid=$!
set -e
for _ in $(seq 1 500); do
  [[ -e $work/ready ]] && break
  kill -0 "$mutation_pid" 2>/dev/null || break
  sleep 0.01
done
[[ -e $work/ready ]] || fail 'receipt module mutation child did not enter its sandbox'
printf '%s\n' \
  'import { writeFileSync } from "node:fs";' \
  "writeFileSync('$work_real/mutated-imported', 'mutated\\n');" \
  >"$script_directory/r2-complete-proof-xfs-receipt.mjs"
set +e
wait "$mutation_pid"
mutation_status=$?
set -e
[[ $mutation_status -eq 125 ]] || fail 'receipt helper accepted an in-place candidate-module mutation'
[[ -f $work/original-imported && ! -e $work/mutated-imported ]] ||
  fail 'receipt child imported mutable host bytes instead of the candidate snapshot'
r2_public_close_work_directory
script_directory=$repository_script_directory

# Once production has activated the descriptor boundary, privileged I/O must
# not resolve a work-derived path through the caller-renamable spelling.
if awk '/^supervisor\(\)/,/^if \[\[ \$\{1:-\}/' "$wrapper" |
  awk '/r2_pin_work_directory/{seen=1; next} seen' |
  grep -Eq '<"\$work_real/|>"\$work_real/|\(\s*"\$work_real/'; then
  fail 'supervisor retains a privileged work-real path I/O after pinning'
fi

run_expect_fail() {
  local output status
  set +e
  output=$("$@" 2>&1)
  status=$?
  set -e
  [[ $status -ne 0 ]] || { printf 'expected failure: %s\n' "$*" >&2; return 1; }
  printf '%s\n' "$output"
}

mkdir -- "$test_root/work"
run_expect_fail "$wrapper" >"$test_root/usage.log"
grep -Fq 'usage: r2-complete-proof-xfs.sh --source CLEAN --work EMPTY' "$test_root/usage.log"
[[ -z $(find "$test_root/work" -mindepth 1 -print -quit) ]]

# The caller PATH must not redirect any helper that resolves the exact script
# later passed to sudo. The shebang still finds the real Bash because this
# hostile directory deliberately contains only the security-sensitive helpers.
hostile_bin=$test_root/hostile-bin
hostile_marker=$test_root/hostile-helper-called
mkdir -- "$hostile_bin"
for helper in dirname pwd realpath find id readlink; do
  printf '%s\n' '#!/usr/bin/env bash' "printf '%s\\n' '$helper' >>'$hostile_marker'" 'exit 97' \
    >"$hostile_bin/$helper"
  chmod 755 "$hostile_bin/$helper"
done
run_expect_fail /usr/bin/env PATH="$hostile_bin:$PATH" "$wrapper" >"$test_root/hostile-usage.log"
grep -Fq 'usage: r2-complete-proof-xfs.sh --source CLEAN --work EMPTY' "$test_root/hostile-usage.log"
[[ ! -e $hostile_marker ]]

# The root supervisor is read from an already-open file descriptor. Replacing
# its canonical pathname after that open must not change the bytes a child
# executes through the handoff.
pinned_wrapper_copy=$test_root/pinned-wrapper.sh
pinned_replacement_marker=$test_root/pinned-replacement-executed
cp -- "$wrapper" "$pinned_wrapper_copy"
exec {pinned_wrapper_fd}<"$pinned_wrapper_copy"
exec {pinned_helper_fd}<"$workdir_helper"
rm -- "$pinned_wrapper_copy"
printf '%s\n' '#!/usr/bin/env bash' "touch '$pinned_replacement_marker'" >"$pinned_wrapper_copy"
chmod 755 "$pinned_wrapper_copy"
if [[ $(id -u) == 0 ]]; then
  pinned_expected='private supervisor argument shape is invalid'
else
  pinned_expected='private supervisor is root-only'
fi
run_expect_fail /usr/bin/bash "/proc/$BASHPID/fd/$pinned_wrapper_fd" \
  --pinned-supervise "$script_directory" "/proc/$BASHPID/fd/$pinned_helper_fd" \
  >"$test_root/pinned-supervisor.log"
grep -Fq "$pinned_expected" "$test_root/pinned-supervisor.log"
[[ ! -e $pinned_replacement_marker ]] || fail 'canonical replacement executed instead of pinned supervisor bytes'
exec {pinned_wrapper_fd}<&-
exec {pinned_helper_fd}<&-

# Network isolation belongs to the post-drop inner harness. Prove both the
# static ownership boundary and Bubblewrap's actual no-capability entry state.
# This suite runs in the unprivileged outer proof because its adversarial
# receipt tests and this topology regression deliberately create sandboxes of
# their own.
if grep -Eq 'unshare[[:space:]]+--net|ip[[:space:]]+link[[:space:]]+set[[:space:]]+lo' "$wrapper"; then
  printf 'privileged wrapper owns a forbidden network-namespace operation\n' >&2
  exit 1
fi
grep -Fq -- '--unshare-net --unshare-pid' "$outer"
# shellcheck disable=SC2016 # The confined child expands its own procfs fields.
if ! /usr/bin/setpriv --no-new-privs /usr/bin/bwrap \
  --die-with-parent --new-session --unshare-net --unshare-pid \
  --ro-bind / / --dev /dev --proc /proc /usr/bin/bash -ceu '
    for field in CapInh CapPrm CapEff CapBnd CapAmb; do
      [[ $(/usr/bin/awk -v wanted="$field:" "\$1 == wanted {print \$2}" /proc/self/status) == 0000000000000000 ]]
    done
    [[ $(/usr/bin/awk "/^NoNewPrivs:/ {print \$2}" /proc/self/status) == 1 ]]
    /usr/sbin/ip -j link show lo | /usr/bin/jq -e ".[0].ifname == \"lo\" and (.[0].flags | index(\"UP\")) != null" >/dev/null

    exec {confined_findmnt_fd}</proc
    confined_findmnt_descriptor=/proc/self/fd/$confined_findmnt_fd
    confined_findmnt_raw=$(/usr/bin/findmnt --json -T "$confined_findmnt_descriptor" -o TARGET)
    /usr/bin/jq -e "(.filesystems | length) >= 2 and all(.filesystems[]; .target == \"/proc\")" \
      <<<"$confined_findmnt_raw" >/dev/null
    confined_findmnt_unique=$(/usr/bin/findmnt --uniq --json -T "$confined_findmnt_descriptor" -o TARGET)
    [[ $(/usr/bin/jq -er "if (.filesystems | length) == 1 then .filesystems[0].target else empty end" \
      <<<"$confined_findmnt_unique") == /proc ]]
    [[ $(/usr/bin/realpath -e -- "$confined_findmnt_descriptor") == /proc ]]
    exec {confined_findmnt_fd}<&-
  '; then
  fail 'confined capability or duplicate-/proc regression failed'
fi

# A canonical-path private entry is never accepted; only the descriptor-backed
# handoff above may reach the supervisor.
run_expect_fail "$wrapper" --supervise a b c >"$test_root/private.log"
grep -Fq 'private supervisor requires the pinned descriptor handoff' "$test_root/private.log"

run_expect_fail "$wrapper" --source "$test_root/missing-source" --work "$test_root/work" >"$test_root/source.log"
grep -Fq 'source does not exist' "$test_root/source.log"
[[ -z $(find "$test_root/work" -mindepth 1 -print -quit) ]]

source=$test_root/source
mkdir -- "$source"
git -C "$source" init --quiet
git -C "$source" config user.email test@example.invalid
git -C "$source" config user.name 'R2 XFS Test'
printf 'test\n' >"$source/file"
git -C "$source" add file
git -C "$source" commit --quiet -m test
git -C "$source" checkout --quiet --detach HEAD

hostile_work=$test_root/hostile-work
mkdir -- "$hostile_work"
run_expect_fail /usr/bin/env PATH="$hostile_bin:$PATH" RUSTUP_HOME="$test_root/missing-rustup-home" \
  "$wrapper" --source "$source" --work "$hostile_work" >"$test_root/hostile-path.log"
grep -Fq 'RUSTUP_HOME does not exist' "$test_root/hostile-path.log"
[[ ! -e $hostile_marker ]]
[[ -z $(find "$hostile_work" -mindepth 1 -print -quit) ]]

symlink_work=$test_root/symlink-work
mkdir -- "$test_root/real-work"
ln -s -- "$test_root/real-work" "$symlink_work"
run_expect_fail "$wrapper" --source "$source" --work "$symlink_work" >"$test_root/symlink.log"
grep -Fq 'work is not canonical' "$test_root/symlink.log"
[[ -z $(find "$test_root/real-work" -mindepth 1 -print -quit) ]]

# Keep the overlap target under Git's private metadata so the source remains
# clean long enough for the wrapper to reach its canonical non-overlap check.
nested_work=$source/.git/nested-work
mkdir -- "$nested_work"
run_expect_fail "$wrapper" --source "$source" --work "$nested_work" >"$test_root/overlap.log"
grep -Fq 'source and work paths overlap' "$test_root/overlap.log"
[[ -z $(find "$nested_work" -mindepth 1 -print -quit) ]]

printf 'r2-complete-proof-xfs shell validation tests: PASS\n'

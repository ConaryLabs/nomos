#!/usr/bin/env bash
# shellcheck disable=SC2016,SC2034,SC2154 # Static plants and sourced dynamic-scope helpers are intentional.
set -Eeuo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_script_directory=$script_directory
wrapper=$script_directory/r2-complete-proof-xfs.sh
workdir_helper=$script_directory/r2-complete-proof-xfs-workdir.sh
inner=$script_directory/r2-complete-proof.sh
workflow=$script_directory/../../.github/workflows/nomos-viewer.yml
[[ -f $workdir_helper && ! -L $workdir_helper ]]
fail() {
  printf 'r2-complete-proof-xfs shell validation tests: FAIL: %s\n' "$*" >&2
  exit 1
}
# shellcheck source=docs/evaluation/r2-complete-proof-xfs-workdir.sh
source "$workdir_helper"
outer=$script_directory/r2-complete-proof-outer.sh
# shellcheck source=docs/evaluation/r2-complete-proof-outer.sh
source "$outer"
grep -Fq 'r2_pin_work_directory' "$wrapper"
grep -Fq 'work_fd_path=/proc/self/fd/' "$workdir_helper"
grep -Fq 'r2_public_open_work_directory' "$workdir_helper"
grep -Fq 'r2_public_pinned_exec' "$workdir_helper"
grep -Fq 'r2_public_pinned_exec /usr/bin/node' "$wrapper"
grep -Fq '/usr/bin/bash "$pinned_supervisor_path" --pinned-supervise' "$wrapper"
grep -Fq -- '--config core.hooksPath=/dev/null "$source_fd_path" "$checkout"' "$wrapper"
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
if grep -Fq 'chmod 0733' "$wrapper"; then
  printf 'wrapper temporarily reopens caller top-level write authority\n' >&2
  exit 1
fi
test_root=$(mktemp -d "${TMPDIR:-/tmp}/nomos-r2-xfs-shell-test.XXXXXX")
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
record_wrapper_tools "$test_root/wrapper-tools.tsv"
pwd_tool_path=$(/usr/bin/awk -F '\t' '$1 == "pwd" {print $2}' "$test_root/wrapper-tools.tsv")
[[ $pwd_tool_path == /* && -x $pwd_tool_path ]] || fail 'wrapper tool recorder did not resolve external pwd'

# GNU find does not descend through a command-line descriptor symlink under
# its default -P policy. The production inventory must explicitly follow that
# one root link while retaining -P behavior for every entry beneath it.
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
  /usr/bin/node -e 'process.stdout.write("clean-env\n")' >"$test_root/clean-env.stdout"
[[ $(<"$test_root/clean-env.stdout") == clean-env ]] || fail 'receipt sandbox inherited caller Node options'
set +e
r2_public_pinned_exec /usr/bin/node "$script_directory/r2-complete-proof-xfs-receipt.mjs" \
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
r2_public_pinned_exec /usr/bin/node --input-type=module -e '
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
if grep -Eq 'unshare[[:space:]]+--net|ip[[:space:]]+link[[:space:]]+set[[:space:]]+lo' "$wrapper"; then
  printf 'privileged wrapper owns a forbidden network-namespace operation\n' >&2
  exit 1
fi
grep -Fq -- '--unshare-net --unshare-pid' "$outer"
# shellcheck disable=SC2016 # The confined child expands its own procfs fields.
/usr/bin/setpriv --no-new-privs /usr/bin/bwrap \
  --die-with-parent --new-session --unshare-net --unshare-pid \
  --ro-bind / / --dev /dev --proc /proc /usr/bin/bash -ceu '
    for field in CapInh CapPrm CapEff CapBnd CapAmb; do
      [[ $(/usr/bin/awk -v wanted="$field:" "\$1 == wanted {print \$2}" /proc/self/status) == 0000000000000000 ]]
    done
    [[ $(/usr/bin/awk "/^NoNewPrivs:/ {print \$2}" /proc/self/status) == 1 ]]
    /usr/sbin/ip -j link show lo | /usr/bin/jq -e ".[0].ifname == \"lo\" and (.[0].flags | index(\"UP\")) != null" >/dev/null
  '

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

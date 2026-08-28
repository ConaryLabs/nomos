#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'R2 complete proof plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'

# This suite is itself part of the complete isolated proof. Its nested harness
# plants must start as ordinary outer invocations rather than inheriting the
# parent proof's private inner-namespace authority markers.
unset \
  NOMOS_R2_PROOF_INNER \
  NOMOS_R2_HOST_NETNS \
  NOMOS_R2_HOST_PIDNS \
  NOMOS_R2_CALLER_UID \
  NOMOS_R2_CALLER_GID \
  NOMOS_R2_EXPECTED_HEAD \
  NOMOS_R2_EXPECTED_TREE \
  NOMOS_R2_OUTPUT_REAL \
  NOMOS_R2_OUTPUT_RELATIVE \
  NOMOS_R2_PROOF_TOKEN \
  NOMOS_R2_EXTERNAL_POSITIVE \
  NOMOS_R2_OUTER_PREFLIGHT_LOG \
  NOMOS_R2_OUTER_POSITIVE_STDOUT \
  NOMOS_R2_OUTER_POSITIVE_STDERR \
  NOMOS_R2_XFS_WRAPPER \
  NOMOS_R2_XFS_UUID \
  NOMOS_R2_XFS_FRAGMENT_SIZE \
  NOMOS_R2_XFS_DEVICE \
  NOMOS_R2_XFS_MAJOR_MINOR

# The inner preflight now requires the wrapper's filesystem identity. These
# deliberately synthetic values let refusal plants reach the behavior they
# intend to exercise without pretending to be a real wrapper run.
export NOMOS_R2_XFS_WRAPPER=1
export NOMOS_R2_XFS_UUID=11111111-1111-1111-1111-111111111111
export NOMOS_R2_XFS_FRAGMENT_SIZE=4096
export NOMOS_R2_XFS_DEVICE=/dev/loop0
export NOMOS_R2_XFS_MAJOR_MINOR=7:0

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
harness_source=$script_directory/r2-complete-proof.sh
harness_lib_source=$script_directory/r2-complete-proof-lib.sh
harness_control_source=$script_directory/r2-complete-proof-control.sh
harness_outer_source=$script_directory/r2-complete-proof-outer.sh
control_test_source=$script_directory/r2-complete-proof-control.test.sh
[[ -f $harness_source && ! -L $harness_source ]] || fail 'complete-proof harness is absent'
[[ -f $harness_lib_source && ! -L $harness_lib_source ]] || fail 'complete-proof library is absent'
[[ -f $harness_control_source && ! -L $harness_control_source ]] ||
  fail 'complete-proof control library is absent'
[[ -f $harness_outer_source && ! -L $harness_outer_source ]] ||
  fail 'complete-proof outer library is absent'
[[ -f $control_test_source && ! -L $control_test_source ]] ||
  fail 'complete-proof control plants are absent'

for command in git cp ln chmod mkdir mktemp mv grep jq ps realpath readlink wc sed find id sleep setsid taskset; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-complete-proof-plants.XXXXXX")
seed=$temporary/seed
linked=
leaked_child_pid=
allowed_session_pid=
same_group_root=
same_group_job_pid=
group_leak_pid=
cleanup() {
  case $temporary in
    "$repo_root"/target/r2-complete-proof-plants.*)
      if [[ -n ${leaked_child_pid:-} ]] && kill -0 "$leaked_child_pid" 2>/dev/null; then
        kill "$leaked_child_pid" 2>/dev/null || true
        wait "$leaked_child_pid" 2>/dev/null || true
      fi
      if [[ -n ${allowed_session_pid:-} ]] && kill -0 "$allowed_session_pid" 2>/dev/null; then
        kill -- "-$allowed_session_pid" 2>/dev/null ||
          kill "$allowed_session_pid" 2>/dev/null || true
        wait "$allowed_session_pid" 2>/dev/null || true
      fi
      if [[ -n ${same_group_root:-} ]]; then
        kill -- "-$same_group_root" 2>/dev/null || true
        [[ -z ${same_group_job_pid:-} ]] || wait "$same_group_job_pid" 2>/dev/null || true
      fi
      if [[ -n ${group_leak_pid:-} ]] && kill -0 "$group_leak_pid" 2>/dev/null; then
        kill "$group_leak_pid" 2>/dev/null || true
        wait "$group_leak_pid" 2>/dev/null || true
      fi
      # Retain the closed plant fixture beneath checkout-local target so a
      # failed refusal can be inspected without touching the candidate tree.
      ;;
    *)
      printf 'R2 complete proof plants: refusing unsafe cleanup path: %s\n' "$temporary" >&2
      ;;
  esac
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Give every plant a minimal ordinary repository rather than teaching the
# proof a test root. Only the exact harness, its libraries, browser discovery,
# toolchain pin, and ignore rule are needed before these preflight refusals;
# copying the full candidate would make the filesystem sampler measure the
# plant fixture rather than the candidate workload.
mkdir -p "$seed/docs/evaluation" "$seed/apps/nomos-viewer/smoke"
cp "$repo_root/.gitignore" "$repo_root/rust-toolchain.toml" "$seed/"
cp "$harness_source" "$seed/docs/evaluation/r2-complete-proof.sh"
cp "$harness_lib_source" "$seed/docs/evaluation/r2-complete-proof-lib.sh"
cp "$harness_control_source" "$seed/docs/evaluation/r2-complete-proof-control.sh"
cp "$harness_outer_source" "$seed/docs/evaluation/r2-complete-proof-outer.sh"
cp "$control_test_source" "$seed/docs/evaluation/r2-complete-proof-control.test.sh"
cp "$repo_root/apps/nomos-viewer/smoke/chrome.mjs" \
  "$seed/apps/nomos-viewer/smoke/chrome.mjs"
chmod 755 "$seed/docs/evaluation/r2-complete-proof.sh"
git -C "$seed" init -q --object-format=sha1 -b plant-main
git -C "$seed" add --all
git -C "$seed" \
  -c user.name='Nomos proof plant' \
  -c user.email='proof-plant.invalid@example.invalid' \
  commit -q -m 'temporary complete-proof plant seed'

clone_detached() {
  local destination=$1
  git clone -q --local --no-hardlinks "$seed" "$destination"
  git -C "$destination" checkout -q --detach HEAD
}

in_repo() {
  local root=$1
  shift
  (cd -- "$root" && "$@")
}

guard_directory=$temporary/guard-bin
sudo_log=$temporary/sudo.log
mkdir "$guard_directory"
# The single-quoted rows are the literal body of the planted executable.
# shellcheck disable=SC2016
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'printf '\''%s\n'\'' "$*" >>"${R2_TEST_SUDO_LOG:?}"' \
  'exit "${R2_TEST_SUDO_STATUS:-97}"' \
  >"$guard_directory/sudo"
chmod 755 "$guard_directory/sudo"
guard_path=$guard_directory:$PATH

plant_count=0
expect_failure() {
  local label=$1
  local expected=$2
  local sudo_calls=$3
  shift 3
  local log=$temporary/$label.log status actual_sudo_calls
  : >"$sudo_log"
  set +e
  "$@" >"$log" 2>&1
  status=$?
  set -e
  [[ $status -ne 0 ]] || fail "$label plant passed"
  grep -F -- "$expected" "$log" >/dev/null || {
    sed -n '1,80p' "$log" >&2
    fail "$label plant failed for the wrong reason"
  }
  actual_sudo_calls=$(wc -l <"$sudo_log")
  [[ $actual_sudo_calls -eq $sudo_calls ]] ||
    fail "$label reached sudo $actual_sudo_calls times, expected $sudo_calls"
  plant_count=$((plant_count + 1))
}

detached=$temporary/detached
clone_detached "$detached"
harness=$detached/docs/evaluation/r2-complete-proof.sh
shared_output=$detached/target/shared-output
mkdir -p "$shared_output"

# Exact public argument grammar.
expect_failure no-arguments 'usage: r2-complete-proof.sh --output <empty-directory>' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness"
expect_failure missing-value 'usage: r2-complete-proof.sh --output <empty-directory>' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output
expect_failure wrong-flag 'usage: r2-complete-proof.sh --output <empty-directory>' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --out "$shared_output"
expect_failure extra-argument 'usage: r2-complete-proof.sh --output <empty-directory>' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output "$shared_output" extra

# Checkout topology and identity must fail before privileged isolation checks.
attached=$temporary/attached
git clone -q --local --no-hardlinks "$seed" "$attached"
mkdir -p "$attached/target/proof"
expect_failure attached-head 'checkout must be detached at the candidate commit' 0 \
  in_repo "$attached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

linked=$temporary/linked
git -C "$seed" worktree add -q --detach "$linked" HEAD
mkdir -p "$linked/target/proof"
expect_failure linked-worktree 'checkout must have a real local .git directory' 0 \
  in_repo "$linked" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

shallow=$temporary/shallow
git clone -q --depth 1 "file://$seed" "$shallow"
git -C "$shallow" checkout -q --detach HEAD
mkdir -p "$shallow/target/proof"
[[ $(git -C "$shallow" rev-parse --is-shallow-repository) == true ]] || fail 'shallow fixture is not shallow'
expect_failure shallow-checkout 'checkout must be a full, non-shallow clone' 0 \
  in_repo "$shallow" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

alternates=$temporary/alternates
git clone -q --shared "$seed" "$alternates"
git -C "$alternates" checkout -q --detach HEAD
mkdir -p "$alternates/target/proof"
[[ -s $alternates/.git/objects/info/alternates ]] || fail 'alternates fixture has no alternates file'
expect_failure alternate-objects 'Git object alternates are forbidden' 0 \
  in_repo "$alternates" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

dirty=$temporary/dirty
clone_detached "$dirty"
mkdir -p "$dirty/target/proof"
printf '\nplant\n' >>"$dirty/README.md"
expect_failure dirty-checkout 'checkout is not clean' 0 \
  in_repo "$dirty" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

# Output confinement, identity, emptiness, and ignore status.
outside=$temporary/outside-output
mkdir "$outside"
expect_failure outside-output 'output must be physically inside the checkout' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output "$outside"
expect_failure checkout-root 'output cannot be a filesystem or checkout root' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output .
mkdir -p "$detached/target"
expect_failure target-root 'output cannot be the checkout root or target/ root' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output target

target_symlink=$temporary/target-symlink
clone_detached "$target_symlink"
mkdir "$target_symlink/proof-output" "$temporary/target-symlink-sink"
printf '/proof-output\n' >>"$target_symlink/.git/info/exclude"
ln -s "$temporary/target-symlink-sink" "$target_symlink/target"
expect_failure target-symlink 'checkout target must be absent or one real directory' 0 \
  in_repo "$target_symlink" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output proof-output

mkdir -p "$detached/target/real-final"
ln -s real-final "$detached/target/link-final"
expect_failure symlink-output 'output must already exist as a real directory' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output target/link-final

mkdir -p "$detached/target/real-parent/child"
ln -s real-parent "$detached/target/link-parent"
expect_failure symlink-ancestor 'output path traverses a symlink' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output target/link-parent/child

mkdir -p "$detached/target/nonempty"
printf 'plant\n' >"$detached/target/nonempty/file"
expect_failure nonempty-output 'output directory must be empty' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output target/nonempty

mkdir "$detached/unignored-output"
expect_failure unignored-output 'output directory must be Git-ignored' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output unignored-output

preexisting=$temporary/preexisting
clone_detached "$preexisting"
mkdir -p "$preexisting/target/proof" "$preexisting/target/debug"
expect_failure preexisting-target 'pre-existing proof target is forbidden: target/debug' 0 \
  in_repo "$preexisting" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

dangling_target=$temporary/dangling-target
clone_detached "$dangling_target"
mkdir -p "$dangling_target/target/proof"
ln -s nowhere "$dangling_target/target/debug"
expect_failure dangling-proof-target 'pre-existing proof target is forbidden: target/debug' 0 \
  in_repo "$dangling_target" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  /usr/bin/bash docs/evaluation/r2-complete-proof.sh --output target/proof

# A missing required tool is rejected before repository or isolation work.
missing_tools=$temporary/missing-tools
mkdir "$missing_tools"
for command in git realpath readlink find grep awk sed sort cmp cut sha256sum \
  stat date dirname du ionice; do
  ln -s "$(command -v "$command")" "$missing_tools/$command"
done
expect_failure missing-tool 'required executable not found: jq' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$missing_tools" \
  /usr/bin/bash "$harness" --output "$shared_output"

# Browser failures are rejected during unprivileged host validation, before a
# sudo capability probe, nested bubblewrap attempt, or proof entry.
missing_browser_output=$detached/target/missing-browser-output
mkdir "$missing_browser_output"
expect_failure missing-browser 'Chrome/Chromium is not installed or cannot start' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" R2_TEST_SUDO_STATUS=0 \
  CHROME_BIN="$temporary/no-browser" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output "$missing_browser_output"

fake_browser=$temporary/not-a-browser
printf '#!/usr/bin/env bash\nexit 0\n' >"$fake_browser"
chmod 755 "$fake_browser"
wrong_browser_output=$detached/target/wrong-browser-output
mkdir "$wrong_browser_output"
expect_failure wrong-browser 'Chrome/Chromium is not installed or cannot start' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" R2_TEST_SUDO_STATUS=0 \
  CHROME_BIN="$fake_browser" PATH="$guard_path" \
  /usr/bin/bash "$harness" --output "$wrong_browser_output"

# The private marker is not authority. Supply every binding exactly while
# leaving the process in the caller namespace; it must still fail closed.
forged_output=$detached/target/forged-inner
mkdir "$forged_output"
forged_head=$(git -C "$detached" rev-parse 'HEAD^{commit}')
forged_tree=$(git -C "$detached" rev-parse 'HEAD^{tree}')
forged_netns=$(readlink /proc/self/ns/net)
printf -v forged_token '%064x' 0
expect_failure forged-inner-marker 'forged isolation marker or unchanged network namespace' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  NOMOS_R2_PROOF_INNER=1 NOMOS_R2_HOST_NETNS="$forged_netns" \
  NOMOS_R2_CALLER_UID="$(id -u)" NOMOS_R2_CALLER_GID="$(id -g)" \
  NOMOS_R2_EXPECTED_HEAD="$forged_head" NOMOS_R2_EXPECTED_TREE="$forged_tree" \
  NOMOS_R2_PROOF_TOKEN="$forged_token" \
  NOMOS_R2_OUTPUT_REAL="$(realpath -e "$forged_output")" \
  NOMOS_R2_OUTPUT_RELATIVE=target/forged-inner \
  /usr/bin/bash "$harness" --output "$forged_output"

forged_pid_output=$detached/target/forged-pid-inner
mkdir "$forged_pid_output"
forged_pidns=$(readlink /proc/self/ns/pid)
expect_failure forged-pid-marker 'forged isolation marker or unchanged PID namespace' 0 \
  in_repo "$detached" env R2_TEST_SUDO_LOG="$sudo_log" PATH="$guard_path" \
  NOMOS_R2_PROOF_INNER=1 NOMOS_R2_HOST_NETNS='net:[0]' \
  NOMOS_R2_HOST_PIDNS="$forged_pidns" \
  NOMOS_R2_CALLER_UID="$(id -u)" NOMOS_R2_CALLER_GID="$(id -g)" \
  NOMOS_R2_EXPECTED_HEAD="$forged_head" NOMOS_R2_EXPECTED_TREE="$forged_tree" \
  NOMOS_R2_PROOF_TOKEN="$forged_token" \
  NOMOS_R2_OUTPUT_REAL="$(realpath -e "$forged_pid_output")" \
  NOMOS_R2_OUTPUT_RELATIVE=target/forged-pid-inner \
  /usr/bin/bash "$harness" --output "$forged_pid_output"

# Exercise the closure primitive with a real child that clears the proof token,
# then prove the same namespace is clean after that child is reaped. The full
# proof uses the stronger fresh-namespace branch, which treats every
# non-ancestor process as proof-owned.
# shellcheck source=docs/evaluation/r2-complete-proof.sh
source "$harness_source"
# shellcheck source=docs/evaluation/r2-complete-proof-outer.sh
source "$harness_outer_source"
fail() {
  printf 'R2 complete proof plants: FAIL: %s\n' "$*" >&2
  exit 1
}

# Outer preflight is machine-readable and must reject any capability, namespace,
# or no-new-privileges mutation before a host log can be accepted.
outer_preflight_fixture=$temporary/outer-preflight.json
printf '%s\n' \
  '{"cap_ambient":"0000000000000000","cap_bounding":"0000000000000000","cap_effective":"0000000000000000","cap_inheritable":"0000000000000000","cap_permitted":"0000000000000000","host_network_namespace":"net:[10]","host_pid_namespace":"pid:[20]","network_namespace":"net:[30]","no_new_privs":1,"pid_namespace":"pid:[40]"}' \
  >"$outer_preflight_fixture"
r2_validate_outer_preflight_log "$outer_preflight_fixture" 'net:[10]' 'pid:[20]' ||
  fail 'valid outer preflight evidence was rejected'
jq -c '.cap_effective = "0000000000000001"' "$outer_preflight_fixture" \
  >"$outer_preflight_fixture.mutated"
if r2_validate_outer_preflight_log "$outer_preflight_fixture.mutated" 'net:[10]' 'pid:[20]'; then
  fail 'outer preflight capability mutation was accepted'
fi
plant_count=$((plant_count + 1))

# Retained positive-control streams must be byte-identical regular files; a
# changed staging stream and a symlinked output stream are both terminal.
outer_positive_fixture=$temporary/outer-positive
mkdir "$outer_positive_fixture"
outer_host_stdout=$outer_positive_fixture/host.stdout
outer_host_stderr=$outer_positive_fixture/host.stderr
outer_stage_stdout=$outer_positive_fixture/stage.stdout
outer_stage_stderr=$outer_positive_fixture/stage.stderr
outer_output_stdout=$outer_positive_fixture/output.stdout
outer_output_stderr=$outer_positive_fixture/output.stderr
printf 'connected\n' >"$outer_host_stdout"
: >"$outer_host_stderr"
cp "$outer_host_stdout" "$outer_stage_stdout"
cp "$outer_host_stderr" "$outer_stage_stderr"
cp "$outer_host_stdout" "$outer_output_stdout"
cp "$outer_host_stderr" "$outer_output_stderr"
r2_compare_outer_positive \
  "$outer_host_stdout" "$outer_host_stderr" \
  "$outer_stage_stdout" "$outer_stage_stderr" \
  "$outer_output_stdout" "$outer_output_stderr" ||
  fail 'matching outer positive-control evidence was rejected'
printf 'mutated\n' >"$outer_stage_stdout"
if r2_compare_outer_positive \
  "$outer_host_stdout" "$outer_host_stderr" \
  "$outer_stage_stdout" "$outer_stage_stderr" \
  "$outer_output_stdout" "$outer_output_stderr"; then
  fail 'positive-control staging mutation was accepted'
fi
cp "$outer_host_stdout" "$outer_stage_stdout"
mv "$outer_output_stdout" "$outer_positive_fixture/output.real"
ln -s output.real "$outer_output_stdout"
if r2_compare_outer_positive \
  "$outer_host_stdout" "$outer_host_stderr" \
  "$outer_stage_stdout" "$outer_stage_stderr" \
  "$outer_output_stdout" "$outer_output_stderr"; then
  fail 'symlinked positive-control output was accepted'
fi
plant_count=$((plant_count + 1))

# The sidecar binds each actual executor argv to its ordinal and command id;
# changing one argument must fail the same row-level validation.
argv_ledger=$temporary/command-argv.ndjson
r2_init_command_argv_ledger "$argv_ledger" || fail 'argv ledger was not initialized'
r2_record_command_argv "$argv_ledger" 1 argv-mutation /bin/printf hello ||
  fail 'argv ledger row was not recorded'
argv_record=$(<"$argv_ledger")
r2_validate_command_argv_record 1 argv-mutation "$argv_record" /bin/printf hello ||
  fail 'argv ledger row did not validate'
argv_mutation=$(jq -c '.argv[1] = "mutated"' "$argv_ledger")
if r2_validate_command_argv_record 1 argv-mutation "$argv_mutation" /bin/printf hello; then
  fail 'argv mutation was accepted'
fi
plant_count=$((plant_count + 1))

# Procfs can expose a live task's stat file while it is between updates. Keep
# the bounded generic reader plants here after retiring the observer-specific
# source file: transient emptiness retries exactly twice, while persistent
# emptiness and parsed malformation remain terminal at their fixed bounds.
process_stat_once_source=$(declare -f r2_read_process_stat_once)
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  if [[ $process_stat_attempt -lt 3 ]]; then
    R2_PROC_READ_CLASS=incomplete
    return 1
  fi
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=2
  R2_PROC_SESSION=2
  R2_PROC_START=3
  R2_PROC_READ_CLASS=ok
}
r2_read_process_stat /proc/self/stat ||
  fail 'transient empty procfs stat snapshots were not retried'
[[ $process_stat_attempt -eq 3 && $R2_PROC_READ_CLASS == ok &&
  $R2_PROC_STATE == S && $R2_PROC_PARENT -eq 1 &&
  $R2_PROC_GROUP -eq 2 && $R2_PROC_SESSION -eq 2 &&
  $R2_PROC_START -eq 3 ]] ||
  fail 'procfs stat retry count or result differs'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  R2_PROC_READ_CLASS=incomplete
  return 1
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 1 && $process_stat_attempt -eq 3 &&
  $R2_PROC_READ_CLASS == incomplete ]] ||
  fail 'persistent empty procfs stat snapshots did not preserve absence'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  R2_PROC_READ_CLASS=malformed
  return 2
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 2 && $process_stat_attempt -eq 1 &&
  $R2_PROC_READ_CLASS == malformed ]] ||
  fail 'malformed procfs stat snapshot was retried'
eval "$process_stat_once_source"
unset process_stat_once_source process_stat_attempt

printf -v closure_token '%064x' "$$"
closure_namespace=$(readlink /proc/self/ns/net)
closure_report=$temporary/closure-report.txt
env -i PATH="$PATH" sleep 30 &
leaked_child_pid=$!
closure_child_ready=0
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if [[ -r /proc/$leaked_child_pid/environ ]] &&
    ! grep -Fzq -- 'NOMOS_R2_PROOF_TOKEN=' "/proc/$leaked_child_pid/environ"; then
    closure_child_ready=1
    break
  fi
  sleep 0.01
done
[[ $closure_child_ready -eq 1 ]] || fail 'planted child did not clear the proof token'
expect_failure leaked-child 'R2 process closure: live namespace children:' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" "$closure_report"
grep -Fx -- "$leaked_child_pid" "$closure_report" >/dev/null ||
  fail 'closure report did not identify the planted child'
kill "$leaked_child_pid"
wait "$leaked_child_pid" 2>/dev/null || true
leaked_child_pid=
r2_measure_process_closure "$closure_namespace" "$closure_token" "$closure_report" ||
  fail 'closure primitive did not pass after the planted child closed'
[[ ! -s $closure_report ]] || fail 'clean closure report is not empty'

# The one pre-stop allowance is a dedicated session, not merely a process
# group. Session membership can only be inherited from this root; a process
# outside it cannot join later. Keep a real descendant alive so the positive
# check exercises both root and child membership.
session_child_file=$temporary/allowed-session-child.pid
# The single-quoted program is expanded by the planted child shell.
# shellcheck disable=SC2016
setsid env -i PATH="$PATH" bash -c '
  sleep 30 &
  child=$!
  printf '\''%s\n'\'' "$child" >"$1"
  wait "$child"
' r2-allowed-session "$session_child_file" &
allowed_session_pid=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -s $session_child_file ]] || break
  kill -0 "$allowed_session_pid" 2>/dev/null ||
    fail 'dedicated allowed session exited before its child was ready'
  sleep 0.01
done
[[ -s $session_child_file ]] || fail 'dedicated allowed session child did not become ready'
allowed_session_child=$(<"$session_child_file")
r2_read_process_stat "/proc/$allowed_session_pid/stat" ||
  fail 'dedicated allowed session root is unreadable'
[[ $R2_PROC_GROUP == "$allowed_session_pid" &&
  $R2_PROC_SESSION == "$allowed_session_pid" && $R2_PROC_STATE != Z ]] ||
  fail 'dedicated allowed session root does not own its session and group'
allowed_session_start=$R2_PROC_START
r2_read_process_stat "/proc/$allowed_session_child/stat" ||
  fail 'dedicated allowed session child is unreadable'
[[ $R2_PROC_PARENT == "$allowed_session_pid" &&
  $R2_PROC_SESSION == "$allowed_session_pid" ]] ||
  fail 'dedicated allowed session child does not inherit the sampler session'

expect_failure session-root-mismatch 'allowed sampler session root is not stable' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$allowed_session_pid" "$((allowed_session_pid + 1))" \
  "$allowed_session_start"
expect_failure session-start-mismatch 'allowed sampler session root is not stable' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$allowed_session_pid" "$allowed_session_pid" \
  "$((allowed_session_start + 1))"
env -i PATH="$PATH" sleep 30 &
group_leak_pid=$!
expect_failure session-does-not-hide-leak 'R2 process closure: live namespace children:' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$allowed_session_pid" "$allowed_session_pid" \
  "$allowed_session_start"
grep -Fx -- "$group_leak_pid" "$closure_report" >/dev/null ||
  fail 'dedicated sampler session hid an unrelated live child'
kill "$group_leak_pid"
wait "$group_leak_pid" 2>/dev/null || true
group_leak_pid=
r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$allowed_session_pid" "$allowed_session_pid" \
  "$allowed_session_start" ||
  fail 'closure primitive rejected a dedicated sampler session'
[[ ! -s $closure_report ]] || fail 'allowed sampler session appeared as a leak'
kill -- "-$allowed_session_pid"
wait "$allowed_session_pid" 2>/dev/null || true
closed_session_pid=$allowed_session_pid
allowed_session_pid=
expect_failure closed-session-root 'allowed sampler session root is not stable' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$closed_session_pid" "$closed_session_pid" \
  "$allowed_session_start"

# A same-session sibling may join an existing PGID, which is the exact attack
# that made PGID-only allowance unsound. A background pipeline gives two
# non-descendant siblings one group. Passing its leader as a purported sampler
# root must now be refused because that leader does not own a dedicated session.
set -m
env -i PATH="$PATH" sleep 30 | env -i PATH="$PATH" sleep 30 &
same_group_job_pid=$!
set +m
for ((attempt = 0; attempt < 100; attempt += 1)); do
  r2_read_process_stat "/proc/$same_group_job_pid/stat" && break
  sleep 0.01
done
r2_read_process_stat "/proc/$same_group_job_pid/stat" ||
  fail 'same-PGID sibling is unreadable'
same_group_root=$R2_PROC_GROUP
same_group_session=$R2_PROC_SESSION
same_group_sibling_parent=$R2_PROC_PARENT
[[ $same_group_root != "$same_group_job_pid" ]] ||
  fail 'same-PGID plant did not create a distinct sibling'
r2_read_process_stat "/proc/$same_group_root/stat" ||
  fail 'same-PGID group root is unreadable'
same_group_start=$R2_PROC_START
[[ $R2_PROC_GROUP == "$same_group_root" &&
  $R2_PROC_SESSION == "$same_group_session" &&
  $R2_PROC_PARENT == "$$" && $same_group_sibling_parent == "$$" &&
  $R2_PROC_SESSION != "$same_group_root" ]] ||
  fail 'same-PGID plant is not two non-session sibling processes'
expect_failure same-pgid-non-session-root 'allowed sampler session root is not stable' 0 \
  r2_measure_process_closure "$closure_namespace" "$closure_token" \
  "$closure_report" "$same_group_root" "$same_group_root" "$same_group_start"
kill -- "-$same_group_root"
wait "$same_group_job_pid" 2>/dev/null || true
same_group_root=
same_group_job_pid=

# Version evidence must invoke the exact canonical path recorded and digested
# in tools.txt. This fake reports its argv[0] basename, so invoking its `cc`
# symlink would produce different evidence from invoking the recorded file.
fake_version_real=$temporary/version-real
fake_version_link=$temporary/cc
fake_version_tools=$temporary/version-tools.tsv
# The parameter expansion belongs to the planted executable, not this shell.
# shellcheck disable=SC2016
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'printf '\''%s\n'\'' "${0##*/}"' \
  >"$fake_version_real"
chmod 755 "$fake_version_real"
ln -s "$fake_version_real" "$fake_version_link"
printf 'tool\tpath\tsha256\ncc\t%s\t%s\n' \
  "$fake_version_real" "$(sha256sum "$fake_version_real" | cut -d' ' -f1)" \
  >"$fake_version_tools"
fake_version=$(PATH="$temporary:$PATH" \
  r2_emit_recorded_tool_version "$fake_version_tools" cc cc --version) ||
  fail 'recorded tool version helper rejected a canonical executable'
[[ $fake_version == 'cc=version-real' ]] ||
  fail 'tool version evidence invoked a command symlink instead of its recorded path'

# `run_step` must not let a later successful command mask an earlier failure
# inside a compound proof function. Exercise its exact sourceable executor
# outside a conditional context, because Bash deliberately suppresses errexit
# for a function evaluated directly by `if` or `||`.
masked_stdout=$temporary/masked-step.stdout
masked_stderr=$temporary/masked-step.stderr
fail_then_succeed() {
  false
  printf 'failure was masked\n'
}
set +e
r2_execute_step "$masked_stdout" "$masked_stderr" fail_then_succeed
masked_status=$?
set -e
[[ $masked_status -ne 0 ]] || fail 'step executor masked an early failure'
[[ ! -s $masked_stdout ]] || fail 'step executor continued after an early failure'
plant_count=$((plant_count + 1))

[[ -z $(find "$temporary" -name commands.tsv -print -quit) ]] ||
  fail 'a plant reached the heavy command ledger'

bash "$script_directory/r2-complete-proof-control.test.sh"
printf 'R2_COMPLETE_PROOF_PLANTS PASS\n'
printf 'planted_failures %s\n' "$plant_count"
printf 'plant_scratch retained_in_checkout_target\n'
printf 'clean_outer_preflight not_run_no_documented_test_hook\n'

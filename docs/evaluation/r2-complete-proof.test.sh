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
  NOMOS_R2_EXTERNAL_POSITIVE

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
harness_source=$script_directory/r2-complete-proof.sh
harness_lib_source=$script_directory/r2-complete-proof-lib.sh
[[ -f $harness_source && ! -L $harness_source ]] || fail 'complete-proof harness is absent'
[[ -f $harness_lib_source && ! -L $harness_lib_source ]] || fail 'complete-proof library is absent'

for command in git cp ln chmod mkdir mktemp grep jq ps realpath readlink wc sed find id nice sleep setsid taskset; do
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
stop_test_pid=
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
      if [[ -n ${stop_test_pid:-} ]] && kill -0 "$stop_test_pid" 2>/dev/null; then
        kill -KILL -- "-$stop_test_pid" 2>/dev/null || true
        wait "$stop_test_pid" 2>/dev/null || true
      fi
      if [[ -n ${linked:-} && -d $seed/.git ]]; then
        git -C "$seed" worktree remove --force "$linked" >/dev/null 2>&1 || true
      fi
      [[ ! -e $temporary ]] || find "$temporary" -depth -delete
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
# proof a test root. Only the exact harness, its library, browser discovery,
# toolchain pin, and ignore rule are needed before these preflight refusals;
# copying the full candidate would make the disk-budget sampler measure the
# plant fixture rather than the candidate workload.
mkdir -p "$seed/docs/evaluation" "$seed/apps/nomos-viewer/smoke"
cp "$repo_root/.gitignore" "$repo_root/rust-toolchain.toml" "$seed/"
cp "$harness_source" "$seed/docs/evaluation/r2-complete-proof.sh"
cp "$harness_lib_source" "$seed/docs/evaluation/r2-complete-proof-lib.sh"
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
fail() {
  printf 'R2 complete proof plants: FAIL: %s\n' "$*" >&2
  exit 1
}
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

# A `du` walk can race Cargo's atomic deletion of an intermediate file. The
# sampler must discard that incomplete walk, retain a subsequent complete
# result, and fail closed if no complete result can be obtained.
fake_disk_bin=$temporary/fake-disk-bin
fake_disk_state=$temporary/fake-disk-state
mkdir "$fake_disk_bin"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  '[[ $1 == -c && $2 == 3 ]] || exit 91' \
  'shift 2' \
  'exec "$@"' \
  >"$fake_disk_bin/ionice"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  '[[ $# -eq 3 && $1 == -sm && $2 == -- && -d $3 ]] || exit 93' \
  'process_nice=$(ps -o ni= -p $$) || exit 92' \
  'process_nice=${process_nice// /}' \
  '[[ $process_nice == 19 ]] || exit 92' \
  'if [[ -n ${R2_TEST_DU_AFFINITY_FILE:-} ]]; then' \
  '  while IFS= read -r line; do' \
  '    [[ $line != Cpus_allowed_list:* ]] || printf '\''%s\n'\'' "${line#*:}" >"$R2_TEST_DU_AFFINITY_FILE"' \
  '  done </proc/self/status' \
  'fi' \
  'if [[ ${R2_TEST_DU_STABLE:-0} == 1 ]]; then' \
  '  sleep "${R2_TEST_DU_DELAY:-0}"' \
  '  [[ ${R2_TEST_DU_FAIL:-0} != 1 ]] || exit 94' \
  '  printf '\''17\t%s\n'\'' "${@: -1}"' \
  '  exit 0' \
  'fi' \
  'count=0' \
  '[[ ! -f ${R2_TEST_DU_STATE:?} ]] || read -r count <"$R2_TEST_DU_STATE"' \
  'count=$((count + 1))' \
  'printf '\''%s\n'\'' "$count" >"$R2_TEST_DU_STATE"' \
  'if [[ ${R2_TEST_DU_ALWAYS_FAIL:-0} == 1 || $count -eq 1 ]]; then' \
  '  printf '\''du: cannot access transient: No such file or directory\n'\'' >&2' \
  '  exit 1' \
  'fi' \
  'printf '\''17\t%s\n'\'' "${@: -1}"' \
  >"$fake_disk_bin/du"
chmod 755 "$fake_disk_bin/du" "$fake_disk_bin/ionice"

# A retry is a new sampling attempt. Its own start timestamp, not the failed
# attempt's timestamp or the controller's launch timestamp, must be the one
# retained in the raw row. Use a deterministic clock to make that distinction
# exact in both the measurement helper and the record worker.
disk_affinity_line=$(taskset -pc $$)
disk_test_affinity=${disk_affinity_line##*: }
[[ $disk_test_affinity =~ ^[0-9,-]+$ ]] ||
  fail 'could not derive the test process CPU affinity'
r2_partition_cpu_list 0-11 || fail 'canonical CPU partition fixture was refused'
[[ $R2_CONTROLLER_CPUS == 0 && $R2_DISK_CPUS == 1,2,3,4,5 &&
  $R2_WORKLOAD_CPUS == 6,7,8,9,10,11 ]] || fail 'twelve-CPU partition differs'
if r2_partition_cpu_list 0-1 || r2_partition_cpu_list 02,3,4 ||
  r2_partition_cpu_list 3,2,4; then
  fail 'malformed or undersized CPU partition was accepted'
fi
plant_count=$((plant_count + 1))
r2_partition_cpu_list "$disk_test_affinity" ||
  fail 'disk sampler plants require at least three available CPUs'
disk_test_controller_cpus=$R2_CONTROLLER_CPUS
export R2_DISK_WALK_CPUS=$R2_DISK_CPUS

# Stop requests carry their own timestamp and wait only for an identity-bound
# session. Exercise both normal closure and a stopped root whose unwritable
# marker forces bounded TERM/KILL cleanup.
normal_stop=$temporary/normal-sampler.stop
stop_test_start=
setsid taskset -c "$disk_test_controller_cpus" bash -c \
  'while [[ ! -e $1 ]]; do sleep 0.01; done' r2-stop-plant "$normal_stop" &
stop_test_pid=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$stop_test_pid/stat" &&
    [[ $R2_PROC_GROUP == "$stop_test_pid" && $R2_PROC_SESSION == "$stop_test_pid" ]]; then
    stop_test_start=$R2_PROC_START
    break
  fi
  sleep 0.01
done
[[ ${stop_test_start:-} =~ ^[0-9]+$ ]] || fail 'normal stop sampler identity was not stable'
unset R2_DISK_STOP_REQUESTED_NS
r2_stop_disk_sampler "$stop_test_pid" "$stop_test_start" \
  "$disk_test_controller_cpus" "$normal_stop" 0 || fail 'normal sampler stop failed'
[[ ${R2_DISK_STOP_REQUESTED_NS:-} =~ ^(0|[1-9][0-9]*)$ &&
  $(<"$normal_stop") == "$R2_DISK_STOP_REQUESTED_NS" && ! -e /proc/$stop_test_pid ]] ||
  fail 'normal sampler stop did not bind its marker and close'
stop_test_pid=

blocked_stop=$temporary/blocked-sampler.stop
mkdir "$blocked_stop"
stop_test_start=
setsid taskset -c "$disk_test_controller_cpus" sleep 30 &
stop_test_pid=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$stop_test_pid/stat" &&
    [[ $R2_PROC_GROUP == "$stop_test_pid" && $R2_PROC_SESSION == "$stop_test_pid" ]]; then
    stop_test_start=$R2_PROC_START
    break
  fi
  sleep 0.01
done
kill -STOP -- "-$stop_test_pid"
blocked_stop_started_seconds=$SECONDS
set +e
r2_stop_disk_sampler "$stop_test_pid" "$stop_test_start" \
  "$disk_test_controller_cpus" "$blocked_stop" 0 \
  >"$temporary/blocked-stop.stdout" 2>"$temporary/blocked-stop.stderr"
blocked_stop_status=$?
set -e
blocked_stop_elapsed_seconds=$((SECONDS - blocked_stop_started_seconds))
if [[ $blocked_stop_status -eq 0 || $blocked_stop_elapsed_seconds -gt 5 ||
  -e /proc/$stop_test_pid ]] ||
  r2_sampler_session_has_members "$stop_test_pid"; then
  fail 'failed stop marker leaked or hung its stopped sampler group'
fi
stop_test_pid=
plant_count=$((plant_count + 1))

fake_retry_clock_bin=$temporary/fake-retry-clock-bin
fake_retry_clock_state=$temporary/fake-retry-clock-state
mkdir "$fake_retry_clock_bin"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  '[[ $# -eq 1 && $1 == +%s%N ]] || exit 95' \
  'count=0' \
  '[[ ! -f ${R2_TEST_DATE_STATE:?} ]] || read -r count <"$R2_TEST_DATE_STATE"' \
  'count=$((count + 1))' \
  'printf '\''%s\n'\'' "$count" >"$R2_TEST_DATE_STATE"' \
  'case $count in' \
  '  1) printf '\''1000000000\n'\'' ;;' \
  '  2) printf '\''1200000000\n'\'' ;;' \
  '  *) exit 96 ;;' \
  'esac' \
  >"$fake_retry_clock_bin/date"
chmod 755 "$fake_retry_clock_bin/date"

retry_row=$(R2_TEST_DU_STATE="$fake_disk_state" \
  R2_TEST_DATE_STATE="$fake_retry_clock_state" \
  PATH="$fake_retry_clock_bin:$fake_disk_bin:$PATH" \
  r2_measure_checkout_mib "$repo_root") ||
  fail 'disk sampler did not refresh a successful retry timestamp'
IFS=$'\t' read -r retry_started retry_mib <<<"$retry_row"
[[ $retry_started == 1200000000 && $retry_mib == 17 &&
  $(<"$fake_disk_state") == 2 && $(<"$fake_retry_clock_state") == 2 ]] ||
  fail 'disk sampler retained the failed attempt timestamp'

find "$fake_disk_state" "$fake_retry_clock_state" -delete
retry_raw=$temporary/retry-raw.tsv
retry_affinity=$temporary/retry-affinity.txt
retry_started_signal=$temporary/retry-started.txt
: >"$retry_raw"
R2_TEST_DU_STATE="$fake_disk_state" \
  R2_TEST_DATE_STATE="$fake_retry_clock_state" \
  R2_TEST_DU_AFFINITY_FILE="$retry_affinity" \
  PATH="$fake_retry_clock_bin:$fake_disk_bin:$PATH" \
  r2_record_checkout_mib "$repo_root" "$retry_raw" 900000000 7 scheduled \
  "$retry_started_signal" ||
  fail 'disk record worker rejected a successful retry'
[[ $(<"$retry_raw") == $'7\t1200000000\t300000000\t17\tscheduled' &&
  $(<"$retry_started_signal") == 1200000000 &&
  $(<"$fake_disk_state") == 2 && $(<"$fake_retry_clock_state") == 2 ]] ||
  fail 'disk record worker did not retain the refreshed retry timestamp'
retry_affinity_value=$(<"$retry_affinity")
retry_affinity_value=${retry_affinity_value//[$'\t ']/}
r2_expand_cpu_list "$retry_affinity_value" || fail 'disk worker affinity is malformed'
[[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]] ||
  fail 'disk worker did not enter its isolated CPU set'
find "$fake_disk_state" "$fake_retry_clock_state" "$retry_started_signal" -delete

disk_row=$(R2_TEST_DU_STATE="$fake_disk_state" PATH="$fake_disk_bin:$PATH" \
  r2_measure_checkout_mib "$repo_root") || fail 'disk sampler did not recover from a raced walk'
IFS=$'\t' read -r disk_started disk_mib <<<"$disk_row"
[[ $disk_started =~ ^[0-9]+$ && $disk_mib == 17 && $(<"$fake_disk_state") == 2 ]] ||
  fail 'disk sampler retained the wrong recovered result'
find "$fake_disk_state" -delete
set +e
R2_TEST_DU_STATE="$fake_disk_state" R2_TEST_DU_ALWAYS_FAIL=1 \
  PATH="$fake_disk_bin:$PATH" r2_measure_checkout_mib "$repo_root" \
  >"$temporary/disk-failure.stdout" 2>"$temporary/disk-failure.stderr"
disk_failure_status=$?
set -e
[[ $disk_failure_status -ne 0 && ! -s $temporary/disk-failure.stdout ]] ||
  fail 'disk sampler accepted twenty incomplete walks'
grep -Fx 'R2 disk sampler: no complete du result after 20 attempts' \
  "$temporary/disk-failure.stderr" >/dev/null ||
  fail 'disk sampler permanent-failure diagnostic differs'
plant_count=$((plant_count + 1))

async_samples=$temporary/async-disk-samples.tsv
async_parts=$temporary/async-disk-state
async_stop=$temporary/async-disk.stop
async_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$async_samples"
mkdir "$async_parts"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_TEST_DU_STABLE=1 \
  R2_TEST_DU_DELAY=0.2 \
  R2_DISK_WALK_CPUS="$R2_DISK_WALK_CPUS" \
  PATH="$fake_disk_bin:$PATH" \
  bash -c '
  set -euo pipefail
  source "$1"
  shift
  r2_sample_checkout_disk \
    "$@"
' r2-async-sampler "$harness_lib_source" \
  "$repo_root" "$async_samples" "$async_stop" "$async_parts" \
  "$async_started" 50000000 &
async_sampler_pid=$!
async_session_bound=0
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$async_sampler_pid/stat" &&
    [[ $R2_PROC_GROUP == "$async_sampler_pid" &&
      $R2_PROC_SESSION == "$async_sampler_pid" && $R2_PROC_STATE != Z ]]; then
    async_session_bound=1
    break
  fi
  kill -0 "$async_sampler_pid" 2>/dev/null || break
  sleep 0.001
done
[[ $async_session_bound -eq 1 ]] ||
  fail 'asynchronous sampler does not own its session and process group'
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $async_parts/ready ]] || break
  kill -0 "$async_sampler_pid" 2>/dev/null || fail 'asynchronous sampler exited before readiness'
  sleep 0.01
done
[[ -f $async_parts/ready ]] || fail 'asynchronous sampler did not become ready'
sleep 0.26
async_stop_started=$(date +%s%N)
: >"$async_stop"
wait "$async_sampler_pid" || fail 'asynchronous sampler rejected complete walks'
async_count=0
async_gap_ns=0
async_previous_started=0
async_terminal_count=0
async_terminal_started=0
async_terminal_seen=0
{
  IFS= read -r async_header
  [[ $async_header == $'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind' ]] ||
    fail 'asynchronous sampler raw header differs'
  while IFS=$'\t' read -r async_ordinal async_sample_started async_elapsed \
    async_mib async_kind async_extra; do
    [[ $async_ordinal == "$async_count" && $async_sample_started =~ ^[0-9]+$ &&
      $async_elapsed =~ ^[0-9]+$ && $async_mib =~ ^[0-9]+$ &&
      -z $async_extra && $async_sample_started -ge $async_started &&
      $async_elapsed -eq $((async_sample_started - async_started)) ]] ||
      fail 'asynchronous sampler raw row arithmetic differs'
    [[ $async_terminal_seen -eq 0 ]] ||
      fail 'asynchronous sampler retained a row after its terminal row'
    if [[ $async_count -gt 0 ]]; then
      async_gap=$((async_sample_started - async_previous_started))
      [[ $async_gap -gt 0 ]] || fail 'asynchronous sampler starts are not increasing'
      ((async_gap <= async_gap_ns)) || async_gap_ns=$async_gap
    fi
    case $async_kind in
      scheduled) ;;
      terminal)
        async_terminal_seen=1
        async_terminal_count=$((async_terminal_count + 1))
        async_terminal_started=$async_sample_started
        ;;
      *) fail 'asynchronous sampler row kind differs' ;;
    esac
    async_previous_started=$async_sample_started
    async_count=$((async_count + 1))
  done
} <"$async_samples"
[[ $async_count -ge 5 && $async_gap_ns -le 100000000 &&
  $async_terminal_count -eq 1 && $async_terminal_started -ge $async_stop_started &&
  ! -e $async_parts ]] ||
  fail 'asynchronous sampler did not preserve exact cadence/session/terminal evidence'

# Publication is chronological even when workers start out of launch order;
# launch ordinals remain a separate, complete identity set.
chronology_samples=$temporary/chronology-samples.tsv
chronology_state=$temporary/chronology-state
chronology_raw=$chronology_state/samples.unsorted.tsv
chronology_sorted=$chronology_state/samples.sorted.tsv
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$chronology_samples"
mkdir "$chronology_state"
printf '%s\n' \
  $'0\t1000000000\t0\t17\tscheduled' \
  $'2\t1050000000\t50000000\t17\tscheduled' \
  $'1\t1075000000\t75000000\t17\tscheduled' \
  $'3\t1100000000\t100000000\t17\tterminal' >"$chronology_raw"
r2_publish_checkout_disk_samples "$chronology_samples" "$chronology_state" \
  "$chronology_raw" "$chronology_sorted" 1000000000 4 ||
  fail 'chronological publication rejected complete out-of-order ordinals'
[[ $(sed -n '2,5p' "$chronology_samples") == $'0\t1000000000\t0\t17\tscheduled\n2\t1050000000\t50000000\t17\tscheduled\n1\t1075000000\t75000000\t17\tscheduled\n3\t1100000000\t100000000\t17\tterminal' ]] ||
  fail 'chronological publication reordered by launch identity'
chronology_stop=$temporary/chronology.stop
chronology_summary=$temporary/chronology-summary.json
printf '1090000000\n' >"$chronology_stop"
r2_write_checkout_disk_summary "$chronology_samples" "$chronology_stop" \
  "$chronology_summary" 1000000000 50000000 1090000000 ||
  fail 'disk summary refused chronological raw evidence and its stop marker'
jq -e '.stop_requested_ns == "1090000000" and .maximum_gap_ns == "50000000"' \
  "$chronology_summary" >/dev/null || fail 'disk summary arithmetic differs'

# Drive the production raw-row validator with deterministic record workers.
# Exactly 100,000,000 ns is admitted; one nanosecond more is refused. A
# pre-existing stop marker gives the controller exactly one scheduled row and
# its required final terminal row without depending on host scheduling.
run_exact_gap_sampler() {
  local planted_gap=$1
  local planted_samples=$2
  local planted_stop=$3
  local planted_state=$4
  local planted_origin=1000000000
  (
    r2_record_checkout_mib() {
      [[ $# -eq 6 ]] || return 2
      local raw=$2 origin=$3 ordinal=$4 kind=$5 signal=$6
      local started=$((origin + ordinal * planted_gap))
      if [[ $kind == scheduled ]]; then
        sleep 0.01
      else
        [[ -f ${signal%/*}/started.0 ]] || return 2
      fi
      printf '%s\n' "$started" >"$signal"
      printf '%s\t%s\t%s\t17\t%s\n' \
        "$ordinal" "$started" "$((started - origin))" "$kind" >>"$raw"
    }
    r2_sample_checkout_disk \
      "$repo_root" "$planted_samples" "$planted_stop" "$planted_state" \
      "$planted_origin" 50000000
  )
}

gap_pass_samples=$temporary/exact-gap-pass.tsv
gap_pass_stop=$temporary/exact-gap-pass.stop
gap_pass_state=$temporary/exact-gap-pass-state
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$gap_pass_samples"
: >"$gap_pass_stop"
mkdir "$gap_pass_state"
run_exact_gap_sampler 100000000 "$gap_pass_samples" "$gap_pass_stop" "$gap_pass_state" ||
  fail 'disk sampler refused an exact 100000000 ns retained-start gap'
[[ $(wc -l <"$gap_pass_samples") -eq 3 &&
  $(sed -n '2p' "$gap_pass_samples") == $'0\t1000000000\t0\t17\tscheduled' &&
  $(sed -n '3p' "$gap_pass_samples") == $'1\t1100000000\t100000000\t17\tterminal' &&
  ! -e $gap_pass_state ]] ||
  fail 'exact 100000000 ns gap evidence differs'

gap_fail_samples=$temporary/exact-gap-fail.tsv
gap_fail_stop=$temporary/exact-gap-fail.stop
gap_fail_state=$temporary/exact-gap-fail-state
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$gap_fail_samples"
: >"$gap_fail_stop"
mkdir "$gap_fail_state"
set +e
run_exact_gap_sampler 100000001 "$gap_fail_samples" "$gap_fail_stop" "$gap_fail_state" \
  >"$temporary/exact-gap-fail.stdout" 2>"$temporary/exact-gap-fail.stderr"
gap_fail_status=$?
set -e
[[ $gap_fail_status -ne 0 && ! -s $temporary/exact-gap-fail.stdout ]] ||
  fail 'disk sampler admitted a 100000001 ns retained-start gap'
grep -Fx 'R2 disk sampler: retained sample-start gap exceeds 100000000 ns' \
  "$temporary/exact-gap-fail.stderr" >/dev/null ||
  fail 'disk sampler gap-overflow diagnostic differs'
plant_count=$((plant_count + 1))

# A long proof launches thousands of samples. Reaping must consult only the
# bounded active child set on every launch rather than retaining and rescanning
# every historical PID. Count the process probes and bind them to the number of
# retained samples while quick walks repeatedly finish.
history_samples=$temporary/history-disk-samples.tsv
history_parts=$temporary/history-disk-state
history_stop=$temporary/history-disk.stop
history_probes=$temporary/history-disk-probes.txt
history_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$history_samples"
: >"$history_probes"
mkdir "$history_parts"
run_history_sampler() {
  eval "$(declare -f r2_read_process_stat | sed \
    '1s/r2_read_process_stat/r2_read_process_stat_unprobed/')"
  r2_read_process_stat() {
    printf 'probe\n' >>"$history_probes"
    r2_read_process_stat_unprobed "$@"
  }
  export R2_TEST_DU_STABLE=1 R2_TEST_DU_DELAY=0.02
  export PATH="$fake_disk_bin:$PATH"
  r2_sample_checkout_disk \
    "$repo_root" "$history_samples" "$history_stop" "$history_parts" \
    "$history_started" 10000000
}
run_history_sampler &
history_sampler_pid=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $history_parts/ready ]] || break
  kill -0 "$history_sampler_pid" 2>/dev/null || fail 'history sampler exited before readiness'
  sleep 0.01
done
[[ -f $history_parts/ready ]] || fail 'history sampler did not become ready'
sleep 0.8
: >"$history_stop"
wait "$history_sampler_pid" || fail 'history sampler rejected quick complete walks'
history_count=$(awk 'NR > 1 { count += 1 } END { print count + 0 }' "$history_samples")
history_probe_count=$(wc -l <"$history_probes")
# These quick two-period walks should need only a small active set; sixteen
# probes per launch is deliberately generous but still rejects O(n²) history.
[[ $history_count -ge 40 && $history_probe_count -gt 0 &&
  $history_probe_count -le $(((history_count + 1) * 16)) &&
  ! -e $history_parts ]] ||
  fail 'disk sampler retained historical jobs instead of the bounded active set'

# A failed asynchronous walk must be reaped, make the controller fail without
# publishing a row, and leave no live child behind in the controller shell.
child_failure_samples=$temporary/child-failure-disk-samples.tsv
child_failure_state=$temporary/child-failure-disk-state
child_failure_stop=$temporary/child-failure-disk.stop
child_failure_waits=$temporary/child-failure-disk-waits.txt
child_failure_jobs=$temporary/child-failure-disk-jobs.txt
child_failure_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$child_failure_samples"
: >"$child_failure_waits"
mkdir "$child_failure_state"
set +e
(
  wait() {
    printf '%s\n' "$*" >>"$child_failure_waits"
    builtin wait "$@"
  }
  export R2_TEST_DU_STABLE=1 R2_TEST_DU_FAIL=1
  export PATH="$fake_disk_bin:$PATH"
  r2_sample_checkout_disk \
    "$repo_root" "$child_failure_samples" "$child_failure_stop" \
    "$child_failure_state" "$child_failure_started" 50000000
  child_failure_status=$?
  jobs -pr >"$child_failure_jobs"
  exit "$child_failure_status"
) >"$temporary/child-failure.stdout" 2>"$temporary/child-failure.stderr"
child_failure_status=$?
set -e
[[ $child_failure_status -ne 0 && ! -s $temporary/child-failure.stdout &&
  $(wc -l <"$child_failure_samples") -eq 1 &&
  $(wc -l <"$child_failure_waits") -ge 1 &&
  ! -s $child_failure_jobs ]] ||
  fail 'disk sampler published or leaked a failed asynchronous walk'
grep -Fx 'R2 disk sampler: one or more scheduled samples failed' \
  "$temporary/child-failure.stderr" >/dev/null ||
  fail 'disk sampler child-failure diagnostic differs'
plant_count=$((plant_count + 1))

overload_samples=$temporary/overload-disk-samples.tsv
overload_state=$temporary/overload-disk-state
overload_stop=$temporary/overload-disk.stop
overload_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$overload_samples"
mkdir "$overload_state"
set +e
(
  export R2_TEST_DU_STABLE=1 R2_TEST_DU_DELAY=2.0
  export PATH="$fake_disk_bin:$PATH"
  r2_sample_checkout_disk \
    "$repo_root" "$overload_samples" "$overload_stop" "$overload_state" \
    "$overload_started" 50000000
) >"$temporary/overload.stdout" 2>"$temporary/overload.stderr"
overload_status=$?
set -e
[[ $overload_status -ne 0 && ! -s $temporary/overload.stdout ]] ||
  fail 'asynchronous sampler permitted unbounded concurrent walks'
grep -Fx 'R2 disk sampler: thirty-two concurrent du walks are still active' \
  "$temporary/overload.stderr" >/dev/null ||
  fail 'asynchronous sampler concurrency-limit diagnostic differs'
plant_count=$((plant_count + 1))

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

printf 'R2_COMPLETE_PROOF_PLANTS PASS\n'
printf 'planted_failures %s\n' "$plant_count"
printf 'clean_outer_preflight not_run_no_documented_test_hook\n'

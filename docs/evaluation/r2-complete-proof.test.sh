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

for command in git cp ln chmod mkdir mktemp grep realpath readlink wc sed find id sleep; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-complete-proof-plants.XXXXXX")
seed=$temporary/seed
linked=
leaked_child_pid=
cleanup() {
  case $temporary in
    "$repo_root"/target/r2-complete-proof-plants.*)
      if [[ -n ${leaked_child_pid:-} ]] && kill -0 "$leaked_child_pid" 2>/dev/null; then
        kill "$leaked_child_pid" 2>/dev/null || true
        wait "$leaked_child_pid" 2>/dev/null || true
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
  stat date dirname du; do
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

# A `du` walk can race Cargo's atomic deletion of an intermediate file. The
# sampler must discard that incomplete walk, retain a subsequent complete
# result, and fail closed if no complete result can be obtained.
fake_disk_bin=$temporary/fake-disk-bin
fake_disk_state=$temporary/fake-disk-state
mkdir "$fake_disk_bin"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'if [[ ${R2_TEST_DU_STABLE:-0} == 1 ]]; then' \
  '  sleep "${R2_TEST_DU_DELAY:-0}"' \
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
chmod 755 "$fake_disk_bin/du"
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
printf 'ordinal\telapsed_ms\tmebibytes\n' >"$async_samples"
mkdir "$async_parts"
(
  export R2_TEST_DU_STABLE=1 R2_TEST_DU_DELAY=0.2
  export PATH="$fake_disk_bin:$PATH"
  r2_sample_checkout_disk \
    "$repo_root" "$async_samples" "$async_stop" "$async_parts" \
    "$async_started" 50000000
) &
async_sampler_pid=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $async_parts/ready ]] || break
  kill -0 "$async_sampler_pid" 2>/dev/null || fail 'asynchronous sampler exited before readiness'
  sleep 0.01
done
[[ -f $async_parts/ready ]] || fail 'asynchronous sampler did not become ready'
sleep 0.26
: >"$async_stop"
wait "$async_sampler_pid" || fail 'asynchronous sampler rejected slow complete walks'
async_count=$(awk 'NR > 1 { count += 1 } END { print count + 0 }' "$async_samples")
async_gap=$(awk 'NR == 2 { previous = $2 } NR > 2 { gap = $2 - previous; if (gap > maximum) maximum = gap; previous = $2 } END { print maximum + 0 }' "$async_samples")
[[ $async_count -ge 5 && $async_gap -le 100 && ! -e $async_parts ]] ||
  fail 'asynchronous sampler did not preserve cadence across slow walks'

overload_samples=$temporary/overload-disk-samples.tsv
overload_state=$temporary/overload-disk-state
overload_stop=$temporary/overload-disk.stop
overload_started=$(date +%s%N)
printf 'ordinal\telapsed_ms\tmebibytes\n' >"$overload_samples"
mkdir "$overload_state"
set +e
(
  export R2_TEST_DU_STABLE=1 R2_TEST_DU_DELAY=0.6
  export PATH="$fake_disk_bin:$PATH"
  r2_sample_checkout_disk \
    "$repo_root" "$overload_samples" "$overload_stop" "$overload_state" \
    "$overload_started" 50000000
) >"$temporary/overload.stdout" 2>"$temporary/overload.stderr"
overload_status=$?
set -e
[[ $overload_status -ne 0 && ! -s $temporary/overload.stdout ]] ||
  fail 'asynchronous sampler permitted unbounded concurrent walks'
grep -Fx 'R2 disk sampler: eight concurrent du walks are still active' \
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

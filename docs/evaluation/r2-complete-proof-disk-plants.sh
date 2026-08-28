# shellcheck shell=bash
# This source-only fixture intentionally consumes parent-suite globals, defines
# functions invoked indirectly by the candidate, and emits literal Bash source.
# shellcheck disable=SC2016,SC2030,SC2031,SC2154,SC2329

# Source-only disk-observer plants split from r2-complete-proof.test.sh so each
# routinely edited proof file remains reviewable. The parent supplies its
# fixture paths, lifecycle variables, fail helper, and plant counter.

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'R2 complete proof disk plants: source this file from the parent suite\n' >&2
  exit 2
fi

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
  '[[ $process_nice == 0 ]] || exit 92' \
  'if [[ -n ${R2_TEST_DU_AFFINITY_DIRECTORY:-} ]]; then' \
  '  [[ -d $R2_TEST_DU_AFFINITY_DIRECTORY && ! -L $R2_TEST_DU_AFFINITY_DIRECTORY ]] || exit 97' \
  '  du_allowed=' \
  '  worker_allowed=' \
  '  while IFS= read -r line; do' \
  '    [[ $line != Cpus_allowed_list:* ]] || du_allowed=${line#*:}' \
  '  done </proc/self/status' \
  '  while IFS= read -r line; do' \
  '    [[ $line != Cpus_allowed_list:* ]] || worker_allowed=${line#*:}' \
  '  done <"/proc/$PPID/status"' \
  '  IFS= read -r du_stat </proc/self/stat || exit 97' \
  '  du_fields=${du_stat##*) }' \
  '  [[ $du_fields != "$du_stat" ]] || exit 97' \
  '  read -r _ du_parent du_group du_session _ <<<"$du_fields"' \
  '  [[ $du_parent == "$PPID" ]] || exit 97' \
  '  IFS= read -r worker_stat <"/proc/$PPID/stat" || exit 97' \
  '  worker_fields=${worker_stat##*) }' \
  '  [[ $worker_fields != "$worker_stat" ]] || exit 97' \
  '  read -r _ worker_parent worker_group worker_session _ <<<"$worker_fields"' \
  '  du_allowed=${du_allowed//[$'\''\t '\'']/}' \
  '  worker_allowed=${worker_allowed//[$'\''\t '\'']/}' \
  '  affinity_record=$R2_TEST_DU_AFFINITY_DIRECTORY/$BASHPID' \
  '  set -o noclobber' \
  '  printf '\''%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n'\'' "$BASHPID" "$PPID" "$du_group" "$du_session" "$du_allowed" "$worker_parent" "$worker_group" "$worker_session" "$worker_allowed" >"$affinity_record" || exit 97' \
  '  set +o noclobber' \
  'fi' \
  'if [[ -n ${R2_TEST_DU_AFFINITY_FILE:-} ]]; then' \
  '  while IFS= read -r line; do' \
  '    [[ $line != Cpus_allowed_list:* ]] || printf '\''%s\n'\'' "${line#*:}" >"$R2_TEST_DU_AFFINITY_FILE"' \
  '  done </proc/self/status' \
  'fi' \
  'if [[ -n ${R2_TEST_DU_HOLD_MARKER:-} ]]; then' \
  '  [[ $R2_TEST_DU_HOLD_MARKER == */* ]] || exit 97' \
  '  while [[ ! -e $R2_TEST_DU_HOLD_MARKER && ! -L $R2_TEST_DU_HOLD_MARKER ]]; do sleep 0.01; done' \
  '  [[ -f $R2_TEST_DU_HOLD_MARKER && ! -L $R2_TEST_DU_HOLD_MARKER ]] || exit 97' \
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
r2_expand_cpu_list "$disk_test_affinity" || fail 'disk sampler test affinity is malformed'
disk_test_affinity=$R2_EXPANDED_CPU_LIST
plant_count=$((plant_count + 1))
disk_test_controller_cpus=${disk_test_affinity%%,*}
export R2_DISK_WALK_CPUS=$disk_test_affinity

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
  $(<"$retry_started_signal") == $'7\t1000000000' &&
  $(<"$fake_disk_state") == 2 && $(<"$fake_retry_clock_state") == 2 ]] ||
  fail 'disk record worker did not separate launch acknowledgement from retained retry time'
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
async_affinity_directory=$temporary/async-disk-affinity
async_taskset_bin=$temporary/async-taskset-bin
async_taskset_log=$temporary/async-taskset.tsv
async_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$async_samples"
mkdir "$async_parts" "$async_affinity_directory" "$async_taskset_bin"
IFS=, read -r -a disk_test_cpu_array <<<"$disk_test_affinity"
[[ ${#disk_test_cpu_array[@]} -ge 2 ]] ||
  fail 'the real pool-affinity plant requires two allowed logical CPUs'
printf -v async_walk_cpus '%s,' "${disk_test_cpu_array[@]:1}"
async_walk_cpus=${async_walk_cpus%,}
real_taskset=$(type -P taskset)
[[ -n $real_taskset && -f $real_taskset && ! -L $real_taskset ]] ||
  fail 'the real pool-affinity plant could not bind taskset'
printf '%s\n' \
  '#!/usr/bin/env bash' \
  '[[ $# -eq 3 && $1 == -pc && $2 == "${R2_TEST_WALK_CPUS:?}" && $3 =~ ^[1-9][0-9]*$ && $3 == "$PPID" ]] || exit 98' \
  'worker_parent=$(ps -o ppid= -p "$3") || exit 98' \
  'worker_group=$(ps -o pgid= -p "$3") || exit 98' \
  'worker_session=$(ps -o sid= -p "$3") || exit 98' \
  'worker_parent=${worker_parent// /}' \
  'worker_group=${worker_group// /}' \
  'worker_session=${worker_session// /}' \
  'printf '\''%s\t%s\t%s\t%s\t%s\t%s\n'\'' "$1" "$2" "$3" "$worker_parent" "$worker_group" "$worker_session" >>"${R2_TEST_TASKSET_LOG:?}"' \
  'exec "${R2_TEST_REAL_TASKSET:?}" "$@"' \
  >"$async_taskset_bin/taskset"
chmod 755 "$async_taskset_bin/taskset"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_TEST_DU_STABLE=1 \
  R2_TEST_DU_DELAY=0.005 \
  R2_TEST_DU_AFFINITY_DIRECTORY="$async_affinity_directory" \
  R2_TEST_TASKSET_LOG="$async_taskset_log" \
  R2_TEST_REAL_TASKSET="$real_taskset" \
  R2_TEST_WALK_CPUS="$async_walk_cpus" \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  PATH="$async_taskset_bin:$fake_disk_bin:$PATH" \
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
async_sampler_start=
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$async_sampler_pid/stat" &&
    [[ $R2_PROC_GROUP == "$async_sampler_pid" &&
      $R2_PROC_SESSION == "$async_sampler_pid" && $R2_PROC_STATE != Z ]]; then
    async_sampler_start=$R2_PROC_START
    async_session_bound=1
    break
  fi
  kill -0 "$async_sampler_pid" 2>/dev/null || break
  sleep 0.001
done
[[ $async_session_bound -eq 1 ]] ||
  fail 'asynchronous sampler does not own its session and process group'
r2_sampler_identity_stable "$async_sampler_pid" "$async_sampler_start" \
  "$disk_test_controller_cpus" ||
  fail 'asynchronous sampler left its controller-only CPU mask'
r2_monotonic_now_ns || fail 'could not start real pool-affinity readiness deadline'
r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 12000000000 ||
  fail 'real pool-affinity readiness deadline overflowed'
async_ready_deadline=$R2_DISK_DEADLINE_NS
while :; do
  [[ ! -e $async_parts/ready ]] || break
  kill -0 "$async_sampler_pid" 2>/dev/null || fail 'asynchronous sampler exited before readiness'
  r2_monotonic_now_ns || fail 'real pool-affinity readiness clock failed'
  [[ $R2_MONOTONIC_NS -lt $async_ready_deadline ]] || break
  sleep 0.01
done
[[ -f $async_parts/ready ]] || fail 'asynchronous sampler did not become ready'
async_live_rows=0
r2_monotonic_now_ns || fail 'could not start real pool-affinity row deadline'
r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 8000000000 ||
  fail 'real pool-affinity row deadline overflowed'
async_row_deadline=$R2_DISK_DEADLINE_NS
while :; do
  async_live_rows=$(wc -l <"$async_parts/samples.unsorted.tsv")
  [[ $async_live_rows -lt 40 ]] || break
  kill -0 "$async_sampler_pid" 2>/dev/null ||
    fail 'asynchronous sampler exited before its fortieth row'
  r2_monotonic_now_ns || fail 'real pool-affinity row clock failed'
  [[ $R2_MONOTONIC_NS -lt $async_row_deadline ]] || break
  sleep 0.01
done
[[ $async_live_rows -ge 40 ]] ||
  fail 'real pool-affinity plant did not dispatch more than its pool size'
# Drain/request/ready ordering is independently planted by the terminal-order
# suite. Keep this process-level affinity plant focused on the real pool and
# exact walks: publish its canonical stop directly instead of making success
# depend on the host scheduling a second parent-side handoff within 100 ms.
async_stop_started=$(date +%s%N)
r2_publish_decimal_control_marker "$async_stop" "$async_stop_started" ||
  fail 'could not publish asynchronous sampler stop marker'
wait "$async_sampler_pid" || fail 'asynchronous sampler rejected complete walks'
async_sampler_identity=$async_sampler_pid
if r2_sampler_session_has_members "$async_sampler_identity"; then
  fail 'asynchronous sampler left a live session member'
else
  async_session_status=$?
fi
[[ $async_session_status -eq 1 ]] ||
  fail 'asynchronous sampler session closure could not be proved'
async_sampler_pid=
async_count=0
async_gap_ns=0
async_previous_started=0
async_terminal_count=0
async_terminal_started=0
async_terminal_seen=0
async_scheduled_count=0
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
      scheduled)
        async_scheduled_count=$((async_scheduled_count + 1))
        ;;
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
[[ $async_count -ge 41 && $async_scheduled_count -ge 40 &&
  $async_gap_ns -le 100000000 && $async_terminal_count -eq 1 &&
  $async_terminal_started -ge $async_stop_started && ! -e $async_parts ]] ||
  fail 'asynchronous sampler did not preserve exact cadence/session/terminal evidence'
async_affinity_count=0
declare -A async_affinity_parent_workers=()
for async_affinity_record in "$async_affinity_directory"/*; do
  [[ -f $async_affinity_record && ! -L $async_affinity_record ]] ||
    fail 'real pool-affinity plant did not retain a regular du record'
  IFS=$'\t' read -r async_du_pid async_du_parent async_du_group \
    async_du_session async_du_affinity async_worker_parent async_worker_group \
    async_worker_session async_worker_affinity async_extra <"$async_affinity_record"
  [[ $async_du_pid =~ ^[1-9][0-9]*$ && $async_du_parent =~ ^[1-9][0-9]*$ &&
    $async_du_group == "$async_sampler_identity" &&
    $async_du_session == "$async_sampler_identity" &&
    $async_worker_parent =~ ^[1-9][0-9]*$ &&
    $async_worker_group == "$async_sampler_identity" &&
    $async_worker_session == "$async_sampler_identity" && -z $async_extra ]] ||
    fail 'a real du walk was not descended from the dedicated sampler session'
  r2_expand_cpu_list "$async_du_affinity" ||
    fail 'a real du walk recorded malformed CPU affinity'
  [[ $R2_EXPANDED_CPU_LIST == "$async_walk_cpus" ]] ||
    fail 'a real du walk escaped the walk-only CPU mask'
  r2_expand_cpu_list "$async_worker_affinity" ||
    fail 'a real pool worker recorded malformed CPU affinity'
  [[ $R2_EXPANDED_CPU_LIST == "$async_walk_cpus" ]] ||
    fail 'a real pool worker escaped the walk-only CPU mask'
  async_affinity_parent_workers["$async_worker_parent"]=1
  async_affinity_count=$((async_affinity_count + 1))
done
[[ $async_affinity_count -gt 32 ]] ||
  fail 'real pool-affinity plant did not observe more walks than the pool size'
async_taskset_count=0
declare -A async_taskset_workers=()
while IFS=$'\t' read -r async_taskset_mode async_taskset_mask \
  async_taskset_worker async_taskset_parent async_taskset_group \
  async_taskset_session async_taskset_extra; do
  [[ $async_taskset_mode == -pc && $async_taskset_mask == "$async_walk_cpus" &&
    $async_taskset_worker =~ ^[1-9][0-9]*$ &&
    $async_taskset_parent == "$async_sampler_identity" &&
    $async_taskset_group == "$async_sampler_identity" &&
    $async_taskset_session == "$async_sampler_identity" &&
    -z $async_taskset_extra &&
    -z ${async_taskset_workers[$async_taskset_worker]+present} ]] ||
    fail 'a pool worker was not affinitized exactly once as a direct child'
  async_taskset_workers["$async_taskset_worker"]=1
  async_taskset_count=$((async_taskset_count + 1))
done <"$async_taskset_log"
for async_worker_parent in "${!async_affinity_parent_workers[@]}"; do
  [[ -n ${async_taskset_workers[$async_worker_parent]+present} ]] ||
    fail 'a real du walk did not descend from a recorded pool worker'
done
[[ $async_taskset_count -eq 32 &&
  ${#async_taskset_workers[@]} -eq 32 &&
  $async_affinity_count -gt 32 && $async_taskset_count -lt $async_count ]] ||
  fail 'worker affinity was applied per sample or the bounded pool size differs'
plant_count=$((plant_count + 1))
# Bash can start a process-substitution child even when the dynamic descriptor
# duplication then fails. Exhaust the child's descriptor limit at that exact
# boundary and require the launch-failure branch to close its dedicated group;
# returning normally would leave the deliberately held child in that session.
channel_samples=$temporary/channel-failure-samples.tsv
channel_state=$temporary/channel-failure-state
channel_stop=$temporary/channel-failure.stop
channel_stdout=$temporary/channel-failure.stdout
channel_stderr=$temporary/channel-failure.stderr
channel_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$channel_samples"
mkdir "$channel_state"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  bash -c '
  set -euo pipefail
  source "$1"
  R2_TEST_CHANNEL_RECORD=$5/worker-0.results
  eval "$(declare -f r2_read_process_stat |
    sed "1s/^r2_read_process_stat /r2_real_read_process_stat /")"
  r2_channel_controller_reads=0
  r2_read_process_stat() {
    [[ $# -eq 1 ]] || return 2
    if [[ $1 == "/proc/$BASHPID/stat" ]]; then
      r2_channel_controller_reads=$((r2_channel_controller_reads + 1))
      if [[ $r2_channel_controller_reads -eq 2 ]]; then for ((attempt = 0; attempt < 5000; attempt += 1)); do [[ -s $R2_TEST_CHANNEL_RECORD ]] && break; sleep 0.001; done; fi
    fi
    r2_real_read_process_stat "$1"
  }
  r2_disk_pool_worker() {
    [[ $# -eq 5 ]] || return 2
    local descriptor
    local -a inherited_descriptors=()
    for descriptor in {3..9}; do eval "exec ${descriptor}>&-"; done
    IFS=, read -r -a inherited_descriptors <<<"$5"
    for descriptor in "${inherited_descriptors[@]}"; do eval "exec ${descriptor}>&-"; done
    r2_real_read_process_stat "/proc/$BASHPID/stat" || return 2
    printf "held\t%s\t%s\t%s\t%s\t%s\n" "$1" "$BASHPID" \
      "$R2_PROC_PARENT" "$R2_PROC_GROUP" "$R2_PROC_SESSION"
    while :; do sleep 1; done
  }
  ulimit -n 13
  for descriptor in {3..9}; do eval "exec ${descriptor}<>/dev/null"; done
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-channel-failure "$harness_lib_source" "$repo_root" "$channel_samples" \
  "$channel_stop" "$channel_state" "$channel_started" \
  >"$channel_stdout" 2>"$channel_stderr" &
stop_test_pid=$!
set +e
wait "$stop_test_pid" 2>>"$channel_stderr"
channel_status=$?
set -e
IFS=$'\t' read -r channel_tag channel_index channel_child channel_parent \
  channel_group channel_session channel_extra <"$channel_state/worker-0.results" ||
  fail 'worker-channel plant did not retain its started child record'
[[ $channel_status -eq 137 && $(wc -l <"$channel_samples") -eq 1 ]] ||
  fail 'failed worker-channel launch published a sample or returned success'
[[ $channel_tag == held && $channel_index == 0 &&
  $channel_child =~ ^[1-9][0-9]*$ && $channel_parent == "$stop_test_pid" &&
  $channel_group == "$stop_test_pid" && $channel_session == "$stop_test_pid" &&
  -z $channel_extra ]] || fail 'worker-channel plant did not bind its live child'
grep -Fx 'R2 disk sampler: worker channel launch failed' "$channel_stderr" >/dev/null ||
  fail 'worker-channel launch failure diagnostic differs'
channel_session_status=0
for ((attempt = 0; attempt < 500 && channel_session_status == 0; attempt += 1)); do
  if r2_sampler_session_has_members "$stop_test_pid"; then sleep 0.001; else channel_session_status=$?; fi
done
[[ $channel_session_status -eq 1 ]] ||
  fail 'worker-channel failure could not prove sampler-session closure'
stop_test_pid=
plant_count=$((plant_count + 1))

# A process-substitution child exists as soon as `$!` is assigned. Refuse its
# /proc identity capture while the child deliberately remains live: the
# controller must already have registered the child, then close its verified
# dedicated group instead of returning and orphaning an untracked worker.
startup_samples=$temporary/startup-capture-samples.tsv
startup_state=$temporary/startup-capture-state
startup_stop=$temporary/startup-capture.stop
startup_child_record=$temporary/startup-capture-child.tsv
startup_attempts=$temporary/startup-capture-attempts.txt
startup_stdout=$temporary/startup-capture.stdout
startup_stderr=$temporary/startup-capture.stderr
startup_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$startup_samples"
mkdir "$startup_state"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  R2_TEST_STARTUP_CHILD_RECORD="$startup_child_record" \
  R2_TEST_STARTUP_ATTEMPTS="$startup_attempts" \
  bash -c '
  set -euo pipefail
  source "$1"
  eval "$(declare -f r2_read_process_stat |
    sed "1s/^r2_read_process_stat /r2_real_read_process_stat /")"
  r2_read_process_stat() {
    [[ $# -eq 1 ]] || return 2
    if [[ $1 == "/proc/$BASHPID/stat" ]]; then
      r2_real_read_process_stat "$1"
    else
      if [[ ! -s $R2_TEST_STARTUP_CHILD_RECORD ]]; then
        r2_monotonic_now_ns || return 2
        r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 6000000000 || return 2
        startup_record_deadline=$R2_DISK_DEADLINE_NS
        while [[ ! -s $R2_TEST_STARTUP_CHILD_RECORD ]]; do
          r2_monotonic_now_ns || return 2
          [[ $R2_MONOTONIC_NS -lt $startup_record_deadline ]] || return 2
          sleep 0.001 || return 2
        done
      fi
      printf "%s\n" "$1" >>"$R2_TEST_STARTUP_ATTEMPTS"
      return 1
    fi
  }
  r2_disk_pool_worker() {
    [[ $# -eq 5 ]] || return 2
    r2_real_read_process_stat "/proc/$BASHPID/stat" || return 2
    printf "%s\t%s\t%s\t%s\n" "$BASHPID" "$R2_PROC_PARENT" \
      "$R2_PROC_GROUP" "$R2_PROC_SESSION" >"$R2_TEST_STARTUP_CHILD_RECORD"
    while :; do sleep 1; done
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-startup-capture "$harness_lib_source" "$repo_root" "$startup_samples" \
  "$startup_stop" "$startup_state" "$startup_started" \
  >"$startup_stdout" 2>"$startup_stderr" &
stop_test_pid=$!
for ((attempt = 0; attempt < 5000; attempt += 1)); do
  [[ ! -s $startup_child_record ]] || break
  kill -0 "$stop_test_pid" 2>/dev/null || break
  sleep 0.001
done
[[ -s $startup_child_record ]] ||
  fail 'startup-capture plant did not hold its process-substitution child'
set +e
wait "$stop_test_pid" 2>/dev/null
startup_status=$?
set -e
IFS=$'\t' read -r startup_child_pid startup_child_parent startup_child_group \
  startup_child_session startup_extra <"$startup_child_record"
[[ $startup_status -ne 0 && $startup_child_pid =~ ^[1-9][0-9]*$ &&
  $startup_child_parent == "$stop_test_pid" &&
  $startup_child_group == "$stop_test_pid" &&
  $startup_child_session == "$stop_test_pid" && -z $startup_extra ]] ||
  fail 'startup-capture child was not bound to the dedicated sampler group'
grep -Fx "/proc/$startup_child_pid/stat" "$startup_attempts" >/dev/null ||
  fail 'startup-capture plant did not refuse the launched child identity'
if r2_sampler_session_has_members "$stop_test_pid"; then
  fail 'failed startup identity capture orphaned a live pool worker'
else
  startup_session_status=$?
fi
[[ $startup_session_status -eq 1 && ! -e /proc/$startup_child_pid ]] ||
  fail 'startup identity failure did not close the dedicated sampler session'
stop_test_pid=
plant_count=$((plant_count + 1))

# A worker that changes affinity after its last result is no longer an
# admissible idle worker. Gate ordinal zero, move worker 31 onto the
# controller-only mask, then release the sample under a pre-existing stop. The
# controller must reject the idle worker before orderly shutdown; this record
# override intentionally removes the independent per-walk affinity defense.
idle_samples=$temporary/idle-affinity-samples.tsv
idle_state=$temporary/idle-affinity-state
idle_stop=$temporary/idle-affinity.stop
idle_entered=$temporary/idle-affinity-entered
idle_release=$temporary/idle-affinity-release
idle_stdout=$temporary/idle-affinity.stdout
idle_stderr=$temporary/idle-affinity.stderr
idle_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$idle_samples"
mkdir "$idle_state"
r2_publish_decimal_control_marker "$idle_stop" "$idle_started" ||
  fail 'could not publish idle-affinity stop marker'
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  R2_TEST_IDLE_ENTERED="$idle_entered" \
  R2_TEST_IDLE_RELEASE="$idle_release" \
  bash -c '
  set -euo pipefail
  source "$1"
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5 started
    if [[ $ordinal -eq 0 ]]; then
      : >"$R2_TEST_IDLE_ENTERED"
      while [[ ! -e $R2_TEST_IDLE_RELEASE ]]; do sleep 0.001; done
    fi
    started=$(date +%s%N) || return 2
    printf "%s\t%s\t%s\t17\t%s\n" "$ordinal" "$started" \
      "$((started - origin))" "$kind" >>"$raw"
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-idle-affinity "$harness_lib_source" "$repo_root" "$idle_samples" \
  "$idle_stop" "$idle_state" "$idle_started" \
  >"$idle_stdout" 2>"$idle_stderr" &
stop_test_pid=$!
idle_worker_record=$idle_state/worker-31.results
for ((attempt = 0; attempt < 5000; attempt += 1)); do
  [[ -s $idle_worker_record && -e $idle_entered ]] && break
  kill -0 "$stop_test_pid" 2>/dev/null ||
    fail 'idle-affinity sampler exited before its worker became mutable'
  sleep 0.001
done
[[ -s $idle_worker_record && -e $idle_entered ]] ||
  fail 'idle-affinity plant did not reach its gated idle-worker state'
IFS=$'\t' read -r idle_ready_tag idle_worker_index idle_worker_pid idle_extra \
  <"$idle_worker_record"
[[ $idle_ready_tag == ready && $idle_worker_index == 31 &&
  $idle_worker_pid =~ ^[1-9][0-9]*$ && -z $idle_extra ]] ||
  fail 'idle-affinity worker readiness record differs'
"$real_taskset" -pc "$disk_test_controller_cpus" "$idle_worker_pid" >/dev/null ||
  fail 'could not plant idle-worker affinity drift'
r2_read_allowed_cpu_list "/proc/$idle_worker_pid/status" ||
  fail 'mutated idle-worker affinity is unreadable'
[[ $R2_EXPANDED_CPU_LIST == "$disk_test_controller_cpus" ]] ||
  fail 'idle-worker affinity mutation did not take effect'
: >"$idle_release"
set +e
wait "$stop_test_pid"
idle_status=$?
set -e
[[ $idle_status -ne 0 && $(wc -l <"$idle_samples") -eq 1 ]] ||
  fail 'sampler accepted an idle worker whose affinity changed'
if r2_sampler_session_has_members "$stop_test_pid"; then
  fail 'idle-affinity refusal leaked a sampler-session member'
else
  idle_session_status=$?
fi
[[ $idle_session_status -eq 1 ]] ||
  fail 'idle-affinity refusal could not prove sampler-session closure'
stop_test_pid=
plant_count=$((plant_count + 1))

# A live worker whose recorded PID/start/parent/group/session tuple changes is
# not safe to mark reaped. Override only the controller's /proc view after all
# workers are ready, report a false PGID for idle worker 31, and require the
# controller to abort the real dedicated group rather than merely forgetting
# that still-live child.
structural_samples=$temporary/structural-identity-samples.tsv
structural_state=$temporary/structural-identity-state
structural_stop=$temporary/structural-identity.stop
structural_entered=$temporary/structural-identity-entered
structural_release=$temporary/structural-identity-release
structural_target=$temporary/structural-identity-target
structural_stdout=$temporary/structural-identity.stdout
structural_stderr=$temporary/structural-identity.stderr
structural_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$structural_samples"
mkdir "$structural_state"
r2_publish_decimal_control_marker "$structural_stop" "$structural_started" ||
  fail 'could not publish structural-identity stop marker'
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  R2_TEST_STRUCTURAL_ENTERED="$structural_entered" \
  R2_TEST_STRUCTURAL_RELEASE="$structural_release" \
  R2_TEST_STRUCTURAL_TARGET="$structural_target" \
  bash -c '
  set -euo pipefail
  source "$1"
  eval "$(declare -f r2_disk_pool_worker |
    sed "1s/^r2_disk_pool_worker /r2_real_disk_pool_worker /")"
  r2_disk_pool_worker() {
    [[ $# -eq 5 ]] || return 2
    if [[ $1 != 31 ]]; then
      r2_real_disk_pool_worker "$@"
      return
    fi
    local descriptor
    local -a inherited_descriptors=()
    if [[ $5 != - ]]; then
      IFS=, read -r -a inherited_descriptors <<<"$5"
      for descriptor in "${inherited_descriptors[@]}"; do
        eval "exec ${descriptor}>&-"
      done
    fi
    taskset -pc "$R2_DISK_WALK_CPUS" "$BASHPID" >/dev/null || return 2
    r2_read_allowed_cpu_list "/proc/$BASHPID/status" || return 2
    [[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]] || return 2
    printf "ready\t31\t%s\n" "$BASHPID"
    while :; do sleep 1; done
  }
  eval "$(declare -f r2_read_process_stat |
    sed "1s/^r2_read_process_stat /r2_real_read_process_stat /")"
  r2_read_process_stat() {
    [[ $# -eq 1 ]] || return 2
    r2_real_read_process_stat "$1" || return
    if [[ -s $R2_TEST_STRUCTURAL_TARGET ]]; then
      structural_worker=$(<"$R2_TEST_STRUCTURAL_TARGET")
      if [[ $1 == "/proc/$structural_worker/stat" ]]; then
        R2_PROC_GROUP=$((R2_PROC_GROUP + 1))
      fi
    fi
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5 started
    if [[ $ordinal -eq 0 ]]; then
      : >"$R2_TEST_STRUCTURAL_ENTERED"
      while [[ ! -e $R2_TEST_STRUCTURAL_RELEASE ]]; do sleep 0.001; done
    fi
    started=$(date +%s%N) || return 2
    printf "%s\t%s\t%s\t17\t%s\n" "$ordinal" "$started" \
      "$((started - origin))" "$kind" >>"$raw"
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-structural-identity "$harness_lib_source" "$repo_root" \
  "$structural_samples" "$structural_stop" "$structural_state" \
  "$structural_started" >"$structural_stdout" 2>"$structural_stderr" &
stop_test_pid=$!
structural_worker_record=$structural_state/worker-31.results
for ((attempt = 0; attempt < 5000; attempt += 1)); do
  [[ -s $structural_worker_record && -e $structural_entered ]] && break
  kill -0 "$stop_test_pid" 2>/dev/null ||
    fail 'structural-identity sampler exited before its mutation point'
  sleep 0.001
done
[[ -s $structural_worker_record && -e $structural_entered ]] ||
  fail 'structural-identity plant did not reach its gated idle-worker state'
IFS=$'\t' read -r structural_ready_tag structural_worker_index \
  structural_worker_pid structural_extra <"$structural_worker_record"
[[ $structural_ready_tag == ready && $structural_worker_index == 31 &&
  $structural_worker_pid =~ ^[1-9][0-9]*$ && -z $structural_extra ]] ||
  fail 'structural-identity worker readiness record differs'
printf '%s\n' "$structural_worker_pid" >"$structural_target"
: >"$structural_release"
set +e
wait "$stop_test_pid" 2>>"$structural_stderr"
structural_status=$?
set -e
[[ $structural_status -eq 137 && $(wc -l <"$structural_samples") -eq 1 ]] ||
  fail 'live structural worker mismatch did not abort the dedicated group'
grep -Fx 'R2 disk sampler: live worker identity changed during shutdown' \
  "$structural_stderr" >/dev/null ||
  fail 'live structural worker mismatch diagnostic differs'
if r2_sampler_session_has_members "$stop_test_pid"; then
  fail 'structural-identity refusal leaked a sampler-session member'
else
  structural_session_status=$?
fi
[[ $structural_session_status -eq 1 ]] ||
  fail 'structural-identity refusal could not prove sampler-session closure'
stop_test_pid=
plant_count=$((plant_count + 1))

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
r2_publish_decimal_control_marker "$chronology_stop" 1090000000 ||
  fail 'could not publish chronology stop marker'
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
      [[ $# -eq 5 ]] || return 2
      local raw=$2 origin=$3 ordinal=$4 kind=$5
      local started=$((origin + ordinal * planted_gap))
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
r2_publish_decimal_control_marker "$gap_pass_stop" 1050000000 ||
  fail 'could not publish exact-gap pass stop marker'
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
r2_publish_decimal_control_marker "$gap_fail_stop" 1050000000 ||
  fail 'could not publish exact-gap failure stop marker'
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

# shellcheck source=docs/evaluation/r2-disk-history-plant.sh
source "$script_directory/r2-disk-history-plant.sh"

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
overload_launches=$temporary/overload-disk-launches
overload_hold=$temporary/overload-disk.release
overload_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$overload_samples"
mkdir "$overload_state" "$overload_launches"
set +e
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_TEST_DU_STABLE=1 \
  R2_TEST_DU_HOLD_MARKER="$overload_hold" \
  R2_TEST_DU_AFFINITY_DIRECTORY="$overload_launches" \
  R2_DISK_WALK_CPUS="$R2_DISK_WALK_CPUS" \
  PATH="$fake_disk_bin:$PATH" \
  bash -c '
  set -euo pipefail
  source "$1"
  shift
  r2_sample_checkout_disk "$@"
' r2-overload-sampler "$harness_lib_source" \
  "$repo_root" "$overload_samples" "$overload_stop" "$overload_state" \
  "$overload_started" 50000000 \
  >"$temporary/overload.stdout" 2>"$temporary/overload.stderr" &
overload_sampler_pid=$!
overload_session=$overload_sampler_pid
wait "$overload_sampler_pid" 2>>"$temporary/overload.stderr"
overload_status=$?
set -e
overload_launch_count=$(find "$overload_launches" -maxdepth 1 -type f | wc -l)
[[ $overload_status -ne 0 && ! -s $temporary/overload.stdout &&
  $(wc -l <"$overload_samples") -eq 1 && $overload_launch_count -eq 32 ]] ||
  fail 'asynchronous sampler permitted unbounded concurrent walks'
[[ $(grep -Fxc 'R2 disk sampler: thirty-two concurrent du walks are still active' \
  "$temporary/overload.stderr") -eq 1 ]] ||
  fail 'asynchronous sampler concurrency-limit diagnostic differs'
if r2_sampler_session_has_members "$overload_session"; then
  fail 'concurrency-limit sampler left a live session member'
else
  overload_session_status=$?
fi
[[ $overload_session_status -eq 1 ]] ||
  fail 'concurrency-limit sampler session closure could not be proved'
overload_sampler_pid=
plant_count=$((plant_count + 1))

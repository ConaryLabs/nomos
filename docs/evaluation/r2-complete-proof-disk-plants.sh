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
for ((attempt = 0; attempt < 500; attempt += 1)); do
  [[ ! -e $async_parts/ready ]] || break
  kill -0 "$async_sampler_pid" 2>/dev/null || fail 'asynchronous sampler exited before readiness'
  sleep 0.01
done
[[ -f $async_parts/ready ]] || fail 'asynchronous sampler did not become ready'
async_live_rows=0
for ((attempt = 0; attempt < 500; attempt += 1)); do
  async_live_rows=$(wc -l <"$async_parts/samples.unsorted.tsv")
  [[ $async_live_rows -lt 40 ]] || break
  sleep 0.01
done
[[ $async_live_rows -ge 40 ]] ||
  fail 'real pool-affinity plant did not dispatch more than its pool size'
unset R2_DISK_STOP_REQUESTED_NS
r2_prepare_and_stop_disk_sampler "$async_sampler_pid" "$async_sampler_start" \
  "$disk_test_controller_cpus" "$async_stop" "$async_parts" 0 ||
  fail 'asynchronous sampler rejected complete walks'
async_stop_started=$R2_DISK_STOP_REQUESTED_NS
async_sampler_identity=$async_sampler_pid
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
[[ $async_affinity_count -ge 5 ]] ||
  fail 'real pool-affinity plant observed too few exact du walks'
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
  $async_affinity_count -gt 0 && $async_taskset_count -lt $async_count ]] ||
  fail 'worker affinity was applied per sample or the bounded pool size differs'
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
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5
    local started=$((origin + ordinal * 10000000))
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - origin))" "$kind" >>"$raw"
  }
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
r2_publish_decimal_control_marker "$history_stop" "$(date +%s%N)" ||
  fail 'could not publish history-sampler stop marker'
wait "$history_sampler_pid" || fail 'history sampler rejected quick complete walks'
history_sampler_pid=
history_count=$(awk 'NR > 1 { count += 1 } END { print count + 0 }' "$history_samples")
history_probe_count=$(wc -l <"$history_probes")
# Deterministic quick workers need only a small active set; sixteen probes per
# launch is deliberately generous but still rejects O(n²) history.
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
overload_launches=$temporary/overload-disk-launches
overload_started=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$overload_samples"
mkdir "$overload_state" "$overload_launches"
set +e
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_TEST_DU_STABLE=1 \
  R2_TEST_DU_DELAY=2.0 \
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
wait "$overload_sampler_pid"
overload_status=$?
overload_sampler_pid=
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
plant_count=$((plant_count + 1))

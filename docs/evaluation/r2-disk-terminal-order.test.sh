#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'R2 disk terminal-order plant: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
# shellcheck source=docs/evaluation/r2-complete-proof-lib.sh
source "$script_directory/r2-complete-proof-lib.sh"

r2_disk_deadline_ns 1000000000 1 50000000
phase_one=$R2_DISK_DEADLINE_NS
r2_disk_deadline_ns 1000000000 2 50000000
phase_two=$R2_DISK_DEADLINE_NS
r2_disk_deadline_ns 1000000000 3 50000000
phase_three=$R2_DISK_DEADLINE_NS
[[ $phase_one -eq 1050000000 && $phase_two -eq 1100000000 &&
  $phase_three -eq 1150000000 ]] || fail 'absolute 50 ms deadlines differ'
r2_disk_deadline_ns 1000000000 1 50000001
[[ $R2_DISK_DEADLINE_NS -eq 1050000001 ]] || fail 'an odd positive period moved'
if r2_disk_deadline_ns 1000000000 1 0 ||
  r2_disk_deadline_ns 9223372036854775800 1 100 ||
  r2_disk_deadline_ns 9223372036854775808 0 1 ||
  r2_disk_deadline_ns 0 9223372036854775807 2; then
  fail 'a zero, out-of-range, or overflowing nominal period was accepted'
fi

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-disk-terminal-order.XXXXXX")
atomic_publisher=
handshake_pid=
deadline_sampler_pid=
hung_sampler_pid=
partial_ready_sampler_pid=
wait_setup_sampler_pid=
shutdown_sampler_pid=
capture_sampler_pid=
cleanup() {
  local session_pid
  [[ -z ${atomic_publisher:-} ]] || kill "$atomic_publisher" 2>/dev/null || true
  [[ -z ${atomic_publisher:-} ]] || wait "$atomic_publisher" 2>/dev/null || true
  for session_pid in "${handshake_pid:-}" "${deadline_sampler_pid:-}" \
    "${hung_sampler_pid:-}" "${partial_ready_sampler_pid:-}" \
    "${wait_setup_sampler_pid:-}" "${shutdown_sampler_pid:-}" \
    "${capture_sampler_pid:-}"; do
    [[ -n $session_pid ]] || continue
    kill -KILL -- "-$session_pid" 2>/dev/null || true
    wait "$session_pid" 2>/dev/null || true
  done
}
trap cleanup EXIT
test_affinity_line=$(taskset -pc $$)
test_affinity=${test_affinity_line##*: }
r2_expand_cpu_list "$test_affinity" || fail 'test CPU affinity is malformed'
test_controller_cpu=${R2_EXPANDED_CPU_LIST%%,*}
export R2_DISK_WALK_CPUS=$R2_EXPANDED_CPU_LIST

# shellcheck source=docs/evaluation/r2-procfs-read-plants.sh
source "$script_directory/r2-procfs-read-plants.sh"

# Handoff validation must stay exact above IEEE-754 integer precision while
# finishing inside the freshness window. The fast path sorts externally and
# uses decimal-string arithmetic in awk; plant both a valid out-of-order ledger
# and one-nanosecond method/coverage mutations.
handoff_origin=1787900000000000000
handoff_raw=$temporary/handoff-fast-raw.tsv
handoff_sorted=$temporary/handoff-fast-sorted.tsv
printf '%s\n' \
  $'0\t1787900000000000000\t0\t17\tscheduled' \
  $'2\t1787900000050000000\t50000000\t17\tscheduled' \
  $'1\t1787900000025000000\t25000000\t17\tscheduled' \
  $'3\t1787900000100000000\t100000000\t17\tscheduled' >"$handoff_raw"
(
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    printf '1787900000100000000\n'
  }
  r2_validate_disk_drain_handoff \
    "$handoff_raw" "$handoff_sorted" "$handoff_origin" 4 \
    1787900000020000000 1 75000000
) || fail 'fast exact handoff validator refused a complete ledger'
[[ ! -e $handoff_sorted ]] || fail 'fast handoff validator retained its sorted scratch'

# Adjacent integer nanoseconds above 2^53 must remain distinct. An awk numeric
# comparison rounds these values together even though both rows are valid.
handoff_one_ns=$temporary/handoff-one-ns.tsv
handoff_one_ns_sorted=$temporary/handoff-one-ns-sorted.tsv
printf '%s\n' \
  $'0\t1787900000000000000\t0\t17\tscheduled' \
  $'1\t1787900000000000001\t1\t17\tscheduled' >"$handoff_one_ns"
(
  date() { printf '1787900000000000001\n'; }
  r2_validate_disk_drain_handoff \
    "$handoff_one_ns" "$handoff_one_ns_sorted" "$handoff_origin" 2 \
    1787900000000000001 1 75000000
) || fail 'fast handoff validator rounded adjacent nanoseconds together'
[[ ! -e $handoff_one_ns_sorted ]] ||
  fail 'one-nanosecond handoff retained its sorted scratch'

handoff_bad_arithmetic=$temporary/handoff-bad-arithmetic.tsv
handoff_bad_arithmetic_sorted=$temporary/handoff-bad-arithmetic-sorted.tsv
sed 's/25000000\t17/25000001\t17/' "$handoff_raw" >"$handoff_bad_arithmetic"
set +e
(
  date() { printf '1787900000100000000\n'; }
  r2_validate_disk_drain_handoff \
    "$handoff_bad_arithmetic" "$handoff_bad_arithmetic_sorted" \
    "$handoff_origin" 4 1787900000020000000 1 75000000
)
handoff_bad_arithmetic_status=$?
set -e
[[ $handoff_bad_arithmetic_status -eq 2 ]] ||
  fail 'fast handoff validator admitted a one-nanosecond arithmetic mutation'

handoff_bad_gap=$temporary/handoff-bad-gap.tsv
handoff_bad_gap_sorted=$temporary/handoff-bad-gap-sorted.tsv
printf '%s\n' \
  $'0\t1787900000000000000\t0\t17\tscheduled' \
  $'1\t1787900000100000001\t100000001\t17\tscheduled' >"$handoff_bad_gap"
set +e
(
  date() { printf '1787900000100000001\n'; }
  r2_validate_disk_drain_handoff \
    "$handoff_bad_gap" "$handoff_bad_gap_sorted" "$handoff_origin" 2 \
    1787900000000000000 1 75000000
) 2>"$temporary/handoff-bad-gap.stderr"
handoff_bad_gap_status=$?
set -e
[[ $handoff_bad_gap_status -eq 1 ]] ||
  fail 'fast handoff validator admitted a 100000001 ns retained-start gap'
[[ $(wc -l <"$temporary/handoff-bad-gap.stderr") -eq 1 ]] &&
  grep -Fx 'R2 disk sampler: retained sample-start gap exceeds 100000000 ns' \
    "$temporary/handoff-bad-gap.stderr" >/dev/null ||
  fail 'fast handoff retained-gap diagnostic differs'

topology_root=$temporary/topology
write_sibling_group() {
  [[ $# -eq 2 && $1 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  mkdir -p "$topology_root/cpu$1/topology"
  printf '%s\n' "$2" >"$topology_root/cpu$1/topology/thread_siblings_list"
}
for pair in '0 6' '1 7' '2 8' '3 9' '4 10' '5 11'; do
  read -r first second <<<"$pair"
  write_sibling_group "$first" "$first,$second"
  write_sibling_group "$second" "$first,$second"
done
r2_partition_cpu_topology 0-11 "$topology_root" ||
  fail 'canonical sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 0,6 && $R2_DISK_CPUS == 1,2,3,4,7,8,9,10 &&
  $R2_WORKLOAD_CPUS == 5,11 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,6;1,7;2,8;3,9;4,10;5,11' &&
  $R2_CONTROLLER_PHYSICAL_GROUPS == '0,6' &&
  $R2_DISK_PHYSICAL_GROUPS == '1,7;2,8;3,9;4,10' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '5,11' ]] ||
  fail 'canonical three-way physical-core role split differs'
r2_validate_physical_cpu_isolation "$R2_CONTROLLER_CPUS" "$R2_DISK_CPUS" \
  "$topology_root" || fail 'controller and disk walks share a physical core'
r2_validate_physical_cpu_isolation "$R2_DISK_CPUS" "$R2_WORKLOAD_CPUS" \
  "$topology_root" || fail 'canonical physical-core role split overlaps'
if r2_validate_physical_cpu_isolation 0-5 6-11 "$topology_root"; then
  fail 'the former SMT-overlapping role split was accepted'
fi
r2_partition_cpu_topology '0-3,6-9' "$topology_root" ||
  fail 'four-group sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 0,6 && $R2_DISK_CPUS == 1,2,7,8 &&
  $R2_WORKLOAD_CPUS == 3,9 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,6;1,7;2,8;3,9' &&
  $R2_CONTROLLER_PHYSICAL_GROUPS == '0,6' &&
  $R2_DISK_PHYSICAL_GROUPS == '1,7;2,8' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '3,9' ]] ||
  fail 'four-group physical-core role split differs'
r2_partition_cpu_topology '0-4,6-10' "$topology_root" ||
  fail 'five-group sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 0,6 && $R2_DISK_CPUS == 1,2,3,7,8,9 &&
  $R2_WORKLOAD_CPUS == 4,10 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,6;1,7;2,8;3,9;4,10' &&
  $R2_CONTROLLER_PHYSICAL_GROUPS == '0,6' &&
  $R2_DISK_PHYSICAL_GROUPS == '1,7;2,8;3,9' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '4,10' ]] ||
  fail 'five-group physical-core role split differs'
r2_partition_cpu_topology '1,3-4,7,9-10' "$topology_root" ||
  fail 'irregular complete sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 1,7 && $R2_DISK_CPUS == 3,9 &&
  $R2_WORKLOAD_CPUS == 4,10 &&
  $R2_CPU_TOPOLOGY_GROUPS == '1,7;3,9;4,10' &&
  $R2_CONTROLLER_PHYSICAL_GROUPS == '1,7' &&
  $R2_DISK_PHYSICAL_GROUPS == '3,9' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '4,10' ]] ||
  fail 'irregular three-way physical-core split differs'
if r2_partition_cpu_topology '0-2,7-8' "$topology_root"; then
  fail 'a sibling group partly outside the allowed affinity was accepted'
fi
if r2_partition_cpu_topology 0,6 "$topology_root" ||
  r2_partition_cpu_topology 0-1,6-7 "$topology_root" ||
  r2_partition_cpu_topology 0,12 "$topology_root"; then
  fail 'an undersized or unreadable topology was accepted'
fi
printf '6\n' >"$topology_root/cpu6/topology/thread_siblings_list"
if r2_partition_cpu_topology 0,6 "$topology_root"; then
  fail 'contradictory sibling topology was accepted'
fi
write_sibling_group 6 0,6
write_sibling_group 0 0,6,99
write_sibling_group 1 1,7,99
if r2_partition_cpu_topology 0,1 "$topology_root"; then
  fail 'sibling groups overlapping outside the allowed mask were accepted'
fi
write_sibling_group 0 0,6
write_sibling_group 1 1,7

# Control markers publish only after their complete decimal line exists. Hold
# the atomic rename and prove that no reader can observe the prepared bytes at
# the destination; an existing destination is never overwritten.
atomic_directory=$temporary/atomic-marker
atomic_marker=$atomic_directory/marker
atomic_observed=$atomic_directory/rename-observed
atomic_release=$atomic_directory/release
mkdir "$atomic_directory"
(
  mv() {
    : >"$atomic_observed"
    while [[ ! -e $atomic_release ]]; do command sleep 0.001; done
    command mv "$@"
  }
  r2_publish_decimal_control_marker "$atomic_marker" 123456789
) &
atomic_publisher=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $atomic_observed ]] || break
  kill -0 "$atomic_publisher" 2>/dev/null || fail 'atomic marker publisher exited early'
  sleep 0.001
done
atomic_temporary=$(find "$atomic_directory" -maxdepth 1 -name '.marker.*.publish' -print -quit)
[[ -f $atomic_observed && ! -e $atomic_marker && -f $atomic_temporary ]] ||
  fail 'control marker became visible before its atomic rename'
r2_read_decimal_control_marker "$atomic_temporary" ||
  fail 'prepared control-marker bytes are not canonical'
[[ $R2_CONTROL_MARKER == 123456789 ]] || fail 'prepared control-marker value differs'
: >"$atomic_release"
wait "$atomic_publisher" || fail 'atomic marker publication failed'
atomic_publisher=
r2_read_decimal_control_marker "$atomic_marker" || fail 'published marker is not canonical'
[[ $R2_CONTROL_MARKER == 123456789 ]] || fail 'published control-marker value differs'
if r2_publish_decimal_control_marker "$atomic_marker" 987654321; then
  fail 'atomic control-marker publication overwrote an existing destination'
fi
printf '%s' 42 >"$atomic_directory/no-final-line-feed"
if r2_read_decimal_control_marker "$atomic_directory/no-final-line-feed"; then
  fail 'control-marker reader accepted a decimal without its final line feed'
fi
printf '%s\n' 9223372036854775808 >"$atomic_directory/out-of-range"
if r2_read_decimal_control_marker "$atomic_directory/out-of-range" ||
  r2_publish_decimal_control_marker \
    "$atomic_directory/out-of-range-published" 9223372036854775808; then
  fail 'control-marker arithmetic admitted a value above signed 64-bit range'
fi
[[ ! -e $atomic_directory/out-of-range-published ]] ||
  fail 'out-of-range control-marker publication left a destination'
if r2_wait_for_disk_stop_marker \
  "$atomic_directory/out-of-range-stop" "$atomic_marker" 123456789 \
  9223372036854775808 1; then
  fail 'stop-wait arithmetic admitted an out-of-range latest timestamp'
else
  out_of_range_wait_status=$?
fi
[[ $out_of_range_wait_status -eq 2 ]] ||
  fail 'out-of-range stop-wait timestamp did not fail as invalid input'

# The sampler's post-ready wait must outlive the parent's separate six-second
# preparation window by elapsed monotonic time, not by a smaller polling count.
# Force the historical 5,000-iteration boundary without spending wall time and
# publish a valid stop only on iteration 5,001.
delayed_stop_directory=$temporary/delayed-stop
delayed_stop_request=$delayed_stop_directory/drain-request
delayed_stop_marker=$delayed_stop_directory/stop
delayed_stop_result=$delayed_stop_directory/result
mkdir "$delayed_stop_directory"
r2_publish_decimal_control_marker "$delayed_stop_request" 1787900000000000000 ||
  fail 'could not publish delayed-stop request'
(
  delayed_stop_iterations=0
  r2_monotonic_now_ns() { R2_MONOTONIC_NS=1000000000; }
  sleep() {
    [[ $# -eq 1 && $1 == 0.001 ]] || return 2
    delayed_stop_iterations=$((delayed_stop_iterations + 1))
    if [[ $delayed_stop_iterations -eq 5001 ]]; then
      r2_publish_decimal_control_marker \
        "$delayed_stop_marker" 1787900000050000000
    fi
  }
  r2_wait_for_disk_stop_marker \
    "$delayed_stop_marker" "$delayed_stop_request" 1787900000000000000 \
    1787900000000000000 6000000000
  printf '%s\t%s\n' "$delayed_stop_iterations" "$R2_DISK_STOP_NS" \
    >"$delayed_stop_result"
) || fail 'monotonic stop wait expired at the former polling boundary'
[[ $(<"$delayed_stop_result") == $'5001\t1787900000050000000' ]] ||
  fail 'monotonic stop-wait result differs'

# The same wait must expire at its six-second monotonic deadline. Advance a
# synthetic clock by one second per probe; a bounded fake sleep also prevents
# an accidentally unbounded implementation from hanging this plant.
expiry_directory=$temporary/stop-expiry
expiry_request=$expiry_directory/drain-request
expiry_stop=$expiry_directory/stop
expiry_trace=$expiry_directory/trace
expiry_stdout=$expiry_directory/stdout
expiry_stderr=$expiry_directory/stderr
mkdir "$expiry_directory"
r2_publish_decimal_control_marker "$expiry_request" 1787900000000000000 ||
  fail 'could not publish stop-expiry request'
set +e
(
  expiry_probe=0
  expiry_sleep=0
  r2_monotonic_now_ns() {
    expiry_probe=$((expiry_probe + 1))
    [[ $expiry_probe -le 7 ]] || return 2
    R2_MONOTONIC_NS=$((expiry_probe * 1000000000))
    printf '%s\n' "$R2_MONOTONIC_NS" >>"$expiry_trace"
  }
  sleep() {
    [[ $# -eq 1 && $1 == 0.001 ]] || return 2
    expiry_sleep=$((expiry_sleep + 1))
    [[ $expiry_sleep -le 5 ]] || return 97
    printf '%s\n' "$expiry_sleep" >"$expiry_directory/sleeps"
  }
  r2_wait_for_disk_stop_marker \
    "$expiry_stop" "$expiry_request" 1787900000000000000 \
    1787900000000000000 6000000000
) >"$expiry_stdout" 2>"$expiry_stderr"
expiry_status=$?
set -e
[[ $expiry_status -eq 1 && ! -s $expiry_stdout &&
  $(<"$expiry_directory/sleeps") == 5 && $(wc -l <"$expiry_trace") -eq 7 &&
  $(<"$expiry_stderr") == \
    'R2 disk sampler: stop marker did not arrive before timeout' ]] ||
  fail 'monotonic stop wait did not expire at exactly six seconds'

# The parent-side stop helper must request and observe a completed pre-stop
# drain before it writes the canonical stop marker.
handshake_state=$temporary/handshake-state
handshake_stop=$temporary/handshake.stop
mkdir "$handshake_state"
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  while [[ ! -f $1/drain-request ]]; do sleep 0.001; done
  request=$(<"$1/drain-request")
  source "$3"
  r2_publish_decimal_control_marker "$1/drain-ready" "$request"
  while [[ ! -f $2 ]]; do sleep 0.001; done
' r2-drain-handshake "$handshake_state" "$handshake_stop" \
  "$script_directory/r2-disk-control-lib.sh" &
handshake_pid=$!
handshake_start=
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$handshake_pid/stat" &&
    [[ $R2_PROC_GROUP == "$handshake_pid" && $R2_PROC_SESSION == "$handshake_pid" ]]; then
    handshake_start=$R2_PROC_START
    break
  fi
  sleep 0.001
done
[[ $handshake_start =~ ^[0-9]+$ ]] || fail 'drain-handshake identity was not stable'
session_members_source=$(declare -f r2_sampler_session_has_members)
eval "$(declare -f r2_sampler_session_has_members | sed \
  '1s/r2_sampler_session_has_members/r2_sampler_session_has_members_live/')"
session_indeterminate=0
r2_sampler_session_has_members() {
  if [[ $1 == "$handshake_pid" && ! -e /proc/$1 &&
    $session_indeterminate -eq 0 ]]; then
    session_indeterminate=1
    return 2
  fi
  r2_sampler_session_has_members_live "$@"
}
unset R2_DISK_STOP_REQUESTED_NS
r2_prepare_and_stop_disk_sampler "$handshake_pid" "$handshake_start" \
  "$test_controller_cpu" "$handshake_stop" "$handshake_state" 0 ||
  fail 'parent-side drain handshake was refused'
[[ ${R2_DISK_STOP_REQUESTED_NS:-} =~ ^(0|[1-9][0-9]*)$ &&
  $session_indeterminate -eq 1 &&
  $(<"$handshake_stop") == "$R2_DISK_STOP_REQUESTED_NS" &&
  $(<"$handshake_state/drain-request") == $(<"$handshake_state/drain-ready") &&
  ! -e /proc/$handshake_pid ]] || fail 'drain handshake did not precede stop and closure'
eval "$session_members_source"
unset -f r2_sampler_session_has_members_live
unset session_members_source
handshake_pid=
find "$handshake_state/drain-request" "$handshake_state/drain-ready" -delete

# A drain observed after the initial worker has already completed still owes
# one post-intent bridge. The worker itself publishes the request after writing
# ordinal zero, making the zero-live-root boundary deterministic.
zero_root_samples=$temporary/zero-root-samples.tsv
zero_root_stop=$temporary/zero-root.stop
zero_root_state=$temporary/zero-root-state
zero_root_trace=$temporary/zero-root-trace.tsv
zero_root_request_file=$temporary/zero-root-request.txt
zero_root_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$zero_root_samples"
mkdir "$zero_root_state"
(
  zero_root_coordinator=
  stop_zero_root_coordinator() {
    [[ -n ${zero_root_coordinator:-} ]] || return 0
    kill "$zero_root_coordinator" 2>/dev/null || true
    wait "$zero_root_coordinator" 2>/dev/null || true
  }
  trap stop_zero_root_coordinator EXIT
  (
    for ((attempt = 0; attempt < 5000; attempt += 1)); do
      [[ ! -f $zero_root_state/drain-ready ]] || break
      command sleep 0.001
    done
    [[ -f $zero_root_state/drain-ready ]] || exit 2
    r2_publish_decimal_control_marker "$zero_root_stop" "$(date +%s%N)"
  ) &
  zero_root_coordinator=$!
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 started request
    started=$(date +%s%N)
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    printf '%s\t%s\t%s\n' "$ordinal" "$started" "$kind" >>"$zero_root_trace"
    if [[ $ordinal -eq 0 ]]; then
      request=$(date +%s%N)
      printf '%s\n' "$request" >"$zero_root_request_file"
      r2_publish_decimal_control_marker "$zero_root_state/drain-request" "$request"
    fi
  }
  if r2_sample_checkout_disk "$repo_root" "$zero_root_samples" "$zero_root_stop" \
    "$zero_root_state" "$zero_root_origin" 50000000; then zero_root_status=0
  else zero_root_status=$?; fi
  if wait "$zero_root_coordinator"; then coordinator_status=0
  else coordinator_status=$?; fi
  zero_root_coordinator=
  trap - EXIT
  [[ $zero_root_status -eq 0 && $coordinator_status -eq 0 ]]
) || fail 'zero-live-root drain did not retain a post-intent bridge'
zero_root_request=$(<"$zero_root_request_file")
zero_root_scheduled_after=$(awk -F '\t' -v request="$zero_root_request" \
  '$3 == "scheduled" && $2 >= request { count += 1 } END { print count + 0 }' \
  "$zero_root_trace")
[[ ! -e $zero_root_state && $zero_root_scheduled_after -ge 1 &&
  $(awk -F '\t' '$5 == "terminal" { count += 1 } END { print count + 0 }' \
    "$zero_root_samples") -eq 1 ]] || fail 'zero-live-root handoff evidence differs'

# Exercise the real controller against a deterministic clock. The worker
# override accepts exactly the production five-argument call, so restoring a
# synchronous acknowledgement fails this plant. Trace the production helper's
# exact fixed origin, ordinal, and 50 ms period independently from the worker's
# authentic attempt timestamp; ordinal 1 begins 7 ms after its nominal slot.
absolute_samples=$temporary/absolute-samples.tsv
absolute_stop=$temporary/absolute-stop
absolute_state=$temporary/absolute-state
absolute_clock=$temporary/absolute-clock
absolute_trace=$temporary/absolute-trace
absolute_deadlines=$temporary/absolute-deadlines
absolute_origin=1000000000
printf '%s\n' "$absolute_origin" >"$absolute_clock"
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$absolute_samples"
: >"$absolute_deadlines"
mkdir "$absolute_state"
(
  eval "$(declare -f r2_disk_deadline_ns | sed \
    '1s/r2_disk_deadline_ns/r2_real_disk_deadline_ns/')"
  r2_disk_deadline_ns() {
    if [[ $# -eq 3 && $1 == "$absolute_origin" && $3 == 50000000 ]]; then
      printf '%s\t%s\t%s\n' "$1" "$2" "$3" >>"$absolute_deadlines"
    fi
    r2_real_disk_deadline_ns "$@"
  }
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    local current
    IFS= read -r current <"$absolute_clock"
    printf '%s\n' "$current"
  }
  sleep() {
    local current delta
    if [[ $# -eq 1 && $1 =~ ^([0-9]+)\.([0-9]{9})$ ]]; then
      current=$(<"$absolute_clock")
      delta=$((10#${BASH_REMATCH[1]} * 1000000000 + 10#${BASH_REMATCH[2]}))
      printf '%s\n' "$((current + delta))" >"$absolute_clock"
    else
      command sleep "$@"
    fi
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 started
    started=$((sampler_origin + ordinal * 50000000))
    [[ $ordinal -ne 1 ]] || started=$((started + 7000000))
    printf '%s\t%s\t%s\n' "$ordinal" "$kind" "$started" >>"$absolute_trace"
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    [[ $ordinal -ne 3 ]] || r2_publish_decimal_control_marker "$absolute_stop" "$started"
  }
  r2_sample_checkout_disk "$repo_root" "$absolute_samples" "$absolute_stop" \
    "$absolute_state" "$absolute_origin" 50000000
) || fail 'deterministic absolute-schedule controller trace was refused'
[[ $(sort -n -t $'\t' -k1,1 "$absolute_trace" | sed -n '1,4p') == \
  $'0\tscheduled\t1000000000\n1\tscheduled\t1057000000\n2\tscheduled\t1100000000\n3\tscheduled\t1150000000' &&
  $(sed -n '1,4p' "$absolute_deadlines") == \
  $'1000000000\t0\t50000000\n1000000000\t1\t50000000\n1000000000\t2\t50000000\n1000000000\t3\t50000000' &&
  $(awk -F '\t' 'END { print $2 }' "$absolute_trace") == terminal &&
  $(<"$absolute_stop") == 1150000000 && ! -e $absolute_state ]] ||
  fail 'controller launches are not on the absolute 50 ms production phase'

samples=$temporary/samples.tsv
stop=$temporary/stop
state=$temporary/state
origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$samples"
mkdir "$state"

# Ordinal 1 remains live when ordinal 3 requests a pre-stop drain. The
# controller must keep launching bridge samples until ordinal 5 releases it,
# quiesce every scheduled worker, acknowledge the drain, and only then accept
# the canonical stop marker and launch the terminal row.
(
  stop_coordinator=
  stop_main_coordinator() {
    [[ -n ${stop_coordinator:-} ]] || return 0
    kill "$stop_coordinator" 2>/dev/null || true
    wait "$stop_coordinator" 2>/dev/null || true
  }
  trap stop_main_coordinator EXIT
  (
    for ((attempt = 0; attempt < 5000; attempt += 1)); do
      [[ ! -f $state/drain-ready ]] || break
      command sleep 0.001
    done
    [[ -f $state/drain-ready ]] || exit 2
    [[ $(<"$state/drain-ready") == $((origin + 180000000)) ]] || exit 2
    r2_publish_decimal_control_marker "$stop" "$((origin + 300000000))"
  ) &
  stop_coordinator=$!
  eval "$(declare -f r2_validate_disk_drain_handoff | sed \
    '1s/r2_validate_disk_drain_handoff/r2_validate_disk_drain_handoff_live/')"
  r2_validate_disk_drain_handoff() {
    local latest validation_status
    latest=$(sort -t $'\t' -k2,2n "$1" | tail -n 1 | cut -f 2)
    date() {
      [[ $# -eq 1 && $1 == +%s%N ]] || return 2
      printf '%s\n' "$latest"
    }
    if r2_validate_disk_drain_handoff_live "$@"; then validation_status=0
    else validation_status=$?; fi
    unset -f date
    return "$validation_status"
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 started
    started=$((sampler_origin + ordinal * 50000000))
    if [[ $ordinal -eq 1 ]]; then
      while [[ ! -e $state/release-original ]]; do command sleep 0.001; done
      find "$state/release-original" -delete
      started=$((sampler_origin + 275000000))
    fi
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    if [[ $ordinal -eq 3 ]]; then
      r2_publish_decimal_control_marker \
        "$state/drain-request" "$((sampler_origin + 180000000))"
    elif [[ $ordinal -eq 5 ]]; then
      : >"$state/release-original"
    fi
  }
  if r2_sample_checkout_disk "$repo_root" "$samples" "$stop" "$state" \
    "$origin" 50000000; then sampler_status=0
  else sampler_status=$?; fi
  if wait "$stop_coordinator"; then coordinator_status=0
  else coordinator_status=$?; fi
  stop_coordinator=
  trap - EXIT
  [[ $sampler_status -eq 0 && $coordinator_status -eq 0 ]]
) || fail 'sampler did not bridge its pre-stop drain before the terminal row'

[[ ! -e $state && $(wc -l <"$samples") -ge 6 ]] ||
  fail 'sampler did not publish the complete planted ledger'
stop_requested=$(<"$stop")
[[ $stop_requested =~ ^(0|[1-9][0-9]*)$ ]] || fail 'stop marker is not canonical'
ordinal_three_started=$(awk -F '\t' '$1 == 3 { print $2 }' "$samples")
[[ $ordinal_three_started -eq $((origin + 150000000)) &&
  $stop_requested -eq $((origin + 300000000)) ]] ||
  fail 'pre-stop drain or canonical stop timestamp differs'
awk -F '\t' -v origin="$origin" -v stop="$stop_requested" '
  NR > 1 {
    if (previous != "" && ($2 <= previous || $2 - previous > 100000000)) bad = 1
    previous = $2
  }
  NR > 1 && $1 == 1 { delayed = $3 }
  NR > 1 && $1 == 2 { delayed_two = $3 }
  NR > 1 && $1 == 3 { stopped = $3 }
  NR > 1 && $1 == 5 { bridge = $3 }
  NR > 1 && $5 == "scheduled" && $2 > scheduled_max { scheduled_max = $2 }
  NR > 1 && $5 == "terminal" { terminal_count += 1; terminal = $2 }
  END {
    exit !(bad == 0 && delayed == 275000000 && delayed_two == 100000000 &&
      stopped == 150000000 &&
      bridge == 250000000 && terminal_count == 1 && delayed > bridge &&
      terminal > scheduled_max && terminal >= stop)
  }
' "$samples" || fail 'drain bridges or terminal ordering differ'

# A scripted monotonic clock makes the four-walk backpressure deadline exact
# and proves that a fixed polling-iteration count cannot define or lengthen it.
# Four workers retain their starts and remain live; the fifth nominal launch
# must time out and abort the dedicated session without publishing a ledger.
deadline_samples=$temporary/deadline-samples.tsv
deadline_stop=$temporary/deadline.stop
deadline_state=$temporary/deadline-state
deadline_clock=$temporary/deadline-clock
deadline_trace=$temporary/deadline-trace
deadline_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$deadline_samples"
mkdir "$deadline_state"
printf '1000000000\n' >"$deadline_clock"
: >"$deadline_trace"
deadline_wall_start=$(date +%s%N)
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  deadline_clock=$7
  deadline_trace=$8
  r2_monotonic_now_ns() {
    local current caller=${FUNCNAME[1]:-}
    if [[ $caller != wait_for_launch_slot ]]; then
      R2_MONOTONIC_NS=1000000000
      return 0
    fi
    current=$(<"$deadline_clock")
    [[ $current =~ ^(0|[1-9][0-9]*)$ ]] || return 2
    printf "%s\n" "$current" >>"$deadline_trace"
    printf "%s\n" "$((current + 1000000000))" >"$deadline_clock"
    R2_MONOTONIC_NS=$current
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5 started
    started=$((origin + ordinal * 50000000))
    printf "%s\t%s\t%s\t17\t%s\n" \
      "$ordinal" "$started" "$((started - origin))" "$kind" >>"$raw"
    while :; do sleep 30; done
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-deadline-sampler "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$deadline_samples" "$deadline_stop" "$deadline_state" \
  "$deadline_origin" "$deadline_clock" "$deadline_trace" \
  >"$temporary/deadline-controller.stdout" \
  2>"$temporary/deadline-controller.stderr" &
deadline_sampler_pid=$!
deadline_session=$deadline_sampler_pid
set +e
wait "$deadline_sampler_pid" 2>"$temporary/deadline-wait.stderr"
deadline_status=$?
set -e
deadline_wall_end=$(date +%s%N)
deadline_trace_text=$(paste -sd, "$deadline_trace")
[[ $deadline_status -eq 137 && $deadline_trace_text == \
  '1000000000,2000000000,3000000000,4000000000,5000000000' &&
  $((deadline_wall_end - deadline_wall_start)) -lt 2000000000 &&
  $(wc -l <"$deadline_samples") -eq 1 ]] ||
  fail 'scripted four-walk deadline was extended or published a ledger'
[[ $(grep -Fxc \
  'R2 disk sampler: four concurrent du walks did not make room before timeout' \
  "$temporary/deadline-controller.stderr") -eq 1 ]] ||
  fail 'scripted four-walk timeout diagnostic differs'
if r2_sampler_session_has_members "$deadline_session"; then
  fail 'scripted worker-set deadline left a live session member'
else
  deadline_session_status=$?
fi
[[ $deadline_session_status -eq 1 ]] ||
  fail 'scripted four-walk session closure could not be proved'
deadline_sampler_pid=

# A sampler with one live worker that never acknowledges the drain must be
# killed as its exact dedicated session before either a ledger or terminal row
# can be published. Depending on live host scheduling, the shared absolute
# drain deadline is observed either at a scheduled-launch boundary or while
# waiting for the request-time sample set; the synthetic slot plant pins the
# former path independently.
hung_samples=$temporary/hung-samples.tsv
hung_stop=$temporary/hung.stop
hung_state=$temporary/hung-state
hung_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$hung_samples"
mkdir "$hung_state"
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  r2_record_checkout_mib() {
    local raw=$2 origin=$3 ordinal=$4 kind=$5 started
    if [[ $ordinal -eq 1 ]]; then
      while :; do sleep 30; done
    fi
    started=$(date +%s%N)
    printf "%s\t%s\t%s\t17\t%s\n" \
      "$ordinal" "$started" "$((started - origin))" "$kind" >>"$raw"
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-hung-sampler "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$hung_samples" "$hung_stop" "$hung_state" "$hung_origin" \
  >"$temporary/hung-controller.stdout" 2>"$temporary/hung-controller.stderr" &
hung_sampler_pid=$!
hung_sampler_start=
for ((attempt = 0; attempt < 200; attempt += 1)); do
  if r2_read_process_stat "/proc/$hung_sampler_pid/stat" &&
    [[ $R2_PROC_GROUP == "$hung_sampler_pid" &&
      $R2_PROC_SESSION == "$hung_sampler_pid" ]]; then
    hung_sampler_start=$R2_PROC_START
    break
  fi
  sleep 0.001
done
[[ $hung_sampler_start =~ ^[0-9]+$ ]] || fail 'hung sampler identity was not stable'
for ((attempt = 0; attempt < 200; attempt += 1)); do
  [[ ! -e $hung_state/ready ]] || break
  kill -0 "$hung_sampler_pid" 2>/dev/null || fail 'hung sampler exited before readiness'
  sleep 0.01
done
[[ -f $hung_state/ready ]] || fail 'hung sampler did not retain its initial row'
r2_monotonic_now_ns || fail 'could not start the hung-sampler clock'
hung_started_ns=$R2_MONOTONIC_NS
set +e
r2_prepare_and_stop_disk_sampler "$hung_sampler_pid" "$hung_sampler_start" \
  "$test_controller_cpu" "$hung_stop" "$hung_state" 0 \
  >"$temporary/hung-stop.stdout" 2>"$temporary/hung-stop.stderr"
hung_status=$?
set -e
r2_monotonic_now_ns || fail 'could not stop the hung-sampler clock'
hung_elapsed_ns=$((R2_MONOTONIC_NS - hung_started_ns))
[[ $hung_status -ne 0 && $hung_elapsed_ns -ge 3000000000 &&
  $hung_elapsed_ns -le 8000000000 &&
  ! -e /proc/$hung_sampler_pid && $(wc -l <"$hung_samples") -eq 1 &&
  -f $hung_state/drain-request ]] ||
  fail 'hung sampler was accepted, leaked, published, or exceeded its bound'
hung_deadline_diagnostics=$(grep -Ec \
  '^R2 disk sampler: (drain deadline expired before scheduled launch|sample workers did not close before timeout)$' \
  "$temporary/hung-controller.stderr")
[[ $hung_deadline_diagnostics -eq 1 ]] ||
  fail 'hung sampler did not reach exactly one shared drain-deadline abort'
if grep -F 'four concurrent du walks' \
  "$temporary/hung-controller.stderr" >/dev/null; then
  fail 'single hung worker incorrectly reached the concurrency cap'
fi
if r2_sampler_session_has_members "$hung_sampler_pid"; then
  fail 'hung sampler left a live session member'
else
  hung_session_status=$?
fi
[[ $hung_session_status -eq 1 ]] ||
  fail 'hung sampler session closure could not be proved'
hung_sampler_pid=

# Readiness and wait-reaping are distinct lifecycle states. Make worker 31
# publish a malformed readiness record only after workers 0-30 have published
# valid ones, then require cleanup to explicitly wait all 32 children.
partial_samples=$temporary/partial-ready-samples.tsv
partial_state=$temporary/partial-ready-state
partial_stop=$temporary/partial-ready.stop
partial_waits=$temporary/partial-ready-waits.txt
partial_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$partial_samples"
: >"$partial_waits"
mkdir "$partial_state"
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  partial_waits=$7
  eval "$(declare -f r2_disk_pool_worker | sed \
    "1s/r2_disk_pool_worker/r2_real_disk_pool_worker/")"
  r2_disk_pool_worker() {
    [[ $1 == 31 ]] || { r2_real_disk_pool_worker "$@"; return; }
    local descriptor command ordinal kind extra
    local -a inherited_fds=()
    IFS=, read -r -a inherited_fds <<<"$5"
    for descriptor in "${inherited_fds[@]}"; do eval "exec ${descriptor}>&-"; done
    taskset -pc "$R2_DISK_WALK_CPUS" "$BASHPID" >/dev/null || return 2
    r2_read_allowed_cpu_list "/proc/$BASHPID/status" || return 2
    [[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]] || return 2
    printf "broken\t31\t%s\n" "$BASHPID"
    while IFS=$'\''\t'\'' read -r command ordinal kind extra; do
      [[ $command == stop && -z $ordinal && -z $kind && -z $extra ]] || return 2
      return 0
    done
  }
  wait() { printf "%s\n" "$1" >>"$partial_waits"; builtin wait "$@"; }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-partial-ready "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$partial_samples" "$partial_stop" "$partial_state" \
  "$partial_origin" "$partial_waits" \
  >"$temporary/partial-ready.stdout" 2>"$temporary/partial-ready.stderr" &
partial_ready_sampler_pid=$!
set +e
wait "$partial_ready_sampler_pid" 2>/dev/null
partial_status=$?
set -e
partial_wait_count=$(wc -l <"$partial_waits")
partial_unique_waits=$(sort -u "$partial_waits" | wc -l)
[[ $partial_status -ne 0 && $partial_wait_count -eq 32 &&
  $partial_unique_waits -eq 32 && $(wc -l <"$partial_samples") -eq 1 ]] ||
  fail 'partial readiness did not explicitly reap the complete worker pool'
if grep -Ev '^[1-9][0-9]*$' "$partial_waits" >/dev/null; then
  fail 'partial-readiness wait log contains a non-PID argument'
fi
if r2_sampler_session_has_members "$partial_ready_sampler_pid"; then
  fail 'partial-readiness cleanup left a live session member'
else
  partial_session_status=$?
fi
[[ $partial_session_status -eq 1 ]] ||
  fail 'partial-readiness session closure could not be proved'
partial_ready_sampler_pid=

# A failure while constructing the active-set wait deadline must abort instead
# of falling through to a stop helper that refuses `active > 0`.
wait_setup_samples=$temporary/wait-setup-samples.tsv
wait_setup_state=$temporary/wait-setup-state
wait_setup_stop=$temporary/wait-setup.stop
wait_setup_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$wait_setup_samples"
mkdir "$wait_setup_state"
r2_publish_decimal_control_marker "$wait_setup_stop" "$wait_setup_origin" ||
  fail 'could not publish wait-setup stop marker'
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  eval "$(declare -f r2_monotonic_now_ns | sed \
    "1s/r2_monotonic_now_ns/r2_real_monotonic_now_ns/")"
  r2_monotonic_now_ns() {
    [[ ${FUNCNAME[1]:-} != wait_for_sample_set ]] || return 1
    r2_real_monotonic_now_ns
  }
  r2_record_checkout_mib() {
    local started
    started=$(date +%s%N) || return 2
    printf "%s\t%s\t%s\t17\t%s\n" "$4" "$started" \
      "$((started - $3))" "$5" >>"$2"
    while :; do sleep 30; done
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-wait-setup "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$wait_setup_samples" "$wait_setup_stop" "$wait_setup_state" \
  "$wait_setup_origin" >"$temporary/wait-setup.stdout" \
  2>"$temporary/wait-setup.stderr" &
wait_setup_sampler_pid=$!
set +e
wait "$wait_setup_sampler_pid" 2>>"$temporary/wait-setup.stderr"
wait_setup_status=$?
set -e
[[ $wait_setup_status -eq 137 && $(wc -l <"$wait_setup_samples") -eq 1 ]] ||
  fail 'active-set wait setup failure returned or published evidence'
wait_setup_session_status=0
for ((attempt = 0; attempt < 5000 && wait_setup_session_status == 0; attempt += 1)); do
  if r2_sampler_session_has_members "$wait_setup_sampler_pid"; then sleep 0.001; else wait_setup_session_status=$?; fi
done
[[ $wait_setup_session_status -eq 1 ]] ||
  fail 'active-set wait setup failure did not close the sampler session'
wait_setup_sampler_pid=

# A shutdown helper failure cannot return while the ready worker pool remains
# live. Fail the first monotonic probe whose caller is `stop_worker_pool` and
# require the controller's verified process group to be killed before any
# ledger publication.
shutdown_samples=$temporary/shutdown-helper-samples.tsv
shutdown_stop=$temporary/shutdown-helper.stop
shutdown_state=$temporary/shutdown-helper-state
shutdown_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$shutdown_samples"
mkdir "$shutdown_state"
r2_publish_decimal_control_marker "$shutdown_stop" "$shutdown_origin" ||
  fail 'could not publish shutdown-helper stop marker'
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  eval "$(declare -f r2_monotonic_now_ns | sed \
    "1s/r2_monotonic_now_ns/r2_real_monotonic_now_ns/")"
  r2_monotonic_now_ns() {
    [[ ${FUNCNAME[1]:-} != stop_worker_pool ]] || return 1
    r2_real_monotonic_now_ns
  }
  r2_record_checkout_mib() {
    local started
    started=$(date +%s%N) || return 2
    printf "%s\t%s\t%s\t17\t%s\n" "$4" "$started" \
      "$((started - $3))" "$5" >>"$2"
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-shutdown-helper "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$shutdown_samples" "$shutdown_stop" "$shutdown_state" \
  "$shutdown_origin" >"$temporary/shutdown-helper.stdout" \
  2>"$temporary/shutdown-helper.stderr" &
shutdown_sampler_pid=$!
set +e
wait "$shutdown_sampler_pid" 2>>"$temporary/shutdown-helper.stderr"
shutdown_status=$?
set -e
[[ $shutdown_status -eq 137 && $(wc -l <"$shutdown_samples") -eq 1 ]] ||
  fail 'shutdown helper failure returned with live workers or published evidence'
shutdown_session_status=0
for ((attempt = 0; attempt < 5000 && shutdown_session_status == 0; attempt += 1)); do
  if r2_sampler_session_has_members "$shutdown_sampler_pid"; then sleep 0.001; else shutdown_session_status=$?; fi
done
[[ $shutdown_session_status -eq 1 ]] ||
  fail 'shutdown helper failure did not close the sampler session'
shutdown_sampler_pid=

# A live child whose captured identity differs is never passed to `wait` and
# never dropped from cleanup authority: the verified dedicated sampler group
# is aborted immediately.
capture_samples=$temporary/capture-mismatch-samples.tsv
capture_stop=$temporary/capture-mismatch.stop
capture_state=$temporary/capture-mismatch-state
capture_waits=$temporary/capture-mismatch-waits.txt
capture_child_started=$temporary/capture-mismatch-child-started
capture_origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$capture_samples"
: >"$capture_waits"
mkdir "$capture_state"
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  capture_wait_log=$7
  capture_child_started=$8
  eval "$(declare -f r2_read_process_stat | sed \
    "1s/r2_read_process_stat/r2_read_process_stat_original/")"
  r2_read_process_stat() {
    r2_read_process_stat_original "$@" || return
    if [[ $1 != "/proc/$BASHPID/stat" && $R2_PROC_PARENT == "$BASHPID" &&
      -f $capture_child_started ]]; then
      R2_PROC_PARENT=1
    fi
  }
  wait() {
    printf "%s\n" "$*" >>"$capture_wait_log"
    builtin wait "$@"
  }
  r2_record_checkout_mib() {
    : >"$capture_child_started"
    while :; do sleep 30; done
  }
  r2_sample_checkout_disk "$2" "$3" "$4" "$5" "$6" 50000000
' r2-capture-mismatch "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$capture_samples" "$capture_stop" "$capture_state" \
  "$capture_origin" "$capture_waits" "$capture_child_started" \
  >"$temporary/capture-mismatch.stdout" 2>"$temporary/capture-mismatch.stderr" &
capture_sampler_pid=$!
set +e
wait "$capture_sampler_pid" 2>/dev/null
capture_status=$?
set -e
[[ $capture_status -ne 0 && ! -e /proc/$capture_sampler_pid &&
  $(wc -l <"$capture_samples") -eq 1 && ! -s $capture_waits &&
  -f $capture_child_started ]] ||
  fail 'capture mismatch waited, leaked, or published sampler evidence'
if r2_sampler_session_has_members "$capture_sampler_pid"; then
  fail 'capture mismatch left a live session member'
else
  capture_session_status=$?
fi
[[ $capture_session_status -eq 1 ]] ||
  fail 'capture mismatch session closure could not be proved'
capture_sampler_pid=

printf 'R2_DISK_TERMINAL_ORDER_PLANT PASS\n'
printf 'retained_fixture %s\n' "${temporary#"$repo_root/"}"

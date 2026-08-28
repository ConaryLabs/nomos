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

interleaved_deadlines=()
for ordinal in 0 1 2 3 4 5; do
  r2_disk_interleaved_deadline_ns 1000000000 "$ordinal" 50000000
  interleaved_deadlines+=("$R2_DISK_DEADLINE_NS")
done
[[ ${interleaved_deadlines[*]} == \
  '1000000000 1025000000 1050000000 1075000000 1100000000 1125000000' &&
  $((interleaved_deadlines[2] - interleaved_deadlines[0])) -eq 50000000 &&
  $((interleaved_deadlines[3] - interleaved_deadlines[1])) -eq 50000000 ]] ||
  fail 'interleaved absolute 50 ms phase deadlines differ'
if r2_disk_interleaved_deadline_ns 1000000000 1 50000001 ||
  r2_disk_interleaved_deadline_ns 1000000000 1 0 ||
  r2_disk_interleaved_deadline_ns 9223372036854775800 1 100 ||
  r2_disk_interleaved_deadline_ns 0 9223372036854775807 50000000; then
  fail 'an odd, zero, out-of-range, or overflowing interleaved period was accepted'
fi

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-disk-terminal-order.XXXXXX")
atomic_publisher=
handshake_pid=
deadline_sampler_pid=
hung_sampler_pid=
capture_sampler_pid=
cleanup() {
  local session_pid
  [[ -z ${atomic_publisher:-} ]] || kill "$atomic_publisher" 2>/dev/null || true
  [[ -z ${atomic_publisher:-} ]] || wait "$atomic_publisher" 2>/dev/null || true
  for session_pid in "${handshake_pid:-}" "${deadline_sampler_pid:-}" \
    "${hung_sampler_pid:-}" \
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
[[ $R2_CONTROLLER_CPUS == 0 && $R2_DISK_CPUS == 0,1,2,6,7,8 &&
  $R2_WORKLOAD_CPUS == 3,4,5,9,10,11 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,6;1,7;2,8;3,9;4,10;5,11' &&
  $R2_DISK_PHYSICAL_GROUPS == '0,6;1,7;2,8' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '3,9;4,10;5,11' ]] ||
  fail 'canonical physical-core role split differs'
r2_validate_physical_cpu_isolation "$R2_DISK_CPUS" "$R2_WORKLOAD_CPUS" \
  "$topology_root" || fail 'canonical physical-core role split overlaps'
if r2_validate_physical_cpu_isolation 0-5 6-11 "$topology_root"; then
  fail 'the former SMT-overlapping role split was accepted'
fi
r2_partition_cpu_topology '1,3-4,7,9-10' "$topology_root" ||
  fail 'irregular complete sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 1 && $R2_DISK_CPUS == 1,7 &&
  $R2_WORKLOAD_CPUS == 3,4,9,10 ]] || fail 'irregular physical-core split differs'
if r2_partition_cpu_topology 0,6 "$topology_root" ||
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
unset R2_DISK_STOP_REQUESTED_NS
r2_prepare_and_stop_disk_sampler "$handshake_pid" "$handshake_start" \
  "$test_controller_cpu" "$handshake_stop" "$handshake_state" 0 ||
  fail 'parent-side drain handshake was refused'
[[ ${R2_DISK_STOP_REQUESTED_NS:-} =~ ^(0|[1-9][0-9]*)$ &&
  $(<"$handshake_stop") == "$R2_DISK_STOP_REQUESTED_NS" &&
  $(<"$handshake_state/drain-request") == $(<"$handshake_state/drain-ready") &&
  ! -e /proc/$handshake_pid ]] || fail 'drain handshake did not precede stop and closure'
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
# synchronous acknowledgement fails this plant. The two absolute 50 ms phases
# remain +0/+25/+50/+75 ms.
absolute_samples=$temporary/absolute-samples.tsv
absolute_stop=$temporary/absolute-stop
absolute_state=$temporary/absolute-state
absolute_clock=$temporary/absolute-clock
absolute_trace=$temporary/absolute-trace
absolute_origin=1000000000
printf '%s\n' "$absolute_origin" >"$absolute_clock"
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$absolute_samples"
mkdir "$absolute_state"
(
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
    started=$((sampler_origin + ordinal * 25000000))
    printf '%s\t%s\t%s\n' "$ordinal" "$kind" "$started" >>"$absolute_trace"
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    [[ $ordinal -ne 3 ]] || r2_publish_decimal_control_marker "$absolute_stop" "$started"
  }
  r2_sample_checkout_disk "$repo_root" "$absolute_samples" "$absolute_stop" \
    "$absolute_state" "$absolute_origin" 50000000
) || fail 'deterministic absolute-schedule controller trace was refused'
[[ $(sort -n -t $'\t' -k1,1 "$absolute_trace" | sed -n '1,4p') == \
  $'0\tscheduled\t1000000000\n1\tscheduled\t1025000000\n2\tscheduled\t1050000000\n3\tscheduled\t1075000000' &&
  $(awk -F '\t' 'END { print $2 }' "$absolute_trace") == terminal &&
  $(<"$absolute_stop") == 1075000000 && ! -e $absolute_state ]] ||
  fail 'controller launches are not on the interleaved absolute 50 ms phases'

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
    [[ $(<"$state/drain-ready") == $((origin + 80000000)) ]] || exit 2
    r2_publish_decimal_control_marker "$stop" "$((origin + 150000000))"
  ) &
  stop_coordinator=$!
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 started
    started=$((sampler_origin + ordinal * 25000000))
    if [[ $ordinal -eq 1 ]]; then
      while [[ ! -e $state/release-original ]]; do command sleep 0.001; done
      find "$state/release-original" -delete
      started=$((sampler_origin + 137500000))
    elif [[ $ordinal -eq 2 ]]; then
      started=$((sampler_origin + 112500000))
    fi
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    if [[ $ordinal -eq 3 ]]; then
      r2_publish_decimal_control_marker \
        "$state/drain-request" "$((sampler_origin + 80000000))"
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
[[ $ordinal_three_started -eq $((origin + 75000000)) &&
  $stop_requested -eq $((origin + 150000000)) ]] ||
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
    exit !(bad == 0 && delayed == 137500000 && delayed_two == 112500000 &&
      stopped == 75000000 &&
      bridge == 125000000 && terminal_count == 1 && delayed > bridge &&
      terminal > scheduled_max && terminal >= stop)
  }
' "$samples" || fail 'drain bridges or terminal ordering differ'

# A scripted monotonic clock makes the worker-set deadline exact and proves
# that a fixed polling-iteration count cannot define or lengthen it. The
# initial worker retains its row and then stays live; a pre-existing stop drives
# the controller directly into the bounded reap without involving parent timing.
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
r2_publish_decimal_control_marker "$deadline_stop" "$deadline_origin" ||
  fail 'could not publish scripted-deadline stop marker'
deadline_wall_start=$(date +%s%N)
setsid taskset -c "$test_controller_cpu" bash -c '
  set -euo pipefail
  source "$1"
  deadline_clock=$7
  deadline_trace=$8
  r2_monotonic_now_ns() {
    local current
    current=$(<"$deadline_clock")
    [[ $current =~ ^(0|[1-9][0-9]*)$ ]] || return 2
    printf "%s\n" "$current" >>"$deadline_trace"
    printf "%s\n" "$((current + 1000000000))" >"$deadline_clock"
    R2_MONOTONIC_NS=$current
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5 started
    started=$(date +%s%N)
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
deadline_sampler_pid=
deadline_wall_end=$(date +%s%N)
deadline_trace_text=$(paste -sd, "$deadline_trace")
[[ $deadline_status -ne 0 && $deadline_trace_text == \
  '1000000000,2000000000,3000000000,4000000000,5000000000' &&
  $((deadline_wall_end - deadline_wall_start)) -lt 2000000000 &&
  $(wc -l <"$deadline_samples") -eq 1 ]] ||
  fail 'scripted worker-set deadline was extended or published a ledger'
[[ $(grep -Fxc 'R2 disk sampler: sample workers did not close before timeout' \
  "$temporary/deadline-controller.stderr") -eq 1 ]] ||
  fail 'scripted worker-set timeout diagnostic differs'
if grep -F 'thirty-two concurrent du walks' \
  "$temporary/deadline-controller.stderr" >/dev/null; then
  fail 'scripted worker-set deadline reached the concurrency cap'
fi
if r2_sampler_session_has_members "$deadline_session"; then
  fail 'scripted worker-set deadline left a live session member'
else
  deadline_session_status=$?
fi
[[ $deadline_session_status -eq 1 ]] ||
  fail 'scripted worker-set session closure could not be proved'

# A sampler with one live worker that never acknowledges the drain must be
# killed as its exact dedicated session before either a ledger or terminal row
# can be published. Later bridge workers complete, so this reaches the absolute
# drain deadline without relying on the separately planted concurrency cap.
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
grep -Fx 'R2 disk sampler: sample workers did not close before timeout' \
  "$temporary/hung-controller.stderr" >/dev/null ||
  fail 'hung sampler did not reach its bounded worker abort'
if grep -F 'thirty-two concurrent du walks' \
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
    local attempt
    r2_read_process_stat_original "$@" || return
    if [[ $1 != "/proc/$BASHPID/stat" && $R2_PROC_PARENT == "$BASHPID" ]]; then
      for ((attempt = 0; attempt < 100; attempt += 1)); do
        [[ ! -e $capture_child_started ]] || break
        sleep 0.001
      done
      [[ -f $capture_child_started ]] || return 2
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
fi
capture_sampler_pid=

printf 'R2_DISK_TERMINAL_ORDER_PLANT PASS\n'
printf 'retained_fixture %s\n' "${temporary#"$repo_root/"}"

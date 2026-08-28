# shellcheck shell=bash
# shellcheck disable=SC2154 # This source-only plant shares its parent suite.

# Saturate the real worker gate, publish stop from the gate's first monotonic
# probe, release the held walks, and wait until a completion is collectible.
# This deterministically places stop between the caller's absence check and a
# newly available slot. Only the three already-dispatched scheduled walks and
# the required terminal walk may reach the retained ledger.
slot_stop_samples=$temporary/slot-stop-samples.tsv
slot_stop_state=$temporary/slot-stop-state
slot_stop_marker=$temporary/slot-stop.marker
slot_stop_release=$temporary/slot-stop.release
slot_stop_selection=$temporary/slot-stop.selection
slot_stop_origin=1000000000
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$slot_stop_samples"
mkdir "$slot_stop_state"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  bash -c '
  set -euo pipefail
  source "$1"
  root=$2
  samples=$3
  stop=$4
  state=$5
  origin=$6
  release=$7
  selection=$8
  raw=$state/samples.unsorted.tsv
  injected=0
  eval "$(declare -f r2_read_allowed_cpu_list | sed \
    "1s/r2_read_allowed_cpu_list/r2_real_read_allowed_cpu_list/")"
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    printf "%s\n" "$((origin + 1000000000))"
  }
  r2_monotonic_now_ns() {
    if [[ ${FUNCNAME[1]:-} == wait_for_launch_slot && $injected -eq 0 ]]; then
      injected=1
      r2_publish_decimal_control_marker "$stop" "$((origin + 100000000))"
      : >"$release"
      for ((attempt = 0; attempt < 1000; attempt += 1)); do
        [[ $(wc -l <"$raw") -gt 1 ]] && break
        command sleep 0.001
      done
      [[ $(wc -l <"$raw") -gt 1 ]] || return 1
    fi
    R2_MONOTONIC_NS=1000000000
  }
  r2_read_allowed_cpu_list() {
    if [[ ${FUNCNAME[1]:-} == pool_worker_identity_stable &&
      ${FUNCNAME[2]:-} == find_available_worker &&
      ${launch_kind:-} == scheduled && $ordinal -eq 3 ]]; then
      : >"$selection"
      return 2
    fi
    r2_real_read_allowed_cpu_list "$@"
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw_output=$2 sample_origin=$3 ordinal=$4 kind=$5 started
    if [[ $kind == scheduled && $ordinal -lt 3 ]]; then
      while [[ ! -e $release ]]; do command sleep 0.001; done
    fi
    started=$((sample_origin + ordinal * 50000000))
    printf "%s\t%s\t%s\t17\t%s\n" "$ordinal" "$started" \
      "$((started - sample_origin))" "$kind" >>"$raw_output"
  }
  r2_sample_checkout_disk "$root" "$samples" "$stop" "$state" \
    "$origin" 50000000
' r2-slot-stop "$harness_lib_source" "$repo_root" "$slot_stop_samples" \
  "$slot_stop_marker" "$slot_stop_state" "$slot_stop_origin" \
  "$slot_stop_release" "$slot_stop_selection" >"$temporary/slot-stop.stdout" \
  2>"$temporary/slot-stop.stderr" &
stop_test_pid=$!
if wait "$stop_test_pid" 2>"$temporary/slot-stop-wait.stderr"; then
  slot_stop_status=0
else
  slot_stop_status=$?
fi
slot_stop_session=$stop_test_pid
slot_stop_scheduled=$(awk -F '\t' '$5 == "scheduled" { count += 1 } END { print count + 0 }' \
  "$slot_stop_samples")
[[ $slot_stop_status -eq 0 && ! -s $temporary/slot-stop.stdout &&
  ! -s $temporary/slot-stop.stderr && ! -e $slot_stop_state &&
  ! -e $slot_stop_selection &&
  $slot_stop_scheduled -eq 3 &&
  $(tail -n 1 "$slot_stop_samples") == $'3\t1150000000\t150000000\t17\tterminal' ]] ||
  fail 'a scheduled walk crossed the stop boundary while a slot became free'
if r2_sampler_session_has_members "$slot_stop_session"; then
  fail 'stop-boundary slot plant left a live session member'
else
  slot_stop_session_status=$?
fi
[[ $slot_stop_session_status -eq 1 ]] ||
  fail 'stop-boundary slot session closure could not be proved'
stop_test_pid=
plant_count=$((plant_count + 1))

# The gate-free path still performs worker identity selection before dispatch.
# Publish stop from that exact selection on ordinal one; the final boundary
# check must refuse the scheduled request while preserving ordinal zero and the
# required terminal row.
dispatch_stop_samples=$temporary/dispatch-stop-samples.tsv
dispatch_stop_state=$temporary/dispatch-stop-state
dispatch_stop_marker=$temporary/dispatch-stop.marker
dispatch_stop_origin=1000000000
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' \
  >"$dispatch_stop_samples"
mkdir "$dispatch_stop_state"
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  bash -c '
  set -euo pipefail
  source "$1"
  root=$2
  samples=$3
  stop=$4
  state=$5
  origin=$6
  eval "$(declare -f r2_read_allowed_cpu_list | sed \
    "1s/r2_read_allowed_cpu_list/r2_real_read_allowed_cpu_list/")"
  injected=0
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    printf "%s\n" "$((origin + 1000000000))"
  }
  r2_monotonic_now_ns() { R2_MONOTONIC_NS=1000000000; }
  r2_read_allowed_cpu_list() {
    r2_real_read_allowed_cpu_list "$@" || return
    if [[ ${FUNCNAME[1]:-} == pool_worker_identity_stable &&
      ${FUNCNAME[2]:-} == find_available_worker && $ordinal -eq 1 &&
      $injected -eq 0 ]]; then
      injected=1
      r2_publish_decimal_control_marker "$stop" "$((origin + 25000000))"
    fi
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    local raw_output=$2 sample_origin=$3 ordinal=$4 kind=$5 started
    started=$((sample_origin + ordinal * 50000000))
    printf "%s\t%s\t%s\t17\t%s\n" "$ordinal" "$started" \
      "$((started - sample_origin))" "$kind" >>"$raw_output"
  }
  r2_sample_checkout_disk "$root" "$samples" "$stop" "$state" \
    "$origin" 50000000
' r2-dispatch-stop "$harness_lib_source" "$repo_root" \
  "$dispatch_stop_samples" "$dispatch_stop_marker" "$dispatch_stop_state" \
  "$dispatch_stop_origin" >"$temporary/dispatch-stop.stdout" \
  2>"$temporary/dispatch-stop.stderr" &
stop_test_pid=$!
if wait "$stop_test_pid" 2>"$temporary/dispatch-stop-wait.stderr"; then
  dispatch_stop_status=0
else
  dispatch_stop_status=$?
fi
dispatch_stop_session=$stop_test_pid
[[ $dispatch_stop_status -eq 0 && ! -s $temporary/dispatch-stop.stdout &&
  ! -s $temporary/dispatch-stop.stderr && ! -e $dispatch_stop_state &&
  $(sed -n '2,3p' "$dispatch_stop_samples") == \
    $'0\t1000000000\t0\t17\tscheduled\n1\t1050000000\t50000000\t17\tterminal' &&
  $(wc -l <"$dispatch_stop_samples") -eq 3 ]] ||
  fail 'the final scheduled-dispatch boundary ignored a new stop marker'
if r2_sampler_session_has_members "$dispatch_stop_session"; then
  fail 'final dispatch-boundary plant left a live session member'
else
  dispatch_stop_session_status=$?
fi
[[ $dispatch_stop_session_status -eq 1 ]] ||
  fail 'final dispatch-boundary session closure could not be proved'
stop_test_pid=
plant_count=$((plant_count + 1))

# Begin a drain with live work, fill the three-walk gate, and advance only gate
# monotonic time. The original drain deadline is one second earlier than the
# gate's independently derived deadline, so the shared deadline must abort the
# session first and no fourth walk or ledger row may be published.
drain_slot_samples=$temporary/drain-slot-samples.tsv
drain_slot_state=$temporary/drain-slot-state
drain_slot_stop=$temporary/drain-slot.stop
drain_slot_trace=$temporary/drain-slot-trace
drain_slot_origin=1000000000
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$drain_slot_samples"
mkdir "$drain_slot_state"
: >"$drain_slot_trace"
set +e
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_DISK_WALK_CPUS="$async_walk_cpus" \
  bash -c '
  set -euo pipefail
  source "$1"
  root=$2
  samples=$3
  stop=$4
  state=$5
  origin=$6
  trace=$7
  eval "$(declare -f r2_disk_deadline_ns | sed \
    "1s/r2_disk_deadline_ns/r2_real_disk_deadline_ns/")"
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    printf "%s\n" "$((origin + 1000000000))"
  }
  r2_disk_deadline_ns() {
    if [[ ${FUNCNAME[1]:-} == r2_sample_checkout_disk &&
      $1 == "$origin" && $2 == 1 && $3 == 50000000 ]]; then
      for ((attempt = 0; attempt < 1000; attempt += 1)); do
        [[ -e $state/drain-request ]] && break
        command sleep 0.001
      done
      [[ -e $state/drain-request ]] || return 1
    fi
    r2_real_disk_deadline_ns "$@"
  }
  gate_probe=1
  r2_monotonic_now_ns() {
    if [[ ${FUNCNAME[1]:-} == wait_for_launch_slot ||
      ${FUNCNAME[2]:-} == wait_for_launch_slot ]]; then
      gate_probe=$((gate_probe + 1))
      R2_MONOTONIC_NS=$((gate_probe * 1000000000))
      printf "%s\n" "$R2_MONOTONIC_NS" >>"$trace"
    else
      R2_MONOTONIC_NS=1000000000
    fi
  }
  r2_record_checkout_mib() {
    [[ $# -eq 5 ]] || return 2
    if [[ $4 -eq 0 ]]; then
      r2_publish_decimal_control_marker \
        "$state/drain-request" "$((origin + 25000000))"
    fi
    while :; do command sleep 30; done
  }
  r2_sample_checkout_disk "$root" "$samples" "$stop" "$state" \
    "$origin" 50000000
' r2-drain-slot "$harness_lib_source" "$repo_root" "$drain_slot_samples" \
  "$drain_slot_stop" "$drain_slot_state" "$drain_slot_origin" \
  "$drain_slot_trace" >"$temporary/drain-slot.stdout" \
  2>"$temporary/drain-slot.stderr" &
stop_test_pid=$!
wait "$stop_test_pid" 2>"$temporary/drain-slot-wait.stderr"
drain_slot_status=$?
drain_slot_session=$stop_test_pid
set -e
[[ $drain_slot_status -eq 137 && ! -s $temporary/drain-slot.stdout &&
  $(wc -l <"$drain_slot_samples") -eq 1 &&
  $(paste -sd, "$drain_slot_trace") == '2000000000,3000000000,4000000000,5000000000' ]] ||
  fail 'the saturated launch gate outlived its shared drain deadline'
[[ $(grep -Fxc \
  'R2 disk sampler: drain deadline expired before scheduled launch' \
  "$temporary/drain-slot.stderr") -eq 1 ]] ||
  fail 'saturated drain-slot deadline diagnostic differs'
if r2_sampler_session_has_members "$drain_slot_session"; then
  fail 'drain-slot deadline plant left a live session member'
else
  drain_slot_session_status=$?
fi
[[ $drain_slot_session_status -eq 1 ]] ||
  fail 'drain-slot deadline session closure could not be proved'
stop_test_pid=
plant_count=$((plant_count + 1))

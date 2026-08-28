# shellcheck shell=bash
# shellcheck disable=SC2154 # This source-only plant shares its parent suite.

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'R2 disk overload plant: source this file from the parent suite\n' >&2
  exit 2
fi

overload_samples=$temporary/overload-disk-samples.tsv
overload_state=$temporary/overload-disk-state
overload_stop=$temporary/overload-disk.stop
overload_launches=$temporary/overload-disk-launches
overload_hold=$temporary/overload-disk.release
overload_started=$(date +%s%N)
overload_cpus=${disk_test_cpu_array[0]},${disk_test_cpu_array[1]},${disk_test_cpu_array[2]}
overload_lanes="${disk_test_cpu_array[0]};${disk_test_cpu_array[1]};${disk_test_cpu_array[2]}"
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$overload_samples"
mkdir "$overload_state" "$overload_launches"
# Hold exact fake walks and advance only the launch-slot clock. All 32 pool
# workers still bind normally; three walks must occupy lanes 1/1/1 and no
# fourth exact walk may begin.
set +e
setsid taskset -c "$disk_test_controller_cpus" env \
  R2_TEST_DU_STABLE=1 \
  R2_TEST_DU_HOLD_MARKER="$overload_hold" \
  R2_TEST_DU_AFFINITY_DIRECTORY="$overload_launches" \
  R2_DISK_WALK_CPUS="$overload_cpus" \
  R2_DISK_WALK_GROUPS="$overload_lanes" \
  PATH="$fake_disk_bin:$PATH" \
  bash -c '
  set -euo pipefail
  source "$1"
  eval "$(declare -f r2_monotonic_now_ns | sed \
    "1s/r2_monotonic_now_ns/r2_real_monotonic_now_ns/")"
  gate_probe=0
  r2_monotonic_now_ns() {
    if [[ ${FUNCNAME[1]:-} == wait_for_launch_slot ]]; then
      gate_probe=$((gate_probe + 1))
      R2_MONOTONIC_NS=$((gate_probe * 1000000000))
      return 0
    fi
    r2_real_monotonic_now_ns
  }
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
overload_lane_counts=$(awk -F '\t' -v a="${disk_test_cpu_array[0]}" \
  -v b="${disk_test_cpu_array[1]}" -v c="${disk_test_cpu_array[2]}" \
  '{ if ($5 != $9) bad += 1; if ($5 == a) x += 1; else if ($5 == b) y += 1; else if ($5 == c) z += 1; else bad += 1 } END { printf "%d,%d,%d,%d", x, y, z, bad }' \
  "$overload_launches"/*)
[[ $overload_status -eq 137 && ! -s $temporary/overload.stdout &&
  $(wc -l <"$overload_samples") -eq 1 && $overload_launch_count -eq 3 &&
  $overload_lane_counts == 1,1,1,0 ]] ||
  fail 'asynchronous sampler did not balance or bound concurrent walks'
[[ $(grep -Fxc \
  'R2 disk sampler: three concurrent du walks did not make room before timeout' \
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

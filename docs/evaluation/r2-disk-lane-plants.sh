# shellcheck shell=bash
# shellcheck disable=SC2154 # This source-only plant shares its parent suite.

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'R2 disk lane plants: source this file from the parent suite\n' >&2
  exit 2
fi

lane_saved_cpus=$R2_DISK_WALK_CPUS
lane_saved_groups=$R2_DISK_WALK_GROUPS
R2_DISK_WALK_CPUS=1,2,3,7,8,9
R2_DISK_WALK_GROUPS='1,7;2,8;3,9'
r2_configure_disk_walk_lanes || fail 'canonical disk-worker lanes were refused'
[[ $R2_DISK_LANE_COUNT -eq 3 &&
  ${R2_DISK_LANE_MASKS[0]} == 1,7 &&
  ${R2_DISK_LANE_MASKS[1]} == 2,8 &&
  ${R2_DISK_LANE_MASKS[2]} == 3,9 ]] ||
  fail 'canonical disk-worker lane masks differ'

if (R2_DISK_WALK_CPUS=1,2,3; R2_DISK_WALK_GROUPS='1,2;2,3';
  r2_configure_disk_walk_lanes); then
  fail 'overlapping disk-worker lanes were accepted'
fi
if (R2_DISK_WALK_CPUS=1,2,3; R2_DISK_WALK_GROUPS='1;2';
  r2_configure_disk_walk_lanes); then
  fail 'an incomplete disk-worker lane union was accepted'
fi
if (R2_DISK_WALK_CPUS=1,2,3; R2_DISK_WALK_GROUPS='1,4;2,3';
  r2_configure_disk_walk_lanes); then
  fail 'a disk-worker lane outside the combined mask was accepted'
fi
if (R2_DISK_WALK_CPUS=1,2,3; R2_DISK_WALK_GROUPS='1-2;3';
  r2_configure_disk_walk_lanes); then
  fail 'a noncanonical disk-worker lane was accepted'
fi
if (R2_DISK_WALK_CPUS=1,2,3; R2_DISK_WALK_GROUPS='1;2;3;';
  r2_configure_disk_walk_lanes); then
  fail 'a trailing empty disk-worker lane was accepted'
fi

lane_worker_counts=(0 0 0)
for ((lane_worker_index = 0; lane_worker_index < 32; lane_worker_index += 1)); do
  r2_disk_lane_for_worker "$lane_worker_index" 3 ||
    fail 'a pool worker could not be assigned to its lane'
  lane_worker_counts[R2_DISK_WORKER_LANE]=$((
    lane_worker_counts[R2_DISK_WORKER_LANE] + 1
  ))
done
[[ ${lane_worker_counts[0]} -eq 11 && ${lane_worker_counts[1]} -eq 11 &&
  ${lane_worker_counts[2]} -eq 10 ]] || fail 'the 32-worker lane distribution differs'
if r2_disk_lane_for_worker 0 33; then
  fail 'more lanes than persistent workers were accepted'
fi

lane_active=(0 0 0)
lane_next=0
lane_trace=()
for ((lane_launch = 0; lane_launch < 4; lane_launch += 1)); do
  r2_select_disk_lane "$lane_next" 2 "${lane_active[@]}" ||
    fail 'balanced disk-lane selection refused an available lane'
  lane_selected=$R2_DISK_SELECTED_LANE
  lane_trace+=("$lane_selected")
  lane_active[lane_selected]=$((lane_active[lane_selected] + 1))
  lane_next=$(( (lane_selected + 1) % 3 ))
done
[[ ${lane_trace[*]} == '0 1 2 0' && ${lane_active[*]} == '2 1 1' ]] ||
  fail 'four active walks were not balanced two-one-one across three lanes'
lane_active[0]=$((lane_active[0] - 1))
r2_select_disk_lane "$lane_next" 2 "${lane_active[@]}" ||
  fail 'rotating least-active selection refused a released lane'
[[ $R2_DISK_SELECTED_LANE -eq 1 ]] ||
  fail 'least-active lane tie-breaking did not rotate'
if r2_select_disk_lane 0 2 2 2 2; then
  fail 'a third active walk on every saturated lane was admitted'
else
  lane_selection_status=$?
fi
[[ $lane_selection_status -eq 1 ]] ||
  fail 'fully saturated disk lanes did not report capacity exhaustion'
if r2_select_disk_lane 0 2 3 0 0; then
  fail 'an over-capacity disk-lane counter was accepted'
else
  lane_selection_status=$?
fi
[[ $lane_selection_status -eq 2 ]] ||
  fail 'an invalid disk-lane counter did not fail closed'

IFS=, read -r -a lane_test_cpus <<<"$lane_saved_cpus"
[[ ${#lane_test_cpus[@]} -ge 2 ]] || fail 'lane-affinity plant needs two allowed CPUs'
lane_actual_cpu=${lane_test_cpus[1]}
taskset -c "$lane_actual_cpu" sleep 30 &
lane_affinity_pid=$!
lane_affinity_ready=0
for ((lane_attempt = 0; lane_attempt < 100; lane_attempt += 1)); do
  if r2_disk_worker_affinity_stable "$lane_affinity_pid" "$lane_actual_cpu"; then
    lane_affinity_ready=1
    break
  fi
  kill -0 "$lane_affinity_pid" 2>/dev/null || break
  sleep 0.001
done
[[ $lane_affinity_ready -eq 1 ]] || fail 'an exact worker lane could not be verified'
if r2_disk_worker_affinity_stable "$lane_affinity_pid" "${lane_test_cpus[0]}" ||
  r2_disk_worker_affinity_stable "$lane_affinity_pid" "$lane_saved_cpus"; then
  fail 'worker affinity drift passed through another lane or the aggregate mask'
fi
kill "$lane_affinity_pid"
wait "$lane_affinity_pid" 2>/dev/null || true
lane_affinity_pid=

R2_DISK_WALK_CPUS=$lane_saved_cpus
R2_DISK_WALK_GROUPS=$lane_saved_groups
r2_configure_disk_walk_lanes || fail 'the parent disk-worker lane was not restored'

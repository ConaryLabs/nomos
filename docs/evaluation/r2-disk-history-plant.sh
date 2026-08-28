# shellcheck shell=bash
# This source-only fixture consumes the parent suite's paths, lifecycle PID,
# fail helper, and sourced R2 disk primitives.
# shellcheck disable=SC2154,SC2329

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'R2 disk history plant: source this file from the parent suite\n' >&2
  exit 2
fi

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
setsid bash -c '
  set -euo pipefail
  source "$1"
  history_probes=$7
  eval "$(declare -f r2_read_process_stat | sed \
    "1s/r2_read_process_stat/r2_read_process_stat_unprobed/")"
  r2_read_process_stat() {
    printf "probe\n" >>"$history_probes"
    r2_read_process_stat_unprobed "$@"
  }
  r2_record_checkout_mib() {
    [[ $# -eq 6 ]] || return 2
    local raw=$2 origin=$3 ordinal=$4 kind=$5
    local started=$((origin + ordinal * 10000000))
    printf "%s\t%s\t%s\t17\t%s\n" \
      "$ordinal" "$started" "$((started - origin))" "$kind" >>"$raw"
  }
  r2_sample_checkout_disk \
    "$2" "$3" "$4" "$5" "$6" 10000000
' r2-history-sampler "$script_directory/r2-complete-proof-lib.sh" \
  "$repo_root" "$history_samples" "$history_stop" "$history_parts" \
  "$history_started" "$history_probes" &
history_sampler_pid=$!
history_sampler_bound=0
for ((attempt = 0; attempt < 100; attempt += 1)); do
  if r2_read_process_stat "/proc/$history_sampler_pid/stat" &&
    [[ $R2_PROC_GROUP == "$history_sampler_pid" &&
      $R2_PROC_SESSION == "$history_sampler_pid" && $R2_PROC_STATE != Z ]]; then
    history_sampler_bound=1
    break
  fi
  kill -0 "$history_sampler_pid" 2>/dev/null || break
  sleep 0.001
done
[[ $history_sampler_bound -eq 1 ]] ||
  fail 'history sampler did not own its dedicated session'
r2_monotonic_now_ns || fail 'could not start history-sampler readiness deadline'
r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 12000000000 ||
  fail 'history-sampler readiness deadline overflowed'
history_ready_deadline=$R2_DISK_DEADLINE_NS
while :; do
  [[ ! -e $history_parts/ready ]] || break
  kill -0 "$history_sampler_pid" 2>/dev/null || fail 'history sampler exited before readiness'
  r2_monotonic_now_ns || fail 'history-sampler readiness clock failed'
  [[ $R2_MONOTONIC_NS -lt $history_ready_deadline ]] || break
  sleep 0.01
done
[[ -f $history_parts/ready ]] || fail 'history sampler did not become ready'
r2_monotonic_now_ns || fail 'could not start history-sampler row deadline'
r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 8000000000 ||
  fail 'history-sampler row deadline overflowed'
history_row_deadline=$R2_DISK_DEADLINE_NS
while :; do
  history_live_count=$(wc -l <"$history_parts/samples.unsorted.tsv")
  [[ $history_live_count -lt 40 ]] || break
  kill -0 "$history_sampler_pid" 2>/dev/null ||
    fail 'history sampler exited before its fortieth raw row'
  r2_monotonic_now_ns || fail 'history-sampler row clock failed'
  [[ $R2_MONOTONIC_NS -lt $history_row_deadline ]] || break
  sleep 0.005
done
[[ $history_live_count -ge 40 ]] ||
  fail 'history sampler did not retain forty rows before its deadline'
r2_publish_decimal_control_marker "$history_stop" "$(date +%s%N)" ||
  fail 'could not publish history-sampler stop marker'
wait "$history_sampler_pid" || fail 'history sampler rejected quick complete walks'
if r2_sampler_session_has_members "$history_sampler_pid"; then
  fail 'history sampler left a live session member'
else
  history_session_status=$?
fi
[[ $history_session_status -eq 1 ]] ||
  fail 'history sampler session closure could not be proved'
history_count=$(awk 'NR > 1 { count += 1 } END { print count + 0 }' "$history_samples")
history_probe_count=$(wc -l <"$history_probes")
# Deterministic quick workers need only a small active set; sixteen probes per
# launch is deliberately generous but still rejects O(n²) history.
[[ $history_count -ge 40 && $history_probe_count -gt 0 &&
  $history_probe_count -le $(((history_count + 1) * 16)) &&
  ! -e $history_parts ]] ||
  fail 'disk sampler retained historical jobs instead of the bounded active set'
history_sampler_pid=

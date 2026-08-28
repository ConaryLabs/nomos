# shellcheck shell=bash
# Source-only deterministic plants for the bounded procfs snapshot readers.
# The parent terminal-order suite supplies `temporary`, `fail`, and the sourced
# complete-proof library.
# shellcheck disable=SC2154

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'R2 procfs read plants: source this file from the parent suite\n' >&2
  exit 2
fi

# Two empty stat snapshots are retryable. Three preserve the public absent
# result, while parsed malformation remains terminal on its first attempt.
process_stat_once_source=$(declare -f r2_read_process_stat_once)
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  [[ $process_stat_attempt -ge 3 ]] || return 1
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=2
  R2_PROC_SESSION=2
  R2_PROC_START=3
}
r2_read_process_stat /proc/self/stat ||
  fail 'transient empty procfs stat snapshots were not retried'
[[ $process_stat_attempt -eq 3 && $R2_PROC_STATE == S &&
  $R2_PROC_PARENT == 1 && $R2_PROC_GROUP == 2 &&
  $R2_PROC_SESSION == 2 && $R2_PROC_START == 3 ]] ||
  fail 'procfs stat retry count or result differs'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  return 1
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 1 && $process_stat_attempt -eq 3 ]] ||
  fail 'persistent empty procfs stat snapshots did not preserve absence'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  return 2
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 2 && $process_stat_attempt -eq 1 ]] ||
  fail 'malformed procfs stat snapshot was retried'
eval "$process_stat_once_source"
unset process_stat_once_source

# Status snapshots use the same three-attempt bound. Nonempty files without
# the canonical row, duplicate rows, and malformed values remain immediate
# status-2 failures rather than entering the empty-snapshot retry class.
read_allowed_once_source=$(declare -f r2_read_allowed_cpu_list_once)
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  [[ $affinity_read_attempt -ge 3 ]] || return 1
  R2_EXPANDED_CPU_LIST=0
}
r2_read_allowed_cpu_list /proc/self/status ||
  fail 'transient empty procfs affinity snapshots were not retried'
[[ $affinity_read_attempt -eq 3 && $R2_EXPANDED_CPU_LIST == 0 ]] ||
  fail 'procfs affinity retry count or result differs'
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  return 1
}
if r2_read_allowed_cpu_list /proc/self/status; then affinity_read_status=0
else affinity_read_status=$?; fi
[[ $affinity_read_status -eq 2 && $affinity_read_attempt -eq 3 ]] ||
  fail 'persistent empty procfs affinity snapshots did not fail closed'
eval "$read_allowed_once_source"

printf 'Name:\tmissing-cpu-row\n' >"$temporary/nonempty-status"
printf 'Cpus_allowed_list:\t0\nCpus_allowed_list:\t0\n' \
  >"$temporary/duplicate-status"
printf 'Cpus_allowed_list:\tbad\n' >"$temporary/malformed-status"
eval "$(declare -f r2_read_allowed_cpu_list_once | sed \
  '1s/r2_read_allowed_cpu_list_once/r2_read_allowed_cpu_list_once_live/')"
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  r2_read_allowed_cpu_list_once_live "$@"
}
for status_fixture in nonempty-status duplicate-status malformed-status; do
  affinity_read_attempt=0
  if r2_read_allowed_cpu_list "$temporary/$status_fixture"; then
    affinity_read_status=0
  else
    affinity_read_status=$?
  fi
  [[ $affinity_read_status -eq 2 && $affinity_read_attempt -eq 1 ]] ||
    fail "malformed CPU status fixture was retried: $status_fixture"
done
eval "$read_allowed_once_source"
unset -f r2_read_allowed_cpu_list_once_live
unset read_allowed_once_source

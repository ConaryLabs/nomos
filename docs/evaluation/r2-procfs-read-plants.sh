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
  if [[ $process_stat_attempt -lt 3 ]]; then
    R2_PROC_READ_CLASS=incomplete
    return 1
  fi
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=2
  R2_PROC_SESSION=2
  R2_PROC_START=3
  R2_PROC_READ_CLASS=ok
}
r2_read_process_stat /proc/self/stat ||
  fail 'transient empty procfs stat snapshots were not retried'
[[ $process_stat_attempt -eq 3 && $R2_PROC_READ_CLASS == ok &&
  $R2_PROC_STATE == S &&
  $R2_PROC_PARENT == 1 && $R2_PROC_GROUP == 2 &&
  $R2_PROC_SESSION == 2 && $R2_PROC_START == 3 ]] ||
  fail 'procfs stat retry count or result differs'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  R2_PROC_READ_CLASS=incomplete
  return 1
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 1 && $process_stat_attempt -eq 3 &&
  $R2_PROC_READ_CLASS == incomplete ]] ||
  fail 'persistent empty procfs stat snapshots did not preserve absence'
process_stat_attempt=0
r2_read_process_stat_once() {
  process_stat_attempt=$((process_stat_attempt + 1))
  R2_PROC_READ_CLASS=malformed
  return 2
}
if r2_read_process_stat /proc/self/stat; then process_stat_status=0
else process_stat_status=$?; fi
[[ $process_stat_status -eq 2 && $process_stat_attempt -eq 1 &&
  $R2_PROC_READ_CLASS == malformed ]] ||
  fail 'malformed procfs stat snapshot was retried'
eval "$process_stat_once_source"
unset process_stat_once_source

# Status snapshots use the same three-attempt bound. Static nonempty files
# without the canonical row, duplicate rows, and malformed values remain
# immediate status-2 failures rather than entering the live-procfs retry class.
read_allowed_once_source=$(declare -f r2_read_allowed_cpu_list_once)
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  if [[ $affinity_read_attempt -lt 3 ]]; then
    R2_PROC_READ_CLASS=incomplete
    return 1
  fi
  R2_EXPANDED_CPU_LIST=0
  R2_PROC_READ_CLASS=ok
}
r2_read_allowed_cpu_list /proc/self/status ||
  fail 'transient empty procfs affinity snapshots were not retried'
[[ $affinity_read_attempt -eq 3 && $R2_PROC_READ_CLASS == ok &&
  $R2_EXPANDED_CPU_LIST == 0 ]] ||
  fail 'procfs affinity retry count or result differs'
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  R2_PROC_READ_CLASS=incomplete
  return 1
}
if r2_read_allowed_cpu_list /proc/self/status; then affinity_read_status=0
else affinity_read_status=$?; fi
[[ $affinity_read_status -eq 2 && $affinity_read_attempt -eq 3 &&
  $R2_PROC_READ_CLASS == incomplete ]] ||
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
  [[ $affinity_read_status -eq 2 && $affinity_read_attempt -eq 1 &&
    $R2_PROC_READ_CLASS == malformed ]] ||
    fail "malformed CPU status fixture was retried: $status_fixture"
done
eval "$read_allowed_once_source"

# A nonempty live procfs status snapshot can itself be torn before the
# canonical key prefix. Missing-row content is retryable only for that live
# numeric procfs path; the static missing-row fixture above remains terminal.
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  if [[ $affinity_read_attempt -eq 1 ]]; then
    R2_PROC_READ_CLASS='missing-row'
    return 2
  fi
  R2_EXPANDED_CPU_LIST=0
  R2_PROC_READ_CLASS=ok
}
r2_read_allowed_cpu_list "/proc/$BASHPID/status" ||
  fail 'transient torn live procfs status was not retried'
[[ $affinity_read_attempt -eq 2 && $R2_PROC_READ_CLASS == ok &&
  $R2_EXPANDED_CPU_LIST == 0 ]] ||
  fail 'torn live procfs status retry count or result differs'
affinity_read_attempt=0
r2_read_allowed_cpu_list_once() {
  affinity_read_attempt=$((affinity_read_attempt + 1))
  R2_PROC_READ_CLASS='missing-row'
  return 2
}
if r2_read_allowed_cpu_list "/proc/$BASHPID/status"; then
  affinity_read_status=0
else
  affinity_read_status=$?
fi
[[ $affinity_read_status -eq 2 && $affinity_read_attempt -eq 3 &&
  $R2_PROC_READ_CLASS == incomplete ]] ||
  fail 'persistent torn live procfs status escaped its fixed retry bound'
eval "$read_allowed_once_source"
unset -f r2_read_allowed_cpu_list_once_live
unset read_allowed_once_source

# A genuinely vanished procfs path is classified separately from a live but
# incomplete snapshot and is never eligible for identity-level retry.
if r2_read_process_stat "$temporary/absent-stat"; then absent_stat_status=0
else absent_stat_status=$?; fi
[[ $absent_stat_status -eq 1 && $R2_PROC_READ_CLASS == absent ]] ||
  fail 'absent procfs stat path was classified as incomplete'
if r2_read_allowed_cpu_list "$temporary/absent-status"; then
  absent_affinity_status=0
else
  absent_affinity_status=$?
fi
[[ $absent_affinity_status -eq 2 && $R2_PROC_READ_CLASS == absent ]] ||
  fail 'absent procfs status path was classified as incomplete'

# The staged sampler identity probe exposes only a live incomplete snapshot as
# retryable status 1. Malformation, absence, tuple drift, zombie state, and
# affinity drift are definitive status 2 results with a stage-specific reason.
identity_stat_source=$(declare -f r2_read_process_stat)
identity_affinity_source=$(declare -f r2_read_allowed_cpu_list)
identity_pid=$BASHPID
identity_stat_calls=0
identity_affinity_calls=0
identity_state=S
identity_group=$identity_pid
identity_session=$identity_pid
identity_start=7
identity_affinity=0
r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  R2_PROC_STATE=$identity_state
  R2_PROC_PARENT=1
  R2_PROC_GROUP=$identity_group
  R2_PROC_SESSION=$identity_session
  R2_PROC_START=$identity_start
  R2_PROC_READ_CLASS=ok
}
r2_read_allowed_cpu_list() {
  identity_affinity_calls=$((identity_affinity_calls + 1))
  R2_EXPANDED_CPU_LIST=$identity_affinity
  R2_PROC_READ_CLASS=ok
}

for identity_mutation in start group session zombie; do
  identity_state=S
  identity_group=$identity_pid
  identity_session=$identity_pid
  identity_start=7
  case $identity_mutation in
    start) identity_start=8 ;;
    group) identity_group=1 ;;
    session) identity_session=1 ;;
    zombie) identity_state=Z ;;
  esac
  identity_stat_calls=0
  identity_affinity_calls=0
  if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
  else identity_status=$?; fi
  [[ $identity_status -eq 2 &&
    $R2_SAMPLER_IDENTITY_REASON == initial-identity-changed &&
    $identity_stat_calls -eq 1 && $identity_affinity_calls -eq 0 ]] ||
    fail "structural sampler identity mutation was not terminal: $identity_mutation"
done

r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  R2_PROC_READ_CLASS=incomplete
  return 1
}
identity_stat_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
else identity_status=$?; fi
[[ $identity_status -eq 1 && $identity_stat_calls -eq 1 &&
  $R2_SAMPLER_IDENTITY_REASON == initial-stat-incomplete ]] ||
  fail 'live incomplete initial stat was not the sole retryable identity result'
r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  R2_PROC_READ_CLASS=malformed
  return 2
}
identity_stat_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
else identity_status=$?; fi
[[ $identity_status -eq 2 && $identity_stat_calls -eq 1 &&
  $R2_SAMPLER_IDENTITY_REASON == initial-stat-malformed ]] ||
  fail 'malformed initial stat entered identity retry'

r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=$identity_pid
  R2_PROC_SESSION=$identity_pid
  R2_PROC_START=7
  R2_PROC_READ_CLASS=ok
}
r2_read_allowed_cpu_list() {
  identity_affinity_calls=$((identity_affinity_calls + 1))
  R2_PROC_READ_CLASS=incomplete
  return 2
}
identity_affinity_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then
  identity_status=0
else
  identity_status=$?
fi
[[ $identity_status -eq 1 && $identity_affinity_calls -eq 1 &&
  $R2_SAMPLER_IDENTITY_REASON == affinity-incomplete ]] ||
  fail 'live incomplete affinity was not the sole retryable identity result'
r2_read_allowed_cpu_list() {
  identity_affinity_calls=$((identity_affinity_calls + 1))
  R2_PROC_READ_CLASS=malformed
  return 2
}
identity_affinity_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then
  identity_status=0
else
  identity_status=$?
fi
[[ $identity_status -eq 2 && $identity_affinity_calls -eq 1 &&
  $R2_SAMPLER_IDENTITY_REASON == affinity-malformed ]] ||
  fail 'malformed affinity entered identity retry'
r2_read_allowed_cpu_list() {
  identity_affinity_calls=$((identity_affinity_calls + 1))
  R2_EXPANDED_CPU_LIST=1
  R2_PROC_READ_CLASS=ok
}
identity_affinity_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
else identity_status=$?; fi
[[ $identity_status -eq 2 && $identity_affinity_calls -eq 1 &&
  $R2_SAMPLER_IDENTITY_REASON == affinity-changed ]] ||
  fail 'changed affinity entered identity retry'
# A second-stat incomplete snapshot is retryable, but a second-stat identity
# splice remains terminal. The successful affinity between the two stat reads
# cannot turn either result into a stable identity.
identity_stat_calls=0
r2_read_allowed_cpu_list() {
  identity_affinity_calls=$((identity_affinity_calls + 1))
  R2_EXPANDED_CPU_LIST=0
  R2_PROC_READ_CLASS=ok
}
r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  if [[ $identity_stat_calls -eq 2 ]]; then
    R2_PROC_READ_CLASS=incomplete
    return 1
  fi
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=$identity_pid
  R2_PROC_SESSION=$identity_pid
  R2_PROC_START=7
  R2_PROC_READ_CLASS=ok
}
identity_stat_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
else identity_status=$?; fi
[[ $identity_status -eq 1 && $identity_stat_calls -eq 2 &&
  $R2_SAMPLER_IDENTITY_REASON == confirmation-stat-incomplete ]] ||
  fail 'live incomplete confirmation stat was not retryable'
r2_read_process_stat() {
  identity_stat_calls=$((identity_stat_calls + 1))
  R2_PROC_STATE=S
  R2_PROC_PARENT=1
  R2_PROC_GROUP=$identity_pid
  R2_PROC_SESSION=$identity_pid
  if [[ $identity_stat_calls -eq 1 ]]; then R2_PROC_START=7
  else R2_PROC_START=8; fi
  R2_PROC_READ_CLASS=ok
}
identity_stat_calls=0
if r2_sampler_identity_stable "$identity_pid" 7 0; then identity_status=0
else identity_status=$?; fi
[[ $identity_status -eq 2 && $identity_stat_calls -eq 2 &&
  $R2_SAMPLER_IDENTITY_REASON == confirmation-identity-changed ]] ||
  fail 'confirmation identity splice entered retry'
eval "$identity_stat_source"
eval "$identity_affinity_source"
unset identity_stat_source identity_affinity_source

# Readiness retries only identity status 1 inside the existing 100-poll bound.
# A marker appearing between polls receives a fresh stable identity probe, and
# definitive identity or marker failures stop immediately.
identity_probe_source=$(declare -f r2_sampler_identity_stable)
ready_marker=$temporary/procfs-ready
ready_sleep_calls=0
ready_identity_calls=0
sleep() {
  ready_sleep_calls=$((ready_sleep_calls + 1))
  [[ $ready_sleep_calls -ne 1 ]] || : >"$ready_marker"
}
r2_sampler_identity_stable() {
  ready_identity_calls=$((ready_identity_calls + 1))
  R2_SAMPLER_IDENTITY_REASON=stable
}
r2_wait_for_sampler_ready "$identity_pid" 7 0 "$ready_marker" ||
  fail 'ready marker appearing between polls was refused'
[[ $ready_sleep_calls -eq 1 && $ready_identity_calls -eq 2 &&
  $R2_SAMPLER_READY_REASON == ready ]] ||
  fail 'ready marker was accepted without its final stable identity probe'

ready_sleep_calls=0
ready_identity_calls=0
r2_sampler_identity_stable() {
  ready_identity_calls=$((ready_identity_calls + 1))
  R2_SAMPLER_IDENTITY_REASON=affinity-incomplete
  return 1
}
if r2_wait_for_sampler_ready "$identity_pid" 7 0 "$ready_marker"; then
  ready_wait_status=0
else
  ready_wait_status=$?
fi
[[ $ready_wait_status -eq 1 && $ready_identity_calls -eq 100 &&
  $ready_sleep_calls -eq 100 &&
  $R2_SAMPLER_READY_REASON == affinity-incomplete-timeout ]] ||
  fail 'persistent incomplete readiness did not expire at its fixed bound'

ready_sleep_calls=0
ready_identity_calls=0
r2_sampler_identity_stable() {
  ready_identity_calls=$((ready_identity_calls + 1))
  R2_SAMPLER_IDENTITY_REASON=affinity-malformed
  return 2
}
if r2_wait_for_sampler_ready "$identity_pid" 7 0 "$ready_marker"; then
  ready_wait_status=0
else
  ready_wait_status=$?
fi
[[ $ready_wait_status -eq 2 && $ready_identity_calls -eq 1 &&
  $ready_sleep_calls -eq 0 && $R2_SAMPLER_READY_REASON == affinity-malformed ]] ||
  fail 'definitive readiness identity failure was retried'

mkdir "$temporary/procfs-malformed-ready"
ready_identity_calls=0
if r2_wait_for_sampler_ready "$identity_pid" 7 0 \
  "$temporary/procfs-malformed-ready"; then ready_wait_status=0
else ready_wait_status=$?; fi
[[ $ready_wait_status -eq 2 && $ready_identity_calls -eq 0 &&
  $R2_SAMPLER_READY_REASON == ready-marker-malformed ]] ||
  fail 'malformed ready marker reached identity acceptance'
eval "$identity_probe_source"
unset -f sleep
unset identity_probe_source

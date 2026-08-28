#!/usr/bin/env bash

# Source-only primitives shared by the complete proof and its refusal suite.

case ${BASH_SOURCE[0]} in
  */*) r2_complete_proof_lib_directory=${BASH_SOURCE[0]%/*} ;;
  *) r2_complete_proof_lib_directory=. ;;
esac
# shellcheck source=docs/evaluation/r2-disk-control-lib.sh
source "$r2_complete_proof_lib_directory/r2-disk-control-lib.sh"
unset r2_complete_proof_lib_directory

r2_read_process_stat() {
  [[ $# -eq 1 && -n $1 ]] || return 2
  local process_stat process_fields
  local -a fields=()
  if ! { IFS= read -r process_stat <"$1"; } 2>/dev/null; then
    return 1
  fi
  process_fields=${process_stat##*) }
  [[ $process_fields != "$process_stat" ]] || return 2
  read -r -a fields <<<"$process_fields"
  [[ ${#fields[@]} -ge 20 && ${fields[0]} =~ ^[A-Za-z]$ &&
    ${fields[1]} =~ ^[0-9]+$ && ${fields[2]} =~ ^[0-9]+$ &&
    ${fields[3]} =~ ^[0-9]+$ &&
    ${fields[19]} =~ ^[0-9]+$ ]] || return 2
  R2_PROC_STATE=${fields[0]}
  R2_PROC_PARENT=${fields[1]}
  R2_PROC_GROUP=${fields[2]}
  R2_PROC_SESSION=${fields[3]}
  R2_PROC_START=${fields[19]}
}

r2_expand_cpu_list() {
  [[ $# -eq 1 && $1 != *$'\n'* && $1 != *$'\t'* ]] || return 2
  local entry first_text last_text first last cpu expanded='' previous=-1
  local -a entries=() cpus=()
  IFS=, read -r -a entries <<<"$1"
  for entry in "${entries[@]}"; do
    if [[ $entry =~ ^(0|[1-9][0-9]*)-(0|[1-9][0-9]*)$ ]]; then
      first_text=${BASH_REMATCH[1]}
      last_text=${BASH_REMATCH[2]}
    elif [[ $entry =~ ^(0|[1-9][0-9]*)$ ]]; then
      first_text=$entry
      last_text=$entry
    else
      return 2
    fi
    first=$((10#$first_text))
    last=$((10#$last_text))
    [[ $first -le $last && $first -gt $previous ]] || return 2
    for ((cpu = first; cpu <= last; cpu += 1)); do
      cpus+=("$cpu")
      previous=$cpu
    done
  done
  [[ ${#cpus[@]} -gt 0 ]] || return 2
  printf -v expanded '%s,' "${cpus[@]}"
  R2_EXPANDED_CPU_LIST=${expanded%,}
}

r2_read_cpu_sibling_group() {
  [[ $# -eq 2 && $1 =~ ^(0|[1-9][0-9]*)$ && -d $2 && ! -L $2 ]] || return 2
  local cpu=$1 topology_root=$2 group line group_text sibling_file
  local -a lines=()
  sibling_file=$topology_root/cpu$cpu/topology/thread_siblings_list
  [[ -f $sibling_file && ! -L $sibling_file ]] || return 1
  mapfile -t lines <"$sibling_file" || return 1
  [[ ${#lines[@]} -eq 1 && -n ${lines[0]} ]] || return 2
  line=${lines[0]}
  r2_expand_cpu_list "$line" || return
  group=$R2_EXPANDED_CPU_LIST
  group_text=,$group,
  [[ $group_text == *,$cpu,* ]] || return 2
  # shellcheck disable=SC2034 # Returned global is consumed after sourcing.
  R2_CPU_SIBLING_GROUP=$group
}

r2_validate_physical_cpu_isolation() {
  [[ $# -eq 3 ]] || return 2
  local disk_text workload_text topology_root=$3 cpu sibling group
  local -a disk_cpus=() workload_cpus=() siblings=()
  local -A disk_set=() workload_set=()
  r2_expand_cpu_list "$1" || return
  disk_text=$R2_EXPANDED_CPU_LIST
  IFS=, read -r -a disk_cpus <<<"$disk_text"
  r2_expand_cpu_list "$2" || return
  workload_text=$R2_EXPANDED_CPU_LIST
  IFS=, read -r -a workload_cpus <<<"$workload_text"
  for cpu in "${disk_cpus[@]}"; do disk_set["$cpu"]=1; done
  for cpu in "${workload_cpus[@]}"; do
    [[ -z ${disk_set[$cpu]+present} ]] || return 1
    workload_set["$cpu"]=1
  done
  for cpu in "${disk_cpus[@]}"; do
    r2_read_cpu_sibling_group "$cpu" "$topology_root" || return
    group=$R2_CPU_SIBLING_GROUP
    IFS=, read -r -a siblings <<<"$group"
    for sibling in "${siblings[@]}"; do
      [[ -z ${workload_set[$sibling]+present} ]] || return 1
    done
  done
  for cpu in "${workload_cpus[@]}"; do
    r2_read_cpu_sibling_group "$cpu" "$topology_root" || return
    group=$R2_CPU_SIBLING_GROUP
    IFS=, read -r -a siblings <<<"$group"
    for sibling in "${siblings[@]}"; do
      [[ -z ${disk_set[$sibling]+present} ]] || return 1
    done
  done
  R2_EXPANDED_CPU_LIST=$workload_text
}

r2_partition_cpu_topology() {
  [[ $# -eq 2 ]] || return 2
  local allowed_text topology_root=$2 cpu sibling group group_list=''
  local disk_group_count index disk_text='' workload_text=''
  local disk_groups_text='' workload_groups_text=''
  local -a cpus=() siblings=() groups=() disk_cpus=() workload_cpus=()
  local -A allowed_set=() cpu_group=() group_index=() sibling_group=()
  r2_expand_cpu_list "$1" || return
  allowed_text=$R2_EXPANDED_CPU_LIST
  IFS=, read -r -a cpus <<<"$allowed_text"
  for cpu in "${cpus[@]}"; do allowed_set["$cpu"]=1; done
  for cpu in "${cpus[@]}"; do
    r2_read_cpu_sibling_group "$cpu" "$topology_root" || return
    group=$R2_CPU_SIBLING_GROUP
    cpu_group["$cpu"]=$group
    if [[ -z ${group_index[$group]+present} ]]; then
      group_index["$group"]=${#groups[@]}
      groups+=("$group")
    fi
  done
  [[ ${#groups[@]} -ge 2 ]] || return 1

  # Sibling groups must be disjoint even where they name a logical CPU outside
  # the allowed affinity. Every allowed sibling must also report the same group.
  # Together these checks reject partial or contradictory topology.
  for group in "${groups[@]}"; do
    IFS=, read -r -a siblings <<<"$group"
    for sibling in "${siblings[@]}"; do
      if [[ -n ${sibling_group[$sibling]+present} &&
          ${sibling_group[$sibling]} != "$group" ]]; then
        return 2
      fi
      sibling_group["$sibling"]=$group
    done
  done
  for cpu in "${cpus[@]}"; do
    group=${cpu_group[$cpu]}
    IFS=, read -r -a siblings <<<"$group"
    for sibling in "${siblings[@]}"; do
      if [[ -n ${allowed_set[$sibling]+present} &&
          ${cpu_group[$sibling]} != "$group" ]]; then
        return 2
      fi
    done
  done

  # Give the observer the first half of complete physical-core groups and the
  # workload the rest. On an odd count, the workload receives the extra core.
  disk_group_count=$(( ${#groups[@]} / 2 ))
  for cpu in "${cpus[@]}"; do
    group=${cpu_group[$cpu]}
    index=${group_index[$group]}
    if [[ $index -lt $disk_group_count ]]; then
      disk_cpus+=("$cpu")
    else
      workload_cpus+=("$cpu")
    fi
  done
  [[ ${#disk_cpus[@]} -gt 0 && ${#workload_cpus[@]} -gt 0 ]] || return 1
  printf -v disk_text '%s,' "${disk_cpus[@]}"
  printf -v workload_text '%s,' "${workload_cpus[@]}"
  printf -v group_list '%s;' "${groups[@]}"
  printf -v disk_groups_text '%s;' "${groups[@]:0:disk_group_count}"
  printf -v workload_groups_text '%s;' "${groups[@]:disk_group_count}"
  disk_text=${disk_text%,}
  workload_text=${workload_text%,}
  r2_validate_physical_cpu_isolation "$disk_text" "$workload_text" "$topology_root" ||
    return
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CONTROLLER_CPUS=${disk_cpus[0]}
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_DISK_CPUS=$disk_text
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_WORKLOAD_CPUS=$workload_text
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CPU_TOPOLOGY_GROUPS=${group_list%;}
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_DISK_PHYSICAL_GROUPS=${disk_groups_text%;}
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_WORKLOAD_PHYSICAL_GROUPS=${workload_groups_text%;}
  R2_EXPANDED_CPU_LIST=$allowed_text
}

r2_read_allowed_cpu_list() {
  [[ $# -eq 1 && -f $1 ]] || return 2
  local line found=0 value
  while IFS= read -r line; do
    [[ $line != Cpus_allowed_list:* ]] || {
      [[ $found -eq 0 ]] || return 2
      value=${line#*:}
      value=${value//$'\t'/}
      value=${value// /}
      found=1
    }
  done <"$1"
  [[ $found -eq 1 ]] || return 2
  r2_expand_cpu_list "$value" || return
  # shellcheck disable=SC2034 # Returned global is consumed after sourcing.
  R2_ALLOWED_CPU_LIST=$value
}

r2_sampler_identity_stable() {
  [[ $# -eq 3 && $1 =~ ^[1-9][0-9]*$ && $2 =~ ^[0-9]+$ && -n $3 ]] || return 2
  local pid=$1 start=$2 expected_cpu_list=$3
  r2_read_process_stat "/proc/$pid/stat" &&
    [[ $R2_PROC_GROUP == "$pid" && $R2_PROC_SESSION == "$pid" &&
      $R2_PROC_START == "$start" && $R2_PROC_STATE != Z ]] &&
    r2_read_allowed_cpu_list "/proc/$pid/status" &&
    [[ $R2_EXPANDED_CPU_LIST == "$expected_cpu_list" ]]
}

r2_sampler_session_has_members() {
  [[ $# -eq 1 && $1 =~ ^[1-9][0-9]*$ ]] || return 2
  local session=$1 proc stat_status
  local -a snapshot=(/proc/[0-9]*)
  for proc in "${snapshot[@]}"; do
    if r2_read_process_stat "$proc/stat"; then
      [[ $R2_PROC_SESSION != "$session" || $R2_PROC_STATE == Z ]] || return 0
    else
      stat_status=$?
      [[ $stat_status -eq 1 && ! -e $proc ]] || return 2
    fi
  done
  return 1
}

r2_stop_disk_sampler() {
  [[ $# -eq 5 && $1 =~ ^[1-9][0-9]*$ &&
    ( -z $2 || $2 =~ ^[0-9]+$ ) && -n $3 && -n $4 &&
    $5 =~ ^[0-9]+$ && $5 -le 255 ]] || return 2
  local pid=$1 start=$2 expected_cpu_list=$3 stop=$4 incoming=$5
  local marker_ns marker_status=0 wait_status=0 forced=0 attempt session_status
  local stat_status can_wait=0

  if [[ -z $start ]] ||
    ! r2_sampler_identity_stable "$pid" "$start" "$expected_cpu_list"; then
    marker_status=1
  elif ! marker_ns=$(date +%s%N) ||
    [[ ! $marker_ns =~ ^(0|[1-9][0-9]*)$ ]] ||
    ! r2_publish_decimal_control_marker "$stop" "$marker_ns"; then
    marker_status=1
  else
    # shellcheck disable=SC2034 # Returned global binds summary and marker.
    R2_DISK_STOP_REQUESTED_NS=$marker_ns
  fi

  # A normal stop gets a bounded grace period. A failed marker or a controller
  # that does not close is terminated as its identity-bound dedicated process
  # group, first softly and then unconditionally. Polling /proc avoids an
  # unbounded `wait` on a stopped or hung sampler.
  for ((attempt = 0; attempt < 500; attempt += 1)); do
    r2_read_process_stat "/proc/$pid/stat" || break
    [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
      $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE != Z ]] || break
    [[ $marker_status -eq 0 ]] || break
    sleep 0.01 || break
  done
  if r2_read_process_stat "/proc/$pid/stat" &&
    [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
      $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE != Z ]]; then
    kill -- "-$pid" 2>/dev/null || true
    forced=1
    for ((attempt = 0; attempt < 100; attempt += 1)); do
      r2_read_process_stat "/proc/$pid/stat" || break
      [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
        $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE != Z ]] || break
      sleep 0.01 || break
    done
  fi
  if r2_read_process_stat "/proc/$pid/stat" &&
    [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
      $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE != Z ]]; then
    kill -KILL -- "-$pid" 2>/dev/null || true
    forced=1
    for ((attempt = 0; attempt < 100; attempt += 1)); do
      r2_read_process_stat "/proc/$pid/stat" || break
      [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
        $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE != Z ]] || break
      sleep 0.01 || break
    done
  fi
  if r2_read_process_stat "/proc/$pid/stat"; then
    if [[ $R2_PROC_START == "$start" && $R2_PROC_GROUP == "$pid" &&
      $R2_PROC_SESSION == "$pid" && $R2_PROC_STATE == Z ]]; then
      can_wait=1
    else
      wait_status=1
    fi
  else
    stat_status=$?
    if [[ $stat_status -eq 1 && ! -e /proc/$pid ]]; then
      can_wait=1
    else
      wait_status=1
    fi
  fi
  if [[ $can_wait -eq 1 ]] && ! wait "$pid"; then
    wait_status=1
  fi
  for ((attempt = 0; attempt < 100; attempt += 1)); do
    if r2_sampler_session_has_members "$pid"; then
      sleep 0.01 || break
      continue
    else
      session_status=$?
      [[ $session_status -eq 1 ]] || wait_status=1
      break
    fi
  done
  if r2_sampler_session_has_members "$pid"; then wait_status=1; fi
  [[ $incoming -eq 0 ]] || return "$incoming"
  [[ $marker_status -eq 0 ]] || return "$marker_status"
  [[ $forced -eq 0 ]] || return 1
  return "$wait_status"
}

r2_prepare_and_stop_disk_sampler() {
  [[ $# -eq 6 && $1 =~ ^[1-9][0-9]*$ &&
    ( -z $2 || $2 =~ ^[0-9]+$ ) && -n $3 && -n $4 &&
    -d $5 && ! -L $5 && $6 =~ ^[0-9]+$ && $6 -le 255 ]] || return 2
  local pid=$1 start=$2 expected_cpu_list=$3 stop=$4 state=$5 incoming=$6
  local request=$state/drain-request ready=$state/drain-ready
  local request_ns='' prepare_status=0 prepared=0 attempt cleanup_status
  local prepare_timeout_ns=6000000000 prepare_deadline_ns monotonic_now

  if [[ $incoming -ne 0 ]]; then
    r2_stop_disk_sampler "$pid" "$start" "$expected_cpu_list" "$stop" "$incoming"
    return
  fi

  if [[ -z $start || -e $request || -L $request || -e $ready || -L $ready ]] ||
    ! r2_sampler_identity_stable "$pid" "$start" "$expected_cpu_list" ||
    ! request_ns=$(date +%s%N) ||
    [[ ! $request_ns =~ ^(0|[1-9][0-9]*)$ ]] ||
    ! r2_publish_decimal_control_marker "$request" "$request_ns"; then
    prepare_status=1
  fi

  if [[ $prepare_status -eq 0 ]]; then
    if ! r2_monotonic_now_ns || ! r2_disk_deadline_ns \
      "$R2_MONOTONIC_NS" 1 "$prepare_timeout_ns"; then
      prepare_status=1
    else
      prepare_deadline_ns=$R2_DISK_DEADLINE_NS
    fi
  fi

  if [[ $prepare_status -eq 0 ]]; then
    for ((attempt = 0; attempt < 1000; attempt += 1)); do
      r2_monotonic_now_ns || break
      monotonic_now=$R2_MONOTONIC_NS
      [[ $monotonic_now -lt $prepare_deadline_ns ]] || break
      if r2_read_decimal_control_marker "$ready" 2>/dev/null &&
        [[ $R2_CONTROL_MARKER == "$request_ns" ]]; then
        prepared=1
        break
      fi
      r2_sampler_identity_stable "$pid" "$start" "$expected_cpu_list" || break
      sleep 0.01 || break
    done
    [[ $prepared -eq 1 ]] || prepare_status=1
  fi

  if [[ $prepare_status -ne 0 ]]; then
    cleanup_status=$incoming
    [[ $cleanup_status -ne 0 ]] || cleanup_status=1
    r2_stop_disk_sampler \
      "$pid" "$start" "$expected_cpu_list" "$stop" "$cleanup_status" || return
    return "$cleanup_status"
  fi
  r2_stop_disk_sampler "$pid" "$start" "$expected_cpu_list" "$stop" "$incoming"
}

r2_measure_process_closure() {
  [[ ( $# -eq 3 || $# -eq 6 ) && $1 == net:\[*\] &&
    $2 =~ ^[0-9a-f]{64}$ && -n $3 ]] || {
    printf 'R2 process closure: invalid arguments\n' >&2
    return 2
  }
  local expected_namespace=$1
  local proof_token=$2
  local report=$3
  local allowed_root=${4:-}
  local allowed_session=${5:-}
  local allowed_start=${6:-}
  local ancestor=$$
  local ancestor_pids=" $$ "
  local proc pid process_namespace parent allowed process_session process_start
  local stat_status
  local strict_namespace=0
  local -a process_snapshot=(/proc/[0-9]*)

  [[ -z $allowed_root || $allowed_root =~ ^[1-9][0-9]*$ ]] || return 2
  [[ -z $allowed_session || $allowed_session =~ ^[1-9][0-9]*$ ]] || return 2
  [[ -z $allowed_start || $allowed_start =~ ^[0-9]+$ ]] || return 2
  if [[ -n $allowed_session ]]; then
    [[ $allowed_root == "$allowed_session" ]] || {
      printf 'R2 process closure: allowed sampler session root is not stable\n' >&2
      return 2
    }
    r2_read_process_stat "/proc/$allowed_root/stat" || {
      printf 'R2 process closure: allowed sampler session root is not stable\n' >&2
      return 2
    }
    [[ $R2_PROC_GROUP == "$allowed_session" &&
      $R2_PROC_SESSION == "$allowed_session" &&
      $R2_PROC_START == "$allowed_start" && $R2_PROC_STATE != Z ]] || {
      printf 'R2 process closure: allowed sampler session root is not stable\n' >&2
      return 2
    }
    process_start=$allowed_start
  fi
  if [[ ${NOMOS_R2_HOST_NETNS:-} == net:\[*\] &&
        $expected_namespace != "$NOMOS_R2_HOST_NETNS" ]]; then
    strict_namespace=1
  fi

  : >"$report"
  while [[ $ancestor -gt 1 ]]; do
    r2_read_process_stat "/proc/$ancestor/stat" || {
      printf 'R2 process closure: auditor ancestor changed during inspection\n' >&2
      return 2
    }
    ancestor=$R2_PROC_PARENT
    ancestor_pids+="$ancestor "
  done
  for proc in "${process_snapshot[@]}"; do
    pid=${proc##*/}
    [[ " $ancestor_pids " == *" $pid "* ]] && continue
    if [[ -n $allowed_session ]]; then
      if r2_read_process_stat "$proc/stat"; then
        process_session=$R2_PROC_SESSION
      else
        stat_status=$?
        [[ $stat_status -eq 1 && ! -e $proc ]] && continue
        return 2
      fi
      # A process cannot join an existing session. SID equality therefore
      # admits exactly the live sampler root and its descendants, including a
      # child reparented while this snapshot is inspected. PGID equality does
      # not have that property: a same-session sibling may join the group.
      [[ $process_session != "$allowed_session" ]] || continue
    fi
    if ! process_namespace=$(readlink "$proc/ns/net" 2>/dev/null); then
      [[ ! -e $proc ]] && continue
      [[ $strict_namespace -eq 0 ]] && continue
      return 2
    fi
    [[ $process_namespace == "$expected_namespace" ]] || continue

    parent=$pid
    allowed=0
    while [[ -z $allowed_session && -n $allowed_root && $parent -gt 1 ]]; do
      [[ $parent -ne $allowed_root ]] || { allowed=1; break; }
      if r2_read_process_stat "/proc/$parent/stat"; then
        parent=$R2_PROC_PARENT
      else
        if [[ ! -e $proc ]]; then
          allowed=1
          break
        fi
        printf 'R2 process closure: process ancestry changed during inspection\n' >&2
        return 2
      fi
    done
    [[ $allowed -eq 0 ]] || continue

    if [[ $strict_namespace -eq 0 ]] &&
      ! grep -Fzx -- "NOMOS_R2_PROOF_TOKEN=$proof_token" "$proc/environ" \
        >/dev/null 2>&1; then
      parent=$pid
      while [[ $parent -gt 1 ]]; do
        if r2_read_process_stat "/proc/$parent/stat"; then
          parent=$R2_PROC_PARENT
        else
          if [[ ! -e $proc ]]; then
            parent=0
            break
          fi
          printf 'R2 process closure: process ancestry changed during inspection\n' >&2
          return 2
        fi
        [[ $parent -ne $$ ]] || break
      done
      [[ $parent -eq $$ ]] || continue
    fi
    printf '%s\n' "$pid" >>"$report"
  done
  if [[ -n $allowed_session ]]; then
    r2_read_process_stat "/proc/$allowed_root/stat" || {
      printf 'R2 process closure: allowed sampler session root is not stable\n' >&2
      return 2
    }
    [[ $R2_PROC_GROUP == "$allowed_session" &&
      $R2_PROC_SESSION == "$allowed_session" &&
      $R2_PROC_START == "$process_start" && $R2_PROC_STATE != Z ]] || {
      printf 'R2 process closure: allowed sampler session root is not stable\n' >&2
      return 2
    }
  fi
  if [[ -s $report ]]; then
    printf 'R2 process closure: live namespace children: %s\n' \
      "$(paste -sd, "$report")" >&2
    return 1
  fi
}

r2_execute_step() {
  [[ $# -ge 3 && -n $1 && -n $2 ]] || {
    printf 'R2 step executor: invalid arguments\n' >&2
    return 2
  }
  local stdout_file=$1
  local stderr_file=$2
  shift 2
  (
    set -e
    "$@"
  ) >"$stdout_file" 2>"$stderr_file"
}

r2_emit_recorded_tool_version() {
  [[ $# -ge 4 && -f $1 && ! -L $1 && $2 =~ ^[a-z0-9-]+$ &&
    $3 =~ ^[a-z0-9-]+$ ]] || {
    printf 'R2 tool version: invalid arguments\n' >&2
    return 2
  }
  local register=$1
  local key=$2
  local label=$3
  shift 3
  local path output first
  path=$(awk -F '\t' -v label="$label" '
    $1 == label { count += 1; path = $2 }
    END { if (count != 1 || path !~ /^\//) exit 1; print path }
  ' "$register") || {
    printf 'R2 tool version: recorded path differs for %s\n' "$label" >&2
    return 2
  }
  [[ -f $path && -x $path && ! -L $path && $(realpath -e -- "$path") == "$path" ]] || {
    printf 'R2 tool version: recorded path is not one canonical executable: %s\n' "$label" >&2
    return 2
  }
  output=$("$path" "$@" 2>/dev/null) || {
    printf 'R2 tool version: recorded executable failed: %s\n' "$label" >&2
    return 1
  }
  first=${output%%$'\n'*}
  [[ -n $first && $first != *$'\r'* ]] || {
    printf 'R2 tool version: recorded executable emitted no canonical line: %s\n' "$label" >&2
    return 2
  }
  printf '%s=%s\n' "$key" "$first"
}

r2_measure_checkout_mib() {
  [[ ( $# -eq 1 || $# -eq 3 ) && -d $1 &&
    ( $# -eq 1 || ( -n ${2:-} && ! -L ${2:-} &&
      ${3:-} =~ ^(0|[1-9][0-9]*)$ ) ) &&
    -n ${R2_DISK_WALK_CPUS:-} ]] || {
    printf 'R2 disk sampler: invalid checkout root\n' >&2
    return 2
  }
  local root=$1
  local started_signal=${2:-}
  local signal_ordinal=${3:-}
  local attempt started raw size
  for ((attempt = 0; attempt < 20; attempt += 1)); do
    # Cargo atomically publishes and removes intermediate files while `du`
    # walks the checkout. Retain only a complete, successful `du -sm` result;
    # a raced walk is not a sample and is retried immediately. The caller's
    # retained start timestamps still enforce the contract's maximum gap.
    # Measurement must not become the dominant workload whose budget it is
    # observing. Keep the contract's exact `du -sm` walk at ordinary CPU
    # priority and let measured writes take precedence at the I/O scheduler.
    started=$(date +%s%N) || return 2
    if [[ $attempt -eq 0 && -n $started_signal ]]; then
      printf '%s\t%s\n' "$signal_ordinal" "$started" >"$started_signal" || return 2
    fi
    if raw=$(ionice -c 3 du -sm -- "$root" 2>/dev/null); then
      size=${raw%%$'\t'*}
      [[ $size =~ ^[0-9]+$ && $raw == "$size"$'\t'"$root" ]] || {
        printf 'R2 disk sampler: malformed du result\n' >&2
        return 2
      }
      printf '%s\t%s\n' "$started" "$size"
      return 0
    fi
  done
  printf 'R2 disk sampler: no complete du result after 20 attempts\n' >&2
  return 1
}

r2_record_checkout_mib() {
  [[ ( $# -eq 5 || $# -eq 6 ) && -d $1 && -f $2 && ! -L $2 && $3 =~ ^[0-9]+$ &&
    $4 =~ ^[0-9]+$ && ( $5 == scheduled || $5 == terminal ) &&
    ( $# -eq 5 || ( -n ${6:-} && ! -L ${6:-} ) ) &&
    -n ${R2_DISK_WALK_CPUS:-} ]] || {
    printf 'R2 disk sampler: invalid sample arguments\n' >&2
    return 2
  }
  local root=$1
  local raw_samples=$2
  local sampler_started=$3
  local ordinal=$4
  local kind=$5
  local started_signal=${6:-}
  local row measured_started size elapsed
  taskset -pc "$R2_DISK_WALK_CPUS" "$BASHPID" >/dev/null || return 2
  if [[ -n $started_signal ]]; then
    row=$(r2_measure_checkout_mib "$root" "$started_signal" "$ordinal") || return
  else
    row=$(r2_measure_checkout_mib "$root") || return
  fi
  IFS=$'\t' read -r measured_started size <<<"$row"
  [[ $measured_started =~ ^[0-9]+$ && $measured_started -ge $sampler_started &&
    $size =~ ^[0-9]+$ ]] || return 2
  elapsed=$((measured_started - sampler_started))
  # Each short O_APPEND write is one complete raw row. The controller waits
  # every writer and validates/sorts all rows before publishing the ledger.
  printf '%s\t%s\t%s\t%s\t%s\n' \
    "$ordinal" "$measured_started" "$elapsed" "$size" "$kind" >>"$raw_samples"
}

r2_sample_checkout_disk() {
  [[ $# -eq 6 && -d $1 && -f $2 && ! -L $2 && -n $3 &&
    -d $4 && ! -L $4 && $5 =~ ^[0-9]+$ && $6 =~ ^[1-9][0-9]*$ ]] || {
    printf 'R2 disk sampler: invalid controller arguments\n' >&2
    return 2
  }
  local root=$1
  local samples=$2
  local stop=$3
  local state=$4
  local sampler_started=$5
  local period_ns=$6
  local ready=$state/ready
  local drain_request=$state/drain-request
  local drain_ready=$state/drain-ready
  local raw_samples=$state/samples.unsorted.tsv
  local sorted_samples=$state/samples.sorted.tsv
  local ordinal=0 deadline now monotonic_now delay delay_seconds pid status=0 attempt
  local child_start child_reaped active initial_ready=0 draining=0 drain_remaining
  local request_ns='' stop_ns='' bridge_ordinal='' bridge_pending=0 stat_status
  local worker_timeout_ns=4000000000 drain_deadline_ns=''
  local controller_start controller_group controller_session
  local -a sample_pids=()
  local -A sample_starts=()
  local -A drain_roots=()

  # One controller follows two interleaved absolute nominal-period phases.
  # Deadlines are derived from the fixed origin, never from completion of the
  # prior walk.
  r2_disk_interleaved_deadline_ns "$sampler_started" 0 "$period_ns" || return
  r2_read_process_stat "/proc/$BASHPID/stat" || return 2
  controller_start=$R2_PROC_START
  controller_group=$R2_PROC_GROUP
  controller_session=$R2_PROC_SESSION

  [[ -z $(find "$state" -mindepth 1 -print -quit) ]] || return 2
  : >"$raw_samples"
  [[ -f $raw_samples && ! -L $raw_samples ]] || return 2

  reap_finished_samples() {
    local candidate expected_start candidate_stat
    local -a still_running=()

    # Reap completed children immediately and retain only the bounded active
    # set. Bash preserves a background child's status for `wait` after the
    # process exits, so every completed walk still contributes to the final
    # fail-closed result without making controller work grow with proof age.
    for candidate in "${sample_pids[@]}"; do
      expected_start=${sample_starts[$candidate]}
      if r2_read_process_stat "/proc/$candidate/stat" &&
        [[ $R2_PROC_START == "$expected_start" &&
          $R2_PROC_PARENT == "$BASHPID" && $R2_PROC_GROUP == "$controller_group" &&
          $R2_PROC_SESSION == "$controller_session" ]]; then
        if [[ $R2_PROC_STATE != Z ]]; then
          still_running+=("$candidate")
          continue
        fi
      else
        candidate_stat=$?
        if [[ $candidate_stat -ne 1 || -e /proc/$candidate ]] ||
          kill -0 "$candidate" 2>/dev/null; then
          status=1
          still_running+=("$candidate")
          continue
        fi
      fi
      wait "$candidate" || status=1
      unset 'sample_starts[$candidate]'
    done
    sample_pids=("${still_running[@]}")
    [[ $status -eq 0 ]]
  }

  abort_dedicated_sampler_group() {
    if r2_read_process_stat "/proc/$BASHPID/stat" &&
      [[ $R2_PROC_START == "$controller_start" && $R2_PROC_GROUP == "$BASHPID" &&
        $R2_PROC_SESSION == "$BASHPID" && $R2_PROC_STATE != Z ]]; then
      kill -KILL -- "-$BASHPID"
    fi
    return 1
  }

  wait_for_sample_set() {
    [[ $# -le 1 ]] || return 2
    local wait_deadline=${1:-} wait_now
    if [[ -n $wait_deadline ]]; then
      r2_disk_deadline_ns "$wait_deadline" 0 1 || return 2
    else
      r2_monotonic_now_ns || { status=1; return 1; }
      wait_now=$R2_MONOTONIC_NS
      r2_disk_deadline_ns "$wait_now" 1 "$worker_timeout_ns" || {
        status=1
        return 1
      }
      wait_deadline=$R2_DISK_DEADLINE_NS
    fi
    while :; do
      reap_finished_samples || true
      [[ ${#sample_pids[@]} -ne 0 ]] || { [[ $status -eq 0 ]]; return; }
      r2_monotonic_now_ns || { status=1; break; }
      wait_now=$R2_MONOTONIC_NS
      [[ $wait_now -lt $wait_deadline ]] || break
      sleep 0.01 || { status=1; break; }
    done
    status=1
    printf 'R2 disk sampler: sample workers did not close before timeout\n' >&2
    abort_dedicated_sampler_group || true
    return 1
  }

  launch_sample() {
    [[ $# -eq 1 && ( $1 == scheduled || $1 == terminal ) ]] || return 2
    reap_finished_samples || return
    active=${#sample_pids[@]}
    [[ $active -lt 32 ]] || {
      printf 'R2 disk sampler: thirty-two concurrent du walks are still active\n' >&2
      return 3
    }
    r2_record_checkout_mib \
      "$root" "$raw_samples" "$sampler_started" "$ordinal" "$1" &
    pid=$!
    child_start=
    child_reaped=0
    for ((attempt = 0; attempt < 100; attempt += 1)); do
      if r2_read_process_stat "/proc/$pid/stat"; then
        if [[ $R2_PROC_PARENT != "$BASHPID" ||
          $R2_PROC_GROUP != "$controller_group" ||
          $R2_PROC_SESSION != "$controller_session" ]]; then
          status=1
        elif [[ $R2_PROC_STATE == Z ]]; then
          wait "$pid" || status=1
          child_reaped=1
        else
          child_start=$R2_PROC_START
        fi
        break
      fi
      stat_status=$?
      if [[ $stat_status -eq 1 && ! -e /proc/$pid ]] &&
        ! kill -0 "$pid" 2>/dev/null; then
        wait "$pid" || status=1
        child_reaped=1
        break
      fi
      sleep 0.001 || { status=1; break; }
    done
    if [[ -n $child_start ]]; then
      sample_pids+=("$pid")
      sample_starts["$pid"]=$child_start
    elif [[ $child_reaped -eq 0 ]]; then
      status=1
      abort_dedicated_sampler_group || true
    fi
    ordinal=$((ordinal + 1))
    [[ $status -eq 0 ]]
  }

  initial_sample_retained() {
    local candidate
    while IFS=$'\t' read -r candidate _; do
      [[ $candidate != 0 ]] || return 0
    done <"$raw_samples"
    return 1
  }

  launch_sample scheduled || status=1
  while [[ $status -eq 0 ]]; do
    reap_finished_samples || {
      printf 'R2 disk sampler: initial sample failed\n' >&2
      status=1
      break
    }
    if [[ $initial_ready -eq 0 ]] && initial_sample_retained; then
      : >"$ready"
      initial_ready=1
    fi
    if [[ -e $stop || -L $stop ]]; then
      r2_read_decimal_control_marker "$stop" || { status=1; break; }
      stop_ns=$R2_CONTROL_MARKER
      [[ $initial_ready -eq 0 ]] || break
      sleep 0.001 || status=1
      [[ $status -eq 0 ]] || break
      continue
    fi
    if [[ -e $drain_request || -L $drain_request ]]; then
      if ! r2_read_decimal_control_marker "$drain_request"; then
        status=1
        break
      fi
      if [[ $draining -eq 0 ]]; then
        draining=1
        request_ns=$R2_CONTROL_MARKER
        # Epoch time validates that the control marker was not future-dated;
        # only the independent monotonic clock below governs the timeout.
        if ! now=$(date +%s%N) || ! r2_disk_deadline_ns "$now" 0 1 ||
          [[ $request_ns -gt $now ]] || ! r2_monotonic_now_ns; then
          status=1
          break
        fi
        monotonic_now=$R2_MONOTONIC_NS
        if ! r2_disk_deadline_ns \
          "$monotonic_now" 1 "$worker_timeout_ns"; then
          status=1
          break
        fi
        drain_deadline_ns=$R2_DISK_DEADLINE_NS
        bridge_pending=1
        for pid in "${sample_pids[@]}"; do
          drain_roots["$pid"]=1
        done
      elif [[ $R2_CONTROL_MARKER != "$request_ns" ]]; then
        status=1
        break
      fi
      drain_remaining=0
      for pid in "${!drain_roots[@]}"; do
        if [[ -n ${sample_starts[$pid]+present} ]]; then
          drain_remaining=1
        else
          unset 'drain_roots[$pid]'
        fi
      done
      [[ $drain_remaining -eq 1 || $bridge_pending -eq 1 ]] || break
    fi
    if ! r2_disk_interleaved_deadline_ns \
      "$sampler_started" "$ordinal" "$period_ns"; then
      status=1
      break
    fi
    deadline=$R2_DISK_DEADLINE_NS
    if ! now=$(date +%s%N) || ! r2_disk_deadline_ns "$now" 0 1; then
      status=1
      break
    fi
    if [[ $draining -eq 1 ]]; then
      if ! r2_monotonic_now_ns ||
        [[ $R2_MONOTONIC_NS -ge $drain_deadline_ns ]]; then
        status=1
        break
      fi
    fi
    if [[ $deadline -gt $now ]]; then
      delay=$((deadline - now))
      printf -v delay_seconds '%d.%09d' \
        "$((delay / 1000000000))" "$((delay % 1000000000))"
      if ! sleep "$delay_seconds"; then
        status=1
        break
      fi
    fi
    if [[ $draining -eq 1 ]]; then
      if ! r2_monotonic_now_ns ||
        [[ $R2_MONOTONIC_NS -ge $drain_deadline_ns ]]; then
        status=1
        break
      fi
    fi
    if [[ ! -e $stop && ! -L $stop ]]; then
      if [[ $draining -eq 1 && $bridge_pending -eq 1 ]]; then
        bridge_ordinal=$ordinal
      fi
      if ! launch_sample scheduled; then
        status=1
        break
      fi
      [[ $draining -ne 1 || $bridge_pending -ne 1 ]] || bridge_pending=0
    fi
  done

  # A normal stop is prepared before its canonical marker is written. Continue
  # the absolute schedule while every worker that was live at the drain request
  # quiesces, then wait the bounded bridge set. This prevents controller
  # shutdown itself from opening an uncovered interval.
  wait_for_sample_set "$drain_deadline_ns" || status=1

  if [[ $status -eq 0 && $draining -eq 1 ]]; then
    [[ ! -e $stop && ! -L $stop && ! -e $drain_ready && ! -L $drain_ready &&
      $bridge_ordinal =~ ^(0|[1-9][0-9]*)$ ]] || status=1
    [[ $status -ne 0 ]] || r2_validate_disk_drain_handoff \
      "$raw_samples" "$sorted_samples" "$sampler_started" "$ordinal" \
      "$request_ns" "$bridge_ordinal" "$((period_ns + period_ns / 2))" || status=1
    [[ $status -ne 0 ]] || r2_publish_decimal_control_marker \
      "$drain_ready" "$request_ns" || status=1
    for ((attempt = 0; status == 0 && attempt < 5000; attempt += 1)); do
      r2_read_decimal_control_marker "$drain_request" || { status=1; break; }
      [[ $R2_CONTROL_MARKER == "$request_ns" ]] || { status=1; break; }
      if [[ -e $stop || -L $stop ]]; then
        r2_read_decimal_control_marker "$stop" || { status=1; break; }
        stop_ns=$R2_CONTROL_MARKER
        [[ $stop_ns -ge $request_ns && $stop_ns -ge $R2_DISK_HANDOFF_LATEST_NS &&
          $((stop_ns - R2_DISK_HANDOFF_LATEST_NS)) -le 100000000 ]] || status=1
        break
      fi
      sleep 0.001 || status=1
    done
    [[ $status -ne 0 || $stop_ns =~ ^(0|[1-9][0-9]*)$ ]] || status=1
  elif [[ $status -eq 0 ]]; then
    r2_read_decimal_control_marker "$stop" || status=1
    [[ $status -ne 0 ]] || stop_ns=$R2_CONTROL_MARKER
  fi

  # This distinct final walk begins only after the canonical stop marker exists
  # and every scheduled or pre-stop bridge worker has retained its row.
  [[ $status -ne 0 ]] || launch_sample terminal || status=1

  wait_for_sample_set || status=1
  [[ $status -eq 0 ]] || {
    printf 'R2 disk sampler: one or more scheduled samples failed\n' >&2
    return 1
  }

  [[ -f $ready && ! -L $ready ]] || return 2
  find "$ready" -delete
  if [[ $draining -eq 1 ]]; then
    [[ -f $drain_request && ! -L $drain_request &&
      -f $drain_ready && ! -L $drain_ready ]] || return 2
    find "$drain_request" "$drain_ready" -delete
  fi
  r2_publish_checkout_disk_samples \
    "$samples" "$state" "$raw_samples" "$sorted_samples" \
    "$sampler_started" "$ordinal"
}

r2_network_probe() {
  [[ $# -eq 2 ]] || return 2
  node -e 'const net=require("node:net");const [host,port]=process.argv.slice(1);let done=false;const socket=net.connect({host,port:Number(port)});const timer=setTimeout(()=>finish(24,"blocked: timeout\n"),3000);function finish(code,text){if(done)return;done=true;clearTimeout(timer);socket.destroy();(code===0?process.stdout:process.stderr).write(text,()=>process.exit(code));}socket.once("connect",()=>finish(0,"connected\n"));socket.once("error",error=>finish(23,"blocked: "+(error.code||error.message)+"\n"));' "$1" "$2"
}

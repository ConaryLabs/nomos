#!/usr/bin/env bash

# Source-only primitives shared by the complete proof and its refusal suite.

case ${BASH_SOURCE[0]} in
  */*) r2_complete_proof_lib_directory=${BASH_SOURCE[0]%/*} ;;
  *) r2_complete_proof_lib_directory=. ;;
esac
# shellcheck source=docs/evaluation/r2-disk-control-lib.sh
source "$r2_complete_proof_lib_directory/r2-disk-control-lib.sh"
# shellcheck source=docs/evaluation/r2-disk-sampler-lib.sh
source "$r2_complete_proof_lib_directory/r2-disk-sampler-lib.sh"
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
  local observer_group_count index controller_text='' disk_text='' workload_text=''
  local controller_groups_text='' disk_groups_text='' workload_groups_text=''
  local -a cpus=() siblings=() groups=() controller_cpus=() disk_cpus=() workload_cpus=()
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
  # The controller, disk walks, and measured workload each require a complete
  # physical core group. A shared observer core recreated the very scheduler
  # contention this partition is intended to exclude.
  [[ ${#groups[@]} -ge 3 ]] || return 1

  # Every sibling group must be complete inside the allowed affinity and
  # disjoint from every other reported group. Every sibling must also report
  # the same group when its own topology file is read. Together these checks
  # reject partial, overlapping, or contradictory topology.
  for group in "${groups[@]}"; do
    IFS=, read -r -a siblings <<<"$group"
    for sibling in "${siblings[@]}"; do
      # A role may receive only a complete physical-core group. Accepting a
      # group that names an unavailable sibling would let work outside the
      # recorded affinity contend with an allegedly isolated role.
      [[ -n ${allowed_set[$sibling]+present} ]] || return 1
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

  # Give the observer side the first half of complete physical-core groups,
  # rounding up. Its first group is controller-only; the remaining observer
  # groups execute disk walks. The measured workload receives the rest.
  observer_group_count=$(( (${#groups[@]} + 1) / 2 ))
  for cpu in "${cpus[@]}"; do
    group=${cpu_group[$cpu]}
    index=${group_index[$group]}
    if [[ $index -eq 0 ]]; then
      controller_cpus+=("$cpu")
    elif [[ $index -lt $observer_group_count ]]; then
      disk_cpus+=("$cpu")
    else
      workload_cpus+=("$cpu")
    fi
  done
  [[ ${#controller_cpus[@]} -gt 0 && ${#disk_cpus[@]} -gt 0 &&
    ${#workload_cpus[@]} -gt 0 ]] || return 1
  printf -v controller_text '%s,' "${controller_cpus[@]}"
  printf -v disk_text '%s,' "${disk_cpus[@]}"
  printf -v workload_text '%s,' "${workload_cpus[@]}"
  printf -v group_list '%s;' "${groups[@]}"
  printf -v controller_groups_text '%s;' "${groups[@]:0:1}"
  printf -v disk_groups_text '%s;' "${groups[@]:1:observer_group_count - 1}"
  printf -v workload_groups_text '%s;' "${groups[@]:observer_group_count}"
  controller_text=${controller_text%,}
  disk_text=${disk_text%,}
  workload_text=${workload_text%,}
  r2_validate_physical_cpu_isolation \
    "$controller_text" "$disk_text" "$topology_root" || return
  r2_validate_physical_cpu_isolation \
    "$controller_text" "$workload_text" "$topology_root" || return
  r2_validate_physical_cpu_isolation \
    "$disk_text" "$workload_text" "$topology_root" || return
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CONTROLLER_CPUS=$controller_text
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_DISK_CPUS=$disk_text
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_WORKLOAD_CPUS=$workload_text
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CPU_TOPOLOGY_GROUPS=${group_list%;}
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CONTROLLER_PHYSICAL_GROUPS=${controller_groups_text%;}
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

r2_network_probe() {
  [[ $# -eq 2 ]] || return 2
  node -e 'const net=require("node:net");const [host,port]=process.argv.slice(1);let done=false;const socket=net.connect({host,port:Number(port)});const timer=setTimeout(()=>finish(24,"blocked: timeout\n"),3000);function finish(code,text){if(done)return;done=true;clearTimeout(timer);socket.destroy();(code===0?process.stdout:process.stderr).write(text,()=>process.exit(code));}socket.once("connect",()=>finish(0,"connected\n"));socket.once("error",error=>finish(23,"blocked: "+(error.code||error.message)+"\n"));' "$1" "$2"
}

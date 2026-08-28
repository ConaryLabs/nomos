#!/usr/bin/env bash

# Source-only primitives shared by the complete proof and its refusal suite.

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

r2_partition_cpu_list() {
  [[ $# -eq 1 ]] || return 2
  r2_expand_cpu_list "$1" || return
  local disk_count controller='' disk='' workload=''
  local -a cpus=()
  IFS=, read -r -a cpus <<<"$R2_EXPANDED_CPU_LIST"
  [[ ${#cpus[@]} -ge 3 ]] || return 1
  disk_count=$(( (${#cpus[@]} - 1) / 2 ))
  controller=${cpus[0]}
  printf -v disk '%s,' "${cpus[@]:1:disk_count}"
  printf -v workload '%s,' "${cpus[@]:disk_count+1}"
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_CONTROLLER_CPUS=$controller
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_DISK_CPUS=${disk%,}
  # shellcheck disable=SC2034 # Returned globals are consumed after sourcing.
  R2_WORKLOAD_CPUS=${workload%,}
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
    ! printf '%s\n' "$marker_ns" >"$stop"; then
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
  [[ ( $# -eq 1 || $# -eq 2 ) && -d $1 &&
    ( $# -eq 1 || ( -n ${2:-} && ! -L ${2:-} ) ) &&
    -n ${R2_DISK_WALK_CPUS:-} ]] || {
    printf 'R2 disk sampler: invalid checkout root\n' >&2
    return 2
  }
  local root=$1
  local started_signal=${2:-}
  local attempt started raw size
  for ((attempt = 0; attempt < 20; attempt += 1)); do
    # Cargo atomically publishes and removes intermediate files while `du`
    # walks the checkout. Retain only a complete, successful `du -sm` result;
    # a raced walk is not a sample and is retried immediately. The caller's
    # retained start timestamps still enforce the contract's maximum gap.
    # Measurement must not become the dominant workload whose budget it is
    # observing. Keep the contract's exact `du -sm` walk at low CPU priority
    # and let measured writes take precedence at the I/O scheduler.
    started=$(date +%s%N) || return 2
    [[ -z $started_signal ]] || printf '%s\n' "$started" >"$started_signal" || return 2
    if raw=$(nice -n 19 ionice -c 3 du -sm -- "$root" 2>/dev/null); then
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
  [[ $# -eq 6 && -d $1 && -f $2 && ! -L $2 && $3 =~ ^[0-9]+$ &&
    $4 =~ ^[0-9]+$ && ( $5 == scheduled || $5 == terminal ) &&
    -n $6 && ! -L $6 && -n ${R2_DISK_WALK_CPUS:-} ]] || {
    printf 'R2 disk sampler: invalid sample arguments\n' >&2
    return 2
  }
  local root=$1
  local raw_samples=$2
  local sampler_started=$3
  local ordinal=$4
  local kind=$5
  local started_signal=$6
  local row measured_started size elapsed
  taskset -pc "$R2_DISK_WALK_CPUS" "$BASHPID" >/dev/null || return 2
  row=$(r2_measure_checkout_mib "$root" "$started_signal") || return
  IFS=$'\t' read -r measured_started size <<<"$row"
  [[ $measured_started =~ ^[0-9]+$ && $measured_started -ge $sampler_started &&
    $size =~ ^[0-9]+$ ]] || return 2
  elapsed=$((measured_started - sampler_started))
  # Each short O_APPEND write is one complete raw row. The controller waits
  # every writer and validates/sorts all rows before publishing the ledger.
  printf '%s\t%s\t%s\t%s\t%s\n' \
    "$ordinal" "$measured_started" "$elapsed" "$size" "$kind" >>"$raw_samples"
}

r2_publish_checkout_disk_samples() {
  [[ $# -eq 6 && -f $1 && ! -L $1 && -d $2 && ! -L $2 &&
    -f $3 && ! -L $3 && -n $4 && $5 =~ ^[0-9]+$ &&
    $6 =~ ^[1-9][0-9]*$ ]] || return 2
  local samples=$1 state=$2 raw_samples=$3 sorted_samples=$4
  local sampler_started=$5 expected_count=$6
  local index=0 line row_ordinal sample_started elapsed size kind extra
  local previous_started=0 previous_elapsed=0 gap terminal_ordinal
  local -A seen_ordinals=()

  terminal_ordinal=$((expected_count - 1))
  LC_ALL=C sort -t $'\t' -k2,2n -k1,1n "$raw_samples" >"$sorted_samples"
  [[ $(wc -l <"$sorted_samples") -eq $expected_count ]] || return 2
  while IFS= read -r line; do
    IFS=$'\t' read -r row_ordinal sample_started elapsed size kind extra <<<"$line"
    [[ $row_ordinal =~ ^(0|[1-9][0-9]*)$ &&
      $sample_started =~ ^(0|[1-9][0-9]*)$ &&
      $elapsed =~ ^(0|[1-9][0-9]*)$ && $size =~ ^(0|[1-9][0-9]*)$ &&
      -z $extra && -z ${seen_ordinals[$row_ordinal]+present} &&
      $sample_started -ge $sampler_started &&
      $elapsed -eq $((sample_started - sampler_started)) ]] || return 2
    seen_ordinals["$row_ordinal"]=1
    if [[ $index -eq $terminal_ordinal ]]; then
      [[ $kind == terminal && $row_ordinal == "$terminal_ordinal" ]] || return 2
    else
      [[ $kind == scheduled ]] || return 2
    fi
    if [[ $index -gt 0 ]]; then
      [[ $sample_started -gt $previous_started &&
        $elapsed -gt $previous_elapsed ]] || return 2
      gap=$((elapsed - previous_elapsed))
      [[ $gap -le 100000000 ]] || {
        printf 'R2 disk sampler: retained sample-start gap exceeds 100000000 ns\n' >&2
        return 1
      }
    fi
    previous_started=$sample_started
    previous_elapsed=$elapsed
    index=$((index + 1))
  done <"$sorted_samples"
  [[ $index -eq $expected_count && ${#seen_ordinals[@]} -eq $expected_count ]] || return 2
  for ((index = 0; index < expected_count; index += 1)); do
    [[ -n ${seen_ordinals[$index]+present} ]] || return 2
  done
  while IFS= read -r line; do
    printf '%s\n' "$line" >>"$samples"
  done <"$sorted_samples"
  find "$raw_samples" "$sorted_samples" -delete
  [[ -z $(find "$state" -mindepth 1 -print -quit) ]] || return 2
  find "$state" -depth -delete
}

r2_write_checkout_disk_summary() {
  [[ $# -eq 6 && -f $1 && ! -L $1 && -f $2 && ! -L $2 &&
    ! -e $3 && $4 =~ ^(0|[1-9][0-9]*)$ &&
    $5 =~ ^(0|[1-9][0-9]*)$ && $6 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  local samples=$1 stop=$2 summary=$3 sampler_started=$4
  local period_ns=$5 stop_requested_ns=$6
  local count initial final maximum maximum_gap integer marker

  IFS= read -r marker <"$stop" || return 2
  [[ $marker == "$stop_requested_ns" && $(<"$stop") == "$marker" ]] || return 2
  count=$(awk 'NR > 1 { count += 1 } END { print count + 0 }' "$samples")
  initial=$(awk 'NR == 2 { print $4 }' "$samples")
  final=$(awk 'END { print $4 }' "$samples")
  maximum=$(awk 'NR > 1 && $4 > maximum { maximum = $4 } END { print maximum + 0 }' "$samples")
  maximum_gap=$(awk 'NR == 2 { previous = $3 } NR > 2 { gap = $3 - previous; if (gap > maximum) maximum = gap; previous = $3 } END { printf "%.0f\n", maximum + 0 }' "$samples")
  for integer in "$count" "$initial" "$final" "$maximum" "$maximum_gap"; do
    [[ $integer =~ ^[0-9]+$ ]] || return 2
  done
  [[ $count -ge 2 && $maximum -le 8192 && $maximum_gap -le 100000000 ]] || return 1
  jq -n \
    --arg sampler_origin_ns "$sampler_started" \
    --arg stop_requested_ns "$stop_requested_ns" \
    --arg nominal_interval_ns "$period_ns" \
    --argjson samples "$count" \
    --argjson initial "$initial" \
    --argjson final "$final" \
    --argjson maximum "$maximum" \
    --arg maximum_gap_ns "$maximum_gap" \
    '{outcome:"pass",sampler_origin_ns:$sampler_origin_ns,
      stop_requested_ns:$stop_requested_ns,
      nominal_interval_ns:$nominal_interval_ns,samples:$samples,
      initial_mib:$initial,final_mib:$final,maximum_mib:$maximum,
      maximum_gap_ns:$maximum_gap_ns,du_arguments:["-sm","--","<checkout>"]}' \
    >"$summary"
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
  local raw_samples=$state/samples.unsorted.tsv
  local sorted_samples=$state/samples.sorted.tsv
  local ordinal=0 deadline now delay delay_seconds pid status=0 attempt
  local child_start started_signal signal_value signal_ready active initial_ready=0
  local -a sample_pids=()
  local -A sample_starts=()

  : >"$raw_samples"

  reap_finished_samples() {
    local candidate expected_start
    local -a still_running=()

    # Reap completed children immediately and retain only the bounded active
    # set. Bash preserves a background child's status for `wait` after the
    # process exits, so every completed walk still contributes to the final
    # fail-closed result without making controller work grow with proof age.
    for candidate in "${sample_pids[@]}"; do
      expected_start=${sample_starts[$candidate]}
      if r2_read_process_stat "/proc/$candidate/stat" &&
        [[ $R2_PROC_START == "$expected_start" && $R2_PROC_STATE != Z ]]; then
        still_running+=("$candidate")
      elif ! wait "$candidate"; then
        status=1
        unset 'sample_starts[$candidate]'
      else
        unset 'sample_starts[$candidate]'
      fi
    done
    sample_pids=("${still_running[@]}")
    [[ $status -eq 0 ]]
  }

  launch_sample() {
    [[ $# -eq 1 && ( $1 == scheduled || $1 == terminal ) ]] || return 2
    reap_finished_samples || return
    active=${#sample_pids[@]}
    [[ $active -lt 32 ]] || {
      printf 'R2 disk sampler: thirty-two concurrent du walks are still active\n' >&2
      return 3
    }
    started_signal=$state/started.$ordinal
    [[ ! -e $started_signal && ! -L $started_signal ]] || return 2
    r2_record_checkout_mib \
      "$root" "$raw_samples" "$sampler_started" "$ordinal" "$1" \
      "$started_signal" &
    pid=$!
    child_start=
    if r2_read_process_stat "/proc/$pid/stat"; then
      child_start=$R2_PROC_START
      sample_pids+=("$pid")
      sample_starts["$pid"]=$child_start
    elif ! wait "$pid"; then
      status=1
    fi
    ordinal=$((ordinal + 1))
    signal_ready=0
    for ((attempt = 0; attempt < 100; attempt += 1)); do
      if [[ -f $started_signal && ! -L $started_signal ]]; then
        IFS= read -r signal_value <"$started_signal" || return 2
        [[ $signal_value =~ ^(0|[1-9][0-9]*)$ ]] || return 2
        signal_ready=1
        break
      fi
      sleep 0.001 || return 2
    done
    [[ $status -eq 0 && $signal_ready -eq 1 ]]
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
    [[ ! -e $stop || $initial_ready -eq 0 ]] || break
    if [[ -e $stop ]]; then
      sleep 0.001 || status=1
      [[ $status -eq 0 ]] || break
      continue
    fi
    deadline=$((sampler_started + ordinal * period_ns))
    if ! now=$(date +%s%N); then
      status=1
      break
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
    if [[ ! -e $stop ]] && ! launch_sample scheduled; then
      status=1
      break
    fi
  done

  # This distinct final walk begins only after the stop marker exists.
  [[ $status -ne 0 ]] || launch_sample terminal || status=1

  reap_finished_samples || status=1
  for pid in "${sample_pids[@]}"; do
    wait "$pid" || status=1
    unset 'sample_starts[$pid]'
  done
  [[ $status -eq 0 ]] || {
    printf 'R2 disk sampler: one or more scheduled samples failed\n' >&2
    return 1
  }

  find "$ready" -delete
  for ((active = 0; active < ordinal; active += 1)); do
    started_signal=$state/started.$active
    [[ -f $started_signal && ! -L $started_signal ]] || return 2
    IFS= read -r signal_value <"$started_signal" || return 2
    [[ $signal_value =~ ^(0|[1-9][0-9]*)$ ]] || return 2
    find "$started_signal" -delete
  done
  r2_publish_checkout_disk_samples \
    "$samples" "$state" "$raw_samples" "$sorted_samples" \
    "$sampler_started" "$ordinal"
}

r2_network_probe() {
  [[ $# -eq 2 ]] || return 2
  node -e 'const net=require("node:net");const [host,port]=process.argv.slice(1);let done=false;const socket=net.connect({host,port:Number(port)});const timer=setTimeout(()=>finish(24,"blocked: timeout\n"),3000);function finish(code,text){if(done)return;done=true;clearTimeout(timer);socket.destroy();(code===0?process.stdout:process.stderr).write(text,()=>process.exit(code));}socket.once("connect",()=>finish(0,"connected\n"));socket.once("error",error=>finish(23,"blocked: "+(error.code||error.message)+"\n"));' "$1" "$2"
}

#!/usr/bin/env bash

# Exact checkout measurement and the bounded, pre-affinitized worker-pool
# controller. This file is sourced by r2-complete-proof-lib.sh.

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
  r2_read_allowed_cpu_list "/proc/$BASHPID/status" || return 2
  [[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]] || return 2
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
  # for each dispatched job and validates/sorts every row before publication.
  printf '%s\t%s\t%s\t%s\t%s\n' \
    "$ordinal" "$measured_started" "$elapsed" "$size" "$kind" >>"$raw_samples"
}

r2_disk_pool_worker() {
  [[ $# -eq 5 && $1 =~ ^(0|[1-9][0-9]*)$ && -d $2 && -f $3 && ! -L $3 &&
    $4 =~ ^(0|[1-9][0-9]*)$ &&
    $5 =~ ^(-|[1-9][0-9]*(,[1-9][0-9]*)*)$ &&
    -n ${R2_DISK_WALK_CPUS:-} ]] || return 2
  local worker_index=$1 root=$2 raw_samples=$3 sampler_started=$4 close_fds=$5
  local descriptor command ordinal kind extra result
  local -a inherited_fds=()
  if [[ $close_fds != - ]]; then
    IFS=, read -r -a inherited_fds <<<"$close_fds"
    for descriptor in "${inherited_fds[@]}"; do
      [[ $descriptor -gt 2 ]] || return 2
      eval "exec ${descriptor}>&-"
    done
  fi
  taskset -pc "$R2_DISK_WALK_CPUS" "$BASHPID" >/dev/null || return 2
  r2_read_allowed_cpu_list "/proc/$BASHPID/status" || return 2
  [[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]] || return 2
  printf 'ready\t%s\t%s\n' "$worker_index" "$BASHPID"
  while IFS=$'\t' read -r command ordinal kind extra; do
    case $command in
      sample)
        [[ $ordinal =~ ^(0|[1-9][0-9]*)$ &&
          ( $kind == scheduled || $kind == terminal ) && -z $extra ]] || return 2
        if r2_record_checkout_mib \
          "$root" "$raw_samples" "$sampler_started" "$ordinal" "$kind"; then
          result=0
        else
          result=$?
        fi
        printf 'done\t%s\t%s\t%s\n' "$worker_index" "$ordinal" "$result"
        [[ $result -eq 0 ]] || return "$result"
        ;;
      stop)
        [[ -z $ordinal && -z $kind && -z $extra ]] || return 2
        return 0
        ;;
      *) return 2 ;;
    esac
  done
  return 1
}

r2_sample_checkout_disk() {
  [[ $# -eq 6 && -d $1 && -f $2 && ! -L $2 && -n $3 &&
    -d $4 && ! -L $4 && $5 =~ ^[0-9]+$ && $6 =~ ^[1-9][0-9]*$ ]] || {
    printf 'R2 disk sampler: invalid controller arguments\n' >&2
    return 2
  }
  local root=$1 samples=$2 stop=$3 state=$4 sampler_started=$5 period_ns=$6
  local ready=$state/ready drain_request=$state/drain-request
  local drain_ready=$state/drain-ready raw_samples=$state/samples.unsorted.tsv
  # Keep the full prestarted identity pool, but admit only five exact walks at
  # once on the reference host's isolated three-group disk mask. The fifth
  # buffered walk sustains coverage through the measured full-workload tail;
  # the retained-start validator remains the final cadence authority.
  local sorted_samples=$state/samples.sorted.tsv pool_size=32 active_limit=5
  local ordinal=0 deadline now monotonic_now delay delay_seconds status=0 attempt
  local launch_status
  local active=0 initial_ready=0 draining=0 drain_remaining request_ns=''
  local bridge_ordinal='' bridge_pending=0 worker_timeout_ns=4000000000
  local stop_wait_timeout_ns=6000000000
  local drain_deadline_ns='' controller_start controller_group controller_session
  local worker_index worker_pid worker_start result_file result_fd request_fd
  local close_fds tag result_index result_ordinal result_status extra
  local next_worker=0 available_worker=-1 wait_deadline wait_now
  local pool_started=0 stat_status
  local -a worker_pids=() worker_starts=() worker_results=()
  local -a worker_result_fds=() worker_request_fds=() worker_jobs=()
  local -a worker_ready=() worker_reaped=() inherited_fds=()
  local -A drain_jobs=()

  r2_disk_deadline_ns "$sampler_started" 0 "$period_ns" || return
  r2_read_process_stat "/proc/$BASHPID/stat" || return 2
  controller_start=$R2_PROC_START
  controller_group=$R2_PROC_GROUP
  controller_session=$R2_PROC_SESSION
  [[ -z $(find "$state" -mindepth 1 -print -quit) ]] || return 2
  : >"$raw_samples"
  [[ -f $raw_samples && ! -L $raw_samples ]] || return 2

  pool_worker_process_stable() {
    [[ $# -eq 1 && $1 =~ ^(0|[1-9][0-9]*)$ && $1 -lt $pool_started ]] || return 2
    local index=$1 pid=${worker_pids[$1]} start=${worker_starts[$1]}
    r2_read_process_stat "/proc/$pid/stat" &&
      [[ $R2_PROC_START == "$start" && $R2_PROC_PARENT == "$BASHPID" &&
        $R2_PROC_GROUP == "$controller_group" &&
        $R2_PROC_SESSION == "$controller_session" && $R2_PROC_STATE != Z ]]
  }

  pool_worker_identity_stable() {
    pool_worker_process_stable "$1" &&
      r2_read_allowed_cpu_list "/proc/${worker_pids[$1]}/status" &&
      [[ $R2_EXPANDED_CPU_LIST == "$R2_DISK_WALK_CPUS" ]]
  }

  close_pool_fd() {
    [[ $# -eq 1 && $1 =~ ^[1-9][0-9]*$ ]] || return 2
    eval "exec ${1}>&-"
  }

  abort_dedicated_sampler_group() {
    if r2_read_process_stat "/proc/$BASHPID/stat" &&
      [[ $R2_PROC_START == "$controller_start" && $R2_PROC_GROUP == "$BASHPID" &&
        $R2_PROC_SESSION == "$BASHPID" && $R2_PROC_STATE != Z ]]; then
      kill -KILL -- "-$BASHPID"
    fi
    return 1
  }

  start_worker_pool() {
    local index ready_count=0 ready_deadline ready_now
    for ((index = 0; index < pool_size; index += 1)); do
      result_file=$state/worker-$index.results
      : >"$result_file" || return 1
      [[ -f $result_file && ! -L $result_file ]] || return 2
      exec {result_fd}<"$result_file" || return 1
      inherited_fds=("${worker_request_fds[@]}" "${worker_result_fds[@]}" "$result_fd")
      printf -v close_fds '%s,' "${inherited_fds[@]}"
      close_fds=${close_fds%,}
      if ! exec {request_fd}> >(
        r2_disk_pool_worker "$index" "$root" "$raw_samples" \
          "$sampler_started" "$close_fds" >>"$result_file"
      ); then
        close_pool_fd "$result_fd" || true
        printf 'R2 disk sampler: worker channel launch failed\n' >&2
        abort_dedicated_sampler_group || true
        return 1
      fi
      worker_pid=$!
      # Register every launched child and owned descriptor before attempting
      # identity capture. If /proc cannot bind that live child, the only safe
      # cleanup authority is the controller's already-verified dedicated group.
      worker_pids[index]=$worker_pid
      worker_starts[index]=''
      worker_results[index]=$result_file
      worker_result_fds[index]=$result_fd
      worker_request_fds[index]=$request_fd
      worker_jobs[index]=''
      worker_ready[index]=0
      worker_reaped[index]=0
      pool_started=$((pool_started + 1))
      worker_start=
      for ((attempt = 0; attempt < 100; attempt += 1)); do
        if r2_read_process_stat "/proc/$worker_pid/stat"; then
          if [[ $R2_PROC_PARENT == "$BASHPID" &&
            $R2_PROC_GROUP == "$controller_group" &&
            $R2_PROC_SESSION == "$controller_session" &&
            $R2_PROC_STATE != Z ]]; then
            worker_start=$R2_PROC_START
            break
          fi
          stat_status=2
          break
        fi
        stat_status=$?
        [[ $stat_status -eq 1 && -e /proc/$worker_pid ]] || break
        if ! sleep 0.001; then
          printf 'R2 disk sampler: worker identity capture wait failed\n' >&2
          abort_dedicated_sampler_group || true
          return 1
        fi
      done
      if [[ ! $worker_start =~ ^[0-9]+$ ]]; then
        printf 'R2 disk sampler: launched worker identity could not be bound\n' >&2
        abort_dedicated_sampler_group || true
        return 1
      fi
      worker_starts[index]=$worker_start
    done

    r2_monotonic_now_ns || return 1
    r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 "$worker_timeout_ns" || return 2
    ready_deadline=$R2_DISK_DEADLINE_NS
    while [[ $ready_count -lt $pool_size ]]; do
      for ((index = 0; index < pool_size; index += 1)); do
        [[ ${worker_ready[$index]} -eq 0 ]] || continue
        if IFS=$'\t' read -r -u "${worker_result_fds[$index]}" \
          tag result_index worker_pid extra; then
          [[ $tag == ready && $result_index == "$index" &&
            $worker_pid == "${worker_pids[$index]}" && -z $extra ]] || return 2
          pool_worker_identity_stable "$index" || return 1
          worker_ready[index]=1
          ready_count=$((ready_count + 1))
        fi
      done
      [[ $ready_count -lt $pool_size ]] || break
      r2_monotonic_now_ns || return 1
      ready_now=$R2_MONOTONIC_NS
      [[ $ready_now -lt $ready_deadline ]] || return 1
      sleep 0.005 || return 1
    done
  }

  collect_finished_samples() {
    local index job
    for ((index = 0; index < pool_started; index += 1)); do
      job=${worker_jobs[$index]}
      [[ -n $job ]] || continue
      if IFS=$'\t' read -r -u "${worker_result_fds[$index]}" \
        tag result_index result_ordinal result_status extra; then
        [[ $tag == "done" && $result_index == "$index" &&
          $result_ordinal == "$job" &&
          $result_status =~ ^(0|[1-9][0-9]*)$ && $result_status -le 255 &&
          -z $extra ]] || { status=1; return 1; }
        worker_jobs[index]=''
        active=$((active - 1))
        unset 'drain_jobs[$job]'
        if [[ $result_status -eq 0 ]]; then
          pool_worker_identity_stable "$index" || { status=1; return 1; }
        else
          status=1
        fi
      elif ! pool_worker_process_stable "$index"; then
        status=1
        return 1
      fi
    done
    [[ $status -eq 0 ]]
  }

  find_available_worker() {
    local offset index
    available_worker=-1
    for ((offset = 0; offset < pool_size; offset += 1)); do
      index=$(( (next_worker + offset) % pool_size ))
      [[ -z ${worker_jobs[$index]} ]] || continue
      pool_worker_identity_stable "$index" || { status=1; return 1; }
      available_worker=$index
      next_worker=$(( (index + 1) % pool_size ))
      return 0
    done
    return 1
  }

  scheduled_launch_boundary() {
    [[ $# -eq 1 && ( $1 == scheduled || $1 == terminal ) ]] || return 2
    [[ $1 == scheduled ]] || return 0
    if [[ $ordinal -ne 0 && ( -e $stop || -L $stop ) ]]; then
      return 3
    fi
    [[ $draining -eq 1 ]] || return 0
    [[ $drain_deadline_ns =~ ^(0|[1-9][0-9]*)$ ]] || return 2
    r2_monotonic_now_ns || {
      status=1
      abort_dedicated_sampler_group || true
      return 1
    }
    if [[ $R2_MONOTONIC_NS -ge $drain_deadline_ns ]]; then
      status=1
      printf 'R2 disk sampler: drain deadline expired before scheduled launch\n' >&2
      abort_dedicated_sampler_group || true
      return 1
    fi
  }

  wait_for_launch_slot() {
    [[ $# -eq 1 && ( $1 == scheduled || $1 == terminal ) ]] || return 2
    local kind=$1 boundary_status slot_deadline slot_now
    [[ $active -lt $active_limit ]] && return 0
    r2_monotonic_now_ns || {
      status=1
      abort_dedicated_sampler_group || true
      return 1
    }
    r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 "$worker_timeout_ns" || {
      status=1
      abort_dedicated_sampler_group || true
      return 1
    }
    slot_deadline=$R2_DISK_DEADLINE_NS
    while [[ $active -ge $active_limit ]]; do
      collect_finished_samples || return 1
      if scheduled_launch_boundary "$kind"; then boundary_status=0
      else boundary_status=$?; fi
      [[ $boundary_status -eq 0 ]] || return "$boundary_status"
      [[ $active -ge $active_limit ]] || return 0
      r2_monotonic_now_ns || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
      slot_now=$R2_MONOTONIC_NS
      [[ $slot_now -lt $slot_deadline ]] || {
        status=1
        printf 'R2 disk sampler: five concurrent du walks did not make room before timeout\n' >&2
        abort_dedicated_sampler_group || true
        return 1
      }
      sleep 0.005 || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
    done
  }

  launch_sample() {
    [[ $# -eq 1 && ( $1 == scheduled || $1 == terminal ) ]] || return 2
    local launch_kind=$1 boundary_status slot_status
    collect_finished_samples || return
    if wait_for_launch_slot "$launch_kind"; then slot_status=0
    else slot_status=$?; fi
    [[ $slot_status -eq 0 ]] || return "$slot_status"
    [[ $active -lt $active_limit && $active -lt $pool_size ]] || return 2
    find_available_worker || return
    [[ $available_worker -ge 0 ]] || return 2
    if scheduled_launch_boundary "$launch_kind"; then boundary_status=0
    else boundary_status=$?; fi
    [[ $boundary_status -eq 0 ]] || return "$boundary_status"
    printf 'sample\t%s\t%s\n' "$ordinal" "$launch_kind" \
      >&"${worker_request_fds[$available_worker]}" || { status=1; return 1; }
    worker_jobs[available_worker]=$ordinal
    active=$((active + 1))
    ordinal=$((ordinal + 1))
  }

  wait_for_sample_set() {
    [[ $# -le 1 ]] || return 2
    wait_deadline=${1:-}
    if [[ -n $wait_deadline ]]; then
      r2_disk_deadline_ns "$wait_deadline" 0 1 || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
    else
      r2_monotonic_now_ns || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
      wait_now=$R2_MONOTONIC_NS
      r2_disk_deadline_ns "$wait_now" 1 "$worker_timeout_ns" || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
      wait_deadline=$R2_DISK_DEADLINE_NS
    fi
    while :; do
      collect_finished_samples || true
      [[ $active -ne 0 ]] || { [[ $status -eq 0 ]]; return; }
      r2_monotonic_now_ns || { status=1; break; }
      wait_now=$R2_MONOTONIC_NS
      [[ $wait_now -lt $wait_deadline ]] || break
      sleep 0.005 || { status=1; break; }
    done
    status=1
    printf 'R2 disk sampler: sample workers did not close before timeout\n' >&2
    abort_dedicated_sampler_group || true
    return 1
  }

  stop_worker_pool() {
    local index live=0 pool_deadline pool_now wait_result
    [[ $active -eq 0 ]] || return 2
    for ((index = 0; index < pool_started; index += 1)); do
      if pool_worker_identity_stable "$index"; then
        printf 'stop\n' >&"${worker_request_fds[$index]}" || status=1
      else
        status=1
      fi
      close_pool_fd "${worker_request_fds[$index]}" || status=1
    done
    r2_monotonic_now_ns || {
      status=1
      abort_dedicated_sampler_group || true
      return 1
    }
    r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 "$worker_timeout_ns" || {
      status=1
      abort_dedicated_sampler_group || true
      return 1
    }
    pool_deadline=$R2_DISK_DEADLINE_NS
    while :; do
      live=0
      for ((index = 0; index < pool_started; index += 1)); do
        [[ ${worker_reaped[$index]} -eq 0 ]] || continue
        worker_pid=${worker_pids[$index]}
        worker_start=${worker_starts[$index]}
        if pool_worker_identity_stable "$index"; then
          live=$((live + 1))
          continue
        fi
        # The worker may have exited between the full identity snapshot and
        # this reaping snapshot. A still-live recorded process is an affinity
        # or identity failure; a matching zombie or vanished direct child is
        # safe to wait without turning normal exit into a race-dependent red.
        if r2_read_process_stat "/proc/$worker_pid/stat"; then
          if [[ $R2_PROC_START == "$worker_start" &&
            $R2_PROC_PARENT == "$BASHPID" &&
            $R2_PROC_GROUP == "$controller_group" &&
            $R2_PROC_SESSION == "$controller_session" ]]; then
            if [[ $R2_PROC_STATE != Z ]]; then
              status=1
              live=$((live + 1))
              continue
            fi
          else
            status=1
            printf 'R2 disk sampler: live worker identity changed during shutdown\n' >&2
            abort_dedicated_sampler_group || true
            return 1
          fi
        else
          stat_status=$?
          if [[ $stat_status -ne 1 || -e /proc/$worker_pid ]] ||
            kill -0 "$worker_pid" 2>/dev/null; then
            status=1
            live=$((live + 1))
            continue
          fi
        fi
        if wait "$worker_pid"; then wait_result=0; else wait_result=$?; fi
        [[ $wait_result -eq 0 ]] || status=1
        worker_reaped[index]=1
      done
      [[ $live -ne 0 ]] || break
      r2_monotonic_now_ns || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
      pool_now=$R2_MONOTONIC_NS
      [[ $pool_now -lt $pool_deadline ]] || {
        status=1
        printf 'R2 disk sampler: worker pool did not close before timeout\n' >&2
        abort_dedicated_sampler_group || true
        return 1
      }
      sleep 0.005 || {
        status=1
        abort_dedicated_sampler_group || true
        return 1
      }
    done
    for ((index = 0; index < pool_started; index += 1)); do
      close_pool_fd "${worker_result_fds[$index]}" || status=1
      result_file=${worker_results[$index]}
      [[ -f $result_file && ! -L $result_file ]] || { status=1; continue; }
      find "$result_file" -delete || status=1
    done
    [[ $status -eq 0 ]]
  }

  initial_sample_retained() {
    local candidate
    while IFS=$'\t' read -r candidate _; do
      [[ $candidate != 0 ]] || return 0
    done <"$raw_samples"
    return 1
  }

  if ! start_worker_pool; then
    status=1
    stop_worker_pool || true
    printf 'R2 disk sampler: worker pool did not become ready\n' >&2
    return 1
  fi
  launch_sample scheduled || status=1
  while [[ $status -eq 0 ]]; do
    collect_finished_samples || {
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
        if ! now=$(date +%s%N) || ! r2_disk_deadline_ns "$now" 0 1 ||
          [[ $request_ns -gt $now ]] || ! r2_monotonic_now_ns; then
          status=1
          break
        fi
        monotonic_now=$R2_MONOTONIC_NS
        if ! r2_disk_deadline_ns "$monotonic_now" 1 "$worker_timeout_ns"; then
          status=1
          break
        fi
        drain_deadline_ns=$R2_DISK_DEADLINE_NS
        bridge_pending=1
        for worker_index in "${!worker_jobs[@]}"; do
          [[ -z ${worker_jobs[$worker_index]} ]] ||
            drain_jobs["${worker_jobs[$worker_index]}"]=1
        done
      elif [[ $R2_CONTROL_MARKER != "$request_ns" ]]; then
        status=1
        break
      fi
      drain_remaining=${#drain_jobs[@]}
      [[ $drain_remaining -ne 0 || $bridge_pending -eq 1 ]] || break
    fi
    if ! r2_disk_deadline_ns "$sampler_started" "$ordinal" "$period_ns"; then
      status=1
      break
    fi
    deadline=$R2_DISK_DEADLINE_NS
    if ! now=$(date +%s%N) || ! r2_disk_deadline_ns "$now" 0 1; then
      status=1
      break
    fi
    if [[ $draining -eq 1 ]]; then
      if ! r2_monotonic_now_ns || [[ $R2_MONOTONIC_NS -ge $drain_deadline_ns ]]; then
        status=1
        break
      fi
    fi
    if [[ $deadline -gt $now ]]; then
      delay=$((deadline - now))
      printf -v delay_seconds '%d.%09d' \
        "$((delay / 1000000000))" "$((delay % 1000000000))"
      sleep "$delay_seconds" || { status=1; break; }
    fi
    if [[ $draining -eq 1 ]]; then
      if ! r2_monotonic_now_ns || [[ $R2_MONOTONIC_NS -ge $drain_deadline_ns ]]; then
        status=1
        break
      fi
    fi
    if [[ ! -e $stop && ! -L $stop ]]; then
      if [[ $draining -eq 1 && $bridge_pending -eq 1 ]]; then
        bridge_ordinal=$ordinal
      fi
      if launch_sample scheduled; then launch_status=0
      else launch_status=$?; fi
      if [[ $launch_status -eq 3 ]]; then continue; fi
      [[ $launch_status -eq 0 ]] || { status=1; break; }
      [[ $draining -ne 1 || $bridge_pending -ne 1 ]] || bridge_pending=0
    fi
  done

  wait_for_sample_set "$drain_deadline_ns" || status=1
  if [[ $status -eq 0 && $draining -eq 1 ]]; then
    [[ ! -e $stop && ! -L $stop && ! -e $drain_ready && ! -L $drain_ready &&
      $bridge_ordinal =~ ^(0|[1-9][0-9]*)$ ]] || status=1
    [[ $status -ne 0 ]] || r2_validate_disk_drain_handoff \
      "$raw_samples" "$sorted_samples" "$sampler_started" "$ordinal" \
      "$request_ns" "$bridge_ordinal" "$((period_ns + period_ns / 2))" || status=1
    [[ $status -ne 0 ]] || r2_publish_decimal_control_marker \
      "$drain_ready" "$request_ns" || status=1
    if [[ $status -eq 0 ]] && r2_wait_for_disk_stop_marker \
      "$stop" "$drain_request" "$request_ns" "$R2_DISK_HANDOFF_LATEST_NS" \
      "$stop_wait_timeout_ns"; then
      :
    else
      status=1
    fi
  elif [[ $status -eq 0 ]]; then
    r2_read_decimal_control_marker "$stop" || status=1
  fi

  [[ $status -ne 0 ]] || launch_sample terminal || status=1
  wait_for_sample_set || status=1
  stop_worker_pool || status=1
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

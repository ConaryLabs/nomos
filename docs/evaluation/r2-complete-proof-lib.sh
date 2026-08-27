#!/usr/bin/env bash

# Source-only primitives shared by the complete proof and its refusal suite.

r2_measure_process_closure() {
  [[ $# -ge 3 && $# -le 4 && $1 == net:\[*\] && $2 =~ ^[0-9a-f]{64}$ && -n $3 ]] || {
    printf 'R2 process closure: invalid arguments\n' >&2
    return 2
  }
  local expected_namespace=$1
  local proof_token=$2
  local report=$3
  local allowed_root=${4:-}
  local ancestor=$$
  local ancestor_pids=" $$ "
  local proc pid process_namespace parent allowed
  local strict_namespace=0
  local -a process_snapshot=(/proc/[0-9]*)

  [[ -z $allowed_root || $allowed_root =~ ^[0-9]+$ ]] || return 2
  if [[ ${NOMOS_R2_HOST_NETNS:-} == net:\[*\] &&
        $expected_namespace != "$NOMOS_R2_HOST_NETNS" ]]; then
    strict_namespace=1
  fi

  : >"$report"
  while [[ $ancestor -gt 1 && -r /proc/$ancestor/status ]]; do
    ancestor=$(awk '/^PPid:/ { print $2; exit }' "/proc/$ancestor/status")
    [[ $ancestor =~ ^[0-9]+$ ]] || break
    ancestor_pids+="$ancestor "
  done
  for proc in "${process_snapshot[@]}"; do
    pid=${proc##*/}
    [[ " $ancestor_pids " == *" $pid "* ]] && continue
    process_namespace=$(readlink "$proc/ns/net" 2>/dev/null || true)
    [[ $process_namespace == "$expected_namespace" ]] || continue

    parent=$pid
    allowed=0
    while [[ -n $allowed_root && $parent -gt 1 && -r /proc/$parent/status ]]; do
      [[ $parent -ne $allowed_root ]] || { allowed=1; break; }
      parent=$(awk '/^PPid:/ { print $2; exit }' "/proc/$parent/status")
      [[ $parent =~ ^[0-9]+$ ]] || break
    done
    [[ $allowed -eq 0 ]] || continue

    if [[ $strict_namespace -eq 0 ]] &&
      ! grep -Fzx -- "NOMOS_R2_PROOF_TOKEN=$proof_token" "$proc/environ" \
        >/dev/null 2>&1; then
      parent=$pid
      while [[ $parent -gt 1 && -r /proc/$parent/status ]]; do
        parent=$(awk '/^PPid:/ { print $2; exit }' "/proc/$parent/status")
        [[ $parent =~ ^[0-9]+$ ]] || break
        [[ $parent -ne $$ ]] || break
      done
      [[ $parent -eq $$ ]] || continue
    fi
    printf '%s\n' "$pid" >>"$report"
  done
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

r2_measure_checkout_mib() {
  [[ $# -eq 1 && -d $1 ]] || {
    printf 'R2 disk sampler: invalid checkout root\n' >&2
    return 2
  }
  local root=$1
  local attempt started raw size
  started=$(date +%s%N) || return 2
  for ((attempt = 0; attempt < 20; attempt += 1)); do
    # Cargo atomically publishes and removes intermediate files while `du`
    # walks the checkout. Retain only a complete, successful `du -sm` result;
    # a raced walk is not a sample and is retried immediately. The caller's
    # retained start timestamps still enforce the contract's maximum gap.
    if raw=$(du -sm -- "$root" 2>/dev/null); then
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
  [[ $# -eq 5 && -d $1 && -d $2 && $3 =~ ^[0-9]+$ &&
    $4 =~ ^[0-9]+$ && $5 =~ ^[0-9]+$ && $5 -ge $3 ]] || {
    printf 'R2 disk sampler: invalid sample arguments\n' >&2
    return 2
  }
  local root=$1
  local parts=$2
  local sampler_started=$3
  local ordinal=$4
  local sample_started=$5
  local row measured_started size elapsed part
  row=$(r2_measure_checkout_mib "$root") || return
  IFS=$'\t' read -r measured_started size <<<"$row"
  [[ $measured_started =~ ^[0-9]+$ && $size =~ ^[0-9]+$ ]] || return 2
  elapsed=$(( (sample_started - sampler_started) / 1000000 ))
  printf -v part '%s/%08d.tsv' "$parts" "$ordinal"
  [[ ! -e $part && ! -L $part ]] || return 2
  printf '%s\t%s\t%s\n' "$ordinal" "$elapsed" "$size" >"$part"
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
  local parts=$4
  local sampler_started=$5
  local period_ns=$6
  local ready=$parts/ready
  local ordinal=0 deadline now delay delay_seconds sample_started pid status=0 index part line
  local last_started=0
  local -a sample_pids=()

  launch_sample() {
    sample_started=$(date +%s%N) || return 2
    [[ $sample_started -gt $last_started ]] || return 2
    r2_record_checkout_mib \
      "$root" "$parts" "$sampler_started" "$ordinal" "$sample_started" &
    pid=$!
    sample_pids+=("$pid")
    last_started=$sample_started
    ordinal=$((ordinal + 1))
  }

  launch_sample || return
  : >"$ready"
  while [[ ! -e $stop ]]; do
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
    if [[ ! -e $stop ]] && ! launch_sample; then
      status=1
      break
    fi
  done

  # The terminal row begins only after the stop marker exists and is ordered
  # after the last scheduled launch even at millisecond receipt resolution.
  if [[ $status -eq 0 ]]; then
    while :; do
      if ! now=$(date +%s%N); then
        status=1
        break
      fi
      [[ $((now / 1000000)) -gt $((last_started / 1000000)) ]] && break
      if ! sleep 0.001; then
        status=1
        break
      fi
    done
    if [[ $status -eq 0 ]] && ! launch_sample; then
      status=1
    fi
  fi

  for pid in "${sample_pids[@]}"; do
    wait "$pid" || status=1
  done
  [[ $status -eq 0 ]] || {
    printf 'R2 disk sampler: one or more scheduled samples failed\n' >&2
    return 1
  }

  find "$ready" -delete
  for ((index = 0; index < ordinal; index += 1)); do
    printf -v part '%s/%08d.tsv' "$parts" "$index"
    [[ -f $part && ! -L $part ]] || return 2
    IFS= read -r line <"$part"
    [[ $line == "$index"$'\t'* ]] || return 2
    printf '%s\n' "$line" >>"$samples"
    find "$part" -delete
  done
  [[ -z $(find "$parts" -mindepth 1 -print -quit) ]] || return 2
  find "$parts" -depth -delete
}

r2_network_probe() {
  [[ $# -eq 2 ]] || return 2
  node -e 'const net=require("node:net");const [host,port]=process.argv.slice(1);let done=false;const socket=net.connect({host,port:Number(port)});const timer=setTimeout(()=>finish(24,"blocked: timeout\n"),3000);function finish(code,text){if(done)return;done=true;clearTimeout(timer);socket.destroy();(code===0?process.stdout:process.stderr).write(text,()=>process.exit(code));}socket.once("connect",()=>finish(0,"connected\n"));socket.once("error",error=>finish(23,"blocked: "+(error.code||error.message)+"\n"));' "$1" "$2"
}

#!/usr/bin/env bash

# Timing, control publication, and ledger validation/finalization for the R2
# disk sampler. This file is sourced by r2-complete-proof-lib.sh.

r2_disk_deadline_ns() {
  [[ $# -eq 3 && $1 =~ ^(0|[1-9][0-9]*)$ &&
    $2 =~ ^(0|[1-9][0-9]*)$ && $3 =~ ^[1-9][0-9]*$ ]] || return 2
  local origin=$1 ordinal=$2 period=$3 deadline value
  local maximum=9223372036854775807
  for value in "$origin" "$ordinal" "$period"; do
    # shellcheck disable=SC2071 # Compare equal-length canonical decimals lexically.
    if [[ ${#value} -gt ${#maximum} ||
        ( ${#value} -eq ${#maximum} && $value > "$maximum" ) ]]; then
      return 2
    fi
  done
  if [[ $ordinal -gt 0 && $period -gt $(( (maximum - origin) / ordinal )) ]]; then
    return 2
  fi
  deadline=$((origin + ordinal * period))
  # shellcheck disable=SC2034 # Returned global avoids a fork per sample.
  R2_DISK_DEADLINE_NS=$deadline
}

r2_disk_interleaved_deadline_ns() {
  [[ $# -eq 3 && $1 =~ ^(0|[1-9][0-9]*)$ &&
    $2 =~ ^(0|[1-9][0-9]*)$ && $3 =~ ^[1-9][0-9]*$ ]] || return 2
  local origin=$1 ordinal=$2 period=$3 cycle phase half_period deadline value
  local maximum=9223372036854775807
  for value in "$ordinal" "$period"; do
    # shellcheck disable=SC2071 # Compare equal-length canonical decimals lexically.
    if [[ ${#value} -gt ${#maximum} ||
        ( ${#value} -eq ${#maximum} && $value > "$maximum" ) ]]; then
      return 2
    fi
  done
  [[ $((period % 2)) -eq 0 ]] || return 2
  cycle=$((ordinal / 2))
  phase=$((ordinal % 2))
  half_period=$((period / 2))
  r2_disk_deadline_ns "$origin" "$cycle" "$period" || return
  deadline=$R2_DISK_DEADLINE_NS
  if [[ $phase -eq 1 ]]; then
    [[ $deadline -le $((maximum - half_period)) ]] || return 2
    deadline=$((deadline + half_period))
  fi
  # shellcheck disable=SC2034 # Returned global avoids a fork per sample.
  R2_DISK_DEADLINE_NS=$deadline
}

# shellcheck disable=SC2120 # This public helper explicitly rejects arguments.
r2_monotonic_now_ns() {
  [[ $# -eq 0 ]] || return 2
  local uptime idle extra='' seconds fraction
  { IFS=' ' read -r uptime idle extra && [[ -z $extra ]]; } </proc/uptime || return 1
  [[ $uptime =~ ^(0|[1-9][0-9]*)\.([0-9][0-9])$ ]] || return 2
  seconds=${BASH_REMATCH[1]}
  fraction=${BASH_REMATCH[2]}
  [[ $idle =~ ^(0|[1-9][0-9]*)\.[0-9][0-9]$ ]] || return 2
  # Compare equal-length canonical decimals lexically.
  # shellcheck disable=SC2071
  [[ ${#seconds} -lt 10 ||
    ( ${#seconds} -eq 10 && $seconds < 9223372036 ) ]] || return 2
  # shellcheck disable=SC2034 # Returned global is consumed by controller waits.
  R2_MONOTONIC_NS=$((10#$seconds * 1000000000 + 10#$fraction * 10000000))
}

r2_decimal_fits_signed_i64() {
  [[ $# -eq 1 && $1 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  local value=$1 maximum=9223372036854775807
  # Compare equal-length canonical decimals lexically.
  # shellcheck disable=SC2071
  [[ ${#value} -lt ${#maximum} ||
    ( ${#value} -eq ${#maximum} && ( $value < "$maximum" || $value == "$maximum" ) ) ]]
}

r2_read_decimal_control_marker() {
  [[ $# -eq 1 && -f $1 && ! -L $1 ]] || return 2
  local line extra=''
  { IFS= read -r line && ! IFS= read -r extra && [[ -z $extra ]]; } <"$1" || return 2
  r2_decimal_fits_signed_i64 "$line" || return 2
  # shellcheck disable=SC2034 # Returned global is consumed by protocol peers.
  R2_CONTROL_MARKER=$line
}

r2_publish_decimal_control_marker() {
  [[ $# -eq 2 && $1 == */* && $1 != *$'\n'* && $1 != *$'\t'* &&
    $2 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  local marker=$1 value=$2 parent leaf temporary result=0
  r2_decimal_fits_signed_i64 "$value" || return 2
  parent=${marker%/*}
  leaf=${marker##*/}
  [[ -d $parent && ! -L $parent && $leaf =~ ^[A-Za-z0-9._-]+$ &&
    ! -e $marker && ! -L $marker ]] || return 2
  temporary=$parent/.$leaf.$BASHPID.publish
  [[ ! -e $temporary && ! -L $temporary ]] || return 2
  (
    set -e
    trap 'result=$?; find "$temporary" -maxdepth 0 -type f -delete 2>/dev/null || true; exit "$result"' EXIT
    trap 'exit 1' HUP INT TERM
    umask 077
    printf '%s\n' "$value" >"$temporary"
    [[ -f $temporary && ! -L $temporary ]]
    mv -T -n -- "$temporary" "$marker"
    [[ ! -e $temporary ]]
    trap - EXIT HUP INT TERM
  ) || return 1
  r2_read_decimal_control_marker "$marker" && [[ $R2_CONTROL_MARKER == "$value" ]]
}

r2_wait_for_disk_stop_marker() {
  [[ $# -eq 5 && $1 == */* && $1 != *$'\n'* && $1 != *$'\t'* &&
    -f $2 && ! -L $2 && $3 =~ ^(0|[1-9][0-9]*)$ &&
    $4 =~ ^(0|[1-9][0-9]*)$ && $5 =~ ^[1-9][0-9]*$ ]] || return 2
  local stop=$1 request_file=$2 request_ns=$3 latest_ns=$4 timeout_ns=$5
  local stop_ns deadline now
  r2_decimal_fits_signed_i64 "$request_ns" || return 2
  r2_decimal_fits_signed_i64 "$latest_ns" || return 2
  r2_monotonic_now_ns || return 1
  r2_disk_deadline_ns "$R2_MONOTONIC_NS" 1 "$timeout_ns" || return 2
  deadline=$R2_DISK_DEADLINE_NS
  while :; do
    r2_read_decimal_control_marker "$request_file" || return 1
    [[ $R2_CONTROL_MARKER == "$request_ns" ]] || return 1
    if [[ -e $stop || -L $stop ]]; then
      r2_read_decimal_control_marker "$stop" || return 1
      stop_ns=$R2_CONTROL_MARKER
      [[ $stop_ns -ge $request_ns && $stop_ns -ge $latest_ns &&
        $((stop_ns - latest_ns)) -le 100000000 ]] || return 1
      # shellcheck disable=SC2034 # Returned global binds the terminal row.
      R2_DISK_STOP_NS=$stop_ns
      return 0
    fi
    r2_monotonic_now_ns || return 1
    now=$R2_MONOTONIC_NS
    [[ $now -lt $deadline ]] || {
      printf 'R2 disk sampler: stop marker did not arrive before timeout\n' >&2
      return 1
    }
    sleep 0.001 || return 1
  done
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

r2_validate_disk_drain_handoff() {
  [[ $# -eq 7 && -f $1 && ! -L $1 && ! -e $2 && ! -L $2 &&
    $3 =~ ^(0|[1-9][0-9]*)$ && $4 =~ ^[1-9][0-9]*$ &&
    $5 =~ ^(0|[1-9][0-9]*)$ && $6 =~ ^(0|[1-9][0-9]*)$ &&
    $6 -lt $4 && $7 =~ ^[1-9][0-9]*$ ]] || return 2
  local raw=$1 sorted=$2 origin=$3 expected=$4 request=$5 bridge=$6 freshness=$7
  local latest now validation_status
  LC_ALL=C sort -t $'\t' -k2,2n -k1,1n "$raw" >"$sorted" || return 1
  [[ -f $sorted && ! -L $sorted && $(wc -l <"$sorted") -eq $expected ]] || return 2
  # The handoff freshness window includes validation work. A Bash row loop
  # takes hundreds of milliseconds once a proof has accumulated thousands of
  # samples, so validate the already-sorted ledger in one awk process. Absolute
  # nanoseconds exceed IEEE-754 integer precision: the helpers below compare
  # and add canonical decimals as strings, while small row counts are the only
  # values converted to awk numbers.
  if latest=$(LC_ALL=C awk -F $'\t' \
    -v origin="$origin" -v expected="$expected" \
    -v request="$request" -v bridge="$bridge" '
    function canonical(value) {
      return value ~ /^(0|[1-9][0-9]*)$/
    }
    function decimal_compare(left, right) {
      if (length(left) != length(right))
        return length(left) < length(right) ? -1 : 1
      # Prefix both operands so awk cannot classify canonical decimal fields
      # as numeric strings and compare them through an imprecise double.
      if (("x" left) == ("x" right)) return 0
      return ("x" left) < ("x" right) ? -1 : 1
    }
    function decimal_add(left, right, output, carry, digit, i, j, l, r) {
      output = ""
      carry = 0
      i = length(left)
      j = length(right)
      while (i > 0 || j > 0 || carry > 0) {
        l = i > 0 ? substr(left, i, 1) + 0 : 0
        r = j > 0 ? substr(right, j, 1) + 0 : 0
        digit = l + r + carry
        output = (digit % 10) output
        carry = int(digit / 10)
        i -= 1
        j -= 1
      }
      sub(/^0+/, "", output)
      return output == "" ? "0" : output
    }
    function refuse(code) {
      failure = code
      exit code
    }
    BEGIN {
      if (!canonical(origin) || !canonical(expected) || expected == "0" ||
          !canonical(request) || !canonical(bridge)) refuse(2)
    }
    {
      if (NF != 5 || !canonical($1) || !canonical($2) ||
          !canonical($3) || !canonical($4) || $5 != "scheduled" ||
          decimal_compare($2, origin) < 0 ||
          decimal_compare(decimal_add(origin, $3), $2) != 0 ||
          seen[$1]) refuse(2)
      seen[$1] = 1
      seen_count += 1
      if (row_count > 0 &&
          (decimal_compare($2, previous_started) <= 0 ||
           decimal_compare($3, previous_elapsed) <= 0 ||
           decimal_compare($3,
             decimal_add(previous_elapsed, "100000000")) > 0)) refuse(1)
      if ($1 == bridge) {
        bridge_seen = 1
        bridge_started = $2
      }
      previous_started = $2
      previous_elapsed = $3
      latest = $2
      row_count += 1
    }
    END {
      if (failure) exit failure
      if (row_count != expected + 0 || seen_count != row_count ||
          !bridge_seen || decimal_compare(bridge_started, request) < 0)
        exit 2
      for (ordinal_index = 0; ordinal_index < expected + 0; ordinal_index += 1)
        if (!((ordinal_index "") in seen)) exit 2
      print latest
    }
  ' "$sorted"); then
    validation_status=0
  else
    validation_status=$?
  fi
  [[ $validation_status -eq 0 ]] || return "$validation_status"
  [[ $latest =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  now=$(date +%s%N) || return 2
  [[ $now =~ ^(0|[1-9][0-9]*)$ && $now -ge $latest &&
    $((now - latest)) -le $freshness ]] || return 1
  find "$sorted" -delete || return 1
  # shellcheck disable=SC2034 # Returned global binds the stop handoff window.
  R2_DISK_HANDOFF_LATEST_NS=$latest
}

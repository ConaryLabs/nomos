#!/usr/bin/env bash

# Control-marker publication and pre-stop ledger validation for the R2 disk
# sampler. This file is sourced by r2-complete-proof-lib.sh.

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

r2_read_decimal_control_marker() {
  [[ $# -eq 1 && -f $1 && ! -L $1 ]] || return 2
  local line extra=''
  { IFS= read -r line && ! IFS= read -r extra && [[ -z $extra ]]; } <"$1" || return 2
  [[ $line =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  # shellcheck disable=SC2034 # Returned global is consumed by protocol peers.
  R2_CONTROL_MARKER=$line
}

r2_publish_decimal_control_marker() {
  [[ $# -eq 2 && $1 == */* && $1 != *$'\n'* && $1 != *$'\t'* &&
    $2 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  local marker=$1 value=$2 parent leaf temporary result=0
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

r2_validate_disk_drain_handoff() {
  [[ $# -eq 7 && -f $1 && ! -L $1 && ! -e $2 && ! -L $2 &&
    $3 =~ ^(0|[1-9][0-9]*)$ && $4 =~ ^[1-9][0-9]*$ &&
    $5 =~ ^(0|[1-9][0-9]*)$ && $6 =~ ^(0|[1-9][0-9]*)$ &&
    $6 -lt $4 && $7 =~ ^[1-9][0-9]*$ ]] || return 2
  local raw=$1 sorted=$2 origin=$3 expected=$4 request=$5 bridge=$6 freshness=$7
  local index=0 line ordinal started elapsed size kind extra previous=0
  local bridge_started='' latest=0 now
  local -A seen=()
  LC_ALL=C sort -t $'\t' -k2,2n -k1,1n "$raw" >"$sorted" || return 1
  [[ -f $sorted && ! -L $sorted && $(wc -l <"$sorted") -eq $expected ]] || return 2
  while IFS= read -r line; do
    IFS=$'\t' read -r ordinal started elapsed size kind extra <<<"$line"
    [[ $ordinal =~ ^(0|[1-9][0-9]*)$ && $ordinal -lt $expected &&
      $started =~ ^(0|[1-9][0-9]*)$ && $started -ge $origin &&
      $elapsed =~ ^(0|[1-9][0-9]*)$ && $elapsed -eq $((started - origin)) &&
      $size =~ ^(0|[1-9][0-9]*)$ && $kind == scheduled && -z $extra &&
      -z ${seen[$ordinal]+present} ]] || return 2
    seen["$ordinal"]=1
    [[ $index -eq 0 || ( $started -gt $previous &&
      $((started - previous)) -le 100000000 ) ]] || return 1
    [[ $ordinal -ne $bridge ]] || bridge_started=$started
    previous=$started
    latest=$started
    index=$((index + 1))
  done <"$sorted"
  [[ $index -eq $expected && ${#seen[@]} -eq $expected &&
    $bridge_started =~ ^[0-9]+$ && $bridge_started -ge $request ]] || return 2
  now=$(date +%s%N) || return 2
  [[ $now =~ ^(0|[1-9][0-9]*)$ && $now -ge $latest &&
    $((now - latest)) -le $freshness ]] || return 1
  find "$sorted" -delete || return 1
  # shellcheck disable=SC2034 # Returned global binds the stop handoff window.
  R2_DISK_HANDOFF_LATEST_NS=$latest
}

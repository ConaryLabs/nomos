#!/usr/bin/env bash

# Small fail-closed control primitives shared by the R2 proof parent and its
# persistent filesystem sampler protocol.

r2_control_deadline_ns() {
  [[ $# -eq 3 && $1 =~ ^(0|[1-9][0-9]*)$ &&
    $2 =~ ^(0|[1-9][0-9]*)$ && $3 =~ ^[1-9][0-9]*$ ]] || return 2
  local origin=$1 ordinal=$2 period=$3 deadline value
  local maximum=9223372036854775807
  for value in "$origin" "$ordinal" "$period"; do
    # shellcheck disable=SC2071 # Equal-length canonical decimals compare lexically.
    if [[ ${#value} -gt ${#maximum} ||
        ( ${#value} -eq ${#maximum} && $value > "$maximum" ) ]]; then
      return 2
    fi
  done
  if [[ $ordinal -gt 0 && $period -gt $(( (maximum - origin) / ordinal )) ]]; then
    return 2
  fi
  deadline=$((origin + ordinal * period))
  # shellcheck disable=SC2034 # Returned global avoids a fork in bounded waits.
  R2_CONTROL_DEADLINE_NS=$deadline
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
  # shellcheck disable=SC2071 # Equal-length canonical decimals compare lexically.
  [[ ${#seconds} -lt 10 ||
    ( ${#seconds} -eq 10 && $seconds < 9223372036 ) ]] || return 2
  # shellcheck disable=SC2034 # Returned global is consumed by control peers.
  R2_MONOTONIC_NS=$((10#$seconds * 1000000000 + 10#$fraction * 10000000))
}

r2_decimal_fits_signed_i64() {
  [[ $# -eq 1 && $1 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  local value=$1 maximum=9223372036854775807
  # shellcheck disable=SC2071 # Equal-length canonical decimals compare lexically.
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
    # A same-directory hard link is an atomic create-if-absent publication.
    # Unlike a prechecked rename, it cannot replace a marker won by a racing
    # publisher between the freshness check and this operation.
    ln -- "$temporary" "$marker"
    find "$temporary" -maxdepth 0 -type f -delete
    [[ ! -e $temporary ]]
    trap - EXIT HUP INT TERM
  ) || return 1
  r2_read_decimal_control_marker "$marker" && [[ $R2_CONTROL_MARKER == "$value" ]]
}

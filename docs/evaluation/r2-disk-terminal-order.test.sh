#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'R2 disk terminal-order plant: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
# shellcheck source=docs/evaluation/r2-complete-proof-lib.sh
source "$script_directory/r2-complete-proof-lib.sh"

r2_disk_deadline_ns 1000000000 1 50000000
phase_one=$R2_DISK_DEADLINE_NS
r2_disk_deadline_ns 1000000000 2 50000000
phase_two=$R2_DISK_DEADLINE_NS
r2_disk_deadline_ns 1000000000 3 50000000
phase_three=$R2_DISK_DEADLINE_NS
[[ $phase_one -eq 1025000000 && $phase_two -eq 1050000000 &&
  $phase_three -eq 1075000000 ]] || fail 'interleaved absolute deadlines differ'
if r2_disk_deadline_ns 1000000000 1 50000001; then
  fail 'odd nominal period was accepted'
fi

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-disk-terminal-order.XXXXXX")
cleanup() {
  case $temporary in
    "$repo_root"/target/r2-disk-terminal-order.*)
      [[ ! -e $temporary ]] || find "$temporary" -depth -delete
      ;;
    *) fail "refusing unsafe cleanup path: $temporary" ;;
  esac
}
trap cleanup EXIT

samples=$temporary/samples.tsv
stop=$temporary/stop
state=$temporary/state
origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$samples"
mkdir "$state"

# Ordinal 1 acknowledges its launch immediately, then simulates a raced walk
# whose successful retry starts after ordinal 3 has requested the stop. The
# terminal measurement must wait for that outstanding scheduled row.
(
  r2_record_checkout_mib() {
    [[ $# -eq 6 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 signal=$6 started
    started=$(date +%s%N)
    if [[ $ordinal -eq 1 ]]; then
      printf '%s\t%s\n' "$ordinal" "$started" >"$signal"
      sleep 0.12
      started=$(date +%s%N)
    fi
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    [[ $ordinal -ne 3 ]] || : >"$stop"
    [[ $ordinal -eq 1 ]] || printf '%s\t%s\n' "$ordinal" "$started" >"$signal"
  }
  r2_sample_checkout_disk "$repo_root" "$samples" "$stop" "$state" \
    "$origin" 50000000
) || fail 'sampler did not quiesce scheduled retries before its terminal row'

[[ ! -e $state && $(wc -l <"$samples") -ge 6 ]] ||
  fail 'sampler did not publish the complete planted ledger'
awk -F '\t' '
  NR > 1 && $1 == 1 { delayed = $3 }
  NR > 1 && $1 == 3 { stopped = $3 }
  END { exit !(delayed > stopped && $5 == "terminal") }
' "$samples" || fail 'terminal row is not chronologically last after a retry'

printf 'R2_DISK_TERMINAL_ORDER_PLANT PASS\n'

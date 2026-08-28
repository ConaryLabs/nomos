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
[[ $phase_one -eq 1050000000 && $phase_two -eq 1100000000 &&
  $phase_three -eq 1150000000 ]] || fail 'absolute 50 ms deadlines differ'
r2_disk_deadline_ns 1000000000 1 50000001
[[ $R2_DISK_DEADLINE_NS -eq 1050000001 ]] || fail 'an odd positive period moved'
if r2_disk_deadline_ns 1000000000 1 0 ||
  r2_disk_deadline_ns 9223372036854775800 1 100 ||
  r2_disk_deadline_ns 9223372036854775808 0 1 ||
  r2_disk_deadline_ns 0 9223372036854775807 2; then
  fail 'a zero, out-of-range, or overflowing nominal period was accepted'
fi

interleaved_deadlines=()
for ordinal in 0 1 2 3 4 5; do
  r2_disk_interleaved_deadline_ns 1000000000 "$ordinal" 50000000
  interleaved_deadlines+=("$R2_DISK_DEADLINE_NS")
done
[[ ${interleaved_deadlines[*]} == \
  '1000000000 1025000000 1050000000 1075000000 1100000000 1125000000' &&
  $((interleaved_deadlines[2] - interleaved_deadlines[0])) -eq 50000000 &&
  $((interleaved_deadlines[3] - interleaved_deadlines[1])) -eq 50000000 ]] ||
  fail 'interleaved absolute 50 ms phase deadlines differ'
if r2_disk_interleaved_deadline_ns 1000000000 1 50000001 ||
  r2_disk_interleaved_deadline_ns 1000000000 1 0 ||
  r2_disk_interleaved_deadline_ns 9223372036854775800 1 100 ||
  r2_disk_interleaved_deadline_ns 0 9223372036854775807 50000000; then
  fail 'an odd, zero, out-of-range, or overflowing interleaved period was accepted'
fi

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-disk-terminal-order.XXXXXX")

topology_root=$temporary/topology
write_sibling_group() {
  [[ $# -eq 2 && $1 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  mkdir -p "$topology_root/cpu$1/topology"
  printf '%s\n' "$2" >"$topology_root/cpu$1/topology/thread_siblings_list"
}
for pair in '0 6' '1 7' '2 8' '3 9' '4 10' '5 11'; do
  read -r first second <<<"$pair"
  write_sibling_group "$first" "$first,$second"
  write_sibling_group "$second" "$first,$second"
done
r2_partition_cpu_topology 0-11 "$topology_root" ||
  fail 'canonical sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 0 && $R2_DISK_CPUS == 0,1,2,6,7,8 &&
  $R2_WORKLOAD_CPUS == 3,4,5,9,10,11 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,6;1,7;2,8;3,9;4,10;5,11' &&
  $R2_DISK_PHYSICAL_GROUPS == '0,6;1,7;2,8' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '3,9;4,10;5,11' ]] ||
  fail 'canonical physical-core role split differs'
r2_validate_physical_cpu_isolation "$R2_DISK_CPUS" "$R2_WORKLOAD_CPUS" \
  "$topology_root" || fail 'canonical physical-core role split overlaps'
if r2_validate_physical_cpu_isolation 0-5 6-11 "$topology_root"; then
  fail 'the former SMT-overlapping role split was accepted'
fi
r2_partition_cpu_topology '1,3-4,7,9-10' "$topology_root" ||
  fail 'irregular complete sibling topology was refused'
[[ $R2_CONTROLLER_CPUS == 1 && $R2_DISK_CPUS == 1,7 &&
  $R2_WORKLOAD_CPUS == 3,4,9,10 ]] || fail 'irregular physical-core split differs'
if r2_partition_cpu_topology 0,6 "$topology_root" ||
  r2_partition_cpu_topology 0,12 "$topology_root"; then
  fail 'an undersized or unreadable topology was accepted'
fi
printf '6\n' >"$topology_root/cpu6/topology/thread_siblings_list"
if r2_partition_cpu_topology 0,6 "$topology_root"; then
  fail 'contradictory sibling topology was accepted'
fi
write_sibling_group 6 0,6
write_sibling_group 0 0,6,99
write_sibling_group 1 1,7,99
if r2_partition_cpu_topology 0,1 "$topology_root"; then
  fail 'sibling groups overlapping outside the allowed mask were accepted'
fi
write_sibling_group 0 0,6
write_sibling_group 1 1,7

# Exercise the real controller against a deterministic clock. Each worker adds
# 7 ms of planted bookkeeping before acknowledging its launch. The two
# absolute 50 ms phases therefore remain +0/+25/+50/+75 ms; relative sleeps
# would drift.
absolute_samples=$temporary/absolute-samples.tsv
absolute_stop=$temporary/absolute-stop
absolute_state=$temporary/absolute-state
absolute_clock=$temporary/absolute-clock
absolute_trace=$temporary/absolute-trace
absolute_origin=1000000000
printf '%s\n' "$absolute_origin" >"$absolute_clock"
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$absolute_samples"
mkdir "$absolute_state"
(
  date() {
    [[ $# -eq 1 && $1 == +%s%N ]] || return 2
    local current
    IFS= read -r current <"$absolute_clock"
    printf '%s\n' "$current"
  }
  sleep() {
    local current delta
    if [[ $# -eq 1 && $1 =~ ^([0-9]+)\.([0-9]{9})$ ]]; then
      current=$(<"$absolute_clock")
      delta=$((10#${BASH_REMATCH[1]} * 1000000000 + 10#${BASH_REMATCH[2]}))
      printf '%s\n' "$((current + delta))" >"$absolute_clock"
    else
      command sleep "$@"
    fi
  }
  r2_record_checkout_mib() {
    [[ $# -eq 6 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 signal=$6 started
    started=$(<"$absolute_clock")
    printf '%s\t%s\t%s\n' "$ordinal" "$kind" "$started" >>"$absolute_trace"
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    [[ $ordinal -ne 3 ]] || printf '%s\n' "$started" >"$absolute_stop"
    printf '%s\n' "$((started + 7000000))" >"$absolute_clock"
    printf '%s\t%s\n' "$ordinal" "$started" >"$signal"
  }
  r2_sample_checkout_disk "$repo_root" "$absolute_samples" "$absolute_stop" \
    "$absolute_state" "$absolute_origin" 50000000
) || fail 'deterministic absolute-schedule controller trace was refused'
[[ $(sed -n '1,4p' "$absolute_trace") == \
  $'0\tscheduled\t1000000000\n1\tscheduled\t1025000000\n2\tscheduled\t1050000000\n3\tscheduled\t1075000000' &&
  $(awk -F '\t' 'END { print $2 }' "$absolute_trace") == terminal &&
  $(<"$absolute_stop") == 1075000000 && ! -e $absolute_state ]] ||
  fail 'controller launches are not on the interleaved absolute 50 ms phases'

samples=$temporary/samples.tsv
stop=$temporary/stop
state=$temporary/state
origin=$(date +%s%N)
printf 'ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n' >"$samples"
mkdir "$state"

# Ordinal 1 acknowledges its launch immediately, then waits for ordinal 3's
# stop marker before retaining its planted successful-retry timestamp. Fixed
# clock values keep the two exact 50 ms phases away from the 100 ms gap
# boundary. The terminal measurement must wait for that outstanding scheduled
# row.
(
  r2_record_checkout_mib() {
    [[ $# -eq 6 ]] || return 2
    local raw=$2 sampler_origin=$3 ordinal=$4 kind=$5 signal=$6 started
    started=$((sampler_origin + ordinal * 25000000))
    if [[ $ordinal -eq 1 ]]; then
      printf '%s\t%s\n' "$ordinal" "$started" >"$signal"
      while [[ ! -e $stop ]]; do sleep 0.001; done
      started=$((sampler_origin + 87500000))
    fi
    printf '%s\t%s\t%s\t17\t%s\n' \
      "$ordinal" "$started" "$((started - sampler_origin))" "$kind" >>"$raw"
    if [[ $ordinal -eq 3 ]]; then
      printf '%s\n' "$started" >"$stop"
    fi
    [[ $ordinal -eq 1 ]] || printf '%s\t%s\n' "$ordinal" "$started" >"$signal"
  }
  r2_sample_checkout_disk "$repo_root" "$samples" "$stop" "$state" \
    "$origin" 50000000
) || fail 'sampler did not quiesce scheduled retries before its terminal row'

[[ ! -e $state && $(wc -l <"$samples") -ge 6 ]] ||
  fail 'sampler did not publish the complete planted ledger'
stop_requested=$(<"$stop")
[[ $stop_requested =~ ^(0|[1-9][0-9]*)$ ]] || fail 'stop marker is not canonical'
ordinal_three_started=$(awk -F '\t' '$1 == 3 { print $2 }' "$samples")
[[ $stop_requested == "$ordinal_three_started" &&
  $stop_requested -eq $((origin + 75000000)) ]] ||
  fail 'stop marker differs from ordinal 3'
awk -F '\t' -v origin="$origin" -v stop="$stop_requested" '
  NR > 1 {
    if (previous != "" && ($2 <= previous || $2 - previous > 100000000)) bad = 1
    previous = $2
  }
  NR > 1 && $1 == 1 { delayed = $3 }
  NR > 1 && $1 == 3 { stopped = $3 }
  NR > 1 && $5 == "scheduled" && $2 > scheduled_max { scheduled_max = $2 }
  NR > 1 && $5 == "terminal" { terminal_count += 1; terminal = $2 }
  END {
    exit !(bad == 0 && delayed == 87500000 && stopped == 75000000 &&
      terminal_count == 1 && terminal == origin + 100000000 &&
      delayed > stopped && terminal > scheduled_max && terminal >= stop)
  }
' "$samples" || fail 'terminal row is not uniquely last and after the stop marker'

printf 'R2_DISK_TERMINAL_ORDER_PLANT PASS\n'
printf 'retained_fixture %s\n' "${temporary#"$repo_root/"}"

#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'R2 complete proof control plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
# shellcheck source=docs/evaluation/r2-complete-proof-lib.sh
source "$script_directory/r2-complete-proof-lib.sh"

for command in find ln mkdir mktemp mv rm sleep wc; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

mkdir -p "$repo_root/target"
temporary=$(mktemp -d "$repo_root/target/r2-complete-proof-control.XXXXXX")
atomic_publisher=
cleanup() {
  if [[ -n ${atomic_publisher:-} ]]; then
    kill "$atomic_publisher" 2>/dev/null || true
    wait "$atomic_publisher" 2>/dev/null || true
  fi
  case $temporary in
    "$repo_root"/target/r2-complete-proof-control.*)
      rm -r -- "$temporary"
      ;;
    *)
      printf 'R2 complete proof control plants: refusing unsafe cleanup path: %s\n' \
        "$temporary" >&2
      ;;
  esac
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Deadlines are absolute arithmetic, not a polling-count convention. The
# signed-64-bit bound is part of the control protocol and must fail closed.
r2_control_deadline_ns 1000000000 1 50000000
phase_one=$R2_CONTROL_DEADLINE_NS
r2_control_deadline_ns 1000000000 2 50000000
phase_two=$R2_CONTROL_DEADLINE_NS
r2_control_deadline_ns 1000000000 3 50000000
phase_three=$R2_CONTROL_DEADLINE_NS
[[ $phase_one -eq 1050000000 && $phase_two -eq 1100000000 &&
  $phase_three -eq 1150000000 ]] || fail 'absolute 50 ms deadlines differ'
r2_control_deadline_ns 1000000000 1 50000001
[[ $R2_CONTROL_DEADLINE_NS -eq 1050000001 ]] || fail 'an odd positive period moved'
if r2_control_deadline_ns 1000000000 1 0 ||
  r2_control_deadline_ns 9223372036854775800 1 100 ||
  r2_control_deadline_ns 9223372036854775808 0 1 ||
  r2_control_deadline_ns 0 9223372036854775807 2; then
  fail 'a zero, out-of-range, or overflowing deadline was accepted'
fi

for valid_decimal in 0 42 9223372036854775807; do
  r2_decimal_fits_signed_i64 "$valid_decimal" ||
    fail "canonical signed-64 decimal was refused: $valid_decimal"
done
for invalid_decimal in 00 042 -1 +1 9223372036854775808; do
  if r2_decimal_fits_signed_i64 "$invalid_decimal"; then
    fail "non-canonical or out-of-range decimal was accepted: $invalid_decimal"
  fi
done

marker_directory=$temporary/markers
mkdir "$marker_directory"
marker=$marker_directory/control
r2_publish_decimal_control_marker "$marker" 42 ||
  fail 'canonical control marker publication failed'
r2_read_decimal_control_marker "$marker" ||
  fail 'canonical control marker read failed'
[[ $R2_CONTROL_MARKER == 42 && $(wc -c <"$marker") -eq 3 &&
  $(<"$marker") == 42 ]] ||
  fail 'canonical control marker bytes differ'
if r2_publish_decimal_control_marker "$marker" 43; then
  fail 'control marker publication overwrote an existing destination'
fi
printf '42' >"$marker_directory/no-final-line-feed"
printf '42\n43\n' >"$marker_directory/extra-line"
printf '042\n' >"$marker_directory/leading-zero"
printf -- '-1\n' >"$marker_directory/negative"
for malformed_marker in no-final-line-feed extra-line leading-zero negative; do
  if r2_read_decimal_control_marker "$marker_directory/$malformed_marker"; then
    fail "malformed control marker was accepted: $malformed_marker"
  fi
done
printf '9223372036854775807\n' >"$marker_directory/max"
r2_read_decimal_control_marker "$marker_directory/max" ||
  fail 'maximum signed-64 control marker was refused'
[[ $R2_CONTROL_MARKER == 9223372036854775807 ]] ||
  fail 'maximum signed-64 control marker changed during read'
if r2_publish_decimal_control_marker \
  "$marker_directory/too-large" 9223372036854775808; then
  fail 'out-of-range control marker was published'
fi
[[ ! -e $marker_directory/too-large ]] ||
  fail 'out-of-range control marker left a destination'
ln -s control "$marker_directory/symlink"
if r2_read_decimal_control_marker "$marker_directory/symlink" ||
  r2_publish_decimal_control_marker "$marker_directory/symlink" 43; then
  fail 'control marker symlink was accepted'
fi

# Hold the helper's final link. The prepared file may be read by a diagnostic
# but the destination must remain absent until the publication is released.
atomic_directory=$temporary/atomic-marker
atomic_marker=$atomic_directory/marker
atomic_observed=$atomic_directory/link-observed
atomic_release=$atomic_directory/release
mkdir "$atomic_directory"
(
  # shellcheck disable=SC2329 # Invoked indirectly by the sourced publisher.
  ln() {
    : >"$atomic_observed"
    while [[ ! -e $atomic_release ]]; do command sleep 0.001; done
    command ln "$@"
  }
  r2_publish_decimal_control_marker "$atomic_marker" 123456789
) &
atomic_publisher=$!
atomic_temporary=
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $atomic_observed ]] || break
  kill -0 "$atomic_publisher" 2>/dev/null ||
    fail 'atomic marker publisher exited before its held link'
  sleep 0.001
done
atomic_temporary=$(find "$atomic_directory" -maxdepth 1 -type f \
  -name '.marker.*.publish' -print -quit)
[[ -f $atomic_observed && ! -e $atomic_marker && -f $atomic_temporary ]] ||
  fail 'control marker became visible before its atomic link'
r2_read_decimal_control_marker "$atomic_temporary" ||
  fail 'prepared control-marker bytes are not canonical'
[[ $R2_CONTROL_MARKER == 123456789 ]] ||
  fail 'prepared control-marker value differs'
: >"$atomic_release"
wait "$atomic_publisher" || fail 'atomic marker publication failed'
atomic_publisher=
r2_read_decimal_control_marker "$atomic_marker" ||
  fail 'published marker is not canonical'
[[ $R2_CONTROL_MARKER == 123456789 ]] ||
  fail 'published control-marker value differs'

# A competing destination created after the helper's freshness check must win;
# the held publisher must fail without replacing it or retaining its temporary.
race_directory=$temporary/racing-marker
race_marker=$race_directory/marker
race_observed=$race_directory/link-observed
race_release=$race_directory/release
mkdir "$race_directory"
(
  # shellcheck disable=SC2329 # Invoked indirectly by the sourced publisher.
  ln() {
    : >"$race_observed"
    while [[ ! -e $race_release ]]; do command sleep 0.001; done
    command ln "$@"
  }
  r2_publish_decimal_control_marker "$race_marker" 111
) 2>"$race_directory/publisher.stderr" &
atomic_publisher=$!
for ((attempt = 0; attempt < 100; attempt += 1)); do
  [[ ! -e $race_observed ]] || break
  kill -0 "$atomic_publisher" 2>/dev/null ||
    fail 'racing marker publisher exited before its held link'
  sleep 0.001
done
[[ -f $race_observed && ! -e $race_marker ]] ||
  fail 'racing marker publisher did not reach the atomic link'
printf '222\n' >"$race_marker"
: >"$race_release"
if wait "$atomic_publisher"; then
  fail 'racing control marker publication replaced its competitor'
fi
atomic_publisher=
r2_read_decimal_control_marker "$race_marker" ||
  fail 'competing control marker became malformed'
[[ $R2_CONTROL_MARKER == 222 ]] ||
  fail 'racing publisher replaced the competing control marker'
[[ -s $race_directory/publisher.stderr ]] ||
  fail 'failed racing publication retained no refusal diagnostic'
[[ -z $(find "$race_directory" -maxdepth 1 -type f \
  -name '.marker.*.publish' -print -quit) ]] ||
  fail 'failed racing publication retained its temporary file'

# Use a synthetic procfs topology to keep the two-role physical-isolation
# contract testable on hosts with one visible CPU or no SMT metadata.
topology_root=$temporary/topology
write_sibling_group() {
  [[ $# -eq 2 && $1 =~ ^(0|[1-9][0-9]*)$ ]] || return 2
  mkdir -p "$topology_root/cpu$1/topology"
  printf '%s\n' "$2" >"$topology_root/cpu$1/topology/thread_siblings_list"
}
for pair in '0 2' '1 3'; do
  read -r first second <<<"$pair"
  write_sibling_group "$first" "$first,$second"
  write_sibling_group "$second" "$first,$second"
done
r2_partition_cpu_topology 0-3 "$topology_root" ||
  fail 'canonical two-role sibling topology was refused'
[[ $R2_SAMPLER_CPUS == 0,2 && $R2_WORKLOAD_CPUS == 1,3 &&
  $R2_CPU_TOPOLOGY_GROUPS == '0,2;1,3' &&
  $R2_SAMPLER_PHYSICAL_GROUPS == '0,2' &&
  $R2_WORKLOAD_PHYSICAL_GROUPS == '1,3' ]] ||
  fail 'canonical two-role physical-core split differs'
r2_validate_physical_cpu_isolation 0,2 1,3 "$topology_root" ||
  fail 'canonical physical-core role split overlaps'
if r2_validate_physical_cpu_isolation 0,1 2,3 "$topology_root"; then
  fail 'an SMT-overlapping role split was accepted'
fi
write_sibling_group 0 0,2,99
if r2_partition_cpu_topology 0-3 "$topology_root"; then
  fail 'a sibling group extending outside the allowed mask was accepted'
fi

printf 'R2_COMPLETE_PROOF_CONTROL_PLANTS PASS\n'

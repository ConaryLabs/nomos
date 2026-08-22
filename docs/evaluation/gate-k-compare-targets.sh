#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k cross-target: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 4 ]] || fail 'usage: gate-k-compare-targets.sh <x86-debug> <x86-release> <arm-release> <receipt-dir>'
x86_debug=$1
x86_release=$2
arm_release=$3
receipt_dir=$4

for directory in "$x86_debug" "$x86_release" "$arm_release"; do
  [[ -d $directory/semantic ]] || fail "semantic baseline absent: $directory"
  [[ -f $directory/receipt.txt ]] || fail "target receipt absent: $directory"
  grep -Fx 'GATE_K_DETERMINISM PASS' "$directory/receipt.txt" >/dev/null ||
    fail "target receipt is not passing: $directory"
done
[[ ! -e $receipt_dir ]] || fail "cross-target receipt destination exists: $receipt_dir"
mkdir -p "$receipt_dir"

commit=$(awk '$1 == "commit" { print $2 }' "$x86_debug/receipt.txt")
[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail 'x86_64 debug receipt has no full commit'
for directory in "$x86_release" "$arm_release"; do
  candidate=$(awk '$1 == "commit" { print $2 }' "$directory/receipt.txt")
  [[ $candidate == "$commit" ]] || fail 'target receipts name different commits'
done

grep -Fx 'lane x86_64-debug' "$x86_debug/receipt.txt" >/dev/null ||
  fail 'debug receipt has the wrong lane'
grep -Fx 'lane x86_64-release' "$x86_release/receipt.txt" >/dev/null ||
  fail 'x86_64 release receipt has the wrong lane'
grep -Fx 'lane aarch64-release' "$arm_release/receipt.txt" >/dev/null ||
  fail 'aarch64 release receipt has the wrong lane'
grep -Fx 'profile debug' "$x86_debug/receipt.txt" >/dev/null ||
  fail 'x86_64 debug receipt has the wrong profile'
for directory in "$x86_release" "$arm_release"; do
  grep -Fx 'profile release' "$directory/receipt.txt" >/dev/null ||
    fail "release receipt has the wrong profile: $directory"
done
for directory in "$x86_debug" "$x86_release" "$arm_release"; do
  grep -Fx 'executions 10' "$directory/receipt.txt" >/dev/null ||
    fail "target receipt does not record ten executions: $directory"
done

diff -qr "$x86_debug/semantic" "$x86_release/semantic" >/dev/null ||
  fail 'x86_64 debug and release semantic artifacts differ'
diff -qr "$x86_debug/semantic" "$arm_release/semantic" >/dev/null ||
  fail 'x86_64 and aarch64 semantic artifacts differ'
diff -u "$x86_debug/semantic.sha256" "$x86_release/semantic.sha256" >/dev/null ||
  fail 'x86_64 debug and release semantic digests differ'
diff -u "$x86_debug/semantic.sha256" "$arm_release/semantic.sha256" >/dev/null ||
  fail 'x86_64 and aarch64 semantic digests differ'

cp "$x86_debug/semantic.sha256" "$receipt_dir/semantic.sha256"
for lane in x86_64-debug x86_64-release aarch64-release; do
  case $lane in
    x86_64-debug) source_dir=$x86_debug ;;
    x86_64-release) source_dir=$x86_release ;;
    aarch64-release) source_dir=$arm_release ;;
  esac
  receipt_name=$lane.receipt.txt
  cp "$source_dir/receipt.txt" "$receipt_dir/$receipt_name"
  (cd "$receipt_dir" && sha256sum "$receipt_name") >>"$receipt_dir/source-receipts.sha256"
done
(cd "$receipt_dir" && sha256sum -c source-receipts.sha256 >/dev/null) ||
  fail 'copied target receipts do not verify against their checksum table'

{
  printf 'GATE_K_CROSS_TARGET PASS\n'
  printf 'commit %s\n' "$commit"
  printf 'lanes x86_64-debug x86_64-release aarch64-release\n'
  printf 'executions_per_lane 10\n'
  printf 'semantic_artifacts_byte_identical yes\n'
  printf 'semantic_digest_table_sha256 %s\n' "$(sha256sum "$receipt_dir/semantic.sha256" | cut -d' ' -f1)"
} >"$receipt_dir/receipt.txt"

printf 'gate-k cross-target: PASS\n'

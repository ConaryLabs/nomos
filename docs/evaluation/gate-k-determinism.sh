#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k determinism: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 2 ]] || fail 'usage: gate-k-determinism.sh <x86_64-debug|x86_64-release|aarch64-release> <evidence-dir>'
lane=$1
evidence_arg=$2

case $lane in
  x86_64-debug)
    expected_arch=x86_64
    profile=debug
    binary_rel=target/debug/nomos
    ;;
  x86_64-release)
    expected_arch=x86_64
    profile=release
    binary_rel=target/release/nomos
    ;;
  aarch64-release)
    expected_arch=aarch64
    profile=release
    binary_rel=target/release/nomos
    ;;
  *) fail "unsupported lane: $lane" ;;
esac

for command in git sha256sum diff cmp find sort xargs uname rustc cargo; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
[[ $(git -C "$repo_root" rev-parse --show-toplevel) == "$repo_root" ]] ||
  fail 'script must run from its repository worktree'
cd "$repo_root"

actual_arch=$(uname -m)
[[ $actual_arch == "$expected_arch" ]] ||
  fail "lane $lane requires $expected_arch, observed $actual_arch"

head=$(git -C "$repo_root" rev-parse --verify HEAD)
[[ $head =~ ^[0-9a-f]{40}$ ]] || fail 'HEAD is not a full commit id'
initial_status=$(git -C "$repo_root" status --porcelain=v1 --untracked-files=all)
[[ -z $initial_status ]] || fail 'tracked worktree is not clean before evidence run'

case $evidence_arg in
  /*) evidence_dir=$evidence_arg ;;
  *) evidence_dir=$repo_root/$evidence_arg ;;
esac
[[ ! -e $evidence_dir ]] || fail "evidence destination already exists: $evidence_dir"
mkdir -p "$evidence_dir/runs" "$evidence_dir/semantic"

binary=${NOMOS_BIN:-$repo_root/$binary_rel}
[[ -x $binary ]] || fail "prebuilt lane binary is absent or not executable: $binary"

record_environment() {
  {
    printf 'commit=%s\n' "$head"
    printf 'lane=%s\n' "$lane"
    printf 'profile=%s\n' "$profile"
    printf 'iterations=10\n'
    printf 'runner_arch=%s\n' "${RUNNER_ARCH:-unknown}"
    printf 'runner_image=%s\n' "${ImageOS:-unknown}"
    printf 'runner_image_version=%s\n' "${ImageVersion:-unknown}"
    printf 'uname='; uname -a
    printf 'rustc='; rustc --version
    printf 'cargo='; cargo --version
  } >"$evidence_dir/environment.txt"
  if command -v lscpu >/dev/null 2>&1; then
    lscpu >"$evidence_dir/lscpu.txt"
  else
    printf 'lscpu unavailable\n' >"$evidence_dir/lscpu.txt"
  fi
}

checksum_tree() {
  local root=$1
  local output=$2
  (
    cd "$root"
    find gaol.world gaol.run gaol.replay.run -type f -print0 |
      sort -z |
      xargs -0 sha256sum
  ) >"$output"
}

compare_run_and_replay() {
  local root=$1
  local member
  for member in initial-state.json final-state.json command-log.json \
    causal-receipts.json state-hashes.json result.json; do
    cmp "$root/gaol.run/$member" "$root/gaol.replay.run/$member" ||
      fail "ordinary run and replay differ for $member"
  done
}

record_environment

for iteration in $(seq 1 10); do
  run_name=$(printf 'run-%02d' "$iteration")
  run_dir=$evidence_dir/runs/$run_name
  mkdir "$run_dir"

  "$binary" compile fixtures/gaol.nomos \
    --out "$run_dir/gaol.world" >"$run_dir/compile.stdout"
  "$binary" run "$run_dir/gaol.world" \
    --commands fixtures/gaol.commands \
    --out "$run_dir/gaol.run" >"$run_dir/run.stdout"
  "$binary" replay "$run_dir/gaol.world" \
    --log fixtures/gaol.replay \
    --out "$run_dir/gaol.replay.run" >"$run_dir/replay.stdout"

  compare_run_and_replay "$run_dir"
  checksum_tree "$run_dir" "$run_dir/artifacts.sha256"

  if [[ $iteration -eq 1 ]]; then
    cp -R "$run_dir/gaol.world" "$evidence_dir/semantic/gaol.world"
    cp -R "$run_dir/gaol.run" "$evidence_dir/semantic/gaol.run"
    cp "$run_dir/artifacts.sha256" "$evidence_dir/baseline.sha256"
  else
    diff -qr "$evidence_dir/semantic/gaol.world" "$run_dir/gaol.world" >/dev/null ||
      fail "$run_name world package differs from run-01"
    diff -qr "$evidence_dir/semantic/gaol.run" "$run_dir/gaol.run" >/dev/null ||
      fail "$run_name run bundle differs from run-01"
    diff -u "$evidence_dir/baseline.sha256" "$run_dir/artifacts.sha256" >/dev/null ||
      fail "$run_name artifact digests differ from run-01"
  fi
done

(
  cd "$evidence_dir/semantic"
  find gaol.world gaol.run -type f -print0 | sort -z | xargs -0 sha256sum
) >"$evidence_dir/semantic.sha256"

{
  printf 'GATE_K_DETERMINISM PASS\n'
  printf 'commit %s\n' "$head"
  printf 'lane %s\n' "$lane"
  printf 'profile %s\n' "$profile"
  printf 'executions 10\n'
  printf 'package_manifest_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.world/manifest.json" | cut -d' ' -f1)"
  printf 'world_ir_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.world/world-ir.json" | cut -d' ' -f1)"
  printf 'simulation_semantics_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.world/simulation.json" | cut -d' ' -f1)"
  printf 'state_hashes_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.run/state-hashes.json" | cut -d' ' -f1)"
  printf 'command_log_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.run/command-log.json" | cut -d' ' -f1)"
  printf 'causal_receipts_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.run/causal-receipts.json" | cut -d' ' -f1)"
  printf 'final_state_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.run/final-state.json" | cut -d' ' -f1)"
  printf 'result_sha256 %s\n' "$(sha256sum "$evidence_dir/semantic/gaol.run/result.json" | cut -d' ' -f1)"
  printf 'ordinary_replay_byte_identical yes\n'
  printf 'within_lane_byte_identical yes\n'
} >"$evidence_dir/receipt.txt"

cp "$evidence_dir/semantic/gaol.run/state-hashes.json" "$evidence_dir/state-hashes.json"

final_status=$(git -C "$repo_root" status --porcelain=v1 --untracked-files=all)
[[ -z $final_status ]] || fail 'tracked worktree changed during evidence run'
[[ $(git -C "$repo_root" rev-parse --verify HEAD) == "$head" ]] ||
  fail 'HEAD changed during evidence run'

printf 'gate-k determinism: PASS: %s\n' "$lane"

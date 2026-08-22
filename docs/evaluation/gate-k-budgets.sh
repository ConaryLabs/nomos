#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'gate-k budgets: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail 'usage: gate-k-budgets.sh <evidence-dir>'
evidence_arg=$1

for command in git cargo rustc awk sort du date grep sha256sum /usr/bin/time; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
[[ $(git -C "$repo_root" rev-parse --show-toplevel) == "$repo_root" ]] ||
  fail 'script must run from its repository worktree'
cd "$repo_root"

head=$(git -C "$repo_root" rev-parse --verify HEAD)
[[ $head =~ ^[0-9a-f]{40}$ ]] || fail 'HEAD is not a full commit id'
[[ -z $(git -C "$repo_root" status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'tracked worktree is not clean before budget run'

case $evidence_arg in
  /*) evidence_dir=$evidence_arg ;;
  *) evidence_dir=$repo_root/$evidence_arg ;;
esac
[[ ! -e $evidence_dir ]] || fail "evidence destination already exists: $evidence_dir"
mkdir -p "$evidence_dir/build" "$evidence_dir/operations" "$evidence_dir/inputs"

build_target=$repo_root/target/gate-k-budget-build
[[ ! -e $build_target ]] || fail "clean build target already exists: $build_target"

warmups=3
samples=20
disk_interval_seconds=0.05

{
  printf 'commit=%s\n' "$head"
  printf 'runner_arch=%s\n' "${RUNNER_ARCH:-unknown}"
  printf 'runner_image=%s\n' "${ImageOS:-unknown}"
  printf 'runner_image_version=%s\n' "${ImageVersion:-unknown}"
  printf 'warmups=%s\n' "$warmups"
  printf 'measured_samples=%s\n' "$samples"
  printf 'disk_sample_interval_seconds=%s\n' "$disk_interval_seconds"
  printf 'uname='; uname -a
  printf 'rustc='; rustc --version
  printf 'cargo='; cargo --version
  printf 'gnu_time='; /usr/bin/time --version | head -1
  printf 'cargo_home_kb=%s\n' "$(du -sk "${CARGO_HOME:-$HOME/.cargo}" 2>/dev/null | awk '{print $1}' || printf 'unavailable')"
} >"$evidence_dir/environment.txt"
if command -v lscpu >/dev/null 2>&1; then
  lscpu >"$evidence_dir/lscpu.txt"
else
  printf 'lscpu unavailable\n' >"$evidence_dir/lscpu.txt"
fi

cat >"$evidence_dir/commands.txt" <<EOF
CARGO_TARGET_DIR=$build_target cargo build --workspace --release --locked
$build_target/release/nomos validate fixtures/gaol.nomos
$build_target/release/nomos command <verified-world> --state <verified-six-file-run>/final-state.json "close north_gate" --out <fresh-output>
$build_target/release/nomos replay <verified-world> --log fixtures/gaol.replay --out <fresh-output>
EOF

disk_samples=$evidence_dir/build/disk-samples.tsv
printf 'timestamp_ns\ttarget_kb\n' >"$disk_samples"
build_start_ns=$(date +%s%N)
/usr/bin/time -v -o "$evidence_dir/build/time.txt" \
  env CARGO_TARGET_DIR="$build_target" \
  cargo build --workspace --release --locked \
  >"$evidence_dir/build/stdout.txt" 2>"$evidence_dir/build/stderr.txt" &
build_pid=$!
while kill -0 "$build_pid" 2>/dev/null; do
  target_kb=$(du -sk "$build_target" 2>/dev/null | awk '{print $1}' || printf '0')
  printf '%s\t%s\n' "$(date +%s%N)" "$target_kb" >>"$disk_samples"
  sleep "$disk_interval_seconds"
done
wait "$build_pid" || fail 'clean release workspace build failed'
build_end_ns=$(date +%s%N)
final_target_kb=$(du -sk "$build_target" | awk '{print $1}')
printf '%s\t%s\n' "$build_end_ns" "$final_target_kb" >>"$disk_samples"
peak_target_kb=$(awk -F '\t' 'NR > 1 && $2 > max { max = $2 } END { print max + 0 }' "$disk_samples")
build_rss_kb=$(awk -F ': ' '/Maximum resident set size \(kbytes\)/ { print $2 }' "$evidence_dir/build/time.txt")
[[ $build_rss_kb =~ ^[0-9]+$ ]] || fail 'GNU time did not report build maximum RSS'
{
  printf 'metric\tvalue\tunit\n'
  printf 'clean_release_workspace_build\t%s\tns\n' "$((build_end_ns - build_start_ns))"
  printf 'sampled_peak_target_disk\t%s\tKiB\n' "$peak_target_kb"
  printf 'final_target_disk\t%s\tKiB\n' "$final_target_kb"
  printf 'build_max_rss\t%s\tKiB\n' "$build_rss_kb"
  printf 'disk_samples\t%s\tcount\n' "$(( $(wc -l <"$disk_samples") - 1 ))"
} >"$evidence_dir/build/summary.tsv"

binary=$build_target/release/nomos
[[ -x $binary ]] || fail 'release nomos binary is absent after build'

world=$evidence_dir/inputs/gaol.world
base_run=$evidence_dir/inputs/base.run
"$binary" compile fixtures/gaol.nomos --out "$world" \
  >"$evidence_dir/inputs/compile.stdout"
"$binary" run "$world" --commands fixtures/gaol.commands --out "$base_run" \
  >"$evidence_dir/inputs/run.stdout"

replay_commands=$(grep -o '"ordinal"' fixtures/gaol.replay | wc -l)
[[ $replay_commands -eq 5 ]] || fail "accepted replay command count changed: $replay_commands"

raw_warmups=$evidence_dir/raw-warmups.tsv
raw_samples=$evidence_dir/raw-samples.tsv
printf 'operation\tordinal\tduration_ns\tmax_rss_kb\n' >"$raw_warmups"
printf 'operation\tordinal\tduration_ns\tmax_rss_kb\n' >"$raw_samples"

measure_process() {
  local table=$1
  local operation=$2
  local ordinal=$3
  local output_stem=$4
  shift 4
  local time_file=$output_stem.time
  local start_ns end_ns rss_kb
  start_ns=$(date +%s%N)
  /usr/bin/time -f '%M' -o "$time_file" "$@" \
    >"$output_stem.stdout" 2>"$output_stem.stderr"
  end_ns=$(date +%s%N)
  rss_kb=$(tr -d '[:space:]' <"$time_file")
  [[ $rss_kb =~ ^[0-9]+$ ]] || fail "maximum RSS absent for $operation sample $ordinal"
  printf '%s\t%s\t%s\t%s\n' "$operation" "$ordinal" "$((end_ns - start_ns))" "$rss_kb" >>"$table"
}

run_phase() {
  local phase=$1
  local count=$2
  local table=$3
  local ordinal name root
  for ordinal in $(seq 1 "$count"); do
    name=$(printf '%s-%02d' "$phase" "$ordinal")

    root=$evidence_dir/operations/validate/$name
    mkdir -p "$(dirname "$root")"
    measure_process "$table" validate "$ordinal" "$root" \
      "$binary" validate fixtures/gaol.nomos

    root=$evidence_dir/operations/command/$name
    mkdir -p "$(dirname "$root")"
    measure_process "$table" command "$ordinal" "$root" \
      "$binary" command "$world" --state "$base_run/final-state.json" \
      'close north_gate' --out "$root.run"

    root=$evidence_dir/operations/replay/$name
    mkdir -p "$(dirname "$root")"
    measure_process "$table" replay "$ordinal" "$root" \
      "$binary" replay "$world" --log fixtures/gaol.replay \
      --out "$root.run"
  done
}

run_phase warmup "$warmups" "$raw_warmups"
run_phase sample "$samples" "$raw_samples"

summary=$evidence_dir/summary.tsv
printf 'operation\tminimum_ns\tmedian_ns\tp95_ns\tmaximum_ns\tmaximum_rss_kb\n' >"$summary"
for operation in validate command replay; do
  mapfile -t durations < <(awk -F '\t' -v operation="$operation" '$1 == operation { print $3 }' "$raw_samples" | sort -n)
  [[ ${#durations[@]} -eq $samples ]] || fail "$operation measured sample count is not $samples"
  minimum=${durations[0]}
  maximum=${durations[$((samples - 1))]}
  median=$(( (durations[$((samples / 2 - 1))] + durations[$((samples / 2))]) / 2 ))
  p95_index=$(( (95 * samples + 99) / 100 - 1 ))
  p95=${durations[$p95_index]}
  maximum_rss=$(awk -F '\t' -v operation="$operation" '$1 == operation && $4 > max { max = $4 } END { print max + 0 }' "$raw_samples")
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$operation" "$minimum" "$median" "$p95" "$maximum" "$maximum_rss" >>"$summary"
done

replay_total_ns=$(awk -F '\t' '$1 == "replay" { total += $3 } END { printf "%.0f", total }' "$raw_samples")
replays_per_second=$(awk -v count="$samples" -v ns="$replay_total_ns" 'BEGIN { printf "%.6f", count * 1000000000 / ns }')
commands_per_second=$(awk -v count="$samples" -v commands="$replay_commands" -v ns="$replay_total_ns" 'BEGIN { printf "%.6f", count * commands * 1000000000 / ns }')
{
  printf 'metric\tvalue\tunit\n'
  printf 'complete_replays_per_second\t%s\treplays/s\n' "$replays_per_second"
  printf 'committed_commands_per_second\t%s\tcommands/s\n' "$commands_per_second"
  printf 'commands_per_replay\t%s\tcommands\n' "$replay_commands"
  printf 'aggregate_measured_replay_time\t%s\tns\n' "$replay_total_ns"
} >"$evidence_dir/throughput.tsv"

{
  printf 'GATE_K_BUDGETS PASS\n'
  printf 'commit %s\n' "$head"
  printf 'warmups %s\n' "$warmups"
  printf 'measured_samples %s\n' "$samples"
  printf 'clean_release_build_time_ns %s\n' "$((build_end_ns - build_start_ns))"
  printf 'sampled_peak_target_disk_kib %s\n' "$peak_target_kb"
  printf 'final_target_disk_kib %s\n' "$final_target_kb"
  printf 'build_max_rss_kib %s\n' "$build_rss_kb"
  printf 'complete_replays_per_second %s\n' "$replays_per_second"
  printf 'committed_commands_per_second %s\n' "$commands_per_second"
  printf 'raw_samples_sha256 %s\n' "$(sha256sum "$raw_samples" | cut -d' ' -f1)"
  printf 'summary_sha256 %s\n' "$(sha256sum "$summary" | cut -d' ' -f1)"
} >"$evidence_dir/receipt.txt"

[[ -z $(git -C "$repo_root" status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'tracked worktree changed during budget run'
[[ $(git -C "$repo_root" rev-parse --verify HEAD) == "$head" ]] ||
  fail 'HEAD changed during budget run'

printf 'gate-k budgets: PASS\n'

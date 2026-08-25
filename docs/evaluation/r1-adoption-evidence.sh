#!/usr/bin/env bash

# Measures every RUNTIME.md section 7 row and proves the section 1 offline
# build/test/public-artifact claim. The CI caller runs this inside a network
# namespace with only loopback enabled; this script refuses a normal networked
# environment so its PASS line cannot be produced by the ordinary lanes.

set -euo pipefail
export LC_ALL=C
export CARGO_NET_OFFLINE=true

fail() {
  printf 'R1 adoption evidence: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail 'usage: r1-adoption-evidence.sh <evidence-dir>'
evidence_arg=${1#./}
[[ $evidence_arg == target/* ]] || fail 'evidence destination must be below target/'

for command in git cargo rustc node awk sort date find stat sha256sum ip /usr/bin/time; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
[[ $(git -C "$repo_root" rev-parse --show-toplevel) == "$repo_root" ]] ||
  fail 'script must run from its repository worktree'
cd "$repo_root"

head=$(git rev-parse --verify HEAD)
[[ $head =~ ^[0-9a-f]{40}$ ]] || fail 'HEAD is not a full commit id'
[[ -z $(git status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'tracked worktree is not clean before the evidence run'
[[ ${NOMOS_NETWORK_ISOLATED:-} == 1 ]] ||
  fail 'NOMOS_NETWORK_ISOLATED=1 is required from the network-namespace caller'
[[ -z $(ip -4 route show default) && -z $(ip -6 route show default) ]] ||
  fail 'the evidence process still has a default network route'
ip link show lo | grep -q 'UP' || fail 'loopback is not available for the browser proof'

evidence_dir=$repo_root/$evidence_arg
[[ ! -e $evidence_dir ]] || fail "evidence destination already exists: $evidence_dir"
for path in \
  target/gate-k-budget-build \
  target/executable-gaol \
  target/wasm32-unknown-unknown \
  apps/nomos-viewer/dist; do
  [[ ! -e $path ]] || fail "clean-checkout output already exists: $path"
done
mkdir -p "$evidence_dir/play-replay" "$evidence_dir/workspace-test"

commands=$evidence_dir/commands.txt
printf 'phase\tcommand\n' >"$commands"
record() {
  printf '%s\t%s\n' "$1" "$2" >>"$commands"
}

{
  printf 'commit=%s\n' "$head"
  printf 'runner_arch=%s\n' "${RUNNER_ARCH:-unknown}"
  printf 'runner_image=%s\n' "${ImageOS:-unknown}"
  printf 'runner_image_version=%s\n' "${ImageVersion:-unknown}"
  printf 'uname='; uname -a
  printf 'rustc='; rustc --version
  printf 'cargo='; cargo --version
  printf 'node='; node --version
  printf 'chrome='; "${CHROME_BIN:-google-chrome}" --version
  printf 'network_namespace=no-default-route; loopback-only\n'
  printf 'cargo_net_offline=%s\n' "$CARGO_NET_OFFLINE"
} >"$evidence_dir/environment.txt"

record gate-k-budgets \
  "CARGO_NET_OFFLINE=true docs/evaluation/gate-k-budgets.sh $evidence_arg/gate-k"
docs/evaluation/gate-k-budgets.sh "$evidence_arg/gate-k" \
  >"$evidence_dir/gate-k.stdout" 2>"$evidence_dir/gate-k.stderr"

build_target=$repo_root/target/gate-k-budget-build
record workspace-test \
  "CARGO_TARGET_DIR=target/gate-k-budget-build cargo test --workspace --locked --offline"
CARGO_TARGET_DIR=$build_target cargo test --workspace --locked --offline \
  >"$evidence_dir/workspace-test/stdout.txt" \
  2>"$evidence_dir/workspace-test/stderr.txt"

record viewer-tests 'node --test apps/nomos-viewer/test/*.test.mjs'
node --test apps/nomos-viewer/test/*.test.mjs \
  >"$evidence_dir/viewer-tests.stdout" \
  2>"$evidence_dir/viewer-tests.stderr"

# This is the start of the cold content-to-pixel interval. The committed source
# has conceptually just changed; no derived content, runtime, or public artifact
# exists. `smoke.mjs` closes the interval immediately after the first WebGL
# render, before it drives the rest of the route. Proof-only test time above is
# deliberately outside this product-workflow measurement.
pipeline_start_ms=$(date +%s%3N)
record content-capture \
  'CARGO_NET_OFFLINE=true experiments/executable-gaol/gaol capture'
experiments/executable-gaol/gaol capture \
  >"$evidence_dir/content-capture.stdout" \
  2>"$evidence_dir/content-capture.stderr"

record wasm-build 'crates/nomos-play/build-wasm.sh --offline'
crates/nomos-play/build-wasm.sh --offline \
  >"$evidence_dir/wasm-build.stdout" \
  2>"$evidence_dir/wasm-build.stderr"

viewer_build=$evidence_dir/viewer-build.json
record public-artifact \
  "node apps/nomos-viewer/build.mjs --from target/executable-gaol --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm --out apps/nomos-viewer/dist --receipt $evidence_arg/viewer-build.json"
node apps/nomos-viewer/build.mjs \
  --from target/executable-gaol \
  --wasm target/wasm32-unknown-unknown/wasm/nomos_play.wasm \
  --out apps/nomos-viewer/dist \
  --receipt "$viewer_build" \
  >"$evidence_dir/viewer-build.stdout" \
  2>"$evidence_dir/viewer-build.stderr"

play_binary=$build_target/release/nomos-play
[[ -x $play_binary ]] || fail 'release nomos-play was not produced by the workspace build'
smoke_dir=$evidence_dir/smoke
record browser-smoke \
  "NOMOS_PLAY_BIN=target/gate-k-budget-build/release/nomos-play node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out $evidence_arg/smoke --require-chrome --pipeline-start-ms $pipeline_start_ms"
NOMOS_PLAY_BIN=$play_binary node apps/nomos-viewer/smoke/smoke.mjs \
  --dist apps/nomos-viewer/dist \
  --out "$smoke_dir" \
  --require-chrome \
  --pipeline-start-ms "$pipeline_start_ms" \
  >"$evidence_dir/browser-smoke.stdout" \
  2>"$evidence_dir/browser-smoke.stderr"

session=$smoke_dir/session.json
areas=$repo_root/target/executable-gaol/areas
play_warmups=3
play_samples=20
play_raw=$evidence_dir/play-replay/raw-samples.tsv
printf 'phase\tordinal\tduration_ns\n' >"$play_raw"
measure_play_replay() {
  local phase=$1
  local count=$2
  local ordinal start_ns end_ns output
  for ordinal in $(seq 1 "$count"); do
    output=$evidence_dir/play-replay/$(printf '%s-%02d' "$phase" "$ordinal").stdout
    record play-replay \
      "target/gate-k-budget-build/release/nomos-play replay target/executable-gaol/areas --session $evidence_arg/smoke/session.json"
    start_ns=$(date +%s%N)
    "$play_binary" replay "$areas" --session "$session" >"$output"
    end_ns=$(date +%s%N)
    grep -q '^NOMOS_PLAY_REPLAY PASS ' "$output" ||
      fail "play replay sample $phase/$ordinal did not pass"
    printf '%s\t%s\t%s\n' "$phase" "$ordinal" "$((end_ns - start_ns))" >>"$play_raw"
  done
}
measure_play_replay warmup "$play_warmups"
measure_play_replay sample "$play_samples"

mapfile -t play_durations < <(
  awk -F '\t' '$1 == "sample" { print $3 }' "$play_raw" | sort -n
)
[[ ${#play_durations[@]} -eq $play_samples ]] || fail 'play replay sample count is not twenty'
play_minimum=${play_durations[0]}
play_maximum=${play_durations[$((play_samples - 1))]}
play_median=$((
  (play_durations[play_samples / 2 - 1] + play_durations[play_samples / 2]) / 2
))
play_p95_index=$(( (95 * play_samples + 99) / 100 - 1 ))
play_p95=${play_durations[$play_p95_index]}
play_total_ns=$(awk -F '\t' '$1 == "sample" { total += $3 } END { printf "%.0f", total }' "$play_raw")
play_commands=$(node -e \
  'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).log.length)' \
  "$session")
[[ $play_commands =~ ^[0-9]+$ && $play_commands -gt 0 ]] ||
  fail 'the recorded play session has no command count'
play_commands_per_second=$(awk \
  -v samples="$play_samples" -v commands="$play_commands" -v ns="$play_total_ns" \
  'BEGIN { printf "%.6f", samples * commands * 1000000000 / ns }')
{
  printf 'metric\tvalue\tunit\n'
  printf 'minimum\t%s\tns\n' "$play_minimum"
  printf 'median\t%s\tns\n' "$play_median"
  printf 'p95\t%s\tns\n' "$play_p95"
  printf 'maximum\t%s\tns\n' "$play_maximum"
  printf 'commands_per_replay\t%s\tcommands\n' "$play_commands"
  printf 'committed_commands_per_second\t%s\tcommands/s\n' "$play_commands_per_second"
  printf 'aggregate_measured_time\t%s\tns\n' "$play_total_ns"
} >"$evidence_dir/play-replay/summary.tsv"

package_dir=$evidence_dir/gate-k/inputs/gaol.world
package_files=$(find "$package_dir" -type f | wc -l)
package_bytes=$(find "$package_dir" -type f -printf '%s\n' |
  awk '{ total += $1 } END { print total + 0 }')
public_files=$(node -e \
  'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).files.length)' \
  "$viewer_build")
public_bytes=$(node -e \
  'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).total_bytes)' \
  "$viewer_build")
wasm_bytes=$(stat -c%s target/wasm32-unknown-unknown/wasm/nomos_play.wasm)
wasm_sha=$(sha256sum target/wasm32-unknown-unknown/wasm/nomos_play.wasm | cut -d' ' -f1)
edit_to_visible_ms=$(node -e \
  'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).timing.edit_to_visible_frame_ms)' \
  "$smoke_dir/receipt.json")
navigation_to_visible_ms=$(node -e \
  'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).timing.navigation_to_first_frame_ms)' \
  "$smoke_dir/receipt.json")

build_ns=$(awk -F '\t' '$1 == "clean_release_workspace_build" { print $2 }' \
  "$evidence_dir/gate-k/build/summary.tsv")
validation_median_ns=$(awk -F '\t' '$1 == "validate" { print $3 }' \
  "$evidence_dir/gate-k/summary.tsv")
validation_p95_ns=$(awk -F '\t' '$1 == "validate" { print $4 }' \
  "$evidence_dir/gate-k/summary.tsv")
kernel_replay_commands_per_second=$(awk -F '\t' \
  '$1 == "committed_commands_per_second" { print $2 }' \
  "$evidence_dir/gate-k/throughput.tsv")

for integer in \
  "$build_ns" "$validation_median_ns" "$validation_p95_ns" \
  "$package_files" "$package_bytes" "$public_files" "$public_bytes" \
  "$wasm_bytes" "$edit_to_visible_ms" "$navigation_to_visible_ms"; do
  [[ $integer =~ ^[0-9]+$ ]] || fail "a required integer measurement is invalid: $integer"
done

[[ -z $(git status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'tracked worktree changed during the evidence run'
[[ $(git rev-parse --verify HEAD) == "$head" ]] ||
  fail 'HEAD changed during the evidence run'

receipt=$evidence_dir/receipt.txt
{
  printf 'R1_ADOPTION_EVIDENCE PASS\n'
  printf 'commit %s\n' "$head"
  printf 'runner_arch %s\n' "${RUNNER_ARCH:-unknown}"
  printf 'runner_image %s\n' "${ImageOS:-unknown}"
  printf 'runner_image_version %s\n' "${ImageVersion:-unknown}"
  printf 'network_namespace no_default_route_loopback_only\n'
  printf 'cargo_net_offline true\n'
  printf 'clean_checkout_outputs yes\n'
  printf 'workspace_build_offline yes\n'
  printf 'workspace_test_offline yes\n'
  printf 'public_artifact_offline yes\n'
  printf 'workspace_build_time_ns %s\n' "$build_ns"
  printf 'validation_median_ns %s\n' "$validation_median_ns"
  printf 'validation_p95_ns %s\n' "$validation_p95_ns"
  printf 'kernel_replay_commands_per_second %s\n' "$kernel_replay_commands_per_second"
  printf 'play_replay_commands_per_second %s\n' "$play_commands_per_second"
  printf 'play_replay_median_ns %s\n' "$play_median"
  printf 'play_replay_p95_ns %s\n' "$play_p95"
  printf 'package_regular_files %s\n' "$package_files"
  printf 'package_regular_file_bytes %s\n' "$package_bytes"
  printf 'public_artifact_regular_files %s\n' "$public_files"
  printf 'public_artifact_regular_file_bytes %s\n' "$public_bytes"
  printf 'play_runtime_bytes %s\n' "$wasm_bytes"
  printf 'play_runtime_sha256 %s\n' "$wasm_sha"
  printf 'edit_to_visible_frame_ms %s\n' "$edit_to_visible_ms"
  printf 'navigation_to_first_frame_ms %s\n' "$navigation_to_visible_ms"
  printf 'gate_raw_samples_sha256 %s\n' \
    "$(sha256sum "$evidence_dir/gate-k/raw-samples.tsv" | cut -d' ' -f1)"
  printf 'play_raw_samples_sha256 %s\n' \
    "$(sha256sum "$play_raw" | cut -d' ' -f1)"
  printf 'viewer_build_receipt_sha256 %s\n' \
    "$(sha256sum "$viewer_build" | cut -d' ' -f1)"
  printf 'smoke_receipt_sha256 %s\n' \
    "$(sha256sum "$smoke_dir/receipt.json" | cut -d' ' -f1)"
  printf 'commands_sha256 %s\n' "$(sha256sum "$commands" | cut -d' ' -f1)"
  printf 'initial_tree_clean yes\n'
  printf 'final_tree_clean yes\n'
} >"$receipt"

printf 'R1 adoption evidence: PASS\n'

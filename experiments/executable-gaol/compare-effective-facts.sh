#!/usr/bin/env bash
# Spike harness for issue #126.
#
# Proves that `nomos effective-facts` equals what build-plan.mjs currently
# computes in JavaScript, for all twenty gaol scenarios. Quarantined
# experiment tooling: not Gate K evidence, not part of the accepted proof set.
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
cd "$repo_root"

out=target/effective-facts-comparison
rm -rf "$out"
mkdir -p "$out"
cargo build --quiet --locked -p nomos-cli --bin nomos

cat >"$out/compare.mjs" <<'NODE'
import { readFileSync } from "node:fs";
const [planPath, scenarioId, factsPath] = process.argv.slice(2);
const plan = JSON.parse(readFileSync(planPath, "utf8"));
const kernel = JSON.parse(readFileSync(factsPath, "utf8"));
const scenario = plan.scenarios.find((entry) => entry.id === scenarioId);
if (!scenario) throw new Error(`committed plan has no scenario ${scenarioId}`);

const differences = [];
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
if (scenario.tick !== kernel.tick) differences.push(`tick ${scenario.tick} != ${kernel.tick}`);
if (scenario.stateHash !== kernel.state_hash) differences.push("state hash differs");

const movement = new Map(kernel.effective_facts.ground_movement.map((f) => [f.entity, f.disposition]));
if (!same([...movement.keys()].sort(), Object.keys(scenario.movement).sort())) {
  differences.push("movement subject sets differ");
}
for (const [entity, js] of Object.entries(scenario.movement)) {
  const rust = movement.get(entity);
  if (!rust) { differences.push(`${entity}: absent from kernel output`); continue; }
  if (js.disposition !== rust.kind) differences.push(`${entity}: ${js.disposition} != ${rust.kind}`);
  // build-plan.mjs spells a blocked subject's cost as null; the kernel's
  // Blocked variant carries no cost key. Presentation, not semantics.
  const jsCost = js.cost === null ? undefined : js.cost;
  if (jsCost !== rust.cost) differences.push(`${entity}: cost ${js.cost} != ${rust.cost}`);
  if (!same([...js.reasons].sort(), [...rust.reasons].sort())) {
    differences.push(`${entity}: reasons ${JSON.stringify(js.reasons)} != ${JSON.stringify(rust.reasons)}`);
  }
}

const light = new Map(kernel.effective_facts.light_emission.map((f) => [f.entity, f.emitting]));
if (!same([...light.keys()].sort(), Object.keys(scenario.effectiveLight).sort())) {
  differences.push("light subject sets differ");
}
for (const [entity, emitting] of Object.entries(scenario.effectiveLight)) {
  if (light.get(entity) !== emitting) differences.push(`${entity}: light ${emitting} != ${light.get(entity)}`);
}

if (differences.length) { console.error(differences.join("; ")); process.exit(1); }
NODE

failures=0
scenarios=0
for area_dir in experiments/executable-gaol/areas/*/; do
  area=$(basename "$area_dir")
  target/debug/nomos compile "${area_dir}world.nomos" --out "$out/$area/world" >/dev/null
  for script in "${area_dir}"scenarios/*.commands; do
    name=$(basename "$script" .commands)
    scenarios=$((scenarios + 1))
    # 01-baseline is a declared rejection with zero committed commands.
    target/debug/nomos run "$out/$area/world" --commands "$script" \
      --out "$out/$area/runs/$name" >/dev/null || true
    target/debug/nomos effective-facts "$out/$area/world" \
      --state "$out/$area/runs/$name/final-state.json" \
      >"$out/$area/runs/$name/effective-facts.json"
    if node "$out/compare.mjs" "${area_dir}rendering-plan.example.json" "$name" \
      "$out/$area/runs/$name/effective-facts.json"; then
      printf 'OK    %-14s %s\n' "$area" "$name"
    else
      printf 'DIFF  %-14s %s\n' "$area" "$name"
      failures=$((failures + 1))
    fi
  done
done

printf '\n%d scenarios compared, %d differences\n' "$scenarios" "$failures"
exit $((failures > 0))

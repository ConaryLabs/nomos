#!/usr/bin/env bash
# Equivalence harness for issue #139, mirroring compare-effective-facts.sh.
#
# Proves that the Rust `nomos-render-plan` compiler emits, for all four areas,
# the same rendering plan the committed `rendering-plan.example.json` fixtures
# carry. Quarantined experiment tooling: not Gate K evidence, not part of the
# accepted proof set.
#
# THE DOCUMENTED NORMALIZATION, and the only one:
#
#   1. Both documents are parsed as JSON. Whitespace and key order are
#      therefore ignored; the JavaScript compiler wrote
#      `JSON.stringify(plan, null, 2)` with insertion-ordered keys and the Rust
#      compiler writes canonical bytes with byte-sorted keys, and that
#      difference is not a difference in the document.
#   2. The `schema` field is ignored on both sides. The identity moves from
#      `nomos.experiment.rendering_plan@1` to `nomos.rendering_plan@1`, which is
#      the point of the slice.
#   3. Nothing else is normalized. Array order is compared exactly, numbers are
#      compared by their JSON text, and `"cost": null` on a blocked movement
#      subject is a value that must be present on both sides — it is NOT
#      treated as equivalent to an absent key.
#
# Every difference is reported with its JSON path and both values.
#
# Run it against the JavaScript-generated fixtures to prove the replacement, and
# against the regenerated fixtures afterwards to prove the pipeline still agrees
# with what is committed.
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
cd "$repo_root"

out=target/rendering-plan-comparison
rm -rf "$out"
mkdir -p "$out"
cargo build --quiet --locked -p nomos-cli --bin nomos
cargo build --quiet --locked -p nomos-render-plan --bin nomos-render-plan

cat >"$out/compare.mjs" <<'NODE'
import { readFileSync } from "node:fs";
const [expectedPath, actualPath] = process.argv.slice(2);
const expected = JSON.parse(readFileSync(expectedPath, "utf8"));
const actual = JSON.parse(readFileSync(actualPath, "utf8"));
delete expected.schema;
delete actual.schema;

const differences = [];
const kindOf = (value) => (value === null ? "null" : Array.isArray(value) ? "array" : typeof value);
const walk = (path, left, right) => {
  if (kindOf(left) !== kindOf(right)) {
    differences.push(`${path}: ${kindOf(left)} ${JSON.stringify(left)} != ${kindOf(right)} ${JSON.stringify(right)}`);
    return;
  }
  if (kindOf(left) === "array") {
    if (left.length !== right.length) {
      differences.push(`${path}: array length ${left.length} != ${right.length}`);
      return;
    }
    left.forEach((item, index) => walk(`${path}[${index}]`, item, right[index]));
    return;
  }
  if (kindOf(left) === "object") {
    const keys = [...new Set([...Object.keys(left), ...Object.keys(right)])].sort();
    for (const key of keys) {
      if (!(key in left)) { differences.push(`${path}.${key}: absent != ${JSON.stringify(right[key])}`); continue; }
      if (!(key in right)) { differences.push(`${path}.${key}: ${JSON.stringify(left[key])} != absent`); continue; }
      walk(`${path}.${key}`, left[key], right[key]);
    }
    return;
  }
  if (!Object.is(left, right)) {
    differences.push(`${path}: ${JSON.stringify(left)} != ${JSON.stringify(right)}`);
  }
};
walk("$", expected, actual);

if (differences.length) { console.error(differences.join("\n")); process.exit(1); }
NODE

failures=0
areas=0
for area_dir in experiments/executable-gaol/areas/*/; do
  area=$(basename "$area_dir")
  areas=$((areas + 1))
  work="$out/$area"
  mkdir -p "$work/facts"
  target/debug/nomos compile "${area_dir}world.nomos" --out "$work/world" >/dev/null
  for script in "${area_dir}"scenarios/*.commands; do
    name=$(basename "$script" .commands)
    # 01-baseline is a declared rejection with zero committed commands.
    target/debug/nomos run "$work/world" --commands "$script" \
      --out "$work/runs/$name" >/dev/null || true
    target/debug/nomos effective-facts "$work/world" \
      --state "$work/runs/$name/final-state.json" >"$work/facts/$name.json"
  done
  target/debug/nomos entity-catalog "$work/world" >"$work/entity-catalog.json"
  target/debug/nomos-render-plan \
    --catalog "$work/entity-catalog.json" \
    --facts "$work/facts" \
    --runs "$work/runs" \
    --world "$work/world" \
    --source "${area_dir}presentation.json" \
    --out "$work/rendering-plan.json" >/dev/null

  if node "$out/compare.mjs" "${area_dir}rendering-plan.example.json" \
    "$work/rendering-plan.json"; then
    printf 'OK    %-14s\n' "$area"
  else
    printf 'DIFF  %-14s\n' "$area"
    failures=$((failures + 1))
  fi
done

printf '\n%d areas compared, %d differences\n' "$areas" "$failures"
exit $((failures > 0))

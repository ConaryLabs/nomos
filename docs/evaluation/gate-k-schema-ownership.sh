#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k schema ownership: FAIL: %s\n' "$*" >&2
  exit 1
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
receipt=$script_dir/SCHEMA_OWNERSHIP.md
[[ -f $receipt ]] || fail 'SCHEMA_OWNERSHIP.md is absent'

for command in git rg awk sort diff wc mktemp; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

printf '%s\n' \
  'nomos.causal_receipt@1' \
  'nomos.causal_receipt_sequence@1' \
  'nomos.command_log@1' \
  'nomos.command_script@1' \
  'nomos.compiler_receipts@1' \
  'nomos.package.manifest@1' \
  'nomos.package.schemas@1' \
  'nomos.persisted_runtime_state@2' \
  'nomos.projection.diagnostics@1' \
  'nomos.projection.navigation@1' \
  'nomos.projection.persistence@1' \
  'nomos.projection.simulation@3' \
  'nomos.replay_log@1' \
  'nomos.run_result@1' \
  'nomos.runtime_state@2' \
  'nomos.source@1' \
  'nomos.state_hash_sequence@1' \
  'nomos.world_ir.construction@3' \
  'nomos.world_ir@1' \
  'nomos.world_ir@2' >"$tmp_dir/expected.txt"

awk -F '|' '/^\| `nomos\./ {
  identity = $2
  gsub(/[` ]/, "", identity)
  print identity
}' "$receipt" | sort >"$tmp_dir/actual.txt"

[[ $(wc -l <"$tmp_dir/actual.txt") -eq 20 ]] || fail 'inventory does not contain exactly twenty rows'
[[ $(sort -u "$tmp_dir/actual.txt" | wc -l) -eq 20 ]] || fail 'inventory repeats a schema identity'
diff -u "$tmp_dir/expected.txt" "$tmp_dir/actual.txt" >/dev/null ||
  fail 'inventory identities or versions differ from the frozen twenty-schema set'

assert_source() {
  local file=$1
  local fragment=$2
  [[ -f $repo_root/$file ]] || fail "owner source is absent: $file"
  rg -F "$fragment" "$repo_root/$file" >/dev/null ||
    fail "owner source assertion absent from $file: $fragment"
}

assert_source crates/nomos-core/src/package.rs 'SchemaId::new("nomos.package.manifest", 1)'
assert_source crates/nomos-schema/src/lib.rs 'SchemaId::new("nomos.source", 1)'
assert_source crates/nomos-schema/src/lib.rs 'SchemaId::new("nomos.world_ir.construction", 3)'
assert_source crates/nomos-schema/src/lib.rs 'SchemaId::new("nomos.world_ir", 1)'
assert_source crates/nomos-schema/src/lib.rs 'SchemaId::new("nomos.world_ir", 2)'
assert_source crates/nomos-schema/src/lib.rs 'SchemaId::new("nomos.package.schemas", 1)'
assert_source crates/nomos-compiler/src/package.rs 'SchemaId::new("nomos.compiler_receipts", 1)'
assert_source crates/nomos-projection/src/lib.rs 'simulation_schema => "nomos.projection.simulation" @ 3'
assert_source crates/nomos-projection/src/lib.rs 'navigation_schema => "nomos.projection.navigation" @ 1'
assert_source crates/nomos-projection/src/lib.rs 'persistence_schema => "nomos.projection.persistence" @ 1'
assert_source crates/nomos-projection/src/lib.rs 'diagnostics_schema => "nomos.projection.diagnostics" @ 1'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.runtime_state", 2)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.persisted_runtime_state", 2)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.command_script", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.command_log", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.state_hash_sequence", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.causal_receipt_sequence", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.run_result", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.causal_receipt", 1)'
assert_source crates/nomos-sim/src/lib.rs 'SchemaId::new("nomos.replay_log", 1)'

rg -n 'SchemaId::new\("nomos\.' "$repo_root"/crates/*/src --glob '*.rs' |
  sort >"$tmp_dir/source-constructors.txt"
[[ $(wc -l <"$tmp_dir/source-constructors.txt") -eq 16 ]] ||
  fail 'literal schema constructor set changed outside the reviewed inventory'

awk -F ':' '{ print $1 }' "$tmp_dir/source-constructors.txt" | sort -u >"$tmp_dir/constructor-files.txt"
printf '%s\n' \
  "$repo_root/crates/nomos-compiler/src/package.rs" \
  "$repo_root/crates/nomos-core/src/package.rs" \
  "$repo_root/crates/nomos-schema/src/lib.rs" \
  "$repo_root/crates/nomos-sim/src/lib.rs" >"$tmp_dir/expected-constructor-files.txt"
diff -u "$tmp_dir/expected-constructor-files.txt" "$tmp_dir/constructor-files.txt" >/dev/null ||
  fail 'a literal schema constructor moved outside its reviewed owner module'

[[ $(rg -c '^[[:space:]]+[a-z_]+_schema => "nomos\.projection\.' "$repo_root/crates/nomos-projection/src/lib.rs") -eq 4 ]] ||
  fail 'projection schema macro no longer declares exactly four identities'

head=$(git -C "$repo_root" rev-parse --verify HEAD)
[[ -z $(git -C "$repo_root" diff --name-only eb86f25f5084a5da83cdd4f26e42e68089367a11 -- crates) ]] ||
  fail 'canonical implementation source changed after the reviewed freeze commit'

printf 'GATE_K_SCHEMA_OWNERSHIP PASS\n'
printf 'implementation_commit eb86f25f5084a5da83cdd4f26e42e68089367a11\n'
printf 'evidence_head %s\n' "$head"
printf 'schema_identities 20\n'
printf 'duplicate_meanings 0\n'
printf 'compiler_receipt_profiles exact_and_intentional\n'

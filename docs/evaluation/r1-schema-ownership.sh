#!/usr/bin/env bash

# The R1 canonical-schema ownership lane (issue #133).
#
# This replaces docs/evaluation/gate-k-schema-ownership.sh as the pull-request
# gate. That script remains valid evidence, but only at the Gate K freeze commit
# eb86f25f5084a5da83cdd4f26e42e68089367a11: it fails whenever
# `git diff eb86f25 -- crates` is non-empty, and under RUNTIME.md section 3 —
# option (a), owner-authorized 2026-08-25 — kernel crates may gain read-only R1
# surface, so every R1 slice makes that diff non-empty.
#
# Kept from the Gate K script, COPIED VERBATIM rather than sourced (the Gate K
# script is one top-to-bottom program with no library form, and sourcing it
# would also run the freeze assertions this lane exists to drop):
#
#   * the twenty-identity assertion over docs/evaluation/SCHEMA_OWNERSHIP.md;
#   * the twenty `assert_source` owner-source assertions.
#
# Dropped: the source-freeze diff against eb86f25, the "exactly sixteen literal
# constructors" count, the "exactly these four constructor files" set, and the
# "exactly four projection macro rows" count. Under RUNTIME.md section 5 R1-1,
# "no Gate K command, artifact, hash, or diagnostic changes" is proved by the
# determinism and verify lanes, not by a byte-frozen tree.
#
# Added: an exhaustive enumeration of every canonical schema identity declared
# under crates/*/src. Each one must be either one of the frozen twenty Gate K
# identities, declared in its frozen owner file, or a row in
# docs/evaluation/R1_SCHEMA_OWNERSHIP.md whose Owner column names the declaring
# crate and whose Owner file column names the declaring file. No identity may
# appear in both registers, twice in one register, or twice in the source.

set -euo pipefail

# Every comparison below is a byte comparison of canonical identifiers, so the
# collation must be byte order and not the caller's locale. Without this the
# twenty-identity `diff` reorders `nomos.world_ir.construction@3` against
# `nomos.world_ir@1` under a UTF-8 locale and fails spuriously; issue #134
# records the same latent defect in the Gate K script this lane replaces.
export LC_ALL=C

fail() {
  printf 'r1 schema ownership: FAIL: %s\n' "$*" >&2
  exit 1
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
receipt=$script_dir/SCHEMA_OWNERSHIP.md
r1_receipt=$script_dir/R1_SCHEMA_OWNERSHIP.md
[[ -f $receipt ]] || fail 'SCHEMA_OWNERSHIP.md is absent'
[[ -f $r1_receipt ]] || fail 'R1_SCHEMA_OWNERSHIP.md is absent'

for command in git grep awk sort diff wc mktemp; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

# ---------------------------------------------------------------------------
# 1. The Gate K identity assertions, copied verbatim from
#    docs/evaluation/gate-k-schema-ownership.sh.
# ---------------------------------------------------------------------------

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

# ---------------------------------------------------------------------------
# 1b. The Gate K owner-source assertions, copied verbatim from
#     docs/evaluation/gate-k-schema-ownership.sh.
# ---------------------------------------------------------------------------

assert_source() {
  local file=$1
  local fragment=$2
  [[ -f $repo_root/$file ]] || fail "owner source is absent: $file"
  grep -F "$fragment" "$repo_root/$file" >/dev/null ||
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

# ---------------------------------------------------------------------------
# 2. The frozen Gate K set as identity@version plus its owner file, exactly the
#    sixteen literal constructors and four projection macro rows the assertions
#    above pin. Nothing else is a Gate K identity.
# ---------------------------------------------------------------------------

printf '%s\t%s\n' \
  'nomos.package.manifest@1' 'crates/nomos-core/src/package.rs' \
  'nomos.source@1' 'crates/nomos-schema/src/lib.rs' \
  'nomos.world_ir.construction@3' 'crates/nomos-schema/src/lib.rs' \
  'nomos.world_ir@1' 'crates/nomos-schema/src/lib.rs' \
  'nomos.world_ir@2' 'crates/nomos-schema/src/lib.rs' \
  'nomos.package.schemas@1' 'crates/nomos-schema/src/lib.rs' \
  'nomos.compiler_receipts@1' 'crates/nomos-compiler/src/package.rs' \
  'nomos.projection.simulation@3' 'crates/nomos-projection/src/lib.rs' \
  'nomos.projection.navigation@1' 'crates/nomos-projection/src/lib.rs' \
  'nomos.projection.persistence@1' 'crates/nomos-projection/src/lib.rs' \
  'nomos.projection.diagnostics@1' 'crates/nomos-projection/src/lib.rs' \
  'nomos.runtime_state@2' 'crates/nomos-sim/src/lib.rs' \
  'nomos.persisted_runtime_state@2' 'crates/nomos-sim/src/lib.rs' \
  'nomos.command_script@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.command_log@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.state_hash_sequence@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.causal_receipt_sequence@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.run_result@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.causal_receipt@1' 'crates/nomos-sim/src/lib.rs' \
  'nomos.replay_log@1' 'crates/nomos-sim/src/lib.rs' >"$tmp_dir/gate-k-pairs.txt"

declare -A gate_k_file=()
while IFS=$'\t' read -r identity file; do
  [[ -n $identity ]] || continue
  [[ -z ${gate_k_file[$identity]+set} ]] || fail "the frozen Gate K set repeats $identity"
  gate_k_file[$identity]=$file
done <"$tmp_dir/gate-k-pairs.txt"
[[ ${#gate_k_file[@]} -eq 20 ]] ||
  fail 'the frozen Gate K set is not exactly twenty identities'

# The frozen set and the receipt inventory must name the same twenty identities.
printf '%s\n' "${!gate_k_file[@]}" | sort >"$tmp_dir/gate-k-identities.txt"
diff -u "$tmp_dir/expected.txt" "$tmp_dir/gate-k-identities.txt" >/dev/null ||
  fail 'the frozen Gate K owner-file table disagrees with the twenty-schema set'

# ---------------------------------------------------------------------------
# 3. The R1 register: identity, owner crate, owner file.
# ---------------------------------------------------------------------------

awk -F '|' '/^\| `nomos\./ {
  identity = $2
  owner = $3
  owner_file = $4
  gsub(/[` ]/, "", identity)
  gsub(/[` ]/, "", owner)
  gsub(/[` ]/, "", owner_file)
  printf "%s\t%s\t%s\n", identity, owner, owner_file
}' "$r1_receipt" >"$tmp_dir/r1-rows.txt"

declare -A r1_owner=()
declare -A r1_file=()
r1_rows=0
while IFS=$'\t' read -r identity owner owner_file; do
  [[ -n $identity ]] || continue
  r1_rows=$((r1_rows + 1))
  [[ $identity == *@* ]] ||
    fail "R1 register row is not spelled identity@version: $identity"
  [[ -n $owner ]] || fail "R1 register row $identity has an empty Owner column"
  [[ -n $owner_file ]] || fail "R1 register row $identity has an empty Owner file column"
  [[ $owner_file == crates/*/src/* ]] ||
    fail "R1 register row $identity names an Owner file outside crates/*/src: $owner_file"
  [[ -z ${r1_owner[$identity]+set} ]] || fail "the R1 register repeats $identity"
  [[ -z ${gate_k_file[$identity]+set} ]] ||
    fail "$identity is in both the Gate K receipt and the R1 register"
  r1_owner[$identity]=$owner
  r1_file[$identity]=$owner_file
done <"$tmp_dir/r1-rows.txt"

# ---------------------------------------------------------------------------
# 4. Enumerate every canonical schema identity declared under crates/*/src and
#    normalise each declaration to identity@version plus its file.
# ---------------------------------------------------------------------------

grep -R -n -E --include='*.rs' \
  'SchemaId::new\("nomos\.|_schema[[:space:]]*=>[[:space:]]*"nomos\.' \
  "$repo_root"/crates/*/src >"$tmp_dir/declaration-lines.txt" ||
  fail 'no canonical schema declaration found under crates/*/src'

awk -v root="$repo_root/" '
BEGIN { rootlen = length(root) }
{
  line = $0
  if (substr(line, 1, rootlen) == root) {
    line = substr(line, rootlen + 1)
  }
  colon = index(line, ":")
  file = substr(line, 1, colon - 1)
  rest = substr(line, colon + 1)
  colon = index(rest, ":")
  body = substr(rest, colon + 1)

  if (match(body, /SchemaId::new\("nomos\.[A-Za-z0-9_.]+"[[:space:]]*,[[:space:]]*[0-9]+[[:space:]]*\)/) > 0) {
    fragment = substr(body, RSTART, RLENGTH)
  } else if (match(body, /_schema[[:space:]]*=>[[:space:]]*"nomos\.[A-Za-z0-9_.]+"[[:space:]]*@[[:space:]]*[0-9]+/) > 0) {
    fragment = substr(body, RSTART, RLENGTH)
  } else {
    printf "UNPARSED\t%s\n", line
    next
  }

  match(fragment, /"nomos\.[A-Za-z0-9_.]+"/)
  identity = substr(fragment, RSTART + 1, RLENGTH - 2)
  match(fragment, /[0-9]+[[:space:]]*\)?$/)
  version = substr(fragment, RSTART, RLENGTH)
  gsub(/[^0-9]/, "", version)
  printf "%s@%s\t%s\n", identity, version, file
}' "$tmp_dir/declaration-lines.txt" >"$tmp_dir/declarations.txt"

if grep -q $'^UNPARSED\t' "$tmp_dir/declarations.txt"; then
  fail "canonical schema declaration is not in a recognised form: $(
    grep -m 1 $'^UNPARSED\t' "$tmp_dir/declarations.txt" | cut -f 2-
  )"
fi

# ---------------------------------------------------------------------------
# 5. Every declaration is owned: frozen Gate K identity in its frozen owner
#    file, or an R1 register row whose Owner and Owner file both match.
# ---------------------------------------------------------------------------

declare -A declared=()
while IFS=$'\t' read -r identity file; do
  [[ -n $identity ]] || continue
  [[ -z ${declared[$identity]+set} ]] ||
    fail "$identity is declared more than once under crates/*/src: ${declared[$identity]} and $file"
  declared[$identity]=$file

  crate=${file#crates/}
  crate=${crate%%/*}

  if [[ -n ${gate_k_file[$identity]+set} ]]; then
    [[ ${gate_k_file[$identity]} == "$file" ]] ||
      fail "Gate K identity $identity is declared in $file, not in its frozen owner file ${gate_k_file[$identity]}"
    continue
  fi

  [[ -n ${r1_owner[$identity]+set} ]] ||
    fail "$identity is declared in $file but is neither one of the twenty frozen Gate K identities nor a row in docs/evaluation/R1_SCHEMA_OWNERSHIP.md"
  [[ ${r1_owner[$identity]} == "$crate" ]] ||
    fail "$identity is declared in crate $crate but the R1 register names owner ${r1_owner[$identity]}"
  [[ ${r1_file[$identity]} == "$file" ]] ||
    fail "$identity is declared in $file but the R1 register names owner file ${r1_file[$identity]}"
done <"$tmp_dir/declarations.txt"

# There is deliberately no source-freeze diff assertion here. Under RUNTIME.md
# section 5 R1-1, "no Gate K command, artifact, hash, or diagnostic changes" is
# proved by the determinism and verify lanes, not by a byte-frozen crates tree.

head=$(git -C "$repo_root" rev-parse --verify HEAD)

printf 'R1_SCHEMA_OWNERSHIP PASS\n'
printf 'schema_identities_gate_k %d\n' "${#gate_k_file[@]}"
printf 'schema_identities_r1 %d\n' "$r1_rows"
printf 'evidence_head %s\n' "$head"

#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 schema ownership: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this checker accepts no arguments'

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd "$script_dir/../.." && pwd -P)
register=$script_dir/R2_SCHEMA_OWNERSHIP.md
r1_register=$script_dir/R1_SCHEMA_OWNERSHIP.md
gate_register=$script_dir/SCHEMA_OWNERSHIP.md

for command in awk grep sha256sum sort wc mktemp git; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done
for file in "$register" "$r1_register" "$gate_register"; do
  [[ -f $file && ! -L $file ]] || fail "required regular register is absent or symlinked: ${file#$repo_root/}"
done

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

awk -F '|' '/^\| `nomos\./ {
  identity = $2
  owner = $3
  owner_file = $4
  gsub(/[` ]/, "", identity)
  gsub(/[` ]/, "", owner)
  gsub(/[` ]/, "", owner_file)
  printf "%s\t%s\t%s\n", identity, owner, owner_file
}' "$register" >"$tmp_dir/rows"

[[ $(wc -l <"$tmp_dir/rows") -eq 2 ]] || fail 'register does not contain exactly two inventory rows'
[[ $(cut -f 1 "$tmp_dir/rows" | sort -u | wc -l) -eq 2 ]] || fail 'register repeats an identity'

printf '%s\t%s\t%s\n' \
  'nomos.observed_scene@1' 'nomos-observed-scene' 'crates/nomos-observed-scene/src/input.rs' \
  'nomos.observed_scene_plan@1' 'nomos-observed-scene' 'crates/nomos-observed-scene/src/plan.rs' \
  >"$tmp_dir/expected"
diff -u "$tmp_dir/expected" "$tmp_dir/rows" >/dev/null ||
  fail 'register identities, owners, or owner files differ from R2.md section 4'

while IFS=$'\t' read -r identity owner owner_file; do
  [[ $owner == nomos-observed-scene ]] || fail "$identity has unexpected owner $owner"
  [[ -f $repo_root/$owner_file && ! -L $repo_root/$owner_file ]] ||
    fail "$identity owner file is absent, non-regular, or symlinked: $owner_file"
  declaration="pub const SCHEMA: &str = \"$identity\";"
  [[ $(grep -R -F --include='*.rs' "$declaration" "$repo_root"/crates/*/src | wc -l) -eq 1 ]] ||
    fail "$identity does not have exactly one declaration under crates/*/src"
  grep -F "$declaration" "$repo_root/$owner_file" >/dev/null ||
    fail "$identity is not declared in its registered owner file $owner_file"
  ! grep -F "\`$identity\`" "$r1_register" >/dev/null ||
    fail "$identity duplicates the R1 register"
  ! grep -F "\`$identity\`" "$gate_register" >/dev/null ||
    fail "$identity duplicates the Gate K register"
done <"$tmp_dir/rows"

grep -R -h -E --include='*.rs' \
  '^pub const [A-Z_]*SCHEMA: &str = "nomos\.[a-z0-9_.]+@[1-9][0-9]*";' \
  "$repo_root/crates/nomos-observed-scene/src" | sort >"$tmp_dir/declarations"
printf '%s\n' \
  'pub const SCHEMA: &str = "nomos.observed_scene@1";' \
  'pub const SCHEMA: &str = "nomos.observed_scene_plan@1";' | sort >"$tmp_dir/expected-declarations"
diff -u "$tmp_dir/expected-declarations" "$tmp_dir/declarations" >/dev/null ||
  fail 'R2 crate declares an absent, duplicate, or third schema identity'

head=$(git -C "$repo_root" rev-parse --verify HEAD)
printf 'R2_SCHEMA_OWNERSHIP PASS\n'
printf 'schema_identities_r2 2\n'
printf 'register_sha256 %s\n' "$(sha256sum "$register" | awk '{print $1}')"
while IFS=$'\t' read -r identity _ owner_file; do
  printf 'owner_file_sha256 %s %s\n' "$identity" "$(sha256sum "$repo_root/$owner_file" | awk '{print $1}')"
done <"$tmp_dir/rows"
printf 'evidence_head %s\n' "$head"

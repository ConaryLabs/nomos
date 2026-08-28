#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C

fail() {
  printf 'r2 schema ownership plants: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 0 ]] || fail 'this test accepts no arguments'
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repo_root=$(cd -- "$script_directory/../.." && pwd -P)
checker_relative=docs/evaluation/r2-schema-ownership.sh

for command in awk cmp cut diff find git grep mkdir mktemp realpath sed sha256sum sort tar wc; do
  command -v "$command" >/dev/null 2>&1 || fail "required executable not found: $command"
done

# The checker searches every crate source tree for competing declarations but
# reads only these three registers and its own source. Archive that exact scope
# instead of cloning the complete 175 MiB candidate three times.
member_paths=(
  crates
  docs/evaluation/R2_SCHEMA_OWNERSHIP.md
  docs/evaluation/R1_SCHEMA_OWNERSHIP.md
  docs/evaluation/SCHEMA_OWNERSHIP.md
  "$checker_relative"
)
mkdir -p "$repo_root/target"
plants_parent=${R2_SCHEMA_PLANTS_PARENT:-$repo_root/target}
[[ -d $plants_parent && ! -L $plants_parent ]] ||
  fail 'the retained-fixture parent is absent, non-directory, or symlinked'
plants_parent=$(realpath -e -- "$plants_parent")
case $plants_parent in
  "$repo_root/target" | "$repo_root/target"/*) ;;
  *) fail 'the retained-fixture parent is outside checkout-local target' ;;
esac
plants_root=$(mktemp -d "$plants_parent/r2-schema-ownership-plants.XXXXXX")
expected_paths=$plants_root/expected-paths
git -C "$repo_root" ls-tree -r --name-only HEAD -- "${member_paths[@]}" |
  sort >"$expected_paths"
[[ -s $expected_paths ]] || fail 'the retained member inventory is empty'

assert_exact_regular_tree() {
  [[ $# -eq 1 && -d $1 && ! -L $1 ]] || return 2
  local root=$1 actual=$plants_root/actual-paths
  if find "$root" -mindepth 1 ! -type d ! -type f -print -quit | grep -q .; then
    fail 'a plant archive contains a symlink or non-regular entry'
  fi
  (cd -- "$root" && find . -type f -printf '%P\n' | sort) >"$actual"
  cmp -s "$expected_paths" "$actual" || fail 'a plant archive has missing or extra files'
}

expect_refusal() {
  [[ $# -eq 2 ]] || return 2
  local label=$1 expected=$2 root output status head
  root=$plants_root/$label
  mkdir "$root"
  git -C "$repo_root" archive --format=tar HEAD -- "${member_paths[@]}" |
    tar -xf - -C "$root"
  assert_exact_regular_tree "$root"
  if [[ $label == missing ]]; then
    [[ $(git -C "$root" rev-parse --show-toplevel) == "$repo_root" ]] ||
      fail 'the clean extracted fixture cannot discover the candidate Git root'
    output=$("$root/$checker_relative") ||
      fail 'the clean extracted schema-ownership fixture was refused'
    head=$(git -C "$repo_root" rev-parse --verify HEAD)
    grep -Fx 'R2_SCHEMA_OWNERSHIP PASS' <<<"$output" >/dev/null &&
      grep -Fx "evidence_head $head" <<<"$output" >/dev/null ||
      fail 'the clean extracted schema-ownership fixture evidence differs'
  fi
  case $label in
    missing)
      sed -i 's/nomos\.observed_scene@1/nomos.observed_scene@9/' \
        "$root/crates/nomos-observed-scene/src/input.rs"
      grep -Fx 'pub const SCHEMA: &str = "nomos.observed_scene@9";' \
        "$root/crates/nomos-observed-scene/src/input.rs" >/dev/null ||
        fail 'missing-declaration mutation was not installed'
      ;;
    duplicate)
      printf '\npub const DUPLICATE_SCHEMA: &str = "nomos.observed_scene@1";\n' \
        >>"$root/crates/nomos-observed-scene/src/value.rs"
      grep -Fx 'pub const DUPLICATE_SCHEMA: &str = "nomos.observed_scene@1";' \
        "$root/crates/nomos-observed-scene/src/value.rs" >/dev/null ||
        fail 'duplicate-declaration mutation was not installed'
      ;;
    third)
      printf '\npub const THIRD_SCHEMA: &str = "nomos.observed_third@1";\n' \
        >>"$root/crates/nomos-observed-scene/src/value.rs"
      grep -Fx 'pub const THIRD_SCHEMA: &str = "nomos.observed_third@1";' \
        "$root/crates/nomos-observed-scene/src/value.rs" >/dev/null ||
        fail 'third-identity mutation was not installed'
      ;;
    *) return 2 ;;
  esac
  set +e
  output=$("$root/$checker_relative" 2>&1)
  status=$?
  set -e
  [[ $status -eq 1 ]] || fail "$label plant did not return its canonical refusal status"
  [[ $output == "$expected" ]] || fail "$label plant emitted a non-specific refusal"
  printf 'expected refusal: %s\n' "$label"
}

expect_refusal missing \
  'r2 schema ownership: FAIL: nomos.observed_scene@1 does not have exactly one declaration under crates/*/src'
expect_refusal duplicate \
  'r2 schema ownership: FAIL: R2 crate declares an absent, duplicate, or third schema identity'
expect_refusal third \
  'r2 schema ownership: FAIL: R2 crate declares an absent, duplicate, or third schema identity'

# Retain the bounded fixture so a refusal remains inspectable and teardown does
# not add unrelated allocation churn to the persistent filesystem samples.
# The three refusal lines above are also the frozen receipt-verifier output.

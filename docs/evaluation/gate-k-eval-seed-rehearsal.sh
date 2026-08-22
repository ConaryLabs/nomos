#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k debug rehearsal seed: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 3 ]] || fail \
  'usage: gate-k-eval-seed-rehearsal.sh CANDIDATE COMMIT OUT'
candidate=$1
commit=$2
out=$3
[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail 'commit is not a full lowercase SHA-1'
candidate=$(realpath -e "$candidate")
[[ $(git -C "$candidate" rev-parse --show-toplevel) == "$candidate" ]] ||
  fail 'candidate is not a worktree root'
[[ $(git -C "$candidate" rev-parse HEAD) == "$commit" ]] || fail 'candidate HEAD mismatch'
[[ -z $(git -C "$candidate" status --porcelain=v1 --untracked-files=all) ]] ||
  fail 'candidate worktree is dirty'

out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"
[[ ! -e $out ]] || fail "output already exists: $out"
case $out/ in
  "$candidate"/*) fail 'seed receipt output must remain outside the candidate worktree' ;;
esac

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
seed_commands="$script_dir/rehearsals/debug-seed.commands"
hidden_mutation="$script_dir/rehearsals/debug-hidden-mutation.json"
[[ -f $seed_commands && ! -L $seed_commands ]] || fail 'rehearsal command seed is absent'
[[ -f $hidden_mutation && ! -L $hidden_mutation ]] || fail 'rehearsal mutation record is absent'

cargo build --release --locked --bin nomos --manifest-path "$candidate/Cargo.toml" >/dev/null
binary="$candidate/target/release/nomos"
[[ -x $binary ]] || fail 'candidate binary build did not publish nomos'
seed_id=$(printf '%s\n%s\n' "$commit" "$(sha256sum "$seed_commands" | cut -d' ' -f1)" |
  sha256sum | cut -c1-16)
relative="target/gate-k-debug-rehearsal-$seed_id"
seed="$candidate/$relative"
rm -r -- "$seed" 2>/dev/null || true
mkdir -m 755 "$seed" "$seed/forensics"
install -m 644 "$seed_commands" "$seed/failing.commands"

(
  cd "$candidate"
  target/release/nomos compile fixtures/gaol.nomos --out "$relative/gaol.world" \
    >"$relative/forensics/compile.stdout.json"
  set +e
  target/release/nomos run "$relative/gaol.world" --commands "$relative/failing.commands" \
    --out "$relative/failing.run" >"$relative/forensics/failure.stdout.json" \
    2>"$relative/forensics/failure.stderr.txt"
  status=$?
  set -e
  [[ $status -eq 1 ]] || fail "seeded command script exited $status instead of semantic rejection 1"
  printf '%s\n' "$status" >"$relative/forensics/failure.exit.txt"
  target/release/nomos explain-transition "$relative/failing.run" north_gate --tick 1 \
    --world "$relative/gaol.world" >"$relative/forensics/north-gate-tick-1.json"
)

jq -e '
  .status == "rejected" and
  .committed_command_count == 1 and
  .rejection_diagnostic.code == "EK0804"
  ' "$seed/failing.run/result.json" >/dev/null ||
  fail 'seeded run did not preserve the expected semantic rejection and committed prefix'
jq -e '
  .entity == "north_gate" and .tick == 1 and
  .request.action == "unlock" and
  .resolved_command.action == "unlock"
  ' "$seed/forensics/north-gate-tick-1.json" >/dev/null ||
  fail 'seeded transition explanation did not prove the successful first unlock'

stage=$(mktemp -d "$out_parent/.gate-k-debug-seed.XXXXXX")
cleanup() {
  rm -r -- "$stage"
}
trap cleanup EXIT
install -d -m 755 "$stage"
for tree in gaol.world failing.run forensics; do
  while IFS= read -r -d '' relative_path; do
    relative_path=${relative_path#./}
    if [[ -d $seed/$tree/$relative_path ]]; then
      install -d -m 755 "$stage/$tree/$relative_path"
    else
      install -d -m 755 "$(dirname "$stage/$tree/$relative_path")"
      install -m 644 "$seed/$tree/$relative_path" "$stage/$tree/$relative_path"
    fi
  done < <(cd "$seed/$tree" && find . -mindepth 1 -print0 | sort -z)
done
install -m 644 "$seed/failing.commands" "$stage/failing.commands"
install -m 644 "$hidden_mutation" "$stage/hidden-mutation.json"
printf '%s\n' "$commit" >"$stage/candidate-commit.txt"
printf '%s\n' "$(sha256sum "$binary" | cut -d' ' -f1)" >"$stage/binary.sha256"

mv -- "$stage" "$out"
trap - EXIT
rm -r -- "$seed"
printf 'GATE_K_DEBUG_REHEARSAL_SEEDED commit=%s output=%s\n' "$commit" "$out"

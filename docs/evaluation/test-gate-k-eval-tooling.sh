#!/usr/bin/env bash

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
packet_builder="$repo_root/docs/evaluation/gate-k-eval-packet.sh"
packet_verifier="$repo_root/docs/evaluation/gate-k-eval-verify-packet.sh"
seed_builder="$repo_root/docs/evaluation/gate-k-eval-seed-rehearsal.sh"
task_launcher="$repo_root/docs/evaluation/pi-cold-agent-task.sh"
task_recorder="$repo_root/docs/evaluation/gate-k-eval-record-task.sh"
finalizer="$repo_root/docs/evaluation/gate-k-eval-finalize.sh"
fake_pi="$repo_root/docs/evaluation/fixtures/fake-pi-cold-agent"
fake_bwrap="$repo_root/docs/evaluation/fixtures/fake-bwrap-pi-cold-agent"
rehearsals="$repo_root/docs/evaluation/rehearsals"

tmp_dir=$(mktemp -d)
candidate="$tmp_dir/candidate"
cleanup() {
  git -C "$repo_root" worktree remove --force "$candidate" >/dev/null 2>&1 || true
  rm -r -- "$tmp_dir"
}
trap cleanup EXIT

commit=$(git -C "$repo_root" rev-parse HEAD)
git -C "$repo_root" worktree add --detach "$candidate" "$commit" >/dev/null

assert_blocked() {
  local expected=$1
  shift
  local name=$1
  shift
  if "$@" >"$tmp_dir/$name.out" 2>"$tmp_dir/$name.err"; then
    printf 'expected %s to be blocked\n' "$name" >&2
    exit 1
  fi
  grep -F "$expected" "$tmp_dir/$name.err" >/dev/null
}

"$seed_builder" "$candidate" "$commit" "$tmp_dir/debug-seed" >/dev/null

build_author() {
  local out=$1
  "$packet_builder" author \
    --candidate "$candidate" --commit "$commit" \
    --brief "$rehearsals/author-brief.txt" \
    --prompt "$rehearsals/author-prompt.txt" \
    --fixture "$candidate/fixtures/gaol.nomos" \
    --out "$out" >/dev/null
}

build_debug() {
  local out=$1
  "$packet_builder" debug \
    --candidate "$candidate" --commit "$commit" \
    --brief "$rehearsals/debug-brief.txt" \
    --prompt "$rehearsals/debug-prompt.txt" \
    --world "$tmp_dir/debug-seed/gaol.world" \
    --failure-input "$tmp_dir/debug-seed/failing.commands" \
    --run-artifacts "$tmp_dir/debug-seed/failing.run" \
    --forensics "$tmp_dir/debug-seed/forensics" \
    --out "$out" >/dev/null
}

build_author "$tmp_dir/author-1"
build_author "$tmp_dir/author-2"
build_debug "$tmp_dir/debug-1"
build_debug "$tmp_dir/debug-2"
diff -r "$tmp_dir/author-1" "$tmp_dir/author-2" >/dev/null
diff -r "$tmp_dir/debug-1" "$tmp_dir/debug-2" >/dev/null
"$packet_verifier" "$tmp_dir/author-1" "$commit" >/dev/null
"$packet_verifier" "$tmp_dir/debug-1" "$commit" >/dev/null

cp -R "$tmp_dir/author-2" "$tmp_dir/tampered-packet"
printf 'tampered\n' >>"$tmp_dir/tampered-packet/prompt.txt"
assert_blocked 'mismatch: prompt.txt' tampered-packet \
  "$packet_verifier" "$tmp_dir/tampered-packet" "$commit"
assert_blocked 'wrong candidate' wrong-candidate \
  "$packet_builder" author --candidate "$candidate" \
  --commit 0000000000000000000000000000000000000000 \
  --brief "$rehearsals/author-brief.txt" --prompt "$rehearsals/author-prompt.txt" \
  --fixture "$candidate/fixtures/gaol.nomos" --out "$tmp_dir/wrong-candidate-packet"
touch "$candidate/untracked-boundary-fixture"
assert_blocked 'candidate worktree is dirty' dirty-candidate \
  "$packet_builder" author --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-brief.txt" --prompt "$rehearsals/author-prompt.txt" \
  --fixture "$candidate/fixtures/gaol.nomos" --out "$tmp_dir/dirty-candidate-packet"
rm -- "$candidate/untracked-boundary-fixture"

launch_task() {
  local shape_name=$1
  local packet=$2
  local raw="$tmp_dir/$shape_name-raw"
  mkdir -m 755 "$raw"
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
    "$task_launcher" claude "$candidate" "$commit" "$packet" \
    "$raw/transcript.ndjson" "$raw/stderr.txt" "$raw/qualification.txt" \
    >"$raw/launcher.txt"
}

record_task() {
  local shape_name=$1
  local packet=$2
  local raw="$tmp_dir/$shape_name-raw"
  "$task_recorder" "$packet" "$raw/transcript.ndjson" "$raw/stderr.txt" \
    "$raw/qualification.txt" "$raw/launcher.txt" "$commit" \
    "$tmp_dir/$shape_name-record" >/dev/null
}

launch_task author-subject "$tmp_dir/author-1"
record_task author-subject "$tmp_dir/author-1"
jq -e '.outcome == "eligible-for-checker" and .formalAttempt == false' \
  "$tmp_dir/author-subject-record/task-receipt.json" >/dev/null

"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-artifacts "$tmp_dir/author-subject-record/artifacts" \
  --commands "$tmp_dir/author-subject-record/commands.json" \
  --out "$tmp_dir/author-checker-1" >/dev/null
"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-artifacts "$tmp_dir/author-subject-record/artifacts" \
  --commands "$tmp_dir/author-subject-record/commands.json" \
  --out "$tmp_dir/author-checker-2" >/dev/null
diff -r "$tmp_dir/author-checker-1" "$tmp_dir/author-checker-2" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@1' \
  "$tmp_dir/author-checker-1/prompt.txt" >/dev/null
launch_task author-checker "$tmp_dir/author-checker-1"
record_task author-checker "$tmp_dir/author-checker-1"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/author-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "author"' \
  "$tmp_dir/author-run/result.json" >/dev/null

launch_task debug-subject "$tmp_dir/debug-1"
record_task debug-subject "$tmp_dir/debug-1"
"$packet_builder" debug-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-checker-brief.txt" \
  --prompt "$rehearsals/debug-checker-prompt.txt" \
  --subject-artifacts "$tmp_dir/debug-subject-record/artifacts" \
  --commands "$tmp_dir/debug-subject-record/commands.json" \
  --debug-evidence "$tmp_dir/debug-2/input" \
  --hidden-mutation "$tmp_dir/debug-seed/hidden-mutation.json" \
  --out "$tmp_dir/debug-checker" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@1' \
  "$tmp_dir/debug-checker/prompt.txt" >/dev/null
launch_task debug-checker "$tmp_dir/debug-checker"
record_task debug-checker "$tmp_dir/debug-checker"
"$finalizer" "$tmp_dir/debug-subject-record" "$tmp_dir/debug-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/debug-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "debug"' \
  "$tmp_dir/debug-run/result.json" >/dev/null

negative_task() {
  local mode=$1
  local expected=$2
  cp -R "$tmp_dir/author-2" "$tmp_dir/negative-$mode-packet"
  mkdir -m 755 "$tmp_dir/negative-$mode-raw"
  assert_blocked "$expected" "task-$mode" env \
    FAKE_PI_TASK_MODE="$mode" PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
    "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/negative-$mode-packet" \
    "$tmp_dir/negative-$mode-raw/transcript.ndjson" \
    "$tmp_dir/negative-$mode-raw/stderr.txt" \
    "$tmp_dir/negative-$mode-raw/qualification.txt"
}

negative_task missing-identity 'event stream has no matching terminal assistant identity'
negative_task missing-accounting 'expected one accounting record, found 0'
negative_task packet-hash-mismatch 'task boundary record does not prove the declared packet isolation'
negative_task harness-mutation 'immutable packet file changed: brief.txt'
negative_task wrong-provider 'task boundary record does not prove the declared packet isolation'
negative_task wrong-model 'task boundary record does not prove the declared packet isolation'
negative_task wrong-thinking 'task boundary record does not prove the declared packet isolation'
negative_task reused-session 'task boundary record does not prove the declared packet isolation'
negative_task forbidden-tool 'task boundary record does not prove the declared packet isolation'
negative_task outside-read-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task outside-write-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task missing-session 'expected one task session header, found 0'

cp -R "$tmp_dir/author-2" "$tmp_dir/negative-leak-packet"
mkdir -m 755 "$tmp_dir/negative-leak-raw"
assert_blocked 'provider output leaked credential environment variable NOMOS_PI_TEST_SECRET' \
  task-leak env FAKE_PI_TASK_MODE=leak-secret NOMOS_PI_TEST_SECRET=fixture-secret-never-print \
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/negative-leak-packet" \
  "$tmp_dir/negative-leak-raw/transcript.ndjson" "$tmp_dir/negative-leak-raw/stderr.txt" \
  "$tmp_dir/negative-leak-raw/qualification.txt"

cp -R "$tmp_dir/author-2" "$tmp_dir/budget-packet"
FAKE_PI_TASK_MODE=budget-exhausted PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/budget-packet" \
  "$tmp_dir/budget-events" "$tmp_dir/budget-stderr" "$tmp_dir/budget-qualification" \
  >"$tmp_dir/budget-launcher"
"$task_recorder" "$tmp_dir/budget-packet" "$tmp_dir/budget-events" "$tmp_dir/budget-stderr" \
  "$tmp_dir/budget-qualification" "$tmp_dir/budget-launcher" "$commit" \
  "$tmp_dir/budget-record" >/dev/null
jq -e '.outcome == "fail" and .accounting.budgetExceeded == "validation_compile_cycles"' \
  "$tmp_dir/budget-record/task-receipt.json" >/dev/null

cp -R "$tmp_dir/author-2" "$tmp_dir/budget-aborted-packet"
FAKE_PI_TASK_MODE=budget-aborted PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/budget-aborted-packet" \
  "$tmp_dir/budget-aborted-events" "$tmp_dir/budget-aborted-stderr" \
  "$tmp_dir/budget-aborted-qualification" >"$tmp_dir/budget-aborted-launcher"
"$task_recorder" "$tmp_dir/budget-aborted-packet" "$tmp_dir/budget-aborted-events" \
  "$tmp_dir/budget-aborted-stderr" "$tmp_dir/budget-aborted-qualification" \
  "$tmp_dir/budget-aborted-launcher" "$commit" "$tmp_dir/budget-aborted-record" >/dev/null
jq -e '.outcome == "fail" and .accounting.budgetExceeded == "provider_reported_tokens"' \
  "$tmp_dir/budget-aborted-record/task-receipt.json" >/dev/null

cp -R "$tmp_dir/author-2" "$tmp_dir/absent-command-packet"
FAKE_PI_TASK_MODE=absent-command PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/absent-command-packet" \
  "$tmp_dir/absent-command-events" "$tmp_dir/absent-command-stderr" \
  "$tmp_dir/absent-command-qualification" >"$tmp_dir/absent-command-launcher"
assert_blocked 'completed task has no command record' absent-command-record \
  "$task_recorder" "$tmp_dir/absent-command-packet" "$tmp_dir/absent-command-events" \
  "$tmp_dir/absent-command-stderr" "$tmp_dir/absent-command-qualification" \
  "$tmp_dir/absent-command-launcher" "$commit" "$tmp_dir/absent-command-record"

printf 'gate-k evaluation tooling offline harness: PASS\n'

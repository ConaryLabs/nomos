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

declare_packet_file() {
  local packet=$1
  local relative=$2
  local path="$packet/$relative"
  local size digest update
  size=$(stat -c %s "$path")
  digest=$(sha256sum "$path" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/manifest-update.XXXXXX")
  jq -S -c --arg path "$relative" --argjson size "$size" --arg digest "$digest" '
    .files += [{
      path: $path,
      bytes: $size,
      mode: "644",
      sha256: $digest,
      schemaIdentity: null
    }]
    | .files |= sort_by(.path)
    ' "$packet/packet-manifest.json" >"$update"
  mv -- "$update" "$packet/packet-manifest.json"
}

refresh_packet_file() {
  local packet=$1
  local relative=$2
  local path="$packet/$relative"
  local size digest update
  size=$(stat -c %s "$path")
  digest=$(sha256sum "$path" | cut -d' ' -f1)
  update=$(mktemp "$tmp_dir/manifest-refresh.XXXXXX")
  jq -S -c --arg path "$relative" --argjson size "$size" --arg digest "$digest" '
    .files |= map(if .path == $path then .bytes = $size | .sha256 = $digest else . end)
    ' "$packet/packet-manifest.json" >"$update"
  mv -- "$update" "$packet/packet-manifest.json"
}

tree_sha() {
  local root=$1
  find "$root" -type f -printf '%P\0' | sort -z |
    while IFS= read -r -d '' relative; do
      sha256sum "$root/$relative" | sed "s#  $root/#  #"
    done | sha256sum | cut -d' ' -f1
}

rebind_recorded_commands() {
  local record=$1
  local command=$2
  local commands_update receipt_update digest
  commands_update=$(mktemp "$tmp_dir/commands-update.XXXXXX")
  jq -S -c --arg command "$command" \
    '.commands[0].arguments.command = $command' \
    "$record/commands.json" >"$commands_update"
  mv -- "$commands_update" "$record/commands.json"
  digest=$(sha256sum "$record/commands.json" | cut -d' ' -f1)
  receipt_update=$(mktemp "$tmp_dir/receipt-update.XXXXXX")
  jq -S -c --arg digest "$digest" \
    '.digests.commandsSha256 = $digest' \
    "$record/task-receipt.json" >"$receipt_update"
  mv -- "$receipt_update" "$record/task-receipt.json"
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
grep -F 'Any attempted access outside `/workspace` makes the rehearsal ineligible' \
  "$tmp_dir/author-1/prompt.txt" >/dev/null
grep -F 'Any attempted outside access makes the rehearsal ineligible' \
  "$tmp_dir/debug-1/prompt.txt" >/dev/null
diff -r "$tmp_dir/author-1" "$tmp_dir/author-2" >/dev/null
diff -r "$tmp_dir/debug-1" "$tmp_dir/debug-2" >/dev/null
"$packet_verifier" "$tmp_dir/author-1" "$commit" >/dev/null
"$packet_verifier" "$tmp_dir/debug-1" "$commit" >/dev/null

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-history"
mkdir -m 755 "$tmp_dir/forensics-with-history/.git"
printf '%s\n' '[core]' >"$tmp_dir/forensics-with-history/.git/config"
assert_blocked 'excluded repository metadata' debug-history-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-history" \
  --out "$tmp_dir/debug-history-packet"

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-credentials"
printf '%s\n' 'not-a-real-secret' >"$tmp_dir/forensics-with-credentials/credentials.txt"
assert_blocked 'credential-like file' debug-credential-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-credentials" \
  --out "$tmp_dir/debug-credential-packet"

cp -R "$tmp_dir/debug-seed/forensics" "$tmp_dir/forensics-with-empty-answer"
mkdir -m 755 "$tmp_dir/forensics-with-empty-answer/expected-answer-is-duplicate-unlock"
assert_blocked 'unbound empty directory' debug-empty-directory-input \
  "$packet_builder" debug --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-brief.txt" --prompt "$rehearsals/debug-prompt.txt" \
  --world "$tmp_dir/debug-seed/gaol.world" \
  --failure-input "$tmp_dir/debug-seed/failing.commands" \
  --run-artifacts "$tmp_dir/debug-seed/failing.run" \
  --forensics "$tmp_dir/forensics-with-empty-answer" \
  --out "$tmp_dir/debug-empty-directory-packet"

cp -R "$tmp_dir/debug-2" "$tmp_dir/declared-history-packet"
mkdir -m 755 "$tmp_dir/declared-history-packet/input/forensics/.git"
printf '%s\n' '[core]' >"$tmp_dir/declared-history-packet/input/forensics/.git/config"
declare_packet_file "$tmp_dir/declared-history-packet" input/forensics/.git/config
assert_blocked 'excluded repository metadata' declared-history-packet \
  "$packet_verifier" "$tmp_dir/declared-history-packet" "$commit"

cp -R "$tmp_dir/debug-2" "$tmp_dir/declared-unrelated-packet"
printf '%s\n' 'unrelated' >"$tmp_dir/declared-unrelated-packet/unrelated.md"
declare_packet_file "$tmp_dir/declared-unrelated-packet" unrelated.md
assert_blocked 'outside the shape allowlist' declared-unrelated-packet \
  "$packet_verifier" "$tmp_dir/declared-unrelated-packet" "$commit"

cp -R "$tmp_dir/debug-2" "$tmp_dir/undeclared-empty-directory-packet"
mkdir -m 755 "$tmp_dir/undeclared-empty-directory-packet/input/forensics/expected-answer-is-duplicate-unlock"
assert_blocked 'undeclared empty directory' declared-empty-directory-packet \
  "$packet_verifier" "$tmp_dir/undeclared-empty-directory-packet" "$commit"

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
mkdir -m 755 "$tmp_dir/author-1/workspace/expected-answer-is-watch-lamp"
assert_blocked 'subject artifacts contain an unbound empty directory' \
  recorder-empty-artifact-directory \
  "$task_recorder" "$tmp_dir/author-1" \
  "$tmp_dir/author-subject-raw/transcript.ndjson" \
  "$tmp_dir/author-subject-raw/stderr.txt" \
  "$tmp_dir/author-subject-raw/qualification.txt" \
  "$tmp_dir/author-subject-raw/launcher.txt" "$commit" \
  "$tmp_dir/empty-directory-subject-record"
rmdir "$tmp_dir/author-1/workspace/expected-answer-is-watch-lamp"
record_task author-subject "$tmp_dir/author-1"
jq -e '.outcome == "eligible-for-checker" and .formalAttempt == false' \
  "$tmp_dir/author-subject-record/task-receipt.json" >/dev/null

"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/author-subject-record" \
  --out "$tmp_dir/author-checker-1" >/dev/null
"$packet_builder" author-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/author-subject-record" \
  --out "$tmp_dir/author-checker-2" >/dev/null
diff -r "$tmp_dir/author-checker-1" "$tmp_dir/author-checker-2" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@1' \
  "$tmp_dir/author-checker-1/prompt.txt" >/dev/null
grep -F 'even when the sandbox denied it' \
  "$tmp_dir/author-checker-1/prompt.txt" >/dev/null

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/injected-subject-record"
mkdir -m 755 "$tmp_dir/injected-subject-record/artifacts/source"
printf '%s\n' 'not an answer' >"$tmp_dir/injected-subject-record/artifacts/expected-answer.txt"
printf '%s\n' '{}' >"$tmp_dir/injected-subject-record/artifacts/hidden-mutation.json"
printf '%s\n' 'not source' >"$tmp_dir/injected-subject-record/artifacts/source/kernel.c"
printf '%s\n' 'unrelated' >"$tmp_dir/injected-subject-record/artifacts/unrelated.md"
assert_blocked 'subject artifacts differ from its task receipt' checker-injected-artifacts \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/injected-subject-record" \
  --out "$tmp_dir/injected-checker-packet"

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/empty-directory-subject-record"
mkdir -m 755 "$tmp_dir/empty-directory-subject-record/artifacts/expected-answer-is-watch-lamp"
assert_blocked 'unbound empty directory' checker-empty-directory-artifacts \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/empty-directory-subject-record" \
  --out "$tmp_dir/empty-directory-checker-packet"

cp -R "$tmp_dir/author-subject-record" "$tmp_dir/substituted-command-record"
printf '%s\n' 'SECRET_EXPECTED_ANSWER_AND_UNRECORDED_COMMANDS' \
  >"$tmp_dir/substituted-command-record/commands.json"
assert_blocked 'subject commands are invalid' checker-substituted-commands \
  "$packet_builder" author-checker --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/author-checker-brief.txt" \
  --prompt "$rehearsals/author-checker-prompt.txt" \
  --subject-record "$tmp_dir/substituted-command-record" \
  --out "$tmp_dir/substituted-command-packet"

cp -R "$tmp_dir/author-checker-2" "$tmp_dir/declared-injected-checker-packet"
printf '%s\n' 'not an answer' \
  >"$tmp_dir/declared-injected-checker-packet/subject/artifacts/expected-answer.txt"
declare_packet_file "$tmp_dir/declared-injected-checker-packet" \
  subject/artifacts/expected-answer.txt
assert_blocked 'checker subject artifacts differ from the task receipt' \
  declared-injected-checker \
  "$packet_verifier" "$tmp_dir/declared-injected-checker-packet" "$commit"

cp -R "$tmp_dir/author-checker-2" "$tmp_dir/declared-substituted-command-packet"
printf '%s\n' 'SECRET_EXPECTED_ANSWER_AND_UNRECORDED_COMMANDS' \
  >"$tmp_dir/declared-substituted-command-packet/subject/commands.json"
refresh_packet_file "$tmp_dir/declared-substituted-command-packet" subject/commands.json
assert_blocked 'checker subject commands are invalid' declared-substituted-command \
  "$packet_verifier" "$tmp_dir/declared-substituted-command-packet" "$commit"

launch_task author-checker "$tmp_dir/author-checker-1"
record_task author-checker "$tmp_dir/author-checker-1"
mkdir -m 755 "$tmp_dir/author-subject-record/artifacts/unbound-empty-subject-directory"
assert_blocked 'artifact tree contains an unbound empty directory' \
  finalizer-empty-subject-directory \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/empty-subject-directory-run"
rmdir "$tmp_dir/author-subject-record/artifacts/unbound-empty-subject-directory"
mkdir -m 755 "$tmp_dir/author-checker-record/artifacts/unbound-empty-checker-directory"
assert_blocked 'artifact tree contains an unbound empty directory' \
  finalizer-empty-checker-directory \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/empty-checker-directory-run"
rmdir "$tmp_dir/author-checker-record/artifacts/unbound-empty-checker-directory"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/author-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/author-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "author"' \
  "$tmp_dir/author-run/result.json" >/dev/null
[[ -f $tmp_dir/author-run/checker/artifacts/reproduction.txt ]]
[[ $(tree_sha "$tmp_dir/author-run/checker/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' \
      "$tmp_dir/author-run/checker/task-receipt.json") ]]

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/outside-path-checker-record"
rebind_recorded_commands "$tmp_dir/outside-path-checker-record" \
  'cat /workspace/brief.txt 2>/dev/null'
assert_blocked 'requested verdict pass contradicts mechanically derived verdict fail' \
  finalizer-checker-outside-path-pass \
  "$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/outside-path-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/outside-path-pass-run"
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/outside-path-checker-record" \
  fail fixture-adjudicator fixture-owner "$tmp_dir/outside-path-fail-run" >/dev/null
jq -e '
  .verdict == "fail" and
  .reason == "recorded subject or checker commands requested a path outside the declared workspace" and
  .commandBoundaryAudits.subject.verdict == "pass" and
  .commandBoundaryAudits.checker.verdict == "reject" and
  .commandBoundaryAudits.checker.findings == [{
    commandOrdinal: 0,
    commandSha256: "b06bf6c464f4bd1ca528655a03218278a7def04a1e77f26161ad5937941e2a43",
    kind: "outside_workspace_path",
    pathToken: "/dev/null"
  }]
  ' "$tmp_dir/outside-path-fail-run/result.json" >/dev/null

cp -R "$tmp_dir/author-checker-record" "$tmp_dir/quoted-path-checker-record"
rebind_recorded_commands "$tmp_dir/quoted-path-checker-record" \
  'python3 -c "print('\''scan data: /dev/null'\'')"'
"$finalizer" "$tmp_dir/author-subject-record" "$tmp_dir/quoted-path-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/quoted-path-pass-run" >/dev/null
jq -e '
  .verdict == "pass" and
  .commandBoundaryAudits.checker.verdict == "pass" and
  .commandBoundaryAudits.checker.findings == []
  ' "$tmp_dir/quoted-path-pass-run/result.json" >/dev/null

launch_task debug-subject "$tmp_dir/debug-1"
record_task debug-subject "$tmp_dir/debug-1"
"$packet_builder" debug-checker \
  --candidate "$candidate" --commit "$commit" \
  --brief "$rehearsals/debug-checker-brief.txt" \
  --prompt "$rehearsals/debug-checker-prompt.txt" \
  --subject-record "$tmp_dir/debug-subject-record" \
  --debug-evidence "$tmp_dir/debug-2/input" \
  --hidden-mutation "$tmp_dir/debug-seed/hidden-mutation.json" \
  --out "$tmp_dir/debug-checker" >/dev/null
grep -F 'schema` exactly `nomos.gate_k.checker_result@1' \
  "$tmp_dir/debug-checker/prompt.txt" >/dev/null
grep -F 'even when the sandbox denied it' \
  "$tmp_dir/debug-checker/prompt.txt" >/dev/null
launch_task debug-checker "$tmp_dir/debug-checker"
record_task debug-checker "$tmp_dir/debug-checker"
"$finalizer" "$tmp_dir/debug-subject-record" "$tmp_dir/debug-checker-record" \
  pass fixture-adjudicator fixture-owner "$tmp_dir/debug-run" >/dev/null
jq -e '.verdict == "pass" and .formalAttempt == false and .shape == "debug"' \
  "$tmp_dir/debug-run/result.json" >/dev/null
[[ -f $tmp_dir/debug-run/checker/artifacts/reproduction.txt ]]
[[ $(tree_sha "$tmp_dir/debug-run/checker/artifacts") == \
    $(jq -r '.digests.artifactsTreeSha256' \
      "$tmp_dir/debug-run/checker/task-receipt.json") ]]

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
negative_task temporary-write-succeeded 'task boundary record does not prove the declared packet isolation'
negative_task missing-session 'expected one task session header, found 0'

cp -R "$tmp_dir/author-2" "$tmp_dir/negative-leak-packet"
mkdir -m 755 "$tmp_dir/negative-leak-raw"
assert_blocked 'provider output leaked credential environment variable NOMOS_PI_TEST_SECRET' \
  task-leak env FAKE_PI_TASK_MODE=leak-secret NOMOS_PI_TEST_SECRET=fixture-secret-never-print \
  PI_BIN="$fake_pi" BWRAP_BIN="$fake_bwrap" \
  "$task_launcher" claude "$candidate" "$commit" "$tmp_dir/negative-leak-packet" \
  "$tmp_dir/negative-leak-raw/transcript.ndjson" "$tmp_dir/negative-leak-raw/stderr.txt" \
  "$tmp_dir/negative-leak-raw/qualification.txt"

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

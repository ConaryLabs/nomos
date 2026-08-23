#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k eval packet verification: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 2 ]] || fail 'usage: gate-k-eval-verify-packet.sh PACKET EXPECTED_COMMIT'
packet=$1
expected_commit=$2
[[ $expected_commit =~ ^[0-9a-f]{40}$ ]] || fail 'expected commit is not a full lowercase SHA-1'
[[ -d $packet && ! -L $packet ]] || fail "packet is not a real directory: $packet"
packet=$(realpath -e "$packet")
document_validator="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/gate-k-eval-validate-documents.py"

for name in jq sha256sum stat find sort cmp; do
  command -v "$name" >/dev/null 2>&1 || fail "required executable not found: $name"
done

[[ -z $(find "$packet" -type l -print -quit) ]] || fail 'packet contains a symlink'
[[ -z $(find "$packet" ! -type f ! -type d -print -quit) ]] ||
  fail 'packet contains a special entry'
[[ -f $packet/packet-manifest.json ]] || fail 'packet-manifest.json is absent'
[[ -f $packet/plan.json ]] || fail 'plan.json is absent'
[[ -f $packet/prompt.txt ]] || fail 'prompt.txt is absent'
python3 "$document_validator" manifest "$packet/packet-manifest.json" ||
  fail 'packet manifest does not satisfy its exact schema'
python3 "$document_validator" plan "$packet/plan.json" ||
  fail 'plan does not satisfy its exact schema'

jq -e \
  --arg commit "$expected_commit" '
  .schema == "nomos.gate_k.packet_manifest@1" and
  .candidateCommit == $commit and
  (.shape == "author" or .shape == "debug" or
   .shape == "author-checker" or .shape == "debug-checker") and
  .manifestExcludesSelf == true and
  (.writablePaths | length) == 1 and
  (.writablePaths[0] == "workspace" or .writablePaths[0] == "output") and
  (.files | type) == "array" and
  (.files | length) > 0 and
  ([.files[].path] | length) == ([.files[].path] | unique | length) and
  ([.files[].path] == ([.files[].path] | sort)) and
  all(.files[];
    (.path | test("^[A-Za-z0-9.][A-Za-z0-9._/-]*$")) and
    (.path | startswith("/") | not) and
    (.path | contains("..") | not) and
    (.path | contains("//") | not) and
    (.bytes | type) == "number" and .bytes >= 0 and .bytes == (.bytes | floor) and
    (.mode == "644" or .mode == "755") and
    (.sha256 | test("^[0-9a-f]{64}$")) and
    (.schemaIdentity == null or (.schemaIdentity | test("^[a-z][a-z0-9_.]*@[1-9][0-9]*$"))))
  ' "$packet/packet-manifest.json" >/dev/null || fail 'packet manifest shape is invalid'

tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT
find "$packet" -type f -printf '%P\n' | sort >"$tmp_dir/actual-paths"
{
  jq -r '.files[].path' "$packet/packet-manifest.json"
  printf 'packet-manifest.json\n'
} | sort >"$tmp_dir/declared-paths"
cmp -s "$tmp_dir/actual-paths" "$tmp_dir/declared-paths" ||
  fail 'packet file set differs from its manifest'

jq -r '.files[] | [.path, (.bytes|tostring), .mode, .sha256] | @tsv' \
  "$packet/packet-manifest.json" >"$tmp_dir/rows"
while IFS=$'\t' read -r relative expected_bytes expected_mode expected_sha; do
  path="$packet/$relative"
  [[ -f $path && ! -L $path ]] || fail "declared file is absent or not regular: $relative"
  actual_bytes=$(stat -c %s "$path")
  [[ $actual_bytes == "$expected_bytes" ]] || fail "byte-size mismatch: $relative"
  actual_mode=$(stat -c %a "$path")
  [[ $actual_mode == "$expected_mode" ]] || fail "mode mismatch: $relative"
  actual_sha=$(sha256sum "$path")
  actual_sha=${actual_sha%% *}
  [[ $actual_sha == "$expected_sha" ]] || fail "SHA-256 mismatch: $relative"
done <"$tmp_dir/rows"

shape=$(jq -r '.shape' "$packet/packet-manifest.json")
writable=$(jq -r '.writablePaths[0]' "$packet/packet-manifest.json")
[[ -d $packet/$writable && ! -L $packet/$writable ]] ||
  fail "declared writable directory is absent: $writable"
empty_directories=$(find "$packet" -mindepth 1 -type d -empty -printf '%P\n' | sort)
[[ -z $empty_directories || $empty_directories == "$writable" ]] ||
  fail "packet contains an undeclared empty directory: $empty_directories"
[[ -x $packet/bin/nomos && ! -L $packet/bin/nomos ]] || fail 'candidate binary is absent or not executable'
[[ ! -e $packet/.git ]] || fail 'packet contains Git metadata'
marker=$(<"$packet/.nomos-candidate-commit")
[[ $marker == "$expected_commit" ]] || fail 'candidate marker differs from expected commit'

exact_packet_tree() {
  local prefix=$1
  local label=$2
  shift 2
  local actual expected
  [[ -d $packet/$prefix && ! -L $packet/$prefix ]] || fail "$label tree is absent"
  actual=$(find "$packet/$prefix" -type f -printf '%P\n' | sort)
  expected=$(printf '%s\n' "$@" | sort)
  [[ $actual == "$expected" ]] || fail "$label file allowlist mismatch"
}

tree_sha() {
  local root=$1
  find "$root" -type f -printf '%P\0' | sort -z |
    while IFS= read -r -d '' relative; do
      sha256sum "$root/$relative" | sed "s#  $root/#  #"
    done | sha256sum | cut -d' ' -f1
}

while IFS= read -r relative; do
  lower=${relative,,}
  base=${lower##*/}
  case "/$lower/" in
    */.git/* | */.github/* | */docs/review/* | */reviews/*)
      fail "packet contains excluded repository metadata or review material: $relative"
      ;;
  esac
  case $base in
    agents.md | thesis.md | cargo.toml | cargo.lock | *.rs | *.ts | *transcript* | \
      credentials | credentials.* | auth.json | .env | .env.* | secret | secrets | \
      secret.* | secrets.*)
      fail "packet contains an excluded source, transcript, or credential-like file: $relative"
      ;;
  esac

  allowed=false
  case $relative in
    .nomos-candidate-commit | bin/nomos | brief.txt | plan.json | prompt.txt | \
      packet-manifest.json | reference/README.md | reference/nomos-help.txt)
      allowed=true
      ;;
  esac
  case "$shape:$relative" in
    author:reference/KERNEL-authoring-excerpt.md | author:reference/authoring.md | \
      author:reference/compiler.md | author:input/gaol.nomos | author:workspace/gaol.nomos)
      allowed=true
      ;;
    debug:reference/compiler.md | debug:reference/runtime.md | \
      debug:reference/explanations.md | debug:input/failing.commands | debug:input/world/* | \
      debug:input/failing.run/* | debug:input/forensics/*)
      allowed=true
      ;;
    author-checker:reference/authoring.md | author-checker:reference/compiler.md | \
      author-checker:subject/commands.json | author-checker:subject/task-receipt.json | \
      author-checker:subject/artifacts/*)
      allowed=true
      ;;
    debug-checker:reference/compiler.md | debug-checker:reference/runtime.md | \
      debug-checker:reference/explanations.md | debug-checker:subject/commands.json | \
      debug-checker:subject/task-receipt.json | debug-checker:subject/artifacts/* | \
      debug-checker:input/hidden-mutation.json | \
      debug-checker:input/debug-evidence/*)
      allowed=true
      ;;
  esac
  [[ $allowed == true ]] || fail "packet path is outside the shape allowlist: $relative"
done <"$tmp_dir/actual-paths"

case $shape in
  debug)
    exact_packet_tree input/world 'debug world' \
      compiler-receipts.json diagnostics.json manifest.json navigation.json \
      persistence.json schemas.json simulation.json world-ir.json
    exact_packet_tree input/failing.run 'debug run artifacts' \
      causal-receipts.json command-log.json final-state.json initial-state.json \
      result.json state-hashes.json
    exact_packet_tree input/forensics 'debug forensics' \
      compile.stdout.json failure.exit.txt failure.stderr.txt failure.stdout.json \
      north-gate-tick-1.json
    ;;
  debug-checker)
    exact_packet_tree input/debug-evidence 'debug checker evidence' \
      failing.commands \
      failing.run/causal-receipts.json failing.run/command-log.json \
      failing.run/final-state.json failing.run/initial-state.json \
      failing.run/result.json failing.run/state-hashes.json \
      forensics/compile.stdout.json forensics/failure.exit.txt \
      forensics/failure.stderr.txt forensics/failure.stdout.json \
      forensics/north-gate-tick-1.json \
      world/compiler-receipts.json world/diagnostics.json world/manifest.json \
      world/navigation.json world/persistence.json world/schemas.json \
      world/simulation.json world/world-ir.json
    ;;
esac

manifest_binary_sha=$(jq -r '.files[] | select(.path == "bin/nomos") | .sha256' \
  "$packet/packet-manifest.json")
plan_binary_sha=$(jq -r '.candidate.binarySha256 // empty' "$packet/plan.json")
[[ -n $manifest_binary_sha && $manifest_binary_sha == "$plan_binary_sha" ]] ||
  fail 'binary identity differs between plan and manifest'
prompt_sha=$(sha256sum "$packet/prompt.txt")
prompt_sha=${prompt_sha%% *}

jq -e \
  --arg commit "$expected_commit" \
  --arg shape "$shape" \
  --arg writable "$writable" \
  --arg binary_sha "$manifest_binary_sha" \
  --arg prompt_sha "$prompt_sha" '
  .schema == "nomos.gate_k.eval_plan@1" and
  .task.shape == $shape and
  (.task.classification == "rehearsal" or .task.classification == "formal") and
  (.task.formalAttempt | type) == "boolean" and
  .candidate.commit == $commit and
  .candidate.binaryPath == "bin/nomos" and
  .candidate.binarySha256 == $binary_sha and
  .packet.promptPath == "prompt.txt" and
  .packet.promptSha256 == $prompt_sha and
  .packet.writablePaths == [$writable] and
  .packet.repositoryMounted == false and
  .packet.gitMetadataPresent == false and
  .packet.networkPermitted == false and
  .packet.activeTools == ["bash"] and
  .budgets == {
    "freshSessions": 1,
    "operatorSubstantiveHintsMaximum": 0,
    "operatorRetriesMaximum": 0
  } and
  (.rubric | type) == "array" and (.rubric | length) >= 3 and
  all(.rubric[]; type == "string" and length > 0) and
  .recording == {
    "eventStream": "complete-ndjson",
    "removedProviderFields": ["textSignature", "thinkingSignature"],
    "commandOrderPreserved": true,
    "transcriptLossLimit": "only-the-two-declared-provider-signature-fields"
  } and
  .operatorIntervention == "none" and
  .verdicts == ["pass", "fail", "assisted", "inconclusive"]
  ' "$packet/plan.json" >/dev/null || fail 'plan does not bind the packet and protocol defaults'

for required in reference/README.md reference/nomos-help.txt brief.txt prompt.txt plan.json bin/nomos; do
  [[ -f $packet/$required ]] || fail "required packet file is absent: $required"
done

verify_bound_subject() {
  local expected_shape=$1
  local receipt="$packet/subject/task-receipt.json"
  [[ -f $receipt && ! -L $receipt ]] || fail 'checker subject task receipt is absent'
  python3 "$document_validator" task-receipt "$receipt" ||
    fail 'checker subject task receipt does not satisfy its exact schema'
  jq -e --arg commit "$expected_commit" --arg shape "$expected_shape" \
    --arg classification "$(jq -r '.task.classification' "$packet/plan.json")" '
    .schema == "nomos.gate_k.task_receipt@1" and
    .candidateCommit == $commit and .shape == $shape and
    .classification == $classification and
    .formalAttempt == ($classification == "formal") and
    .outcome == "eligible-for-checker" and
    (.digests.commandsSha256 | test("^[0-9a-f]{64}$")) and
    (.digests.artifactsTreeSha256 | test("^[0-9a-f]{64}$"))
    ' "$receipt" >/dev/null || fail 'checker subject task receipt identity is invalid'
  jq -e '
    .schema == "nomos.gate_k.commands@1" and
    (.commands | type) == "array" and (.commands | length) > 0 and
    (.commands | to_entries | all(.[];
      .value.ordinal == .key and .value.tool == "bash" and
      .value.completed == true and (.value.isError | type) == "boolean" and
      (.value.arguments.command | type) == "string" and
      (.value.arguments.command | length) > 0))
    ' "$packet/subject/commands.json" >/dev/null || fail 'checker subject commands are invalid'
  [[ $(sha256sum "$packet/subject/commands.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.commandsSha256' "$receipt") ]] ||
    fail 'checker subject commands differ from the task receipt'
  [[ $(tree_sha "$packet/subject/artifacts") == \
      $(jq -r '.digests.artifactsTreeSha256' "$receipt") ]] ||
    fail 'checker subject artifacts differ from the task receipt'
}

case $shape in
  author)
    [[ $writable == workspace ]] || fail 'author writable path is not workspace'
    for required in reference/KERNEL-authoring-excerpt.md reference/authoring.md \
      reference/compiler.md input/gaol.nomos workspace/gaol.nomos; do
      [[ -f $packet/$required ]] || fail "required author file is absent: $required"
    done
    ;;
  debug)
    [[ $writable == output ]] || fail 'debug writable path is not output'
    for required in reference/runtime.md reference/explanations.md input/failing.commands \
      input/world/manifest.json input/failing.run/result.json input/forensics/failure.stdout.json \
      input/forensics/failure.stderr.txt input/forensics/failure.exit.txt \
      input/forensics/north-gate-tick-1.json; do
      [[ -f $packet/$required ]] || fail "required debug file is absent: $required"
    done
    ;;
  author-checker)
    [[ $writable == output ]] || fail 'author-checker writable path is not output'
    [[ -f $packet/subject/commands.json ]] || fail 'author-checker commands are absent'
    [[ -d $packet/subject/artifacts ]] || fail 'author-checker artifacts are absent'
    verify_bound_subject author
    ;;
  debug-checker)
    [[ $writable == output ]] || fail 'debug-checker writable path is not output'
    [[ -f $packet/subject/commands.json ]] || fail 'debug-checker commands are absent'
    [[ -d $packet/subject/artifacts ]] || fail 'debug-checker artifacts are absent'
    verify_bound_subject debug
    [[ -f $packet/input/hidden-mutation.json ]] || fail 'debug-checker hidden mutation is absent'
    [[ -d $packet/input/debug-evidence ]] || fail 'debug-checker evidence is absent'
    ;;
esac

case $shape in
  author | debug)
    [[ -z $(find "$packet" -type f \( -name '*.rs' -o -name '*.ts' -o -name 'Cargo.toml' \
      -o -name 'Cargo.lock' -o -name AGENTS.md -o -name THESIS.md -o -iname '*transcript*' \
      -o -iname '*hidden*mutation*' \) -print -quit) ]] ||
      fail 'subject packet contains an excluded source, history, transcript, or hidden-mutation class'
    ;;
esac

packet_sha=$(sha256sum "$packet/packet-manifest.json")
packet_sha=${packet_sha%% *}
printf 'GATE_K_PACKET_VERIFIED shape=%s commit=%s manifest_sha256=%s\n' \
  "$shape" "$expected_commit" "$packet_sha"

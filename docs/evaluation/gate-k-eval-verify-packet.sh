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

for name in jq sha256sum stat find sort cmp; do
  command -v "$name" >/dev/null 2>&1 || fail "required executable not found: $name"
done

[[ -z $(find "$packet" -type l -print -quit) ]] || fail 'packet contains a symlink'
[[ -z $(find "$packet" ! -type f ! -type d -print -quit) ]] ||
  fail 'packet contains a special entry'
[[ -f $packet/packet-manifest.json ]] || fail 'packet-manifest.json is absent'
[[ -f $packet/plan.json ]] || fail 'plan.json is absent'
[[ -f $packet/prompt.txt ]] || fail 'prompt.txt is absent'

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
    (.bytes | type) == "number" and .bytes >= 0 and
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
[[ -x $packet/bin/nomos && ! -L $packet/bin/nomos ]] || fail 'candidate binary is absent or not executable'
[[ ! -e $packet/.git ]] || fail 'packet contains Git metadata'
marker=$(<"$packet/.nomos-candidate-commit")
[[ $marker == "$expected_commit" ]] || fail 'candidate marker differs from expected commit'

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
    ;;
  debug-checker)
    [[ $writable == output ]] || fail 'debug-checker writable path is not output'
    [[ -f $packet/subject/commands.json ]] || fail 'debug-checker commands are absent'
    [[ -d $packet/subject/artifacts ]] || fail 'debug-checker artifacts are absent'
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

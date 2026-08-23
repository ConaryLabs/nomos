#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k eval packet: FAIL: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
usage: gate-k-eval-packet.sh SHAPE --candidate DIR --commit SHA --brief FILE --prompt FILE --out DIR [inputs]

SHAPE and required inputs:
  author          --fixture FILE
  debug           --world DIR --failure-input FILE --run-artifacts DIR --forensics DIR
  author-checker  --subject-record DIR
  debug-checker   --subject-record DIR --debug-evidence DIR --hidden-mutation FILE

Common optional input:
  --classification rehearsal|formal   (default: rehearsal)
EOF
  exit 2
}

[[ $# -gt 0 ]] || usage
shape=$1
shift

candidate=
commit=
brief=
prompt=
out=
classification=rehearsal
fixture=
world=
failure_input=
run_artifacts=
forensics=
subject_record=
debug_evidence=
hidden_mutation=

while [[ $# -gt 0 ]]; do
  case $1 in
    --candidate) candidate=${2:-}; shift 2 ;;
    --commit) commit=${2:-}; shift 2 ;;
    --brief) brief=${2:-}; shift 2 ;;
    --prompt) prompt=${2:-}; shift 2 ;;
    --out) out=${2:-}; shift 2 ;;
    --classification) classification=${2:-}; shift 2 ;;
    --fixture) fixture=${2:-}; shift 2 ;;
    --world) world=${2:-}; shift 2 ;;
    --failure-input) failure_input=${2:-}; shift 2 ;;
    --run-artifacts) run_artifacts=${2:-}; shift 2 ;;
    --forensics) forensics=${2:-}; shift 2 ;;
    --subject-record) subject_record=${2:-}; shift 2 ;;
    --debug-evidence) debug_evidence=${2:-}; shift 2 ;;
    --hidden-mutation) hidden_mutation=${2:-}; shift 2 ;;
    *) usage ;;
  esac
done

case $shape in
  author | debug | author-checker | debug-checker) ;;
  *) fail "unknown packet shape: $shape" ;;
esac
case $classification in
  rehearsal | formal) ;;
  *) fail "classification must be rehearsal or formal: $classification" ;;
esac

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
document_validator="$script_dir/gate-k-eval-validate-documents.py"
json_validator="$script_dir/gate-k-eval-validate-json.py"
transcript_validator="$script_dir/gate-k-eval-validate-transcript.py"

for name in git cargo jq sha256sum stat find sort install awk realpath python3; do
  command -v "$name" >/dev/null 2>&1 || fail "required executable not found: $name"
done

[[ -n $candidate && -n $commit && -n $brief && -n $prompt && -n $out ]] || usage
[[ $commit =~ ^[0-9a-f]{40}$ ]] || fail "candidate commit is not a full lowercase SHA-1: $commit"
[[ -d $candidate ]] || fail "candidate worktree does not exist: $candidate"
candidate=$(realpath -e "$candidate")
git_root=$(git -C "$candidate" rev-parse --show-toplevel 2>/dev/null) ||
  fail "candidate is not a Git worktree: $candidate"
git_root=$(realpath -e "$git_root")
[[ $candidate == "$git_root" ]] || fail "candidate must be the worktree root: $candidate"
actual_commit=$(git -C "$candidate" rev-parse --verify HEAD)
[[ $actual_commit == "$commit" ]] ||
  fail "wrong candidate: expected $commit, found $actual_commit"
[[ -z $(git -C "$candidate" status --porcelain=v1 --untracked-files=all) ]] ||
  fail "candidate worktree is dirty"

out_parent=$(realpath -e "$(dirname "$out")")
out="$out_parent/$(basename "$out")"
[[ ! -e $out ]] || fail "output already exists: $out"
case $out/ in
  "$candidate"/*) fail "packet output must be outside the candidate worktree" ;;
esac

regular_file() {
  [[ -f $1 && ! -L $1 ]] || fail "expected a regular non-symlink file: $1"
}

real_directory() {
  local supplied=$1
  local label=$2
  local stripped=$supplied
  local parent name resolved
  while [[ $stripped != / && $stripped == */ ]]; do
    stripped=${stripped%/}
  done
  [[ -d $stripped && ! -L $stripped ]] || fail "$label: $supplied"
  parent=$(realpath -e "$(dirname -- "$stripped")")
  name=$(basename -- "$stripped")
  [[ $name != . && $name != .. ]] || fail "$label: $supplied"
  resolved=$(realpath -e "$stripped")
  [[ $resolved == "$parent/$name" ]] || fail "$label: $supplied"
  printf '%s\n' "$resolved"
}

regular_tree() {
  local root
  root=$(real_directory "$1" 'expected a directory, not a symlink')
  [[ -z $(find "$root" -type l -print -quit) ]] || fail "tree contains a symlink: $root"
  [[ -z $(find "$root" ! -type f ! -type d -print -quit) ]] ||
    fail "tree contains a special entry: $root"
  empty=$(find "$root" -mindepth 1 -type d -empty -print -quit)
  [[ -z $empty ]] || fail "tree contains an unbound empty directory: ${empty#"$root"/}"
  while IFS= read -r -d '' entry; do
    relative=${entry#"$root"/}
    [[ $relative =~ ^[A-Za-z0-9.][A-Za-z0-9._/-]*$ && $relative != *..* &&
      $relative != *//* ]] || fail "tree contains an unsafe path: $relative"
    lower=${relative,,}
    base=${lower##*/}
    case "/$lower/" in
      */.git/* | */.github/* | */docs/review/* | */reviews/*)
        fail "tree contains excluded repository metadata or review material: $relative"
        ;;
    esac
    case $base in
      agents.md | thesis.md | cargo.toml | cargo.lock | *.rs | *.ts | *transcript* | \
        credentials | credentials.* | auth.json | .env | .env.* | secret | secrets | \
        secret.* | secrets.*)
        fail "tree contains an excluded source, transcript, or credential-like file: $relative"
        ;;
    esac
  done < <(find "$root" -mindepth 1 -print0 | sort -z)
}

exact_tree_files() {
  local source=$1
  local label=$2
  shift 2
  local actual expected
  actual=$(find "$source" -type f -printf '%P\n' | sort)
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

validate_subject_record() {
  local expected_shape=$1
  local actual_top expected_top
  subject_record=$(real_directory "$subject_record" 'subject record is not a real directory')
  [[ -z $(find "$subject_record" -type l -print -quit) ]] ||
    fail 'subject record contains a symlink'
  [[ -z $(find "$subject_record" ! -type f ! -type d -print -quit) ]] ||
    fail 'subject record contains a special entry'
  actual_top=$(find "$subject_record" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort)
  expected_top=$(printf '%s\n' TASK.md accounting.json artifacts boundary.json commands.json \
    launcher.txt packet-manifest.json pi-qualification.txt pi-stderr.txt plan.json prompt.txt \
    task-receipt.json transcript.ndjson | sort)
  [[ $actual_top == "$expected_top" ]] || fail 'subject record top-level allowlist mismatch'
  regular_tree "$subject_record/artifacts"
  for file in TASK.md accounting.json boundary.json commands.json launcher.txt \
    packet-manifest.json pi-qualification.txt pi-stderr.txt plan.json prompt.txt \
    task-receipt.json transcript.ndjson; do
    regular_file "$subject_record/$file"
  done
  python3 "$document_validator" task-receipt "$subject_record/task-receipt.json" ||
    fail 'subject task receipt does not satisfy its exact schema'
  python3 "$document_validator" plan "$subject_record/plan.json" ||
    fail 'subject plan does not satisfy its exact schema'
  python3 "$document_validator" manifest "$subject_record/packet-manifest.json" ||
    fail 'subject packet manifest does not satisfy its exact schema'
  for json_file in accounting.json boundary.json commands.json; do
    python3 "$json_validator" "$subject_record/$json_file" ||
      fail "subject $json_file contains invalid or duplicate-key JSON"
  done
  python3 "$transcript_validator" "$subject_record/transcript.ndjson" --syntax-only ||
    fail 'subject transcript contains invalid or duplicate-key JSON'
  jq -e --arg commit "$commit" --arg shape "$expected_shape" \
    --arg classification "$classification" '
    .schema == "nomos.gate_k.task_receipt@1" and
    .candidateCommit == $commit and .shape == $shape and
    .classification == $classification and
    .formalAttempt == ($classification == "formal") and
    .outcome == "eligible-for-checker" and
    (.digests.packetManifestSha256 | test("^[0-9a-f]{64}$")) and
    (.digests.transcriptSha256 | test("^[0-9a-f]{64}$")) and
    (.digests.commandsSha256 | test("^[0-9a-f]{64}$")) and
    (.digests.artifactsTreeSha256 | test("^[0-9a-f]{64}$")) and
    (.digests.boundarySha256 | test("^[0-9a-f]{64}$")) and
    (.digests.qualificationSha256 | test("^[0-9a-f]{64}$"))
    ' "$subject_record/task-receipt.json" >/dev/null ||
    fail 'subject task receipt identity is invalid'
  jq -e --arg commit "$commit" --arg shape "$expected_shape" \
    --arg classification "$classification" '
    .schema == "nomos.gate_k.eval_plan@1" and .candidate.commit == $commit and
    .task.shape == $shape and .task.classification == $classification and
    .task.formalAttempt == ($classification == "formal")
    ' "$subject_record/plan.json" >/dev/null || fail 'subject plan identity is invalid'
  jq -e --arg commit "$commit" --arg shape "$expected_shape" '
    .schema == "nomos.gate_k.packet_manifest@1" and
    .candidateCommit == $commit and .shape == $shape
    ' "$subject_record/packet-manifest.json" >/dev/null ||
    fail 'subject packet manifest identity is invalid'
  jq -e '
    .schema == "nomos.gate_k.commands@1" and
    (.commands | type) == "array" and (.commands | length) > 0 and
    (.commands | to_entries | all(.[];
      .value.ordinal == .key and .value.tool == "bash" and
      .value.completed == true and (.value.isError | type) == "boolean" and
      (.value.arguments.command | type) == "string" and
      (.value.arguments.command | length) > 0))
    ' "$subject_record/commands.json" >/dev/null || fail 'subject commands are invalid'
  [[ $(sha256sum "$subject_record/packet-manifest.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.packetManifestSha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject packet manifest differs from its task receipt'
  [[ $(sha256sum "$subject_record/transcript.ndjson" | cut -d' ' -f1) == \
      $(jq -r '.digests.transcriptSha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject transcript differs from its task receipt'
  [[ $(sha256sum "$subject_record/commands.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.commandsSha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject commands differ from its task receipt'
  [[ $(tree_sha "$subject_record/artifacts") == \
      $(jq -r '.digests.artifactsTreeSha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject artifacts differ from its task receipt'
  [[ $(sha256sum "$subject_record/boundary.json" | cut -d' ' -f1) == \
      $(jq -r '.digests.boundarySha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject boundary differs from its task receipt'
  [[ $(sha256sum "$subject_record/pi-qualification.txt" | cut -d' ' -f1) == \
      $(jq -r '.digests.qualificationSha256' "$subject_record/task-receipt.json") ]] ||
    fail 'subject qualification differs from its task receipt'
}

regular_file "$brief"
regular_file "$prompt"

case $shape in
  author)
    [[ -n $fixture ]] || usage
    regular_file "$fixture"
    writable_path=workspace
    ;;
  debug)
    [[ -n $world && -n $failure_input && -n $run_artifacts && -n $forensics ]] || usage
    regular_tree "$world"
    regular_file "$failure_input"
    regular_tree "$run_artifacts"
    regular_tree "$forensics"
    exact_tree_files "$world" 'debug world' \
      compiler-receipts.json diagnostics.json manifest.json navigation.json \
      persistence.json schemas.json simulation.json world-ir.json
    exact_tree_files "$run_artifacts" 'debug run artifacts' \
      causal-receipts.json command-log.json final-state.json initial-state.json \
      result.json state-hashes.json
    exact_tree_files "$forensics" 'debug forensics' \
      compile.stdout.json failure.exit.txt failure.stderr.txt failure.stdout.json \
      north-gate-tick-1.json
    writable_path=output
    ;;
  author-checker)
    [[ -n $subject_record ]] || usage
    validate_subject_record author
    writable_path=output
    ;;
  debug-checker)
    [[ -n $subject_record && -n $debug_evidence && -n $hidden_mutation ]] || usage
    validate_subject_record debug
    regular_tree "$debug_evidence"
    regular_file "$hidden_mutation"
    exact_tree_files "$debug_evidence" 'debug checker evidence' \
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
    writable_path=output
    ;;
esac

for path in README.md KERNEL.md docs/authoring.md docs/compiler.md docs/runtime.md docs/explanations.md; do
  regular_file "$candidate/$path"
done

# The binary is produced from the exact clean candidate before isolation. Build
# outputs are ignored by Git and therefore do not weaken the clean-tree check.
cargo build --release --locked --bin nomos --manifest-path "$candidate/Cargo.toml" >/dev/null
binary="$candidate/target/release/nomos"
regular_file "$binary"
[[ -x $binary ]] || fail "candidate binary is not executable: $binary"
[[ -z $(git -C "$candidate" status --porcelain=v1 --untracked-files=all) ]] ||
  fail "candidate worktree changed during binary build"
binary_sha=$(sha256sum "$binary")
binary_sha=${binary_sha%% *}

stage=$(mktemp -d "$out_parent/.gate-k-eval-packet.XXXXXX")
manifest_rows=$(mktemp "$out_parent/.gate-k-eval-manifest.XXXXXX")
cleanup() {
  rm -r -- "$stage"
  rm -f -- "$manifest_rows"
}
trap cleanup EXIT

install -d -m 755 "$stage/bin" "$stage/reference" "$stage/$writable_path"
if [[ $shape != author-checker ]]; then
  install -d -m 755 "$stage/input"
fi
install -m 755 "$binary" "$stage/bin/nomos"
printf '%s\n' "$commit" >"$stage/.nomos-candidate-commit"
install -m 644 "$candidate/README.md" "$stage/reference/README.md"

case $shape in
  author)
    awk '
      /^## 1\. Exact base fixture$/ { emit = 1 }
      /^## 2\. Compile-time and command-time phases$/ { emit = 0 }
      emit { print }
    ' "$candidate/KERNEL.md" >"$stage/reference/KERNEL-authoring-excerpt.md"
    [[ -s $stage/reference/KERNEL-authoring-excerpt.md ]] ||
      fail "could not extract the Gate K authoring section"
    install -m 644 "$candidate/docs/authoring.md" "$stage/reference/authoring.md"
    install -m 644 "$candidate/docs/compiler.md" "$stage/reference/compiler.md"
    install -m 644 "$fixture" "$stage/input/gaol.nomos"
    install -m 644 "$fixture" "$stage/workspace/gaol.nomos"
    ;;
  debug)
    install -m 644 "$candidate/docs/compiler.md" "$stage/reference/compiler.md"
    install -m 644 "$candidate/docs/runtime.md" "$stage/reference/runtime.md"
    install -m 644 "$candidate/docs/explanations.md" "$stage/reference/explanations.md"
    ;;
  author-checker)
    install -m 644 "$candidate/docs/authoring.md" "$stage/reference/authoring.md"
    install -m 644 "$candidate/docs/compiler.md" "$stage/reference/compiler.md"
    ;;
  debug-checker)
    install -m 644 "$candidate/docs/compiler.md" "$stage/reference/compiler.md"
    install -m 644 "$candidate/docs/runtime.md" "$stage/reference/runtime.md"
    install -m 644 "$candidate/docs/explanations.md" "$stage/reference/explanations.md"
    ;;
esac

"$stage/bin/nomos" --help >"$stage/reference/nomos-help.txt"
install -m 644 "$brief" "$stage/brief.txt"
# Bash command substitution deliberately removes trailing line feeds. The
# packet's prompt.txt, not the operator's source file, is the exact launch text.
prompt_text=$(<"$prompt")
[[ -n $prompt_text ]] || fail "prompt is empty"
printf '%s' "$prompt_text" >"$stage/prompt.txt"

copy_tree() {
  local source=$1
  local destination=$2
  regular_tree "$source"
  install -d -m 755 "$destination"
  while IFS= read -r -d '' relative; do
    relative=${relative#./}
    if [[ -d $source/$relative ]]; then
      install -d -m 755 "$destination/$relative"
    else
      install -d -m 755 "$(dirname "$destination/$relative")"
      install -m 644 "$source/$relative" "$destination/$relative"
    fi
  done < <(cd "$source" && find . -mindepth 1 -print0 | sort -z)
}

case $shape in
  debug)
    copy_tree "$world" "$stage/input/world"
    install -m 644 "$failure_input" "$stage/input/failing.commands"
    copy_tree "$run_artifacts" "$stage/input/failing.run"
    copy_tree "$forensics" "$stage/input/forensics"
    ;;
  author-checker)
    install -d -m 755 "$stage/subject"
    copy_tree "$subject_record/artifacts" "$stage/subject/artifacts"
    install -m 644 "$subject_record/commands.json" "$stage/subject/commands.json"
    install -m 644 "$subject_record/task-receipt.json" "$stage/subject/task-receipt.json"
    ;;
  debug-checker)
    install -d -m 755 "$stage/subject"
    copy_tree "$subject_record/artifacts" "$stage/subject/artifacts"
    install -m 644 "$subject_record/commands.json" "$stage/subject/commands.json"
    install -m 644 "$subject_record/task-receipt.json" "$stage/subject/task-receipt.json"
    copy_tree "$debug_evidence" "$stage/input/debug-evidence"
    install -m 644 "$hidden_mutation" "$stage/input/hidden-mutation.json"
    ;;
esac

brief_sha=$(sha256sum "$stage/brief.txt")
brief_sha=${brief_sha%% *}
prompt_sha=$(sha256sum "$stage/prompt.txt")
prompt_sha=${prompt_sha%% *}
formal=false
[[ $classification == formal ]] && formal=true

plan=$(jq -S -c -n \
  --arg shape "$shape" \
  --arg classification "$classification" \
  --arg commit "$commit" \
  --arg binary_sha "$binary_sha" \
  --arg brief_sha "$brief_sha" \
  --arg prompt_sha "$prompt_sha" \
  --arg writable_path "$writable_path" \
  --argjson formal "$formal" '
  {
    schema: "nomos.gate_k.eval_plan@1",
    task: {
      shape: $shape,
      classification: $classification,
      formalAttempt: $formal
    },
    candidate: {
      commit: $commit,
      binaryPath: "bin/nomos",
      binarySha256: $binary_sha
    },
    packet: {
      briefPath: "brief.txt",
      briefSha256: $brief_sha,
      promptPath: "prompt.txt",
      promptSha256: $prompt_sha,
      writablePaths: [$writable_path],
      repositoryMounted: false,
      gitMetadataPresent: false,
      networkPermitted: false,
      activeTools: ["bash"]
    },
    budgets: {
      freshSessions: 1,
      operatorSubstantiveHintsMaximum: 0,
      operatorRetriesMaximum: 0
    },
    rubric: (
      if $shape == "author" then [
        "model_packet_tool_and_intervention_eligibility",
        "declared_brief_satisfied_with_approved_primitives",
        "distinct_typed_symbolic_ids_resolve",
        "validation_and_compile_succeed",
        "kernel_and_unrelated_content_unchanged",
        "subject_explains_the_change",
        "independent_checker_reproduces"
      ]
      elif $shape == "debug" then [
        "model_packet_tool_and_intervention_eligibility",
        "actual_semantic_cause_identified",
        "forensic_evidence_cited",
        "plausible_alternatives_excluded",
        "repair_targets_the_owning_boundary",
        "content_repair_verified_when_possible",
        "independent_checker_confirms_hidden_mutation"
      ]
      else [
        "fresh_independent_checker_identity",
        "subject_result_reproduced_or_rejected",
        "commands_hashes_and_reasons_recorded"
      ] end
    ),
    recording: {
      eventStream: "complete-ndjson",
      removedProviderFields: ["textSignature", "thinkingSignature"],
      commandOrderPreserved: true,
      transcriptLossLimit: "only-the-two-declared-provider-signature-fields"
    },
    verdicts: ["pass", "fail", "assisted", "inconclusive"],
    operatorIntervention: "none"
  }
')
printf '%s\n' "$plan" >"$stage/plan.json"

schema_identity() {
  local path=$1
  local first_line
  case $path in
    *.json)
      python3 "$json_validator" "$path" >/dev/null ||
        fail "packet JSON contains invalid or duplicate-key input: $path"
      jq -r '
        if has("schema") | not then empty
        elif (.schema | type) == "string" then .schema
        elif (.schema | type) == "object" and (.schema.name | type) == "string" and
             (.schema.version | type) == "number" and .schema.version > 0 and
             (.schema.version | floor) == .schema.version
        then "\(.schema.name)@\(.schema.version)"
        else error("invalid schema identity") end
      ' "$path" 2>/dev/null ||
        fail "packet JSON declares an invalid schema identity: $path"
      ;;
    *.nomos | *.commands)
      IFS= read -r first_line <"$path" || true
      case $first_line in
        schema\ *) printf '%s\n' "${first_line#schema }" ;;
      esac
      ;;
  esac
}

: >"$manifest_rows"
while IFS= read -r -d '' path; do
  relative=${path#"$stage"/}
  [[ $relative != packet-manifest.json ]] || continue
  size=$(stat -c %s "$path")
  mode=$(stat -c %a "$path")
  digest=$(sha256sum "$path")
  digest=${digest%% *}
  schema=$(schema_identity "$path")
  if [[ -n $schema ]]; then
    schema_json=$(jq -n --arg value "$schema" '$value')
  else
    schema_json=null
  fi
  jq -S -c -n \
    --arg path "$relative" \
    --argjson size "$size" \
    --arg mode "$mode" \
    --arg sha "$digest" \
    --argjson schema "$schema_json" \
    '{path: $path, bytes: $size, mode: $mode, sha256: $sha, schemaIdentity: $schema}' \
    >>"$manifest_rows"
done < <(find "$stage" -type f -print0 | sort -z)

manifest=$(jq -S -c -n \
  --arg commit "$commit" \
  --arg shape "$shape" \
  --arg writable_path "$writable_path" \
  --slurpfile files "$manifest_rows" '
  {
    schema: "nomos.gate_k.packet_manifest@1",
    candidateCommit: $commit,
    shape: $shape,
    manifestExcludesSelf: true,
    writablePaths: [$writable_path],
    files: $files
  }
')
printf '%s\n' "$manifest" >"$stage/packet-manifest.json"
python3 "$document_validator" manifest "$stage/packet-manifest.json" ||
  fail 'generated packet manifest does not satisfy its exact schema'

expected_count=$(jq '.files | length' "$stage/packet-manifest.json")
actual_count=$(find "$stage" -type f ! -name packet-manifest.json | wc -l)
[[ $expected_count -eq $actual_count ]] || fail "manifest file count mismatch"
[[ -z $(find "$stage" -type l -print -quit) ]] || fail "packet contains a symlink"
[[ -z $(find "$stage" ! -type f ! -type d -print -quit) ]] ||
  fail "packet contains a special entry"
empty_directories=$(find "$stage" -mindepth 1 -type d -empty -printf '%P\n' | sort)
[[ -z $empty_directories || $empty_directories == "$writable_path" ]] ||
  fail "packet contains an undeclared empty directory: $empty_directories"

mv -- "$stage" "$out"
trap - EXIT
rm -f -- "$manifest_rows"
packet_sha=$(sha256sum "$out/packet-manifest.json")
packet_sha=${packet_sha%% *}
printf 'GATE_K_PACKET shape=%s commit=%s manifest_sha256=%s output=%s\n' \
  "$shape" "$commit" "$packet_sha" "$out"

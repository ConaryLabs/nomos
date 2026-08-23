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
  author-checker  --subject-artifacts DIR --commands FILE
  debug-checker   --subject-artifacts DIR --commands FILE --debug-evidence DIR --hidden-mutation FILE

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
subject_artifacts=
commands=
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
    --subject-artifacts) subject_artifacts=${2:-}; shift 2 ;;
    --commands) commands=${2:-}; shift 2 ;;
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

for name in git cargo jq sha256sum stat find sort install awk realpath; do
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

regular_tree() {
  [[ -d $1 && ! -L $1 ]] || fail "expected a directory, not a symlink: $1"
  [[ -z $(find "$1" -type l -print -quit) ]] || fail "tree contains a symlink: $1"
  [[ -z $(find "$1" ! -type f ! -type d -print -quit) ]] ||
    fail "tree contains a special entry: $1"
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
    writable_path=output
    ;;
  author-checker)
    [[ -n $subject_artifacts && -n $commands ]] || usage
    regular_tree "$subject_artifacts"
    regular_file "$commands"
    writable_path=output
    ;;
  debug-checker)
    [[ -n $subject_artifacts && -n $commands && -n $debug_evidence && -n $hidden_mutation ]] || usage
    regular_tree "$subject_artifacts"
    regular_file "$commands"
    regular_tree "$debug_evidence"
    regular_file "$hidden_mutation"
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

install -d -m 755 "$stage/bin" "$stage/reference" "$stage/input" "$stage/$writable_path"
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
    copy_tree "$subject_artifacts" "$stage/subject/artifacts"
    install -m 644 "$commands" "$stage/subject/commands.json"
    ;;
  debug-checker)
    install -d -m 755 "$stage/subject"
    copy_tree "$subject_artifacts" "$stage/subject/artifacts"
    install -m 644 "$commands" "$stage/subject/commands.json"
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
      providerReportedTokensMaximum: 1000000,
      assistantTurnsMaximum: 40,
      validationCompileCyclesMaximum: 12,
      debugDiagnosticCyclesMaximum: 12,
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
      jq -r '
        if (.schema | type) == "string" then .schema
        elif (.schema | type) == "object" and (.schema.name | type) == "string" and
             (.schema.version | type) == "number"
        then "\(.schema.name)@\(.schema.version)"
        else empty end
      ' "$path" 2>/dev/null || true
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

expected_count=$(jq '.files | length' "$stage/packet-manifest.json")
actual_count=$(find "$stage" -type f ! -name packet-manifest.json | wc -l)
[[ $expected_count -eq $actual_count ]] || fail "manifest file count mismatch"
[[ -z $(find "$stage" -type l -print -quit) ]] || fail "packet contains a symlink"
[[ -z $(find "$stage" ! -type f ! -type d -print -quit) ]] ||
  fail "packet contains a special entry"

mv -- "$stage" "$out"
trap - EXIT
rm -f -- "$manifest_rows"
packet_sha=$(sha256sum "$out/packet-manifest.json")
packet_sha=${packet_sha%% *}
printf 'GATE_K_PACKET shape=%s commit=%s manifest_sha256=%s output=%s\n' \
  "$shape" "$commit" "$packet_sha" "$out"

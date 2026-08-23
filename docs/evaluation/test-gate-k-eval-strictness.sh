#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
protocol_dir=$script_dir
documents="$script_dir/gate-k-eval-validate-documents.py"
sanitizer="$script_dir/gate-k-eval-sanitize-transcript.py"
qualification="$script_dir/gate-k-eval-validate-qualification.py"
tmp_dir=$(mktemp -d)
trap 'rm -r -- "$tmp_dir"' EXIT

assert_blocked() {
  local expected=$1
  shift
  set +e
  "$@" >"$tmp_dir/blocked.stdout" 2>"$tmp_dir/blocked.stderr"
  status=$?
  set -e
  [[ $status -ne 0 ]] || { printf 'expected command to be blocked\n' >&2; exit 1; }
  grep -F "$expected" "$tmp_dir/blocked.stderr" >/dev/null
}

assert_blocked 'reasoning is not a non-negative integer' env PYTHONPATH="$protocol_dir" \
  python3 -c 'from gate_k_eval_pi_protocol import validate_usage; validate_usage({"input":1,"output":1,"cacheRead":0,"cacheWrite":0,"totalTokens":2,"reasoning":-1.5,"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0,"total":0}}, "usage")'
assert_blocked 'cacheWrite1h is not a non-negative integer' env PYTHONPATH="$protocol_dir" \
  python3 -c 'from gate_k_eval_pi_protocol import validate_usage; validate_usage({"input":1,"output":1,"cacheRead":0,"cacheWrite":0,"totalTokens":2,"cacheWrite1h":None,"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0,"total":0}}, "usage")'
assert_blocked 'not an RFC 3339 UTC timestamp' env PYTHONPATH="$protocol_dir" \
  python3 -c 'from gate_k_eval_pi_protocol import require_rfc3339_utc; require_rfc3339_utc("2026-08-23T24:00:00Z", "timestamp")'

printf '%s\n' '{"type":"unsupported","textSignature":"cat /etc/passwd"}' >"$tmp_dir/misplaced.ndjson"
assert_blocked 'occurs outside its documented content block' python3 "$sanitizer" "$tmp_dir/misplaced.ndjson"
printf '%s\n' '{"type":"message","content":[{"type":"text","text":"ok","textSignature":"opaque"}]}' \
  >"$tmp_dir/allowed.ndjson"
python3 "$sanitizer" "$tmp_dir/allowed.ndjson" >"$tmp_dir/sanitized.ndjson"
[[ $(jq -r '.content[0] | has("textSignature")' "$tmp_dir/sanitized.ndjson") == false ]]

base="$script_dir/runs/rehearsal/2026-08-23-claude-opus-5-debug-r5"
jq -S -c '.undeclared = true' "$base/plan.json" >"$tmp_dir/plan.json"
assert_blocked 'fields differ from the protocol' python3 "$documents" plan "$tmp_dir/plan.json"
jq -S -c '.files[0].bytes = 1.5' "$base/packet-manifest.json" >"$tmp_dir/manifest.json"
assert_blocked 'not a non-negative integer' python3 "$documents" manifest "$tmp_dir/manifest.json"
jq -S -c '.undeclared = true' "$base/subject/task-receipt.json" >"$tmp_dir/task-receipt.json"
assert_blocked 'fields differ from the protocol' python3 "$documents" task-receipt "$tmp_dir/task-receipt.json"

printf '%s\n' '{"schema":"nomos.gate_k.checker_result@1","verdict":"pass","commands":["x"],"reasons":["x"],"extra":NaN}' \
  >"$tmp_dir/checker.json"
assert_blocked 'non-finite JSON number' python3 "$documents" checker-result "$tmp_dir/checker.json"
assert_blocked 'cannot be authenticated' env PYTHONPATH="$protocol_dir" \
  python3 -c 'import importlib.util; s=importlib.util.spec_from_file_location("q", "'"$qualification"'"); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); m.file_sha256("/tmp/nomos-forged-provider", "provider extension")'

printf 'gate-k evaluation strictness regressions: PASS\n'

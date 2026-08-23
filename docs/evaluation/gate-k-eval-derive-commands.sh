#!/usr/bin/env bash

set -euo pipefail

fail() {
  printf 'gate-k command derivation: FAIL: %s\n' "$*" >&2
  exit 1
}

[[ $# -eq 1 ]] || fail 'usage: gate-k-eval-derive-commands.sh TRANSCRIPT_NDJSON'
transcript=$1
[[ -f $transcript && ! -L $transcript ]] || fail 'transcript is absent or not a regular file'

jq -e -S -c -s '
  [.[] | select(.type == "tool_execution_start")] as $starts |
  [.[] | select(.type == "tool_execution_end")] as $ends |
  if ($starts | length) == 0 then error("transcript has no tool starts")
  elif ([$starts[].toolCallId] | length) != ([$starts[].toolCallId] | unique | length)
  then error("transcript has duplicate tool starts")
  elif ([$ends[].toolCallId] | length) != ([$ends[].toolCallId] | unique | length)
  then error("transcript has duplicate tool ends")
  elif ([$starts[].toolCallId] | sort) != ([$ends[].toolCallId] | sort)
  then error("transcript tool starts and ends do not pair")
  else
    $starts
    | to_entries
    | map(
        .key as $ordinal |
        .value as $start |
        ([$ends[] | select(.toolCallId == $start.toolCallId)] | first) as $end |
        {
          ordinal: $ordinal,
          toolCallId: $start.toolCallId,
          tool: $start.toolName,
          arguments: $start.args,
          result: ($end.result // null),
          isError: $end.isError,
          completed: true
        }
      )
    | {schema: "nomos.gate_k.commands@1", commands: .}
  end
  ' "$transcript" || fail 'transcript cannot derive one complete ordered command record'

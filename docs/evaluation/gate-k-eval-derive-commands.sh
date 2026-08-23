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
  to_entries as $events |
  [$events[] | select(.value.type == "tool_execution_start") |
    {position: .key, event: .value}] as $starts |
  [$events[] | select(.value.type == "tool_execution_end") |
    {position: .key, event: .value}] as $ends |
  if ($starts | length) == 0 then error("transcript has no tool starts")
  elif ([$starts[].event.toolCallId] | length) !=
       ([$starts[].event.toolCallId] | unique | length)
  then error("transcript has duplicate tool starts")
  elif ([$ends[].event.toolCallId] | length) !=
       ([$ends[].event.toolCallId] | unique | length)
  then error("transcript has duplicate tool ends")
  elif ([$starts[].event.toolCallId] | sort) != ([$ends[].event.toolCallId] | sort)
  then error("transcript tool starts and ends do not pair")
  elif any($starts[]; . as $start |
    any($ends[];
      .event.toolCallId == $start.event.toolCallId and
      .position <= $start.position))
  then error("transcript has a tool end before its matching start")
  else
    $starts
    | to_entries
    | map(
        .key as $ordinal |
        .value.event as $start |
        ([$ends[] | select(.event.toolCallId == $start.toolCallId) | .event] | first) as $end |
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

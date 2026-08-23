#!/usr/bin/env python3
"""Fail-closed semantic validation for complete Pi task NDJSON transcripts."""

import argparse
import json
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import (
    fail,
    loads,
    message_text,
    require_keys,
    require_rfc3339_utc,
    require_string,
    require_uuid,
    tool_calls,
    validate_message,
    validate_result_payload,
    validate_update,
)


def load_events(path: Path) -> list[dict[str, object]]:
    events: list[dict[str, object]] = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line:
            fail(f"empty NDJSON line {line_number}")
        event = loads(line, f"NDJSON line {line_number}")
        if type(event) is not dict:
            fail(f"NDJSON line {line_number} is not an object")
        events.append(event)
    if not events:
        fail("transcript is empty")
    return events


def same_call(left: dict[str, object], right: dict[str, object]) -> bool:
    return all(left.get(field) == right.get(field) for field in ("id", "name", "arguments"))


def same_content(left: dict[str, object], right: dict[str, object]) -> bool:
    if left.get("type") != right.get("type"):
        return False
    if left["type"] == "toolCall":
        return same_call(left, right)
    field = "text" if left["type"] == "text" else "thinking"
    return right[field].startswith(left[field])


def validate(events: list[dict[str, object]], args: argparse.Namespace) -> dict[str, object]:
    if len(events) < 10:
        fail("transcript is truncated")
    session = require_keys(events[0], {"type", "version", "id", "timestamp", "cwd"}, set(), "session")
    if session["type"] != "session" or type(session["version"]) is not int or session["version"] != 3:
        fail("transcript does not begin with a Pi v3 session")
    require_keys(events[1], {"type"}, set(), "agent_start")
    if events[1]["type"] != "agent_start":
        fail("transcript does not begin with session then agent_start")
    require_keys(events[-2], {"type", "messages", "willRetry"}, set(), "agent_end")
    require_keys(events[-1], {"type"}, set(), "agent_settled")
    if events[-2]["type"] != "agent_end" or events[-1]["type"] != "agent_settled":
        fail("transcript does not end with agent_end then agent_settled")
    if events[-2]["willRetry"] is not False:
        fail("agent_end does not prove willRetry=false")
    lifecycle = [event.get("type") for event in events]
    for event_type in ("session", "agent_start", "agent_end", "agent_settled"):
        if lifecycle.count(event_type) != 1:
            fail(f"transcript requires exactly one {event_type}")

    session_id = require_uuid(session["id"], "session id")
    if session_id != args.session:
        fail("session id differs from task receipt")
    started = require_rfc3339_utc(session["timestamp"], "session timestamp")
    if started != args.started:
        fail("session timestamp differs from task receipt")
    if require_string(session["cwd"], "session cwd") != args.workspace:
        fail("session cwd differs from packet boundary")

    in_turn = False
    open_message: dict[str, object] | None = None
    open_role: str | None = None
    update_streams: dict[int, dict[str, object]] = {}
    declared: dict[str, dict[str, object]] = {}
    active: dict[str, dict[str, object]] = {}
    executed: dict[str, dict[str, object]] = {}
    result_messages: list[dict[str, object]] = []
    turn_assistant: dict[str, object] | None = None
    completed_ids: set[str] = set()
    ended_messages: list[dict[str, object]] = []
    user_messages = 0
    assistant_turns = 0
    tokens = 0
    terminal_assistant: dict[str, object] | None = None

    for index, event in enumerate(events[2:-2], 3):
        event_type = require_string(event.get("type"), f"event {index} type")
        name = f"event {index}"
        if event_type == "turn_start":
            require_keys(event, {"type"}, set(), name)
            if in_turn or open_message is not None or active:
                fail(f"turn_start at event {index} is nested")
            in_turn = True
            declared, active, executed = {}, {}, {}
            result_messages = []
            turn_assistant = None
        elif event_type == "message_start":
            require_keys(event, {"type", "message"}, set(), name)
            if not in_turn or open_message is not None:
                fail(f"message_start at event {index} is out of sequence")
            message = validate_message(event["message"], f"{name}.message", args.provider, args.model)
            open_role = message["role"]
            if open_role == "user":
                if user_messages != 0 or turn_assistant is not None or message_text(message) != args.prompt:
                    fail("user message_start differs from the one exact task prompt")
            elif open_role == "assistant":
                if turn_assistant is not None or declared or active or executed or result_messages:
                    fail(f"assistant message_start at event {index} is out of sequence")
                update_streams = {}
            elif open_role == "toolResult":
                call_id = message["toolCallId"]
                if call_id not in executed or any(result["toolCallId"] == call_id for result in result_messages):
                    fail(f"tool-result message_start at event {index} is unbound")
            open_message = message
        elif event_type == "message_update":
            if not in_turn or open_role != "assistant" or open_message is None:
                fail(f"message_update at event {index} is outside an assistant message")
            call = validate_update(event, name)
            update = event["assistantMessageEvent"]
            stream_index = update["contentIndex"]
            kind = update["type"]
            family = kind.split("_", 1)[0]
            if kind.endswith("_start"):
                if stream_index in update_streams:
                    fail(f"message_update at event {index} repeats a content start")
                update_streams[stream_index] = {"family": family, "text": "", "ended": False}
            else:
                stream = update_streams.get(stream_index)
                if stream is None or stream["family"] != family or stream["ended"]:
                    fail(f"message_update at event {index} has no matching content start")
                if kind.endswith("_delta"):
                    stream["text"] += update["delta"]
                else:
                    if family in ("text", "thinking") and stream["text"] != update["content"]:
                        fail(f"message_update at event {index} content differs from its deltas")
                    if call is not None:
                        delta_arguments = loads(stream["text"], f"{name} tool-call deltas")
                        if delta_arguments != call["arguments"]:
                            fail(f"message_update at event {index} tool-call arguments differ from its deltas")
                        stream["call"] = call
                    stream["ended"] = True
        elif event_type == "message_end":
            require_keys(event, {"type", "message"}, set(), name)
            if not in_turn or open_message is None or open_role is None:
                fail(f"message_end at event {index} is unpaired")
            message = validate_message(event["message"], f"{name}.message", args.provider, args.model)
            if message["role"] != open_role:
                fail(f"message_end at event {index} differs from message_start role")
            if open_role in ("user", "toolResult") and message != open_message:
                fail(f"message_end at event {index} differs from message_start")
            if open_role == "user":
                if message_text(message) != args.prompt:
                    fail("user message_end differs from the exact task prompt")
                user_messages += 1
            elif open_role == "assistant":
                if any(not stream["ended"] for stream in update_streams.values()):
                    fail(f"assistant message_end at event {index} has an incomplete update stream")
                calls = tool_calls(message)
                start_content = open_message["content"]
                if len(start_content) > len(message["content"]) or any(
                    not same_content(item, message["content"][content_index])
                    for content_index, item in enumerate(start_content)
                ):
                    fail("assistant message_start content differs from message_end")
                if set(update_streams) != set(range(len(message["content"]))):
                    fail("assistant message_update coverage differs from message_end content")
                for content_index, item in enumerate(message["content"]):
                    stream = update_streams[content_index]
                    expected_family = "toolcall" if item["type"] == "toolCall" else item["type"]
                    if stream["family"] != expected_family:
                        fail("assistant message_update content type differs from message_end")
                    if expected_family == "toolcall":
                        if not same_call(stream.get("call", {}), item):
                            fail("assistant message_update tool call differs from message_end")
                    elif stream["text"] != item["text" if expected_family == "text" else "thinking"]:
                        fail("assistant message_update content differs from message_end")
                if calls and message["stopReason"] != "toolUse":
                    fail("assistant tool calls lack stopReason=toolUse")
                if not calls and message["stopReason"] != "stop":
                    fail("assistant result without tool calls lacks stopReason=stop")
                for call in calls:
                    call_id = call["id"]
                    if call_id in completed_ids or call_id in declared:
                        fail("assistant declared a duplicate tool-call ID")
                    declared[call_id] = call
                turn_assistant = terminal_assistant = message
                assistant_turns += 1
                tokens += message["usage"]["totalTokens"]
            else:
                call_id = message["toolCallId"]
                execution = executed[call_id]
                expected = {"role": "toolResult", "toolCallId": call_id, "toolName": "bash",
                            "content": execution["result"]["content"], "isError": execution["isError"],
                            "timestamp": message["timestamp"]}
                if "details" in execution["result"]:
                    expected["details"] = execution["result"]["details"]
                if message != expected:
                    fail(f"tool-result message at event {index} differs from tool execution")
                result_messages.append(message)
                completed_ids.add(call_id)
            ended_messages.append(message)
            open_message = None
            open_role = None
        elif event_type == "tool_execution_start":
            require_keys(event, {"type", "toolCallId", "toolName", "args"}, set(), name)
            if not in_turn or open_message is not None or turn_assistant is None:
                fail(f"tool start at event {index} is out of sequence")
            call_id = require_string(event["toolCallId"], f"{name}.toolCallId")
            if call_id not in declared or call_id in active or call_id in executed:
                fail(f"tool start at event {index} is not declared exactly once")
            call = declared[call_id]
            if event["toolName"] != call["name"] or event["args"] != call["arguments"]:
                fail(f"tool start at event {index} differs from assistant tool call")
            active[call_id] = event
        elif event_type == "tool_execution_update":
            require_keys(event, {"type", "toolCallId", "toolName", "args", "partialResult"}, set(), name)
            call_id = require_string(event["toolCallId"], f"{name}.toolCallId")
            if call_id not in active:
                fail("tool update has no active start")
            if event["toolName"] != "bash" or event["args"] != active[call_id]["args"]:
                fail("tool update identity differs from its active start")
            validate_result_payload(event["partialResult"], f"{name}.partialResult")
        elif event_type == "tool_execution_end":
            require_keys(event, {"type", "toolCallId", "toolName", "result", "isError"}, set(), name)
            call_id = require_string(event["toolCallId"], f"{name}.toolCallId")
            if call_id not in active:
                fail("tool end has no active start")
            if event["toolName"] != "bash" or type(event["isError"]) is not bool:
                fail("tool end is not a structured bash result")
            executed[call_id] = {"result": validate_result_payload(event["result"], f"{name}.result"),
                                 "isError": event["isError"]}
            del active[call_id]
        elif event_type == "turn_end":
            require_keys(event, {"type", "message", "toolResults"}, set(), name)
            if not in_turn or open_message is not None or active or turn_assistant is None:
                fail(f"turn_end at event {index} is unpaired")
            if set(declared) != set(executed) or len(result_messages) != len(declared):
                fail(f"turn_end at event {index} has incomplete tool evidence")
            if event["message"] != turn_assistant or event["toolResults"] != result_messages:
                fail(f"turn_end at event {index} does not mirror its message and tool results")
            in_turn = False
        else:
            fail(f"unexpected transcript event type: {event_type}")

    if in_turn or open_message is not None or active:
        fail("transcript ends with an incomplete turn, message, or tool")
    if user_messages != 1:
        fail("transcript requires exactly one task prompt message")
    if assistant_turns == 0 or terminal_assistant is None:
        fail("transcript has no assistant result")
    if terminal_assistant["stopReason"] != "stop" or not message_text(terminal_assistant):
        fail("terminal assistant result is incomplete")
    if events[-2]["messages"] != ended_messages:
        fail("agent_end does not mirror every completed message")
    return {"assistantTurns": assistant_turns, "providerReportedTokens": tokens, "toolCalls": len(completed_ids)}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("transcript", type=Path)
    parser.add_argument("--syntax-only", action="store_true")
    for name in ("prompt", "provider", "model", "session", "started", "workspace"):
        parser.add_argument(f"--{name}")
    args = parser.parse_args()
    try:
        events = load_events(args.transcript)
        if args.syntax_only:
            return
        for name in ("prompt", "provider", "model", "session", "started", "workspace"):
            if getattr(args, name) is None:
                fail(f"missing --{name}")
        print(
            json.dumps(
                validate(events, args), sort_keys=True, separators=(",", ":"), allow_nan=False
            )
        )
    except (OSError, ValueError) as error:
        print(f"gate-k transcript validation: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

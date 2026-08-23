#!/usr/bin/env python3
"""Fail-closed validation for Pi task NDJSON transcripts."""

import argparse
import json
import re
import sys
from pathlib import Path


def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def fail(message: str) -> None:
    raise ValueError(message)


def load_events(path: Path) -> list[dict[str, object]]:
    events = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line:
            fail(f"empty NDJSON line {line_number}")
        try:
            event = json.loads(line, object_pairs_hook=reject_duplicates)
        except (json.JSONDecodeError, ValueError) as error:
            fail(f"invalid NDJSON line {line_number}: {error}")
        if type(event) is not dict:
            fail(f"NDJSON line {line_number} is not an object")
        events.append(event)
    if not events:
        fail("transcript is empty")
    return events


def require_string(value: object, name: str) -> str:
    if type(value) is not str or not value:
        fail(f"{name} is absent or invalid")
    return value


def message_text(message: dict[str, object]) -> str:
    content = message.get("content")
    if type(content) is not list:
        fail("message content is not an array")
    parts = []
    for item in content:
        if type(item) is not dict:
            fail("message content member is not an object")
        if item.get("type") == "text":
            parts.append(require_string(item.get("text"), "message text"))
    return "".join(parts)


def validate(events: list[dict[str, object]], args: argparse.Namespace) -> dict[str, object]:
    if len(events) < 7 or events[0].get("type") != "session" or events[1].get("type") != "agent_start":
        fail("transcript does not begin with session then agent_start")
    if events[-2].get("type") != "agent_end" or events[-1].get("type") != "agent_settled":
        fail("transcript does not end with agent_end then agent_settled")
    lifecycle = [event.get("type") for event in events]
    for event_type in ("session", "agent_start", "agent_end", "agent_settled"):
        if lifecycle.count(event_type) != 1:
            fail(f"transcript requires exactly one {event_type}")
    if events[-2].get("willRetry") is not False:
        fail("agent_end does not prove willRetry=false")

    session = events[0]
    session_id = require_string(session.get("id"), "session id")
    if not re.fullmatch(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", session_id):
        fail("session id is not a protocol UUID")
    if session_id != args.session:
        fail("session id differs from task receipt")
    if require_string(session.get("timestamp"), "session timestamp") != args.started:
        fail("session timestamp differs from task receipt")
    if require_string(session.get("cwd"), "session cwd") != args.workspace:
        fail("session cwd differs from packet boundary")

    in_turn = False
    open_message: str | None = None
    active_tools: set[str] = set()
    completed_tools: set[str] = set()
    user_starts = 0
    user_ends = 0
    assistant_turns = 0
    tokens = 0
    terminal_assistant: dict[str, object] | None = None
    assistant_in_turn = False
    tools_in_turn = 0
    tool_results_in_turn = 0

    for index, event in enumerate(events[2:-2], 3):
        event_type = require_string(event.get("type"), f"event {index} type")
        if event_type == "turn_start":
            if in_turn or open_message is not None or active_tools:
                fail(f"turn_start at event {index} is nested")
            in_turn = True
            assistant_in_turn = False
            tools_in_turn = 0
            tool_results_in_turn = 0
        elif event_type == "turn_end":
            if not in_turn or open_message is not None or active_tools:
                fail(f"turn_end at event {index} is unpaired")
            if not assistant_in_turn or tool_results_in_turn != tools_in_turn:
                fail(f"turn_end at event {index} has incomplete assistant or tool results")
            in_turn = False
        elif event_type == "message_start":
            if not in_turn or open_message is not None:
                fail(f"message_start at event {index} is out of sequence")
            message = event.get("message")
            if type(message) is not dict:
                fail(f"message_start at event {index} has no message")
            role = message.get("role")
            if role not in ("user", "assistant", "toolResult"):
                fail(f"message_start at event {index} has forbidden role")
            open_message = str(role)
            if role == "user":
                user_starts += 1
                if message_text(message) != args.prompt:
                    fail("user message_start differs from the exact task prompt")
        elif event_type == "message_update":
            if not in_turn or open_message != "assistant":
                fail(f"message_update at event {index} is outside an assistant message")
        elif event_type == "message_end":
            if not in_turn or open_message is None:
                fail(f"message_end at event {index} is unpaired")
            message = event.get("message")
            if type(message) is not dict or message.get("role") != open_message:
                fail(f"message_end at event {index} differs from message_start")
            role = open_message
            open_message = None
            if role == "user":
                user_ends += 1
                if message_text(message) != args.prompt:
                    fail("user message_end differs from the exact task prompt")
            elif role == "assistant":
                if assistant_in_turn:
                    fail("turn contains more than one assistant result")
                if message.get("provider") != args.provider or message.get("model") != args.model:
                    fail("assistant identity differs from task receipt")
                usage = message.get("usage")
                if type(usage) is not dict or type(usage.get("totalTokens")) is not int or usage["totalTokens"] < 0:
                    fail("assistant message has incomplete provider usage")
                tokens += usage["totalTokens"]
                assistant_turns += 1
                assistant_in_turn = True
                terminal_assistant = message
            elif role == "toolResult":
                tool_results_in_turn += 1
        elif event_type == "tool_execution_start":
            if not in_turn or open_message is not None:
                fail(f"tool start at event {index} is out of sequence")
            call_id = require_string(event.get("toolCallId"), "tool call id")
            if call_id in active_tools or call_id in completed_tools:
                fail("transcript has a duplicate tool start")
            if event.get("toolName") != "bash" or type(event.get("args")) is not dict:
                fail("tool start is not a structured bash call")
            require_string(event["args"].get("command"), "bash command")
            active_tools.add(call_id)
        elif event_type == "tool_execution_update":
            if require_string(event.get("toolCallId"), "tool update call id") not in active_tools:
                fail("tool update has no active start")
        elif event_type == "tool_execution_end":
            call_id = require_string(event.get("toolCallId"), "tool end call id")
            if call_id not in active_tools:
                fail("tool end has no active start")
            if type(event.get("isError")) is not bool:
                fail("tool end is missing boolean isError")
            active_tools.remove(call_id)
            completed_tools.add(call_id)
            tools_in_turn += 1
        else:
            fail(f"unexpected transcript event type: {event_type}")

    if in_turn or open_message is not None or active_tools:
        fail("transcript ends with an incomplete turn, message, or tool")
    if user_starts != 1 or user_ends != 1:
        fail("transcript requires exactly one task prompt message")
    if assistant_turns == 0 or terminal_assistant is None:
        fail("transcript has no assistant result")
    if terminal_assistant.get("stopReason") != "stop" or not message_text(terminal_assistant):
        fail("terminal assistant result is incomplete")
    return {
        "assistantTurns": assistant_turns,
        "providerReportedTokens": tokens,
        "toolCalls": len(completed_tools),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("transcript", type=Path)
    parser.add_argument("--syntax-only", action="store_true")
    parser.add_argument("--prompt")
    parser.add_argument("--provider")
    parser.add_argument("--model")
    parser.add_argument("--session")
    parser.add_argument("--started")
    parser.add_argument("--workspace")
    args = parser.parse_args()
    try:
        events = load_events(args.transcript)
        if args.syntax_only:
            return
        for name in ("prompt", "provider", "model", "session", "started", "workspace"):
            if getattr(args, name) is None:
                fail(f"missing --{name}")
        print(json.dumps(validate(events, args), sort_keys=True, separators=(",", ":")))
    except (OSError, ValueError) as error:
        print(f"gate-k transcript validation: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

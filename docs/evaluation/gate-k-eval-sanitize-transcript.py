#!/usr/bin/env python3
"""Remove only documented provider signatures from an otherwise valid Pi stream."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import fail, loads


SIGNATURES = {"textSignature": "text", "thinkingSignature": "thinking"}


def reject_signatures(value: object, location: str) -> None:
    if type(value) is list:
        for index, item in enumerate(value):
            reject_signatures(item, f"{location}[{index}]")
    elif type(value) is dict:
        for key, item in value.items():
            if key in SIGNATURES:
                fail(f"{location}.{key} occurs outside its documented content block")
            reject_signatures(item, f"{location}.{key}")


def sanitize_content(value: object, location: str) -> object:
    if type(value) is not list:
        reject_signatures(value, location)
        return value
    result: list[object] = []
    for index, item in enumerate(value):
        item_location = f"{location}[{index}]"
        if type(item) is not dict:
            reject_signatures(item, item_location)
            result.append(item)
            continue
        cleaned = dict(item)
        for signature, expected_type in SIGNATURES.items():
            if signature not in cleaned:
                continue
            if cleaned.get("type") != expected_type or type(cleaned[signature]) is not str:
                fail(f"{item_location}.{signature} occurs outside its documented content block")
            del cleaned[signature]
        reject_signatures(cleaned, item_location)
        result.append(cleaned)
    return result


def sanitize_message(value: object, location: str) -> object:
    if type(value) is not dict or "content" not in value:
        reject_signatures(value, location)
        return value
    result = dict(value)
    result["content"] = sanitize_content(result["content"], f"{location}.content")
    for key, item in result.items():
        if key != "content":
            if key in SIGNATURES:
                fail(f"{location}.{key} occurs outside its documented content block")
            reject_signatures(item, f"{location}.{key}")
    return result


def sanitize_result(value: object, location: str) -> object:
    if type(value) is not dict or "content" not in value:
        reject_signatures(value, location)
        return value
    result = dict(value)
    result["content"] = sanitize_content(result["content"], f"{location}.content")
    for key, item in result.items():
        if key != "content":
            if key in SIGNATURES:
                fail(f"{location}.{key} occurs outside its documented content block")
            reject_signatures(item, f"{location}.{key}")
    return result


def sanitize_event(value: object, location: str) -> object:
    if type(value) is not dict:
        reject_signatures(value, location)
        return value
    result = dict(value)
    event_type = result.get("type")
    handled: set[str] = set()
    if event_type in ("message_start", "message_end", "turn_end") and "message" in result:
        result["message"] = sanitize_message(result["message"], f"{location}.message")
        handled.add("message")
    if event_type == "turn_end" and "toolResults" in result and type(result["toolResults"]) is list:
        result["toolResults"] = [
            sanitize_message(message, f"{location}.toolResults[{index}]")
            for index, message in enumerate(result["toolResults"])
        ]
        handled.add("toolResults")
    if event_type == "agent_end" and "messages" in result and type(result["messages"]) is list:
        result["messages"] = [
            sanitize_message(message, f"{location}.messages[{index}]")
            for index, message in enumerate(result["messages"])
        ]
        handled.add("messages")
    if event_type == "tool_execution_end" and "result" in result:
        result["result"] = sanitize_result(result["result"], f"{location}.result")
        handled.add("result")
    for key, item in result.items():
        if key not in handled:
            if key in SIGNATURES:
                fail(f"{location}.{key} occurs outside its documented content block")
            reject_signatures(item, f"{location}.{key}")
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("stream", type=Path)
    args = parser.parse_args()
    try:
        lines = args.stream.read_text().splitlines()
        if not lines:
            fail("Pi event stream is empty")
        for index, line in enumerate(lines, 1):
            event = loads(line, f"Pi event line {index}")
            cleaned = sanitize_event(event, f"event[{index}]")
            print(json.dumps(cleaned, separators=(",", ":"), ensure_ascii=True, allow_nan=False))
    except (OSError, ValueError) as error:
        print(f"gate-k transcript sanitization: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

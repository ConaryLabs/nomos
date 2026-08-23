"""Strict, dependency-free validators for the Pi JSON event protocol."""

from __future__ import annotations

import datetime as dt
import json
import math
import re
from typing import Any


UUID = re.compile(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}")
SHA256 = re.compile(r"[0-9a-f]{64}")
RFC3339_UTC = re.compile(
    r"\d{4}-\d{2}-\d{2}T(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:\.\d{1,9})?Z"
)


def fail(message: str) -> None:
    raise ValueError(message)


def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_constant(value: str) -> None:
    fail(f"non-finite JSON number: {value}")


def loads(value: str, description: str) -> Any:
    try:
        return json.loads(value, object_pairs_hook=reject_duplicates, parse_constant=reject_constant)
    except (json.JSONDecodeError, ValueError) as error:
        fail(f"{description} is invalid: {error}")


def require_keys(value: object, required: set[str], optional: set[str], name: str) -> dict[str, object]:
    if type(value) is not dict:
        fail(f"{name} is not an object")
    keys = set(value)
    if not required <= keys or not keys <= required | optional:
        fail(f"{name} fields differ from the protocol")
    return value


def require_string(value: object, name: str) -> str:
    if type(value) is not str or not value:
        fail(f"{name} is absent or invalid")
    return value


def require_uuid(value: object, name: str) -> str:
    text = require_string(value, name)
    if UUID.fullmatch(text) is None:
        fail(f"{name} is not a protocol UUID")
    return text


def require_sha256(value: object, name: str) -> str:
    text = require_string(value, name)
    if SHA256.fullmatch(text) is None:
        fail(f"{name} is not a SHA-256 digest")
    return text


def require_rfc3339_utc(value: object, name: str) -> str:
    text = require_string(value, name)
    if RFC3339_UTC.fullmatch(text) is None:
        fail(f"{name} is not an RFC 3339 UTC timestamp")
    try:
        dt.datetime.fromisoformat(text[:-1] + "+00:00")
    except ValueError:
        fail(f"{name} is not a valid calendar timestamp")
    return text


def require_epoch_millis(value: object, name: str) -> int:
    if type(value) is not int or value < 0:
        fail(f"{name} is not a non-negative integer timestamp")
    return value


def validate_usage(value: object, name: str) -> dict[str, object]:
    usage = require_keys(
        value,
        {"input", "output", "cacheRead", "cacheWrite", "totalTokens", "cost"},
        {"reasoning", "cacheWrite1h"},
        name,
    )
    components = ["input", "output", "cacheRead", "cacheWrite"]
    for field in components + ["totalTokens", "reasoning", "cacheWrite1h"]:
        if field not in usage:
            continue
        if type(usage[field]) is not int or usage[field] < 0:
            fail(f"{name}.{field} is not a non-negative integer")
    if usage["totalTokens"] != sum(usage[field] for field in components):
        fail(f"{name}.totalTokens does not equal its token components")
    cost = require_keys(
        usage["cost"],
        {"input", "output", "cacheRead", "cacheWrite", "total"},
        set(),
        f"{name}.cost",
    )
    for field, amount in cost.items():
        if type(amount) not in (int, float) or not math.isfinite(amount) or amount < 0:
            fail(f"{name}.cost.{field} is not a finite non-negative number")
    return usage


def validate_text_content(value: object, name: str) -> list[dict[str, object]]:
    if type(value) is not list:
        fail(f"{name} is not an array")
    validated: list[dict[str, object]] = []
    for index, item in enumerate(value):
        item_name = f"{name}[{index}]"
        item = require_keys(item, {"type", "text"}, set(), item_name)
        if item["type"] != "text" or type(item["text"]) is not str:
            fail(f"{item_name} is not text content")
        validated.append(item)
    return validated


def validate_assistant_content(value: object, name: str) -> list[dict[str, object]]:
    if type(value) is not list:
        fail(f"{name} is not an array")
    validated: list[dict[str, object]] = []
    call_ids: set[str] = set()
    for index, item in enumerate(value):
        item_name = f"{name}[{index}]"
        if type(item) is not dict:
            fail(f"{item_name} is not an object")
        kind = item.get("type")
        if kind == "text":
            require_keys(item, {"type", "text"}, set(), item_name)
            if type(item["text"]) is not str:
                fail(f"{item_name}.text is not a string")
        elif kind == "thinking":
            require_keys(item, {"type", "thinking"}, set(), item_name)
            if type(item["thinking"]) is not str:
                fail(f"{item_name}.thinking is not a string")
        elif kind == "toolCall":
            require_keys(item, {"type", "id", "name", "arguments"}, {"thoughtSignature"}, item_name)
            call_id = require_string(item["id"], f"{item_name}.id")
            if call_id in call_ids:
                fail(f"{name} repeats a tool-call ID")
            call_ids.add(call_id)
            if item["name"] != "bash":
                fail(f"{item_name} names a forbidden tool")
            arguments = require_keys(item["arguments"], {"command"}, set(), f"{item_name}.arguments")
            require_string(arguments["command"], f"{item_name}.arguments.command")
            if "thoughtSignature" in item and type(item["thoughtSignature"]) is not str:
                fail(f"{item_name}.thoughtSignature is not a string")
        else:
            fail(f"{item_name} has forbidden content type")
        validated.append(item)
    return validated


def validate_message(value: object, name: str, provider: str | None = None, model: str | None = None) -> dict[str, object]:
    if type(value) is not dict:
        fail(f"{name} is not an object")
    role = value.get("role")
    if role == "user":
        message = require_keys(value, {"role", "content", "timestamp"}, set(), name)
        validate_text_content(message["content"], f"{name}.content")
        require_epoch_millis(message["timestamp"], f"{name}.timestamp")
    elif role == "assistant":
        message = require_keys(
            value,
            {"role", "content", "api", "provider", "model", "usage", "stopReason", "timestamp"},
            {"responseId", "rawStopReason"},
            name,
        )
        require_string(message["api"], f"{name}.api")
        require_string(message["provider"], f"{name}.provider")
        require_string(message["model"], f"{name}.model")
        if provider is not None and message["provider"] != provider:
            fail(f"{name} provider differs from the authenticated identity")
        if model is not None and message["model"] != model:
            fail(f"{name} model differs from the authenticated identity")
        validate_assistant_content(message["content"], f"{name}.content")
        validate_usage(message["usage"], f"{name}.usage")
        if message["stopReason"] not in ("pending", "toolUse", "stop"):
            fail(f"{name}.stopReason is invalid")
        require_epoch_millis(message["timestamp"], f"{name}.timestamp")
        for field in ("responseId", "rawStopReason"):
            if field in message and type(message[field]) is not str:
                fail(f"{name}.{field} is not a string")
    elif role == "toolResult":
        message = require_keys(
            value,
            {"role", "toolCallId", "toolName", "content", "isError", "timestamp"},
            {"details"},
            name,
        )
        require_string(message["toolCallId"], f"{name}.toolCallId")
        if message["toolName"] != "bash" or type(message["isError"]) is not bool:
            fail(f"{name} is not a structured bash result")
        validate_text_content(message["content"], f"{name}.content")
        require_epoch_millis(message["timestamp"], f"{name}.timestamp")
    else:
        fail(f"{name} has forbidden role")
    return message


def message_text(message: dict[str, object]) -> str:
    return "".join(item["text"] for item in message["content"] if item["type"] == "text")


def tool_calls(message: dict[str, object]) -> list[dict[str, object]]:
    return [item for item in message["content"] if item["type"] == "toolCall"]


def validate_update(value: object, name: str) -> dict[str, object] | None:
    event = require_keys(value, {"type", "assistantMessageEvent", "usage"}, set(), name)
    validate_usage(event["usage"], f"{name}.usage")
    update = event["assistantMessageEvent"]
    if type(update) is not dict:
        fail(f"{name}.assistantMessageEvent is not an object")
    kind = update.get("type")
    if kind in ("text_start", "thinking_start", "toolcall_start"):
        require_keys(update, {"type", "contentIndex"}, set(), f"{name}.assistantMessageEvent")
    elif kind in ("text_delta", "thinking_delta", "toolcall_delta"):
        require_keys(update, {"type", "contentIndex", "delta"}, set(), f"{name}.assistantMessageEvent")
        if type(update["delta"]) is not str:
            fail(f"{name}.assistantMessageEvent.delta is not a string")
    elif kind in ("text_end", "thinking_end"):
        require_keys(update, {"type", "contentIndex", "content"}, set(), f"{name}.assistantMessageEvent")
        if type(update["content"]) is not str:
            fail(f"{name}.assistantMessageEvent.content is not a string")
    elif kind == "toolcall_end":
        require_keys(update, {"type", "contentIndex", "toolCall"}, set(), f"{name}.assistantMessageEvent")
        calls = validate_assistant_content([update["toolCall"]], f"{name}.toolCall")
        return calls[0]
    else:
        fail(f"{name}.assistantMessageEvent type is forbidden")
    if type(update["contentIndex"]) is not int or update["contentIndex"] < 0:
        fail(f"{name}.assistantMessageEvent.contentIndex is invalid")
    return None


def validate_result_payload(value: object, name: str) -> dict[str, object]:
    result = require_keys(value, {"content"}, {"details"}, name)
    validate_text_content(result["content"], f"{name}.content")
    return result

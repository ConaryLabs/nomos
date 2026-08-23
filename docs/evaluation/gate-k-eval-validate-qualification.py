#!/usr/bin/env python3
"""Validate the complete, ordered Pi neutral-qualification receipt."""

import argparse
import json
import re
import sys
from pathlib import Path


HEADERS = [
    "PI_VERSION", "PI_INSTALL", "PI_NPM_INTEGRITY", "PI_PACKAGE_TREE_SHA256",
    "PI_NODE", "PI_NPM", "PI_BWRAP", "PI_BWRAP_SHA256", "PI_RUST_TOOLCHAIN",
    "PI_HOST_OS", "PI_TARGET_COMMIT", "PI_LANE", "PI_MODEL", "PI_AUTH_TYPE",
    "PI_EXTENSION", "PI_PROVIDER_EXTENSION", "PI_PROVIDER_PACKAGE",
    "PI_PROVIDER_INSTALL", "PI_PROVIDER_ENV", "PI_MODEL_CATALOG",
    "PI_SYSTEM_PROMPT", "PI_WORKSPACE", "PI_WORKTREE_STATUS", "PI_CONFIG_ROOT",
    "PI_SESSION", "PI_INVOCATION", "PI_BOUNDARY",
]
NEUTRAL_PROMPT = "Output exactly this line and nothing else: pi boundary preflight"


def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def fail(message: str) -> None:
    raise ValueError(message)


def parse_json(value: str, description: str) -> dict[str, object]:
    try:
        parsed = json.loads(value, object_pairs_hook=reject_duplicates)
    except (json.JSONDecodeError, ValueError) as error:
        fail(f"{description} is invalid: {error}")
    if type(parsed) is not dict:
        fail(f"{description} is not an object")
    return parsed


def text(message: object) -> str:
    if type(message) is not dict or type(message.get("content")) is not list:
        fail("neutral message content is invalid")
    return "".join(
        item.get("text", "")
        for item in message["content"]
        if type(item) is dict and item.get("type") == "text" and type(item.get("text")) is str
    )


def validate(args: argparse.Namespace) -> None:
    lines = args.qualification.read_text().splitlines()
    if len(lines) < len(HEADERS) + 8:
        fail("qualification receipt is truncated")
    values: dict[str, str] = {}
    for index, name in enumerate(HEADERS):
        prefix = f"{name} "
        if not lines[index].startswith(prefix) or not lines[index][len(prefix):]:
            fail(f"qualification header {index + 1} must be {name}")
        values[name] = lines[index][len(prefix):]
    if lines[len(HEADERS)] != "PI_EVENTS_BEGIN":
        fail("qualification event envelope is absent")
    try:
        end = lines.index("PI_EVENTS_END", len(HEADERS) + 1)
    except ValueError:
        fail("qualification event envelope is truncated")
    if lines[end + 1:] != ["PI_COLD_AGENT_BOUNDARY PASS"]:
        fail("qualification disposition is incomplete or has trailing records")
    events = [parse_json(line, "qualification event") for line in lines[len(HEADERS) + 1:end]]
    if not events:
        fail("qualification event stream is empty")

    if values["PI_VERSION"] != args.version or values["PI_HOST_OS"] != args.host:
        fail("qualification client or host identity differs from task receipt")
    if values["PI_TARGET_COMMIT"] != args.commit or values["PI_LANE"] != args.lane:
        fail("qualification candidate or lane differs from task receipt")
    model = values["PI_MODEL"].split("\t")
    if len(model) != 4 or model[0] != args.provider or model[1] != args.model or model[3] != args.thinking or not model[2]:
        fail("qualification model identity differs from task receipt")
    if values["PI_WORKTREE_STATUS"] != args.worktree:
        fail("qualification worktree status differs from task receipt")
    session_fields = values["PI_SESSION"].split()
    if len(session_fields) != 2 or session_fields[1] != "ephemeral" or not re.fullmatch(
        r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", session_fields[0]
    ):
        fail("qualification session identity is invalid")
    if values["PI_PROVIDER_ENV"] != "overrides-cleared prewarm-disabled":
        fail("qualification provider environment is not neutral")
    invocation = values["PI_INVOCATION"]
    for required in ("--mode json", "--no-session", "--offline", "--no-context-files", "--no-builtin-tools", "--tools bash"):
        if required not in invocation:
            fail(f"qualification invocation omits {required}")

    boundary = parse_json(values["PI_BOUNDARY"], "qualification boundary")
    required_boundary = {
        "schema": "nomos.pi_cold_agent_boundary@2",
        "boundaryKind": "source-preflight",
        "mode": "json",
        "targetCommit": args.commit,
        "provider": args.provider,
        "model": args.model,
        "thinking": args.thinking,
        "sessionId": session_fields[0],
        "sessionFile": None,
        "projectTrusted": False,
        "activeTools": ["bash"],
        "contextFiles": [],
        "skills": [],
        "packetManifestSha256": None,
        "binarySha256": None,
        "taskPromptSha256": None,
        "taskShape": None,
        "writablePaths": [],
        "budgets": None,
    }
    for key, expected in required_boundary.items():
        if boundary.get(key) != expected:
            fail(f"qualification boundary field {key} differs")
    sandbox = boundary.get("sandbox")
    if type(sandbox) is not dict or sandbox.get("root") != "read-only" or sandbox.get("workspace") != "read-write-only-host-mount" or sandbox.get("network") != "unshared" or sandbox.get("selfTest") != "pass":
        fail("qualification sandbox proof is incomplete")
    checks = sandbox.get("checks")
    required_checks = {
        "targetCommitResolved", "workspaceRead", "workspaceWrite", "outsideReadDenied",
        "outsideWriteDenied", "credentialEnvironmentAbsent", "networkDenied", "cargoAvailable",
    }
    if type(checks) is not dict or set(checks) != required_checks or any(checks[key] is not True for key in required_checks):
        fail("qualification sandbox checks are incomplete")

    types = [event.get("type") for event in events]
    if len(events) < 7 or types[0] != "session" or types[1] != "agent_start" or types[-2:] != ["agent_end", "agent_settled"]:
        fail("qualification lifecycle ordering is invalid")
    for kind in ("session", "agent_start", "turn_start", "turn_end", "agent_end", "agent_settled"):
        if types.count(kind) != 1:
            fail(f"qualification requires exactly one {kind}")
    if any(kind.startswith("tool_execution_") for kind in types if type(kind) is str):
        fail("qualification unexpectedly used a tool")
    if events[-2].get("willRetry") is not False:
        fail("qualification agent_end permits a retry")
    session = events[0]
    if session.get("id") != session_fields[0] or session.get("cwd") != values["PI_WORKSPACE"] or type(session.get("timestamp")) is not str:
        fail("qualification session header differs from its receipt")
    users = [event for event in events if event.get("type") == "message_end" and type(event.get("message")) is dict and event["message"].get("role") == "user"]
    assistants = [event for event in events if event.get("type") == "message_end" and type(event.get("message")) is dict and event["message"].get("role") == "assistant"]
    if len(users) != 1 or text(users[0]["message"]) != NEUTRAL_PROMPT:
        fail("qualification neutral prompt is not exact")
    if len(assistants) != 1 or assistants[0]["message"].get("provider") != args.provider or assistants[0]["message"].get("model") != args.model or assistants[0]["message"].get("stopReason") != "stop" or text(assistants[0]["message"]) != "pi boundary preflight":
        fail("qualification neutral response is not exact")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("qualification", type=Path)
    for name in ("commit", "version", "host", "provider", "model", "thinking", "lane", "worktree"):
        parser.add_argument(f"--{name}", required=True)
    args = parser.parse_args()
    try:
        validate(args)
    except (OSError, ValueError) as error:
        print(f"gate-k qualification validation: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

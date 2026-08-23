#!/usr/bin/env python3

"""Validate a human command review against immutable Gate K task records."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


SHA256 = re.compile(r"^[0-9a-f]{64}$")
ROOT_KEYS = {
    "adjudicator",
    "candidateCommit",
    "checkerCommandsSha256",
    "checkerTaskReceiptSha256",
    "findings",
    "ownerDisposition",
    "reason",
    "reviewedAllCommands",
    "reviewedCommandCounts",
    "schema",
    "subjectCommandsSha256",
    "subjectTaskReceiptSha256",
    "verdict",
}
FINDING_KEYS = {
    "commandOrdinal",
    "commandSha256",
    "kind",
    "pathToken",
    "reason",
    "record",
}
COMMAND_KEYS = {
    "arguments",
    "completed",
    "isError",
    "ordinal",
    "result",
    "tool",
    "toolCallId",
}


def fail(message: str) -> None:
    raise SystemExit(f"gate-k command adjudication: FAIL: {message}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"), object_pairs_hook=reject_duplicate_keys
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {path}: {error}")
    if not isinstance(value, dict):
        fail(f"JSON root is not an object: {path}")
    return value


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str:
    try:
        return digest_bytes(path.read_bytes())
    except OSError as error:
        fail(f"cannot hash {path}: {error}")


def nonempty_string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        fail(f"{field} must be a nonempty string")
    return value


def load_commands(path: Path) -> list[dict[str, Any]]:
    document = load_json(path)
    if set(document) != {"schema", "commands"}:
        fail(f"commands document fields differ from the schema allowlist: {path}")
    if document.get("schema") != "nomos.gate_k.commands@1":
        fail(f"commands schema differs: {path}")
    commands = document.get("commands")
    if not isinstance(commands, list) or not commands:
        fail(f"commands array is empty: {path}")
    tool_call_ids: set[str] = set()
    for ordinal, row in enumerate(commands):
        if not isinstance(row, dict) or set(row) != COMMAND_KEYS:
            fail(f"command {ordinal} fields differ from the schema allowlist: {path}")
        row_ordinal = row.get("ordinal")
        if isinstance(row_ordinal, bool) or row_ordinal != ordinal:
            fail(f"commands are not contiguous at ordinal {ordinal}: {path}")
        tool_call_id = row.get("toolCallId")
        if not isinstance(tool_call_id, str) or not tool_call_id.strip():
            fail(f"command {ordinal} has an invalid toolCallId: {path}")
        if tool_call_id in tool_call_ids:
            fail(f"command {ordinal} duplicates a toolCallId: {path}")
        tool_call_ids.add(tool_call_id)
        if row.get("tool") != "bash":
            fail(f"command {ordinal} is not a bash command: {path}")
        if row.get("completed") is not True:
            fail(f"command {ordinal} is not complete: {path}")
        if not isinstance(row.get("isError"), bool):
            fail(f"command {ordinal} has an invalid isError flag: {path}")
        arguments = row.get("arguments")
        if not isinstance(arguments, dict) or set(arguments) != {"command"}:
            fail(f"command {ordinal} arguments differ from the bash allowlist: {path}")
        command = arguments.get("command")
        if not isinstance(command, str) or not command:
            fail(f"command {ordinal} has no shell command string: {path}")
    return commands


def command_text(row: dict[str, Any]) -> str:
    arguments = row["arguments"]
    assert isinstance(arguments, dict)
    command = arguments["command"]
    assert isinstance(command, str)
    return command


def validate(
    subject: Path, checker: Path, adjudication_path: Path
) -> dict[str, Any]:
    document = load_json(adjudication_path)
    if set(document) != ROOT_KEYS:
        fail("adjudication fields differ from the schema allowlist")
    if document.get("schema") != "nomos.gate_k.command_adjudication@1":
        fail("adjudication schema differs")
    if document.get("reviewedAllCommands") is not True:
        fail("adjudicator did not affirm review of every recorded command")
    for field in ("adjudicator", "ownerDisposition", "reason"):
        nonempty_string(document.get(field), field)

    candidate = document.get("candidateCommit")
    if not isinstance(candidate, str) or not re.fullmatch(r"[0-9a-f]{40}", candidate):
        fail("candidateCommit is not a full lowercase Git commit")

    subject_receipt = load_json(subject / "task-receipt.json")
    checker_receipt = load_json(checker / "task-receipt.json")
    if subject_receipt.get("candidateCommit") != candidate:
        fail("candidateCommit differs from the subject task receipt")
    if checker_receipt.get("candidateCommit") != candidate:
        fail("candidateCommit differs from the checker task receipt")

    subject_commands_path = subject / "commands.json"
    checker_commands_path = checker / "commands.json"
    subject_receipt_path = subject / "task-receipt.json"
    checker_receipt_path = checker / "task-receipt.json"
    expected_digests = {
        "subjectTaskReceiptSha256": digest_file(subject_receipt_path),
        "checkerTaskReceiptSha256": digest_file(checker_receipt_path),
        "subjectCommandsSha256": digest_file(subject_commands_path),
        "checkerCommandsSha256": digest_file(checker_commands_path),
    }
    for field, expected in expected_digests.items():
        if document.get(field) != expected:
            fail(f"{field} does not bind the supplied task record")

    commands = {
        "subject": load_commands(subject_commands_path),
        "checker": load_commands(checker_commands_path),
    }
    counts = document.get("reviewedCommandCounts")
    expected_counts = {record: len(rows) for record, rows in commands.items()}
    if counts != expected_counts:
        fail("reviewedCommandCounts do not cover the supplied command records")

    findings = document.get("findings")
    if not isinstance(findings, list):
        fail("findings must be an array")
    seen: set[tuple[str, int, str, str]] = set()
    for index, finding in enumerate(findings):
        if not isinstance(finding, dict) or set(finding) != FINDING_KEYS:
            fail(f"finding {index} fields differ from the schema allowlist")
        record = finding.get("record")
        if not isinstance(record, str) or record not in commands:
            fail(f"finding {index} has an invalid record")
        ordinal = finding.get("commandOrdinal")
        if isinstance(ordinal, bool) or not isinstance(ordinal, int):
            fail(f"finding {index} commandOrdinal is not an integer")
        rows = commands[record]
        if ordinal < 0 or ordinal >= len(rows):
            fail(f"finding {index} commandOrdinal is outside the record")
        command_sha = finding.get("commandSha256")
        if not isinstance(command_sha, str) or not SHA256.fullmatch(command_sha):
            fail(f"finding {index} commandSha256 is invalid")
        actual_command_sha = digest_bytes(command_text(rows[ordinal]).encode("utf-8"))
        if command_sha != actual_command_sha:
            fail(f"finding {index} commandSha256 does not bind its command")
        if finding.get("kind") != "outside_workspace_path":
            fail(f"finding {index} kind is unsupported")
        path_token = nonempty_string(finding.get("pathToken"), f"finding {index} pathToken")
        nonempty_string(finding.get("reason"), f"finding {index} reason")
        identity = (record, ordinal, command_sha, path_token)
        if identity in seen:
            fail(f"finding {index} duplicates an earlier finding")
        seen.add(identity)

    expected_verdict = "fail" if findings else "pass"
    if document.get("verdict") != expected_verdict:
        fail(f"adjudication verdict must be {expected_verdict}")
    return document


def main() -> None:
    if len(sys.argv) != 4:
        fail("usage: gate-k-eval-validate-adjudication.py SUBJECT CHECKER ADJUDICATION")
    result = validate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()

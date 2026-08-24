#!/usr/bin/env python3

"""Validate a human command review against immutable Gate K task records."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

from gate_k_eval_pi_protocol import loads


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
ROOT_KEYS_V2 = ROOT_KEYS | {"protocolRevision", "records"}
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
ACCOUNTING_KEYS = {"assistantTurns", "providerReportedTokens", "toolCalls"}
DIMENSIONS = (
    "semantic_merit",
    "independence_integrity",
    "operational_compliance",
)
DIMENSION_RESULTS = {"pass", "fail", "inconclusive"}
RECORD_RESULT_KEYS = {"dimensions", "verdict", "reason"}
DIMENSION_KEYS = {"verdict", "reason", "evidence"}
EVIDENCE_KEYS = {"path", "sha256"}
SAFE_EVIDENCE_PATH = re.compile(r"^[A-Za-z0-9.][A-Za-z0-9._/-]*$")


def fail(message: str) -> None:
    raise SystemExit(f"gate-k command adjudication: FAIL: {message}")


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = loads(path.read_text(encoding="utf-8"), str(path))
    except (OSError, UnicodeError, ValueError) as error:
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
        if (
            isinstance(row_ordinal, bool)
            or not isinstance(row_ordinal, int)
            or row_ordinal != ordinal
        ):
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


def load_accounting(path: Path) -> dict[str, Any]:
    document = load_json(path)
    if set(document) != ACCOUNTING_KEYS:
        fail(f"accounting fields differ from the schema allowlist: {path}")
    for field in ("assistantTurns", "toolCalls"):
        value = document.get(field)
        if isinstance(value, bool) or not isinstance(value, int) or value < 1:
            fail(f"accounting {field} must be a positive integer: {path}")
    tokens = document.get("providerReportedTokens")
    if tokens is not None and (
        isinstance(tokens, bool) or not isinstance(tokens, int) or tokens < 0
    ):
        fail(f"accounting providerReportedTokens must be null or an integer: {path}")
    return document


def command_text(row: dict[str, Any]) -> str:
    arguments = row["arguments"]
    assert isinstance(arguments, dict)
    command = arguments["command"]
    assert isinstance(command, str)
    return command


def derive_record_verdict(dimensions: dict[str, Any]) -> str:
    values = [dimensions[name]["verdict"] for name in DIMENSIONS]
    if "fail" in values:
        return "fail"
    if "inconclusive" in values:
        return "inconclusive"
    return "pass"


def validate_evidence(record: Path, value: object, location: str) -> None:
    if not isinstance(value, list) or not value:
        fail(f"{location} evidence must be a nonempty array")
    seen: set[str] = set()
    record_root = record.resolve(strict=True)
    for index, item in enumerate(value):
        if not isinstance(item, dict) or set(item) != EVIDENCE_KEYS:
            fail(f"{location} evidence {index} fields differ from the schema allowlist")
        path = nonempty_string(item.get("path"), f"{location} evidence {index} path")
        if (
            SAFE_EVIDENCE_PATH.fullmatch(path) is None
            or path.startswith("/")
            or ".." in path
            or "//" in path
            or path in seen
        ):
            fail(f"{location} evidence {index} path is unsafe or duplicated")
        seen.add(path)
        supplied_sha = item.get("sha256")
        if not isinstance(supplied_sha, str) or SHA256.fullmatch(supplied_sha) is None:
            fail(f"{location} evidence {index} SHA-256 is invalid")
        evidence_path = record / path
        try:
            resolved = evidence_path.resolve(strict=True)
        except OSError as error:
            fail(f"{location} evidence {index} is absent: {error}")
        if record_root not in resolved.parents or not resolved.is_file() or evidence_path.is_symlink():
            fail(f"{location} evidence {index} escapes or is not a regular file")
        if digest_file(resolved) != supplied_sha:
            fail(f"{location} evidence {index} digest differs")


def validate_record_dimensions(record: Path, value: object, label: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != RECORD_RESULT_KEYS:
        fail(f"{label} dimension record fields differ from the schema allowlist")
    dimensions = value.get("dimensions")
    if not isinstance(dimensions, dict) or set(dimensions) != set(DIMENSIONS):
        fail(f"{label} dimension names differ from the protocol")
    for name in DIMENSIONS:
        row = dimensions[name]
        if not isinstance(row, dict) or set(row) != DIMENSION_KEYS:
            fail(f"{label}.{name} fields differ from the schema allowlist")
        if row.get("verdict") not in DIMENSION_RESULTS:
            fail(f"{label}.{name} verdict is invalid")
        nonempty_string(row.get("reason"), f"{label}.{name} reason")
        validate_evidence(record, row.get("evidence"), f"{label}.{name}")
    expected = derive_record_verdict(dimensions)
    if value.get("verdict") != expected:
        fail(f"{label} verdict must derive as {expected}")
    nonempty_string(value.get("reason"), f"{label} reason")
    return value


def validate_v2(
    document: dict[str, Any],
    subject: Path,
    checker: Path,
    commands: dict[str, list[dict[str, Any]]],
    findings: list[dict[str, Any]],
) -> None:
    if document.get("protocolRevision") != 6:
        fail("revision-6 adjudication protocolRevision differs")
    records = document.get("records")
    if not isinstance(records, dict) or set(records) != {"subject", "checker"}:
        fail("revision-6 adjudication records differ from the protocol")
    results = {
        "subject": validate_record_dimensions(subject, records["subject"], "records.subject"),
        "checker": validate_record_dimensions(checker, records["checker"], "records.checker"),
    }
    for index, finding in enumerate(findings):
        record = finding["record"]
        path_token = finding["pathToken"]
        if path_token == "/dev/null":
            fail(f"finding {index} treats the declared /dev/null exception as forbidden")
        kind = finding["kind"]
        if kind not in ("outside_workspace_path", "undeclared_information_ingress"):
            fail(f"finding {index} kind is unsupported")
        dimensions = results[record]["dimensions"]
        if dimensions["operational_compliance"]["verdict"] != "fail":
            fail(f"finding {index} does not force operational compliance to fail")
        if (
            kind == "undeclared_information_ingress"
            and dimensions["independence_integrity"]["verdict"] != "fail"
        ):
            fail(f"finding {index} does not force independence integrity to fail")

    receipts = {
        "subject": load_json(subject / "task-receipt.json"),
        "checker": load_json(checker / "task-receipt.json"),
    }
    for label, receipt in receipts.items():
        independence = results[label]["dimensions"]["independence_integrity"]
        if (
            receipt.get("identity", {}).get("freshEphemeralSession") is not True
            or receipt.get("operatorRetries") != 0
            or receipt.get("operatorIntervention") != "none"
        ) and independence["verdict"] != "fail":
            fail(f"{label} eligibility, hint, or retry breach does not fail independence")
    assisted = any(
        receipt.get("operatorIntervention") != "none" for receipt in receipts.values()
    )
    if assisted:
        expected = "assisted"
    elif any(result["verdict"] == "fail" for result in results.values()):
        expected = "fail"
    elif any(result["verdict"] == "inconclusive" for result in results.values()):
        expected = "inconclusive"
    else:
        expected = "pass"
    if document.get("verdict") != expected:
        fail(f"revision-6 overall verdict must derive as {expected}")

    for label, receipt in receipts.items():
        outcome = receipt.get("outcome")
        if outcome == "inconclusive" and results[label]["verdict"] == "pass":
            fail(f"{label} transport is inconclusive but its dimensions derive pass")
    checker_result = load_json(checker / "artifacts" / "checker.json")
    if (
        checker_result.get("verdict") != "pass"
        and results["subject"]["dimensions"]["semantic_merit"]["verdict"]
        == "pass"
    ):
        fail("checker rejection is inconsistent with passing subject semantic merit")
    if expected == "pass":
        if receipts["subject"].get("outcome") != "eligible-for-checker":
            fail("passing revision-6 result has an ineligible subject transport")
        if receipts["checker"].get("outcome") != "completed-checker":
            fail("passing revision-6 result has an incomplete checker transport")
        if checker_result.get("verdict") != "pass":
            fail("passing revision-6 result has a rejecting checker")


def validate(
    subject: Path, checker: Path, adjudication_path: Path
) -> dict[str, Any]:
    document = load_json(adjudication_path)
    # The shell finalizer also consumes these documents. Parse them with the
    # duplicate-key-rejecting loader before any final output can be constructed.
    for record in (subject, checker):
        for name in (
            "boundary.json",
            "packet-manifest.json",
            "plan.json",
            "task-receipt.json",
        ):
            load_json(record / name)
        load_accounting(record / "accounting.json")
    load_json(checker / "artifacts" / "checker.json")
    revision_6 = document.get("schema") == "nomos.gate_k.command_adjudication@2"
    if set(document) != (ROOT_KEYS_V2 if revision_6 else ROOT_KEYS):
        fail("adjudication fields differ from the schema allowlist")
    if document.get("schema") not in (
        "nomos.gate_k.command_adjudication@1",
        "nomos.gate_k.command_adjudication@2",
    ):
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
    if (
        not isinstance(counts, dict)
        or set(counts) != set(expected_counts)
        or any(
            isinstance(value, bool) or not isinstance(value, int) or value < 1
            for value in counts.values()
        )
    ):
        fail("reviewedCommandCounts must contain positive integer subject/checker counts")
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
        if not revision_6 and finding.get("kind") != "outside_workspace_path":
            fail(f"finding {index} kind is unsupported")
        path_token = nonempty_string(finding.get("pathToken"), f"finding {index} pathToken")
        nonempty_string(finding.get("reason"), f"finding {index} reason")
        identity = (record, ordinal, command_sha, path_token)
        if identity in seen:
            fail(f"finding {index} duplicates an earlier finding")
        seen.add(identity)

    if revision_6:
        validate_v2(document, subject, checker, commands, findings)
    else:
        expected_verdict = "fail" if findings else "pass"
        if document.get("verdict") != expected_verdict:
            fail(f"adjudication verdict must be {expected_verdict}")
    return document


def main() -> None:
    if len(sys.argv) != 4:
        fail("usage: gate-k-eval-validate-adjudication.py SUBJECT CHECKER ADJUDICATION")
    result = validate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    print(json.dumps(result, sort_keys=True, separators=(",", ":"), allow_nan=False))


if __name__ == "__main__":
    main()

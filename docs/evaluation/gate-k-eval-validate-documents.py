#!/usr/bin/env python3
"""Exact validators for the public Gate K evaluation JSON documents."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import (
    fail,
    loads,
    require_keys,
    require_rfc3339_utc,
    require_sha256,
    require_string,
    require_uuid,
)


SHAPES = {"author", "author-checker", "debug", "debug-checker"}
CLASSES = {"formal", "rehearsal"}
OUTCOMES = {"eligible-for-checker", "completed-checker", "inconclusive"}
LEGACY_CHECKER_SHA256S = {
    "4d929c41181cde3d74bd7f1ede4e493c544abd0229c1c70ec720cffc02f265e1",
    "b295c8cb1baa75e23596381417bc9a53f6c321d52890dfcbab670226a353f055",
    "e5b7cf8bb73f40f320f85679c993c4873b38741a418c06087ed8fc82f3e49e8b",
    "542a95264677d61877259d8ddff7bba9caea6b1554a649aab7dcb2fe2ad4c2cf",
    "561706bdaf737c5957e3c9c629f80340035288f4b7d919be3ce4b76bbfcc3066",
    "0ff0c4b06af93b465bcc7f2231b2bd3f23166f8197b0e2713aee5dc51684d7fb",
    "80040babed0cda1d6581d905431f32ce05d19e16ed783fb14119c7791a0388b0",
    "f4905b68667c6627bf3f2b5b77cc62bfa2389a765615bf1209590e5e43c8e65b",
    "66d1a5ce0ddb6ab3d76646343ce9a3f86e3333277cbf983f977160f3d30c67c4",
    "ee3e7c0a6138210f9b54cef311edb799290c98ab00d1939e62d5617a853270f6",
    "78de3fc96562c7172cc7485ad3925de6732bcb78b91b2b1b68d2be07b497a951",
}
SHA1 = re.compile(r"[0-9a-f]{40}")
SAFE_PATH = re.compile(r"[A-Za-z0-9.][A-Za-z0-9._/-]*")
SCHEMA_ID = re.compile(r"[a-z][a-z0-9_.]*@[1-9][0-9]*")


def canonical(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n"


def load(path: Path, enforce_canonical: bool = True) -> object:
    text = path.read_text()
    value = loads(text, str(path))
    if enforce_canonical and canonical(value) != text:
        fail(f"{path} is not canonical sorted compact JSON")
    return value


def require_int(value: object, name: str) -> int:
    if type(value) is not int or value < 0:
        fail(f"{name} is not a non-negative integer")
    return value


def require_bool(value: object, name: str) -> bool:
    if type(value) is not bool:
        fail(f"{name} is not a boolean")
    return value


def validate_plan(value: object) -> None:
    plan = require_keys(value, {"schema", "task", "candidate", "packet", "budgets", "rubric",
                                "recording", "operatorIntervention", "verdicts"}, set(), "plan")
    if plan["schema"] != "nomos.gate_k.eval_plan@1":
        fail("plan schema differs")
    task = require_keys(plan["task"], {"shape", "classification", "formalAttempt"}, set(), "plan.task")
    if task["shape"] not in SHAPES or task["classification"] not in CLASSES:
        fail("plan task identity is invalid")
    require_bool(task["formalAttempt"], "plan.task.formalAttempt")
    if task["formalAttempt"] != (task["classification"] == "formal"):
        fail("plan formal classification is inconsistent")
    candidate = require_keys(plan["candidate"], {"commit", "binaryPath", "binarySha256"}, set(), "plan.candidate")
    if type(candidate["commit"]) is not str or SHA1.fullmatch(candidate["commit"]) is None:
        fail("plan candidate commit is invalid")
    if candidate["binaryPath"] != "bin/nomos":
        fail("plan candidate binary path differs")
    require_sha256(candidate["binarySha256"], "plan candidate binary")
    packet = require_keys(plan["packet"], {"briefPath", "briefSha256", "promptPath", "promptSha256",
                                "writablePaths", "repositoryMounted", "gitMetadataPresent",
                                "networkPermitted", "activeTools"}, set(), "plan.packet")
    if packet["briefPath"] != "brief.txt" or packet["promptPath"] != "prompt.txt":
        fail("plan packet document paths differ")
    require_sha256(packet["briefSha256"], "plan brief")
    require_sha256(packet["promptSha256"], "plan prompt")
    if packet["writablePaths"] not in (["workspace"], ["output"]):
        fail("plan writable path differs")
    if any(packet[field] is not False for field in ("repositoryMounted", "gitMetadataPresent", "networkPermitted")):
        fail("plan packet isolation flags differ")
    if packet["activeTools"] != ["bash"]:
        fail("plan active tools differ")
    budgets = require_keys(plan["budgets"], {"freshSessions", "operatorRetriesMaximum",
                                             "operatorSubstantiveHintsMaximum"}, set(), "plan.budgets")
    if budgets != {"freshSessions": 1, "operatorRetriesMaximum": 0,
                   "operatorSubstantiveHintsMaximum": 0}:
        fail("plan budgets differ")
    recording = require_keys(plan["recording"], {"eventStream", "removedProviderFields",
                                                  "commandOrderPreserved", "transcriptLossLimit"}, set(), "plan.recording")
    if recording != {"eventStream": "complete-ndjson",
                      "removedProviderFields": ["textSignature", "thinkingSignature"],
                      "commandOrderPreserved": True,
                      "transcriptLossLimit": "only-the-two-declared-provider-signature-fields"}:
        fail("plan recording contract differs")
    if type(plan["rubric"]) is not list or len(plan["rubric"]) < 3 or any(type(x) is not str or not x for x in plan["rubric"]):
        fail("plan rubric is invalid")
    if plan["operatorIntervention"] != "none" or plan["verdicts"] != ["pass", "fail", "assisted", "inconclusive"]:
        fail("plan intervention or verdict vocabulary differs")


def validate_manifest(value: object) -> None:
    manifest = require_keys(value, {"schema", "candidateCommit", "shape", "manifestExcludesSelf",
                                    "writablePaths", "files"}, set(), "packet manifest")
    if manifest["schema"] != "nomos.gate_k.packet_manifest@1" or type(manifest["candidateCommit"]) is not str or SHA1.fullmatch(manifest["candidateCommit"]) is None:
        fail("packet manifest identity is invalid")
    if manifest["shape"] not in SHAPES or manifest["manifestExcludesSelf"] is not True:
        fail("packet manifest shape or self policy differs")
    if manifest["writablePaths"] not in (["workspace"], ["output"]):
        fail("packet manifest writable path differs")
    if type(manifest["files"]) is not list or not manifest["files"]:
        fail("packet manifest file list is empty")
    paths: list[str] = []
    for index, item in enumerate(manifest["files"]):
        row = require_keys(item, {"path", "bytes", "mode", "sha256", "schemaIdentity"}, set(), f"manifest.files[{index}]")
        path = require_string(row["path"], f"manifest.files[{index}].path")
        if SAFE_PATH.fullmatch(path) is None or path.startswith("/") or ".." in path or "//" in path:
            fail(f"manifest.files[{index}].path is unsafe")
        require_int(row["bytes"], f"manifest.files[{index}].bytes")
        if row["mode"] not in ("644", "755"):
            fail(f"manifest.files[{index}].mode is invalid")
        require_sha256(row["sha256"], f"manifest.files[{index}].sha256")
        if row["schemaIdentity"] is not None and (type(row["schemaIdentity"]) is not str or SCHEMA_ID.fullmatch(row["schemaIdentity"]) is None):
            fail(f"manifest.files[{index}].schemaIdentity is invalid")
        paths.append(path)
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        fail("packet manifest paths are not unique and sorted")


def validate_task_receipt(value: object) -> None:
    required = {"schema", "shape", "classification", "formalAttempt", "candidateCommit", "identity",
                "environment", "disclosures", "operatorIntervention", "operatorRetries", "accounting",
                "outcome", "outcomeReason", "digests"}
    receipt = require_keys(value, required, {"attemptReservation", "execution"}, "task receipt")
    if receipt["schema"] != "nomos.gate_k.task_receipt@1" or receipt["shape"] not in SHAPES or receipt["classification"] not in CLASSES:
        fail("task receipt identity is invalid")
    require_bool(receipt["formalAttempt"], "task receipt formalAttempt")
    if receipt["formalAttempt"] != (receipt["classification"] == "formal"):
        fail("task receipt formal classification is inconsistent")
    if type(receipt["candidateCommit"]) is not str or SHA1.fullmatch(receipt["candidateCommit"]) is None:
        fail("task receipt candidate is invalid")
    identity = require_keys(receipt["identity"], {"provider", "model", "thinking", "sessionId", "sessionStartedAt",
                                                  "client", "clientVersion", "mode", "freshEphemeralSession"}, set(), "task receipt identity")
    for field in ("provider", "model", "thinking", "clientVersion"):
        require_string(identity[field], f"task receipt identity {field}")
    require_uuid(identity["sessionId"], "task receipt session")
    require_rfc3339_utc(identity["sessionStartedAt"], "task receipt session start")
    if identity["client"] != "Pi" or identity["mode"] != "json" or identity["freshEphemeralSession"] is not True:
        fail("task receipt client lifecycle differs")
    environment = require_keys(receipt["environment"], {"hostOs"}, set(), "task receipt environment")
    require_string(environment["hostOs"], "task receipt host OS")
    disclosures = require_keys(receipt["disclosures"], {"persistedSession", "projectMemory", "personalContext",
                                                        "contextFiles", "connectors", "webAccess", "toolNetworkAccess",
                                                        "activeTools", "repositoryMounted"}, set(), "task receipt disclosures")
    expected_disclosures = {"persistedSession": False, "projectMemory": False, "personalContext": False,
                            "contextFiles": [], "connectors": [], "webAccess": False,
                            "toolNetworkAccess": False, "activeTools": ["bash"], "repositoryMounted": False}
    if disclosures != expected_disclosures:
        fail("task receipt disclosures differ")
    if receipt["operatorIntervention"] != "none" or receipt["operatorRetries"] != 0:
        fail("task receipt operator accounting differs")
    accounting = require_keys(receipt["accounting"], {"assistantTurns", "providerReportedTokens", "toolCalls"}, set(), "task receipt accounting")
    for field in accounting:
        require_int(accounting[field], f"task receipt accounting {field}")
    if receipt["outcome"] not in OUTCOMES:
        fail("task receipt outcome is invalid")
    require_string(receipt["outcomeReason"], "task receipt outcome reason")
    digests = require_keys(receipt["digests"], {"packetManifestSha256", "transcriptSha256", "commandsSha256",
                                                "artifactsTreeSha256", "boundarySha256", "qualificationSha256"},
                           {"rawTranscriptSha256"}, "task receipt digests")
    for field in digests:
        require_sha256(digests[field], f"task receipt digest {field}")
    if "attemptReservation" in receipt:
        reservation = receipt["attemptReservation"]
        if receipt["formalAttempt"]:
            reservation = require_keys(reservation, {"attemptId", "ledgerSha256", "ledgerCommit"}, set(), "task receipt attempt reservation")
            require_string(reservation["attemptId"], "task receipt attempt ID")
            require_sha256(reservation["ledgerSha256"], "task receipt ledger digest")
            if type(reservation["ledgerCommit"]) is not str or SHA1.fullmatch(reservation["ledgerCommit"]) is None:
                fail("task receipt ledger commit is invalid")
        elif reservation is not None:
            fail("rehearsal task receipt has an attempt reservation")
    if "execution" in receipt:
        validate_execution(receipt["execution"], "task receipt execution")


def validate_execution(value: object, name: str) -> None:
    execution = require_keys(value, {"pi", "providerExtension", "bubblewrap"}, set(), name)
    for field in ("pi", "bubblewrap"):
        row = require_keys(execution[field], {"path", "sha256"}, set(), f"{name}.{field}")
        if not require_string(row["path"], f"{name}.{field}.path").startswith("/"):
            fail(f"{name}.{field}.path is not absolute")
        require_sha256(row["sha256"], f"{name}.{field}.sha256")
    if execution["providerExtension"] is not None:
        row = require_keys(execution["providerExtension"], {"path", "sha256"}, set(), f"{name}.providerExtension")
        if not require_string(row["path"], f"{name}.providerExtension.path").startswith("/"):
            fail(f"{name}.providerExtension.path is not absolute")
        require_sha256(row["sha256"], f"{name}.providerExtension.sha256")


def validate_checker(value: object, digest: str) -> None:
    if type(value) is not dict:
        fail("checker result is not an object")
    keys = set(value)
    base = {"schema", "verdict", "commands", "reasons"}
    legacy = digest in LEGACY_CHECKER_SHA256S
    if keys not in (base, base | {"evidence"}) and not legacy:
        fail("checker result top-level fields differ from a declared protocol shape")
    if value["schema"] != "nomos.gate_k.checker_result@1" or value["verdict"] not in ("pass", "reject"):
        fail("checker result identity or verdict is invalid")
    if type(value["commands"]) is not list or not value["commands"] or type(value["reasons"]) is not list or not value["reasons"]:
        fail("checker result commands or reasons are empty")
    for index, command in enumerate(value["commands"]):
        if type(command) is str:
            require_string(command, f"checker command {index}")
        else:
            row = (
                command
                if legacy and type(command) is dict
                else require_keys(command, {"command"}, set(), f"checker command {index}")
            )
            require_string(row["command"], f"checker command {index}")
    for index, reason in enumerate(value["reasons"]):
        require_string(reason, f"checker reason {index}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=("plan", "manifest", "task-receipt", "checker-result"))
    parser.add_argument("path", type=Path)
    args = parser.parse_args()
    try:
        value = load(args.path, enforce_canonical=args.kind != "checker-result")
        if args.kind == "checker-result":
            validate_checker(value, hashlib.sha256(args.path.read_bytes()).hexdigest())
        else:
            {"plan": validate_plan, "manifest": validate_manifest,
             "task-receipt": validate_task_receipt}[args.kind](value)
    except (OSError, ValueError) as error:
        print(f"gate-k evaluation document validation: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

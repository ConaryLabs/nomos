#!/usr/bin/env python3
"""Validate and extend the append-only formal-attempt reservation ledger."""

import argparse
import hashlib
import json
import re
import stat
import subprocess
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import fail, loads, require_keys, require_sha256, require_string


BASE = {"schema", "sequence", "previousEventSha256", "event", "attemptId",
        "candidateCommit", "shape", "provider", "model", "thinking",
        "packetManifestSha256"}
IDENTITY = ("candidateCommit", "shape", "provider", "model", "thinking", "packetManifestSha256")
FROZEN_IMPORT_EVENT_SHA256S = (
    "3b94641e19bf07907ee7528e9b5ce1b5050d749600f86d6c79e735cd41f930d5",
    "0f0d6e671d655e78918f964469199190d50819914fc9cc5f4bd82811dd8b97ef",
    "800639cd9af0b6b3e9810ecf9cb6caa917057cd4e871e752ba8169c1b84ad4b7",
    "9a226d12a0abc01578c7bdf21472d292a0e422190600490072ed05b191e06f79",
)
FROZEN_INVENTORY_SHA256 = "69162588b5a43b0456e739d49a34a2d329f4f53d8d6527a45997ec064ecba794"


def event_sha(line: str) -> str:
    return hashlib.sha256((line + "\n").encode()).hexdigest()


def canonical(event: dict[str, object]) -> str:
    return json.dumps(
        event, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False
    )


def validate_event(event: object, sequence: int, previous: str | None) -> dict[str, object]:
    if type(event) is not dict:
        fail(f"ledger event {sequence} is not an object")
    kind = event.get("event")
    if kind == "reserve":
        optional = {"promptSha256", "nonce"}
    elif kind in ("close", "import-close"):
        optional = {"taskReceiptSha256", "outcome"}
    else:
        optional = {"outcome", "reason"}
    require_keys(event, BASE | optional, set(), f"ledger event {sequence}")
    if (event["schema"] != "nomos.gate_k.formal_attempt_event@1" or
            type(event["sequence"]) is not int or event["sequence"] != sequence):
        fail(f"ledger event {sequence} has invalid schema or sequence")
    if event["previousEventSha256"] != previous:
        fail(f"ledger event {sequence} breaks the hash chain")
    require_string(event["attemptId"], f"ledger event {sequence} attempt ID")
    if type(event["candidateCommit"]) is not str or re.fullmatch(r"[0-9a-f]{40}", event["candidateCommit"]) is None:
        fail(f"ledger event {sequence} candidate is invalid")
    if event["shape"] not in ("author", "author-checker", "debug", "debug-checker"):
        fail(f"ledger event {sequence} shape is invalid")
    for field in ("provider", "model", "thinking"):
        require_string(event[field], f"ledger event {sequence} {field}")
    require_sha256(event["packetManifestSha256"], f"ledger event {sequence} packet manifest")
    if kind == "reserve":
        require_sha256(event["promptSha256"], f"ledger event {sequence} prompt")
        require_sha256(event["nonce"], f"ledger event {sequence} nonce")
    elif kind in ("close", "import-close"):
        require_sha256(event["taskReceiptSha256"], f"ledger event {sequence} task receipt")
        if event["outcome"] not in ("eligible-for-checker", "completed-checker", "inconclusive"):
            fail(f"ledger event {sequence} outcome is invalid")
    elif kind == "cancel":
        if event["outcome"] != "discarded-before-launch":
            fail(f"ledger event {sequence} cancellation outcome is invalid")
        require_string(event["reason"], f"ledger event {sequence} cancellation reason")
    else:
        fail(f"ledger event {sequence} kind is invalid")
    return event


def load_ledger(path: Path) -> tuple[list[dict[str, object]], list[str]]:
    lines = path.read_text().splitlines()
    if not lines:
        fail("formal-attempt ledger is empty")
    events: list[dict[str, object]] = []
    open_attempts: dict[str, dict[str, object]] = {}
    seen: set[str] = set()
    previous = None
    for sequence, line in enumerate(lines, 1):
        parsed = loads(line, f"ledger event {sequence}")
        event = validate_event(parsed, sequence, previous)
        if canonical(event) != line:
            fail(f"ledger event {sequence} is not canonical JSON")
        if event["event"] == "import-close" and (
            sequence > len(FROZEN_IMPORT_EVENT_SHA256S)
            or event_sha(line) != FROZEN_IMPORT_EVENT_SHA256S[sequence - 1]
        ):
            fail(f"ledger event {sequence} is not an exact frozen Gate K import")
        attempt_id = event["attemptId"]
        if event["event"] in ("reserve", "import-close"):
            if attempt_id in seen:
                fail(f"ledger event {sequence} repeats an attempt ID")
            if open_attempts:
                fail(f"ledger event {sequence} starts before the prior attempt is closed")
            seen.add(attempt_id)
            if event["event"] == "reserve":
                open_attempts[attempt_id] = event
        else:
            if attempt_id not in open_attempts:
                fail(f"ledger event {sequence} closes an absent reservation")
            reservation = open_attempts.pop(attempt_id)
            if any(event[field] != reservation[field] for field in IDENTITY):
                fail(f"ledger event {sequence} identity differs from its reservation")
        events.append(event)
        previous = event_sha(line)
    return events, list(open_attempts)


def next_event(path: Path, values: dict[str, object]) -> str:
    events, open_attempts = load_ledger(path)
    if open_attempts and values["event"] == "reserve":
        fail("an earlier formal attempt remains open")
    if values["event"] in ("close", "cancel"):
        if open_attempts != [values["attemptId"]]:
            fail("close does not name the one open formal attempt")
        reservation = next(event for event in events if event["attemptId"] == values["attemptId"])
        for field in IDENTITY:
            values[field] = reservation[field]
    values.update({
        "schema": "nomos.gate_k.formal_attempt_event@1",
        "sequence": len(events) + 1,
        "previousEventSha256": event_sha(canonical(events[-1])),
    })
    return canonical(values)


def regular_file(record: Path, name: str) -> Path:
    path = record / name
    if not path.is_file() or path.is_symlink():
        fail(f"formal close {name} is not a regular file")
    return path


def validate_record_members(record: Path) -> None:
    files = {
        "TASK.md", "accounting.json", "boundary.json", "commands.json", "launcher.txt",
        "packet-manifest.json", "pi-qualification.txt", "pi-stderr.txt", "plan.json",
        "prompt.txt", "task-receipt.json", "transcript.ndjson",
    }
    if {entry.name for entry in record.iterdir()} != files | {"artifacts"}:
        fail("formal close task record member set differs from the exact schema")
    for name in files:
        regular_file(record, name)
    if not (record / "artifacts").is_dir() or (record / "artifacts").is_symlink():
        fail("formal close artifacts are not a regular directory")


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifacts_sha256(path: Path) -> str:
    if not path.is_dir() or path.is_symlink():
        fail("formal close artifacts are not a regular directory")
    rows = bytearray()
    for entry in sorted(path.rglob("*")):
        if entry.is_symlink() or (not entry.is_file() and not entry.is_dir()):
            fail("formal close artifacts contain a symlink or special entry")
        if entry.is_file():
            relative = entry.relative_to(path).as_posix()
            rows.extend(f"{sha256_file(entry)}  {relative}\n".encode())
    return hashlib.sha256(rows).hexdigest()


def parse_launcher(path: Path) -> dict[str, object]:
    text = path.read_text()
    if not text.endswith("\n"):
        fail("formal close launcher is not newline terminated")
    lines = text.splitlines()
    prefixes = (
        "PI_TASK_STATUS ", "PI_TASK_MODEL ", "PI_TASK_SESSION ", "PI_TASK_COMMIT ",
        "PI_TASK_PACKET_MANIFEST_SHA256 ", "PI_TASK_RAW_EVENTS_SHA256 ",
        "PI_TASK_EVENTS_SHA256 ", "PI_TASK_STDERR_SHA256 ",
        "PI_TASK_QUALIFICATION_SHA256 ", "PI_TASK_ATTEMPT_ID ",
        "PI_TASK_ATTEMPT_LEDGER_SHA256 ", "PI_TASK_ATTEMPT_LEDGER_COMMIT ",
    )
    if len(lines) != len(prefixes) + 1 or lines[-1] != "PI_COLD_AGENT_TASK RECORDED":
        fail("formal close launcher does not satisfy its exact record schema")
    values: dict[str, object] = {}
    for index, prefix in enumerate(prefixes):
        if not lines[index].startswith(prefix) or not lines[index][len(prefix):]:
            fail(f"formal close launcher record {index + 1} must be {prefix.strip()}")
        values[prefix.strip()] = lines[index][len(prefix):]
    status = values["PI_TASK_STATUS"]
    if type(status) is not str or re.fullmatch(r"[0-9]+", status) is None:
        fail("formal close launcher status is invalid")
    values["PI_TASK_STATUS"] = int(status)
    model = str(values["PI_TASK_MODEL"]).split("\t")
    if len(model) != 4 or any(not field for field in model):
        fail("formal close launcher model record is invalid")
    values["PI_TASK_MODEL"] = model
    session = str(values["PI_TASK_SESSION"]).split()
    if len(session) != 2 or re.fullmatch(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", session[0]) is None or session[1] != "ephemeral":
        fail("formal close launcher session record is invalid")
    values["PI_TASK_SESSION"] = session[0]
    if re.fullmatch(r"[0-9a-f]{40}", str(values["PI_TASK_COMMIT"])) is None:
        fail("formal close launcher candidate is invalid")
    for field in ("PI_TASK_PACKET_MANIFEST_SHA256", "PI_TASK_RAW_EVENTS_SHA256",
                  "PI_TASK_EVENTS_SHA256", "PI_TASK_STDERR_SHA256",
                  "PI_TASK_QUALIFICATION_SHA256", "PI_TASK_ATTEMPT_LEDGER_SHA256"):
        require_sha256(values[field], f"formal close launcher {field}")
    require_string(values["PI_TASK_ATTEMPT_ID"], "formal close launcher attempt ID")
    if re.fullmatch(r"[0-9a-f]{40}", str(values["PI_TASK_ATTEMPT_LEDGER_COMMIT"])) is None:
        fail("formal close launcher ledger commit is invalid")
    return values


def authenticated_close(path: Path, attempt_id: str, record: Path,
                        outcome: str, committed_repo: Path) -> str:
    events, open_attempts = load_ledger(path)
    if open_attempts != [attempt_id]:
        fail("close does not name the one open formal attempt")
    if not record.is_dir() or record.is_symlink():
        fail("formal close task record is not a regular directory")
    validate_record_members(record)
    receipt_path = regular_file(record, "task-receipt.json")
    launcher_path = regular_file(record, "launcher.txt")
    manifest_path = regular_file(record, "packet-manifest.json")
    transcript_path = regular_file(record, "transcript.ndjson")
    commands_path = regular_file(record, "commands.json")
    boundary_path = regular_file(record, "boundary.json")
    qualification_path = regular_file(record, "pi-qualification.txt")
    stderr_path = regular_file(record, "pi-stderr.txt")
    plan_path = regular_file(record, "plan.json")
    prompt_path = regular_file(record, "prompt.txt")
    accounting_path = regular_file(record, "accounting.json")
    validator = Path(__file__).with_name("gate-k-eval-validate-documents.py")
    for kind, document in (("task-receipt", receipt_path), ("plan", plan_path),
                           ("manifest", manifest_path)):
        subprocess.run([sys.executable, str(validator), kind, str(document)], check=True)
    receipt_bytes = receipt_path.read_bytes()
    receipt = loads(receipt_bytes.decode(), "formal close task receipt")
    reservation = next(event for event in events if event["attemptId"] == attempt_id)
    if receipt["formalAttempt"] is not True:
        fail("formal close receipt is not a formal attempt")
    receipt_identity = (receipt["candidateCommit"], receipt["shape"], receipt["identity"]["provider"],
                        receipt["identity"]["model"], receipt["identity"]["thinking"],
                        receipt["digests"]["packetManifestSha256"])
    if receipt_identity != tuple(reservation[field] for field in IDENTITY):
        fail("formal close receipt differs from its reservation")
    if receipt["outcome"] != outcome:
        fail("formal close outcome differs from the task receipt")
    plan = loads(plan_path.read_text(), "formal close plan")
    manifest = loads(manifest_path.read_text(), "formal close packet manifest")
    if (plan["candidate"]["commit"] != receipt["candidateCommit"] or
            plan["task"] != {"classification": "formal", "formalAttempt": True,
                             "shape": receipt["shape"]}):
        fail("formal close plan differs from the task receipt")
    if (manifest["candidateCommit"] != receipt["candidateCommit"] or
            manifest["shape"] != receipt["shape"]):
        fail("formal close manifest differs from the task receipt")
    if sha256_file(prompt_path) != plan["packet"]["promptSha256"]:
        fail("formal close prompt differs from the plan")
    if plan["packet"]["promptSha256"] != reservation["promptSha256"]:
        fail("formal close prompt differs from its reservation")
    rows = {row["path"]: row for row in manifest["files"]}
    for name in ("plan.json", "prompt.txt"):
        evidence_path = record / name
        row = rows.get(name)
        if (type(row) is not dict or row["bytes"] != evidence_path.stat().st_size or
                row["mode"] != format(stat.S_IMODE(evidence_path.stat().st_mode), "o") or
                row["sha256"] != sha256_file(evidence_path)):
            fail(f"formal close manifest does not bind {name}")
    accounting = loads(accounting_path.read_text(), "formal close accounting")
    if canonical(accounting) + "\n" != accounting_path.read_text() or accounting != receipt["accounting"]:
        fail("formal close accounting differs from the task receipt")
    bound = receipt.get("attemptReservation")
    if type(bound) is not dict or bound.get("attemptId") != attempt_id:
        fail("formal close receipt lacks its reservation ID")
    ledger_sha = hashlib.sha256(path.read_bytes()).hexdigest()
    if bound.get("ledgerSha256") != ledger_sha:
        fail("formal close receipt ledger digest differs from the open ledger")
    launcher = parse_launcher(launcher_path)
    if launcher["PI_TASK_ATTEMPT_ID"] != attempt_id:
        fail("formal close launcher attempt ID differs")
    if launcher["PI_TASK_ATTEMPT_LEDGER_SHA256"] != ledger_sha:
        fail("formal close launcher ledger digest differs")
    ledger_commit = launcher["PI_TASK_ATTEMPT_LEDGER_COMMIT"]
    if bound.get("ledgerCommit") != ledger_commit:
        fail("formal close launcher ledger commit differs")
    if require_committed(path, committed_repo) != ledger_commit:
        fail("formal close launcher does not name the committed ledger HEAD")
    status = launcher["PI_TASK_STATUS"]
    expected_outcome = (
        "inconclusive" if status != 0 else
        "completed-checker" if str(receipt["shape"]).endswith("-checker") else
        "eligible-for-checker"
    )
    if outcome != expected_outcome:
        fail("formal close launcher status differs from the task outcome")
    expected_reason = (
        f"Pi transport exited {status}" if status != 0 else
        "checker transport and protocol accounting complete; checker artifact requires final assembly"
        if str(receipt["shape"]).endswith("-checker") else
        "subject transport and protocol accounting complete; task merit requires checker adjudication"
    )
    if receipt["outcomeReason"] != expected_reason:
        fail("formal close launcher status differs from the task outcome reason")
    if launcher["PI_TASK_COMMIT"] != receipt["candidateCommit"]:
        fail("formal close launcher candidate differs")
    if launcher["PI_TASK_PACKET_MANIFEST_SHA256"] != receipt["digests"]["packetManifestSha256"] or sha256_file(manifest_path) != receipt["digests"]["packetManifestSha256"]:
        fail("formal close launcher packet differs")
    model = launcher["PI_TASK_MODEL"]
    if len(model) != 4 or (model[0], model[1], model[3]) != tuple(receipt["identity"][field] for field in ("provider", "model", "thinking")):
        fail("formal close launcher model differs")
    if launcher["PI_TASK_SESSION"] != receipt["identity"]["sessionId"]:
        fail("formal close launcher session differs")
    evidence = (
        ("PI_TASK_RAW_EVENTS_SHA256", None, "rawTranscriptSha256"),
        ("PI_TASK_EVENTS_SHA256", transcript_path, "transcriptSha256"),
        ("PI_TASK_QUALIFICATION_SHA256", qualification_path, "qualificationSha256"),
    )
    for launcher_field, evidence_path, receipt_field in evidence:
        digest = receipt["digests"][receipt_field]
        if launcher[launcher_field] != digest or (evidence_path is not None and sha256_file(evidence_path) != digest):
            fail(f"formal close launcher does not bind {receipt_field}")
    if launcher["PI_TASK_STDERR_SHA256"] != sha256_file(stderr_path):
        fail("formal close launcher does not bind stderr evidence")
    for evidence_path, receipt_field in (
        (commands_path, "commandsSha256"), (boundary_path, "boundarySha256")
    ):
        if sha256_file(evidence_path) != receipt["digests"][receipt_field]:
            fail(f"formal close task record does not bind {receipt_field}")
    if artifacts_sha256(record / "artifacts") != receipt["digests"]["artifactsTreeSha256"]:
        fail("formal close task record does not bind artifactsTreeSha256")
    record_validator = Path(__file__).with_name("gate-k-eval-finalize.sh")
    subprocess.run([str(record_validator), "--validate-task-record", str(record)], check=True)
    return next_event(path, {"event": "close", "attemptId": attempt_id,
                             "taskReceiptSha256": hashlib.sha256(receipt_bytes).hexdigest(),
                             "outcome": outcome})


def require_committed(path: Path, repo: Path) -> str:
    repo = repo.resolve()
    path = path.resolve()
    try:
        relative = path.relative_to(repo)
    except ValueError:
        fail("formal-attempt ledger is outside the repository")
    head = subprocess.run(["git", "-C", str(repo), "rev-parse", "HEAD"], check=True,
                          text=True, capture_output=True).stdout.strip()
    tracked = subprocess.run(["git", "-C", str(repo), "show", f"HEAD:{relative}"],
                             check=True, text=True, capture_output=True).stdout
    if tracked != path.read_text():
        fail("formal-attempt reservation is not committed at launcher HEAD")
    history = subprocess.run(
        ["git", "-C", str(repo), "log", "--format=%H", "--follow", "--", str(relative)],
        check=True, text=True, capture_output=True,
    ).stdout.splitlines()
    for commit in history:
        historical = subprocess.run(
            ["git", "-C", str(repo), "show", f"{commit}:{relative}"],
            check=True, text=True, capture_output=True,
        ).stdout
        if not tracked.startswith(historical):
            fail("formal-attempt ledger rewrites or removes committed history")
    return head


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    validate_parser = sub.add_parser("validate")
    validate_parser.add_argument("ledger", type=Path)
    frozen_parser = sub.add_parser("validate-frozen-inventory")
    frozen_parser.add_argument("ledger", type=Path)
    verify = sub.add_parser("verify-reservation")
    verify.add_argument("ledger", type=Path)
    verify.add_argument("attempt_id")
    for field in ("candidate_commit", "shape", "provider", "model", "thinking", "packet_manifest_sha256", "prompt_sha256"):
        verify.add_argument(field)
    verify.add_argument("--committed-repo", type=Path)
    reserve = sub.add_parser("next-reservation")
    reserve.add_argument("ledger", type=Path)
    reserve.add_argument("attempt_id")
    for field in ("candidate_commit", "shape", "provider", "model", "thinking", "packet_manifest_sha256", "prompt_sha256", "nonce"):
        reserve.add_argument(field)
    close = sub.add_parser("next-close")
    close.add_argument("ledger", type=Path)
    close.add_argument("attempt_id")
    close.add_argument("task_record", type=Path)
    close.add_argument("outcome")
    close.add_argument("--committed-repo", type=Path, required=True)
    cancel = sub.add_parser("next-cancel")
    cancel.add_argument("ledger", type=Path)
    cancel.add_argument("attempt_id")
    cancel.add_argument("reason")
    args = parser.parse_args()
    try:
        events, open_attempts = load_ledger(args.ledger)
        if args.command == "validate":
            if open_attempts:
                fail(f"formal attempt remains open: {open_attempts[0]}")
            return
        if args.command == "validate-frozen-inventory":
            if open_attempts:
                fail(f"formal attempt remains open: {open_attempts[0]}")
            if (
                len(events) != len(FROZEN_IMPORT_EVENT_SHA256S)
                or sha256_file(args.ledger) != FROZEN_INVENTORY_SHA256
            ):
                fail("formal-attempt ledger differs from the exact frozen Gate K inventory")
            return
        if args.command == "verify-reservation":
            if open_attempts != [args.attempt_id]:
                fail("requested formal attempt is not the one open reservation")
            reservation = next(event for event in events if event["attemptId"] == args.attempt_id)
            supplied = (args.candidate_commit, args.shape, args.provider, args.model, args.thinking,
                        args.packet_manifest_sha256, args.prompt_sha256)
            expected = tuple(reservation[field] for field in IDENTITY) + (reservation["promptSha256"],)
            if supplied != expected:
                fail("formal launch differs from its prelaunch reservation")
            if args.committed_repo:
                print(require_committed(args.ledger, args.committed_repo))
            return
        if args.command == "next-reservation":
            print(next_event(args.ledger, {
                "event": "reserve", "attemptId": args.attempt_id,
                "candidateCommit": args.candidate_commit, "shape": args.shape,
                "provider": args.provider, "model": args.model, "thinking": args.thinking,
                "packetManifestSha256": args.packet_manifest_sha256,
                "promptSha256": args.prompt_sha256, "nonce": args.nonce,
            }))
        elif args.command == "next-close":
            print(authenticated_close(args.ledger, args.attempt_id, args.task_record,
                                      args.outcome, args.committed_repo))
        else:
            print(next_event(args.ledger, {"event": "cancel", "attemptId": args.attempt_id,
                                          "outcome": "discarded-before-launch",
                                          "reason": require_string(args.reason, "cancellation reason")}))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"gate-k formal-attempt ledger: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

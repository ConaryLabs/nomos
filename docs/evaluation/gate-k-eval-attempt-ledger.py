#!/usr/bin/env python3
"""Validate and extend the append-only formal-attempt reservation ledger."""

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import fail, loads, require_keys, require_sha256, require_string


BASE = {"schema", "sequence", "previousEventSha256", "event", "attemptId",
        "candidateCommit", "shape", "provider", "model", "thinking",
        "packetManifestSha256"}
IDENTITY = ("candidateCommit", "shape", "provider", "model", "thinking", "packetManifestSha256")


def event_sha(line: str) -> str:
    return hashlib.sha256((line + "\n").encode()).hexdigest()


def canonical(event: dict[str, object]) -> str:
    return json.dumps(event, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


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
    if event["schema"] != "nomos.gate_k.formal_attempt_event@1" or event["sequence"] != sequence:
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


def one_launcher_value(text: str, prefix: str) -> str:
    values = [line[len(prefix):] for line in text.splitlines() if line.startswith(prefix)]
    if len(values) != 1 or not values[0]:
        fail(f"formal launcher does not contain exactly one {prefix.strip()} record")
    return values[0]


def authenticated_close(path: Path, attempt_id: str, receipt_path: Path,
                        launcher_path: Path, outcome: str) -> str:
    events, open_attempts = load_ledger(path)
    if open_attempts != [attempt_id]:
        fail("close does not name the one open formal attempt")
    if not receipt_path.is_file() or receipt_path.is_symlink():
        fail("formal close task receipt is not a regular file")
    if not launcher_path.is_file() or launcher_path.is_symlink():
        fail("formal close launcher receipt is not a regular file")
    validator = Path(__file__).with_name("gate-k-eval-validate-documents.py")
    subprocess.run([sys.executable, str(validator), "task-receipt", str(receipt_path)], check=True)
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
    bound = receipt.get("attemptReservation")
    if type(bound) is not dict or bound.get("attemptId") != attempt_id:
        fail("formal close receipt lacks its reservation ID")
    ledger_sha = hashlib.sha256(path.read_bytes()).hexdigest()
    if bound.get("ledgerSha256") != ledger_sha:
        fail("formal close receipt ledger digest differs from the open ledger")
    launcher = launcher_path.read_text()
    if launcher.splitlines().count("PI_COLD_AGENT_TASK RECORDED") != 1:
        fail("formal close launcher did not record a completed provider launch")
    if one_launcher_value(launcher, "PI_TASK_ATTEMPT_ID ") != attempt_id:
        fail("formal close launcher attempt ID differs")
    if one_launcher_value(launcher, "PI_TASK_ATTEMPT_LEDGER_SHA256 ") != ledger_sha:
        fail("formal close launcher ledger digest differs")
    ledger_commit = one_launcher_value(launcher, "PI_TASK_ATTEMPT_LEDGER_COMMIT ")
    if bound.get("ledgerCommit") != ledger_commit or re.fullmatch(r"[0-9a-f]{40}", ledger_commit) is None:
        fail("formal close launcher ledger commit differs")
    if re.fullmatch(r"[0-9]+", one_launcher_value(launcher, "PI_TASK_STATUS ")) is None:
        fail("formal close launcher status is invalid")
    if one_launcher_value(launcher, "PI_TASK_COMMIT ") != receipt["candidateCommit"]:
        fail("formal close launcher candidate differs")
    if one_launcher_value(launcher, "PI_TASK_PACKET_MANIFEST_SHA256 ") != receipt["digests"]["packetManifestSha256"]:
        fail("formal close launcher packet differs")
    model = one_launcher_value(launcher, "PI_TASK_MODEL ").split("\t")
    if len(model) != 4 or (model[0], model[1], model[3]) != tuple(receipt["identity"][field] for field in ("provider", "model", "thinking")):
        fail("formal close launcher model differs")
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
    close.add_argument("task_receipt", type=Path)
    close.add_argument("launcher", type=Path)
    close.add_argument("outcome")
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
            print(authenticated_close(args.ledger, args.attempt_id, args.task_receipt,
                                      args.launcher, args.outcome))
        else:
            print(next_event(args.ledger, {"event": "cancel", "attemptId": args.attempt_id,
                                          "outcome": "discarded-before-launch",
                                          "reason": require_string(args.reason, "cancellation reason")}))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"gate-k formal-attempt ledger: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

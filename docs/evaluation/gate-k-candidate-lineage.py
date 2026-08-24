#!/usr/bin/env python3
"""Prove the semantic lineage of the proposed Gate K round-two candidate."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


RC1_COMMIT = "d8a0b85c55aa33c20f46e5dfd9e0d1f317e1f1c9"

ROOT_PROTECTED = (
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "LICENSE",
    "THESIS.md",
    "rust-toolchain.toml",
)

PREFIX_PROTECTED = (
    ".github/workflows/",
    "crates/",
    "fixtures/",
    "xtask/",
)

STATUS_AND_HYGIENE = {
    ".gitattributes",
    ".gitignore",
    "README.md",
    "docs/HANDOFF.md",
}

FROZEN_ROUND_ONE = {
    "docs/decisions/0013-gate-k-disposition.md": (
        "ee254d177cbd37f4182641b0bb04b88dffead6e196c3a575586f342f93b3c1af"
    ),
    "docs/evaluation/gate-k-formal-attempt-ledger.jsonl": (
        "69162588b5a43b0456e739d49a34a2d329f4f53d8d6527a45997ec064ecba794"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-gemini-3.7-flash-author/result.json": (
        "e6990dacde903f527d1cb46784a54d938a7e130f1193e51bb830a4a2284f07dc"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-gemini-3.7-flash-author/subject-task/task-receipt.json": (
        "732af45918ebc27c02675f6c75c32e7718407545c9fa3a39de327d3591d382a8"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-gemini-3.7-flash-author/checker-task/task-receipt.json": (
        "2e8c97d5a939ddd6fa9b33769f6e24b80fc242b1420c2660eef7f9742d542db3"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-deepseek-v4-flash-vision-exp-debug/result.json": (
        "f09c9214329f7f8bd7d4d4b31476a0f24c825add2f5bb434b7bf780f64d8089c"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-deepseek-v4-flash-vision-exp-debug/subject-task/task-receipt.json": (
        "2820d2f46b2d895abc22b6677f4f3ba908199cdb9d057aee181b477eaeb82390"
    ),
    "docs/evaluation/runs/gate-k/2026-08-23-deepseek-v4-flash-vision-exp-debug/checker-task/task-receipt.json": (
        "0053d3df610e7e31322a2cfd9dfc641e160d3e5c64582df387d34cd4ddd37d37"
    ),
}

OLD_KERNEL_STATUS = (
    b"status: Implementation complete through SW-N; Gate K evidence closure in progress\n"
)
NEW_KERNEL_STATUS = (
    b"status: Gate K failed; acceptance 17 and 18 failed; implementation complete through SW-N\n"
)
KERNEL_DECISION = (
    b"decision_record: docs/decisions/0009-transition-explanation-input-boundary.md\n"
)
KERNEL_DISPOSITION = (
    b"disposition_record: docs/decisions/0013-gate-k-disposition.md\n"
)
KERNEL_INSERTION_POINT = (
    b"criterion because an implementation failed it is not.\n"
)
KERNEL_DISPOSITION_PARAGRAPH = (
    "\nGate K was finally dispositioned `failed` on 2026-08-23 in decision 0013.\n"
    "Criteria 1–16 and 19 passed; the one formal cold-author and cold-debug attempts\n"
    "failed criteria 17 and 18 under their frozen rubric. Contract revision 7 is\n"
    "unchanged, no retry is authorized, and this status does not authorize Gate 0 or\n"
    "renderer work.\n"
).encode("utf-8")


class LineageFailure(RuntimeError):
    pass


def run_git(root: Path, *arguments: str, binary: bool = False) -> str | bytes:
    result = subprocess.run(
        ["git", "-C", str(root), *arguments],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        stderr = result.stderr.decode("utf-8", "replace").strip()
        raise LineageFailure(stderr or f"git {' '.join(arguments)} failed")
    if binary:
        return result.stdout
    return result.stdout.decode("utf-8").strip()


def resolve_commit(root: Path, revision: str) -> str:
    resolved = run_git(root, "rev-parse", "--verify", f"{revision}^{{commit}}")
    assert isinstance(resolved, str)
    return resolved


def read_blob(root: Path, commit: str, path: str) -> bytes:
    blob = run_git(root, "show", f"{commit}:{path}", binary=True)
    assert isinstance(blob, bytes)
    return blob


def changed_paths(root: Path, base: str, candidate: str) -> list[str]:
    raw = run_git(root, "diff", "--name-only", "-z", base, candidate, binary=True)
    assert isinstance(raw, bytes)
    paths = [part.decode("utf-8") for part in raw.split(b"\0") if part]
    if paths != sorted(paths):
        raise LineageFailure("Git returned a non-canonical changed-path order")
    return paths


def root_public_documents(root: Path, commit: str) -> set[str]:
    raw = run_git(
        root,
        "ls-tree",
        "-r",
        "-z",
        "--name-only",
        commit,
        "docs",
        binary=True,
    )
    assert isinstance(raw, bytes)
    documents: set[str] = set()
    for entry in raw.split(b"\0"):
        if not entry:
            continue
        path = entry.decode("utf-8")
        if (
            path.count("/") == 1
            and path.endswith(".md")
            and path != "docs/HANDOFF.md"
        ):
            documents.add(path)
    return documents


def expected_kernel(base_kernel: bytes) -> bytes:
    for needle, description in (
        (OLD_KERNEL_STATUS, "round-one status"),
        (KERNEL_DECISION, "contract decision record"),
        (KERNEL_INSERTION_POINT, "contract-repair paragraph"),
    ):
        if base_kernel.count(needle) != 1:
            raise LineageFailure(f"KERNEL.md has no unique {description}")

    expected = base_kernel.replace(OLD_KERNEL_STATUS, NEW_KERNEL_STATUS, 1)
    expected = expected.replace(
        KERNEL_DECISION,
        KERNEL_DECISION + KERNEL_DISPOSITION,
        1,
    )
    expected = expected.replace(
        KERNEL_INSERTION_POINT,
        KERNEL_INSERTION_POINT + KERNEL_DISPOSITION_PARAGRAPH,
        1,
    )
    return expected


def classify(path: str) -> str:
    if path == "KERNEL.md":
        return "kernel_disposition_only"
    if path in STATUS_AND_HYGIENE:
        return "repository_status_or_hygiene"
    if path.startswith("docs/decisions/"):
        return "owner_authorized_decision"
    if path.startswith("docs/evaluation/"):
        return "evaluation_tooling_or_evidence"
    if path.startswith("docs/review/"):
        return "review_evidence"
    if path.startswith("experiments/gate-0-gaol-target-pack/"):
        return "quarantined_static_experiment"
    raise LineageFailure(f"unclassified path changed since gate-k-rc1: {path}")


def digest_paths(paths: list[str]) -> str:
    payload = b"".join(path.encode("utf-8") + b"\0" for path in paths)
    return hashlib.sha256(payload).hexdigest()


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: gate-k-candidate-lineage.py gate-k-rc1 CANDIDATE",
            file=sys.stderr,
        )
        return 2

    root_text = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=False,
        capture_output=True,
        text=True,
    )
    if root_text.returncode != 0:
        print("candidate-lineage: not inside a Git worktree", file=sys.stderr)
        return 1
    root = Path(root_text.stdout.strip()).resolve()

    try:
        base = resolve_commit(root, sys.argv[1])
        candidate = resolve_commit(root, sys.argv[2])
        if base != RC1_COMMIT:
            raise LineageFailure(
                f"base is {base}, expected frozen gate-k-rc1 {RC1_COMMIT}"
            )

        ancestor = subprocess.run(
            ["git", "-C", str(root), "merge-base", "--is-ancestor", base, candidate],
            check=False,
        )
        if ancestor.returncode != 0:
            raise LineageFailure("gate-k-rc1 is not an ancestor of the candidate")

        protected = set(ROOT_PROTECTED)
        protected.update(root_public_documents(root, base))
        protected.update(root_public_documents(root, candidate))
        for path in sorted(protected):
            if read_blob(root, base, path) != read_blob(root, candidate, path):
                raise LineageFailure(f"protected file changed since gate-k-rc1: {path}")

        paths = changed_paths(root, base, candidate)
        for path in paths:
            if path in protected or path.startswith(PREFIX_PROTECTED):
                raise LineageFailure(f"protected path changed since gate-k-rc1: {path}")

        base_kernel = read_blob(root, base, "KERNEL.md")
        candidate_kernel = read_blob(root, candidate, "KERNEL.md")
        if candidate_kernel != expected_kernel(base_kernel):
            raise LineageFailure(
                "KERNEL.md differs by more than the exact decision-0013 disposition edit"
            )

        for path, expected_sha256 in FROZEN_ROUND_ONE.items():
            actual_sha256 = hashlib.sha256(read_blob(root, candidate, path)).hexdigest()
            if actual_sha256 != expected_sha256:
                raise LineageFailure(f"frozen round-one evidence changed: {path}")

        classes: dict[str, list[str]] = defaultdict(list)
        for path in paths:
            classes[classify(path)].append(path)

        result = {
            "schema": "nomos.gate_k.candidate_lineage@1",
            "status": "pass",
            "base": {
                "commit": base,
                "tag": "gate-k-rc1",
                "tree": run_git(root, "rev-parse", f"{base}^{{tree}}"),
            },
            "candidate": {
                "commit": candidate,
                "tree": run_git(root, "rev-parse", f"{candidate}^{{tree}}"),
            },
            "protected": {
                "rootFiles": list(ROOT_PROTECTED),
                "prefixes": list(PREFIX_PROTECTED),
                "publicDocuments": sorted(protected.difference(ROOT_PROTECTED)),
                "status": "byte-identical",
            },
            "kernelContract": {
                "contractRevision": 7,
                "status": "exact-disposition-only-delta",
                "candidateSha256": hashlib.sha256(candidate_kernel).hexdigest(),
            },
            "changedPaths": {
                "count": len(paths),
                "sha256": digest_paths(paths),
                "classifications": {
                    name: {
                        "count": len(class_paths),
                        "sha256": digest_paths(class_paths),
                    }
                    for name, class_paths in sorted(classes.items())
                },
            },
            "roundOne": {
                "disposition": "failed",
                "criteria17And18": "failed",
                "preservation": "exact-hash-match",
                "frozenFiles": [
                    {"path": path, "sha256": sha256}
                    for path, sha256 in sorted(FROZEN_ROUND_ONE.items())
                ],
            },
        }
        print(json.dumps(result, sort_keys=True, separators=(",", ":")))
        return 0
    except (LineageFailure, UnicodeDecodeError) as error:
        print(f"candidate-lineage: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Authenticate the complete, ordered Pi neutral-qualification receipt."""

import argparse
import hashlib
import importlib.util
import os
import re
import sys
from pathlib import Path

from gate_k_eval_pi_protocol import (
    LEGACY_TASK_RECEIPT_SHA256S,
    fail,
    loads,
    require_keys,
    require_sha256,
    require_string,
    require_uuid,
)


_TRANSCRIPT_PATH = Path(__file__).with_name("gate-k-eval-validate-transcript.py")
_TRANSCRIPT_SPEC = importlib.util.spec_from_file_location("gate_k_eval_validate_transcript", _TRANSCRIPT_PATH)
if _TRANSCRIPT_SPEC is None or _TRANSCRIPT_SPEC.loader is None:
    raise RuntimeError("could not load transcript validator")
_TRANSCRIPT_MODULE = importlib.util.module_from_spec(_TRANSCRIPT_SPEC)
_TRANSCRIPT_SPEC.loader.exec_module(_TRANSCRIPT_MODULE)
validate_transcript = _TRANSCRIPT_MODULE.validate


HEADERS = [
    "PI_VERSION", "PI_INSTALL", "PI_NPM_INTEGRITY", "PI_PACKAGE_TREE_SHA256",
    "PI_NODE", "PI_NPM", "PI_BWRAP", "PI_BWRAP_SHA256", "PI_RUST_TOOLCHAIN",
    "PI_HOST_OS", "PI_TARGET_COMMIT", "PI_LANE", "PI_MODEL", "PI_AUTH_TYPE",
    "PI_EXTENSION", "PI_PROVIDER_EXTENSION", "PI_PROVIDER_PACKAGE",
    "PI_PROVIDER_INSTALL", "PI_PROVIDER_ENV", "PI_MODEL_CATALOG",
    "PI_SYSTEM_PROMPT", "PI_WORKSPACE", "PI_WORKTREE_STATUS", "PI_CONFIG_ROOT",
    "PI_SESSION", "PI_INVOCATION", "PI_BOUNDARY",
]
PROMPT = "Output exactly this line and nothing else: pi boundary preflight"
PI_INTEGRITY = "sha512-l4E+B7hgXKWddRo8bC/eSue2aWZjEgJ9xIpf5p0Og+lq8a2TArCwJ0HCoCPCgaBP/tN4zbYH/wOwvx9pJpeLCA=="
PI_TREE = "63a9dd14b0ae82cee2db30c56822682af19145d145febb58b613d5de4dbb27af"
BWRAP_SHA = "6ad2138a73d592acb43525432965e3c66f6fad8a2f3d610c6ca0b6855e993cbe"
RUST = "1.98.0-x86_64-unknown-linux-gnu"
LEGACY_EXTENSION_SHA = "5076b923aad8ebf6d46110ca0bd45e62911ace563bdfe58e6418b6a14b519f46"
CURRENT_EXTENSION_SHA = "d242e2de63e1228a32d7e890bf7cb852e71680813a7aa8f55fc500af64923043"
PI_CLIENT_SHA = "840d1e8e689ed9e4937bcb00b9a810e02a8567d9afb10a47097f11ca93ea1521"
PROVIDER_EXTENSION_SHA = "1c41a45c2820eb52f1b41955ae5fbb833470cba2203226d3b0c626c6f9dbe10b"
LEGACY_SYSTEM_SHA = "2cec3aeebce2f8359cde337d3b1b2ec1601913711f282ab0289ab276b02dee79"
SYSTEM_SHA = "c1c41bf11dd3fc42f47c174b9d431e36dd87afb60aa04d08062dd6e11963c333"
FINAL_SYSTEM_SHA = "a78cae9025d8b63562a13c111e79e9f27c32ab20e726a53d2d9d8c094712e2b7"
DEEPSEEK_CATALOG_SHA = "7954fb3ef750bed773619c9fe259a8eb923b6f4f8455442a33cf8e1fe2fa3773"
ANTIGRAVITY_INTEGRITY = "sha512-Trl0lWZRDM6TUhw8UjZ+si4Tx2IxCtLLdEwQ10gOS3BUJfgv/C32HY3m/v9PcLNZWYzo+LEfmamiB5+f0jciCg=="
ANTIGRAVITY_TREE = "7980e6825a23f18a9d298953c0efc9f13c1231ce4c814394803b9da9bfb565ce"
LANES = {
    "deepseek": ("deepseek", "deepseek-v4-flash-vision-exp", "DeepSeek V4 Flash Vision Exp", "max", "api_key"),
    "gemini": ("antigravity", "gemini-3.7-flash", "Gemini 3.7 Flash", "high", "oauth"),
    "claude": ("anthropic", "claude-opus-5", "Claude Opus 5", "high", "oauth"),
}


def parse_path_digest(value: str, name: str) -> tuple[str, str]:
    try:
        path, digest = value.rsplit(" ", 1)
    except ValueError:
        fail(f"{name} lacks a path and digest")
    if not os.path.isabs(path):
        fail(f"{name} path is not absolute")
    require_sha256(digest, f"{name} digest")
    return path, digest


def file_sha256(path: str, name: str) -> str:
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
    except OSError as error:
        fail(f"{name} cannot be authenticated: {error}")


def legacy_task_receipt_digest(args: argparse.Namespace) -> str | None:
    if args.task_receipt is None:
        return None
    try:
        return hashlib.sha256(args.task_receipt.read_bytes()).hexdigest()
    except OSError as error:
        fail(f"legacy task receipt cannot be authenticated: {error}")


def parse_receipt(path: Path) -> tuple[dict[str, str], list[dict[str, object]]]:
    lines = path.read_text().splitlines()
    if len(lines) < len(HEADERS) + 11:
        fail("qualification receipt is truncated")
    values: dict[str, str] = {}
    for index, name in enumerate(HEADERS):
        prefix = f"{name} "
        if not lines[index].startswith(prefix) or not lines[index][len(prefix):]:
            fail(f"qualification header {index + 1} must be {name}")
        values[name] = lines[index][len(prefix):]
    cursor = len(HEADERS)
    if lines[cursor].startswith("PI_RAW_EVENTS_SHA256 "):
        values["PI_RAW_EVENTS_SHA256"] = require_sha256(
            lines[cursor].split(" ", 1)[1], "qualification raw event digest"
        )
        cursor += 1
    if lines[cursor] != "PI_EVENTS_BEGIN":
        fail("qualification event envelope is absent")
    try:
        end = lines.index("PI_EVENTS_END", cursor + 1)
    except ValueError:
        fail("qualification event envelope is truncated")
    if lines[end + 1:] != ["PI_COLD_AGENT_BOUNDARY PASS"]:
        fail("qualification disposition is incomplete or has trailing records")
    events = [loads(line, "qualification event") for line in lines[cursor + 1:end]]
    if not events or any(type(event) is not dict for event in events):
        fail("qualification event stream is empty or malformed")
    return values, events


def validate_headers(values: dict[str, str], args: argparse.Namespace) -> tuple[str, str, str]:
    frozen_legacy_receipt = (
        legacy_task_receipt_digest(args) in LEGACY_TASK_RECEIPT_SHA256S
    )
    if values["PI_VERSION"] != args.version or values["PI_VERSION"] != "0.84.2":
        fail("qualification Pi version is not the pinned client")
    if values["PI_INSTALL"] != f"npm install -g --ignore-scripts @earendil-works/pi-coding-agent@{args.version}":
        fail("qualification Pi install recipe differs from the pinned client")
    if values["PI_NPM_INTEGRITY"] != PI_INTEGRITY:
        fail("qualification Pi registry integrity differs")
    fixture = args.worktree == "fixture-may-be-dirty"
    if (not fixture and values["PI_PACKAGE_TREE_SHA256"] != PI_TREE) or (fixture and values["PI_PACKAGE_TREE_SHA256"] != "fixture"):
        fail("qualification Pi package tree differs")
    if re.fullmatch(r"v\d+\.\d+\.\d+", values["PI_NODE"]) is None or re.fullmatch(r"\d+\.\d+\.\d+", values["PI_NPM"]) is None:
        fail("qualification Node/npm identity is invalid")
    if not fixture:
        if values["PI_BWRAP"] != "bubblewrap 0.11.2" or values["PI_BWRAP_SHA256"] != BWRAP_SHA or values["PI_RUST_TOOLCHAIN"] != RUST:
            fail("qualification sandbox or Rust binary identity differs from the pinned environment")
    else:
        require_sha256(values["PI_BWRAP_SHA256"], "fixture bwrap digest")
        if not values["PI_BWRAP"].startswith("bubblewrap 0.11.2") or not values["PI_RUST_TOOLCHAIN"]:
            fail("fixture sandbox or Rust identity is invalid")
    if values["PI_HOST_OS"] != args.host or not values["PI_HOST_OS"].startswith("Linux "):
        fail("qualification host identity differs from task receipt")
    if values["PI_TARGET_COMMIT"] != args.commit or re.fullmatch(r"[0-9a-f]{40}", args.commit) is None:
        fail("qualification candidate differs from task receipt")
    if values["PI_LANE"] != args.lane or args.lane not in LANES:
        fail("qualification lane differs from task receipt")
    provider, model, label, thinking, auth = LANES[args.lane]
    if values["PI_MODEL"].split("\t") != [provider, model, label, thinking]:
        fail("qualification model tuple differs from the pinned lane")
    if (provider, model, thinking) != (args.provider, args.model, args.thinking) or values["PI_AUTH_TYPE"] != auth:
        fail("qualification authenticated model identity differs from task receipt")
    extension, extension_sha = parse_path_digest(values["PI_EXTENSION"], "Pi extension")
    if (not extension.endswith("/docs/evaluation/pi-cold-agent-extension.ts") or
            extension_sha not in (LEGACY_EXTENSION_SHA, CURRENT_EXTENSION_SHA)):
        fail("qualification boundary extension differs from the pinned source")
    source_boundary = loads(values["PI_BOUNDARY"], "qualification boundary")
    expected_system_sha = (
        LEGACY_SYSTEM_SHA
        if isinstance(source_boundary, dict)
        and source_boundary.get("schema") == "nomos.pi_cold_agent_boundary@2"
        else SYSTEM_SHA
    )
    system_prompt, system_sha = parse_path_digest(values["PI_SYSTEM_PROMPT"], "Pi system prompt")
    if (
        not system_prompt.endswith("/docs/evaluation/pi-cold-agent-system-prompt.txt")
        or system_sha != expected_system_sha
    ):
        fail("qualification system prompt differs from the pinned source")
    if values["PI_PROVIDER_ENV"] != "overrides-cleared prewarm-disabled":
        fail("qualification provider environment is not neutral")
    if values["PI_WORKTREE_STATUS"] != args.worktree or not os.path.isabs(values["PI_WORKSPACE"]):
        fail("qualification workspace status is ineligible")
    config = values["PI_CONFIG_ROOT"].rsplit(" ", 1)
    expected_profile = "ephemeral-auth-plus-pinned-model-catalog" if args.lane == "deepseek" else "ephemeral-auth-only"
    if len(config) != 2 or not os.path.isabs(config[0]) or config[1] != expected_profile:
        fail("qualification ephemeral config profile differs")
    session = values["PI_SESSION"].split()
    if len(session) != 2 or session[1] != "ephemeral":
        fail("qualification session header is invalid")
    require_uuid(session[0], "qualification session ID")

    if args.lane == "gemini":
        expected_package = f"pi-antigravity@0.4.0 {ANTIGRAVITY_INTEGRITY} {ANTIGRAVITY_TREE}"
        if fixture:
            if values["PI_PROVIDER_PACKAGE"] != "pi-antigravity fixture" or values["PI_PROVIDER_INSTALL"] != "fixture":
                fail("fixture Gemini provider identity differs")
        elif values["PI_PROVIDER_PACKAGE"] != expected_package or values["PI_PROVIDER_INSTALL"] != "npm install -g --ignore-scripts --legacy-peer-deps pi-antigravity@0.4.0":
            fail("qualification Gemini provider package differs")
        if not os.path.isabs(values["PI_PROVIDER_EXTENSION"]):
            fail("qualification Gemini provider extension is not absolute")
        if not fixture and (
            not values["PI_PROVIDER_EXTENSION"].endswith(
                "/lib/node_modules/pi-antigravity/src/index.ts"
            )
            or (
                not frozen_legacy_receipt
                and file_sha256(
                    values["PI_PROVIDER_EXTENSION"],
                    "Gemini provider extension",
                )
                != PROVIDER_EXTENSION_SHA
            )
        ):
            fail("qualification Gemini provider extension differs from the pinned package entry point")
    elif any(values[field] != "none" for field in ("PI_PROVIDER_EXTENSION", "PI_PROVIDER_PACKAGE", "PI_PROVIDER_INSTALL")):
        fail("qualification lane unexpectedly loads a provider extension")
    if args.lane == "deepseek":
        _, catalog_sha = parse_path_digest(values["PI_MODEL_CATALOG"], "DeepSeek model catalog")
        if catalog_sha != DEEPSEEK_CATALOG_SHA:
            fail("qualification DeepSeek model catalog differs")
    elif values["PI_MODEL_CATALOG"] != "none":
        fail("qualification lane unexpectedly loads a model catalog")

    provider_flag = "" if values["PI_PROVIDER_EXTENSION"] == "none" else f" -e {values['PI_PROVIDER_EXTENSION']}"
    expected_invocation = (
        f"pi --provider {provider} --model {model} --thinking {thinking} --mode json --no-session "
        f"--no-approve --offline --no-extensions{provider_flag} -e {extension} --no-skills "
        f"--no-prompt-templates --no-themes --no-context-files --no-builtin-tools --tools bash "
        f"--system-prompt <sha256:{expected_system_sha}> <neutral-prompt>"
    )
    if values["PI_INVOCATION"] != expected_invocation:
        fail("qualification invocation differs from the exact neutral command")
    return session[0], extension, system_prompt


def validate_boundary(values: dict[str, str], args: argparse.Namespace, session: str, extension: str) -> None:
    _, extension_sha = parse_path_digest(values["PI_EXTENSION"], "Pi extension")
    boundary = loads(values["PI_BOUNDARY"], "qualification boundary")
    boundary = require_keys(boundary, {
        "schema", "boundaryKind", "mode", "targetCommit", "hostWorkspace", "guestWorkspace",
        "provider", "model", "thinking", "sessionId", "sessionFile", "projectTrusted",
        "entryTypesBeforeRun", "activeTools", "configuredTools", "contextFiles", "skills",
        "systemPromptSha256", "finalSystemPromptSha256", "packetManifestSha256", "binarySha256",
        "taskPromptSha256", "taskShape", "writablePaths", "budgets", "sandbox",
    }, {"runtimeIdentity"}, "qualification boundary")
    if boundary["schema"] not in (
        "nomos.pi_cold_agent_boundary@2",
        "nomos.pi_cold_agent_boundary@3",
        "nomos.pi_cold_agent_boundary@4",
    ):
        fail("qualification boundary schema differs")
    expected_system_sha = (
        LEGACY_SYSTEM_SHA
        if boundary["schema"] == "nomos.pi_cold_agent_boundary@2"
        else SYSTEM_SHA
    )
    expected = {
        "boundaryKind": "source-preflight", "mode": "json",
        "targetCommit": args.commit, "hostWorkspace": values["PI_WORKSPACE"], "guestWorkspace": "/workspace",
        "provider": args.provider, "model": args.model, "thinking": args.thinking, "sessionId": session,
        "sessionFile": None, "projectTrusted": False,
        "entryTypesBeforeRun": ["model_change", "thinking_level_change"], "activeTools": ["bash"],
        "configuredTools": [{"name": "bash", "source": {"path": extension, "source": "cli", "scope": "temporary", "origin": "top-level"}}],
        "contextFiles": [], "skills": [], "systemPromptSha256": expected_system_sha,
        "packetManifestSha256": None, "binarySha256": None, "taskPromptSha256": None,
        "taskShape": None, "writablePaths": [], "budgets": None,
    }
    for key, expected_value in expected.items():
        if boundary[key] != expected_value:
            fail(f"qualification boundary field {key} differs")
    fixture = args.worktree == "fixture-may-be-dirty"
    if fixture:
        if boundary["finalSystemPromptSha256"] != "fixture":
            fail("qualification fixture final system prompt identity differs")
    elif boundary["schema"] == "nomos.pi_cold_agent_boundary@4":
        require_sha256(
            boundary["finalSystemPromptSha256"],
            "qualification final system prompt identity",
        )
    elif boundary["finalSystemPromptSha256"] != FINAL_SYSTEM_SHA:
        fail("qualification final system prompt identity differs")
    sandbox = require_keys(boundary["sandbox"], {"backend", "binary", "root", "workspace", "network", "environment", "checks", "selfTest"}, set(), "qualification sandbox")
    if sandbox["backend"] != "bubblewrap" or not os.path.isabs(require_string(sandbox["binary"], "qualification bwrap path")) or sandbox["root"] != "read-only" or sandbox["workspace"] != "read-write-only-host-mount" or sandbox["network"] != "unshared" or sandbox["environment"] != "cleared-and-allowlisted" or sandbox["selfTest"] != "pass":
        fail("qualification sandbox proof differs")
    checks = {"targetCommitResolved", "workspaceRead", "workspaceWrite", "outsideReadDenied", "outsideWriteDenied", "credentialEnvironmentAbsent", "networkDenied", "cargoAvailable"}
    actual_checks = require_keys(sandbox["checks"], checks, set(), "qualification sandbox checks")
    if any(actual_checks[key] is not True for key in checks):
        fail("qualification sandbox checks are incomplete")
    if not fixture and (sandbox["binary"] != "/usr/bin/bwrap" or
                        file_sha256(sandbox["binary"], "Bubblewrap") != values["PI_BWRAP_SHA256"]):
        fail("qualification Bubblewrap path does not name the pinned binary")
    if boundary["schema"] in (
        "nomos.pi_cold_agent_boundary@3",
        "nomos.pi_cold_agent_boundary@4",
    ):
        if "PI_RAW_EVENTS_SHA256" not in values:
            fail("current qualification omits the raw event-stream digest")
        if extension_sha != CURRENT_EXTENSION_SHA:
            fail("current qualification names an obsolete boundary extension")
        if not fixture and file_sha256(extension, "Pi boundary extension") != CURRENT_EXTENSION_SHA:
            fail("qualification boundary extension bytes differ")
        runtime = require_keys(
            boundary.get("runtimeIdentity"),
            {"pi", "providerExtension", "bubblewrap"},
            set(),
            "qualification runtime identity",
        )
        if not fixture:
            pi = require_keys(runtime["pi"], {"path", "sha256"}, set(), "qualification Pi executable")
            if (
                not require_string(pi["path"], "qualification Pi path").endswith(
                    "/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.js"
                )
                or pi["sha256"] != PI_CLIENT_SHA
                or file_sha256(pi["path"], "Pi executable") != PI_CLIENT_SHA
            ):
                fail("qualification Pi executable differs from the pinned package entry point")
            bubblewrap = require_keys(
                runtime["bubblewrap"], {"path", "sha256"}, set(),
                "qualification Bubblewrap identity",
            )
            if bubblewrap != {"path": "/usr/bin/bwrap", "sha256": BWRAP_SHA}:
                fail("qualification runtime Bubblewrap identity differs")
            if args.lane == "gemini":
                provider = require_keys(
                    runtime["providerExtension"], {"path", "sha256"}, set(),
                    "qualification provider extension identity",
                )
                if (provider["path"] != values["PI_PROVIDER_EXTENSION"] or
                        provider["sha256"] != PROVIDER_EXTENSION_SHA):
                    fail("qualification runtime provider extension identity differs")
            elif runtime["providerExtension"] is not None:
                fail("qualification runtime unexpectedly loads a provider extension")
    else:
        if extension_sha != LEGACY_EXTENSION_SHA:
            fail("legacy qualification boundary extension differs")
        if legacy_task_receipt_digest(args) not in LEGACY_TASK_RECEIPT_SHA256S:
            fail("legacy qualification boundary is not bound to one of the four frozen formal task receipts")
        if "runtimeIdentity" in boundary:
            fail("legacy qualification boundary unexpectedly declares runtime identity")


def validate(args: argparse.Namespace) -> None:
    values, events = parse_receipt(args.qualification)
    boundary = loads(values["PI_BOUNDARY"], "qualification boundary")
    if (type(boundary) is dict and boundary.get("schema") == "nomos.pi_cold_agent_boundary@2" and
            legacy_task_receipt_digest(args) not in LEGACY_TASK_RECEIPT_SHA256S):
        fail("legacy qualification boundary is not bound to one of the four frozen formal task receipts")
    session, extension, _ = validate_headers(values, args)
    validate_boundary(values, args, session, extension)
    session_event = events[0]
    if type(session_event) is not dict:
        fail("qualification session event is malformed")
    transcript_args = argparse.Namespace(prompt=PROMPT, provider=args.provider, model=args.model,
                                         session=session, started=session_event.get("timestamp"),
                                         workspace=values["PI_WORKSPACE"])
    accounting = validate_transcript(events, transcript_args)
    if accounting != {"assistantTurns": 1, "providerReportedTokens": accounting["providerReportedTokens"], "toolCalls": 0}:
        fail("qualification neutral lifecycle is not one assistant turn without tools")
    assistant = next(event["message"] for event in events if event["type"] == "message_end" and event["message"]["role"] == "assistant")
    from gate_k_eval_pi_protocol import message_text
    if message_text(assistant) != "pi boundary preflight":
        fail("qualification neutral response is not exact")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("qualification", type=Path)
    for name in ("commit", "version", "host", "provider", "model", "thinking", "lane", "worktree"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--task-receipt", type=Path)
    args = parser.parse_args()
    try:
        validate(args)
    except (OSError, ValueError) as error:
        print(f"gate-k qualification validation: FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()

# `agy` Gemini print-mode repair

**Verdict:** issue #17 preflight PASS; not formal cold-agent evidence

**Date:** 2026-08-22

**Issue:** #17

## Environment and route

- Client: official Antigravity `agy` 1.1.17, Linux amd64 package with installer
  checksum verification.
- Requested and resolved model:
  `gemini-3.7-flash-high` — `Gemini 3.7 Flash (High)`.
- Host: Arch Linux x86_64, kernel `7.1.8-arch1-3`.
- Operator: Codex, with the owner completing Google OAuth in a local browser.
- Repository: `/work/signed-dev/src/signed-world`.

## Diagnosis

The client was absent after the host software reinstall. Restoring 1.1.17 was
necessary but did not by itself explain the three 2026-08-21 failures: the
historical invocation reproduced the canned model greeting on the fresh,
authenticated installation.

The fault was the wrapper invocation, not repository agent configuration or the
upstream model service. `-p` / `--print` takes its prompt as the next argument.
The failed shape put `--model` immediately after `--print`, so `agy` treated the
seven-byte string `--model` as the prompt and ignored the intended trailing
positional text. The local 1.1.17 client log confirmed:

```text
Print mode: starting (promptLength=7, model="", conversationID="")
```

The current official headless documentation likewise places the prompt
immediately after `-p`: <https://www.agy.dev/docs/cli/headless/>.

The repaired invocation is prompt-first, pins the model and high effort, names
the target worktree, disables slash-command expansion, emits streaming JSON,
and pre-approves tool actions for the single-command probe. A first repaired probe
proved tool execution but revealed that an unqualified `pwd` defaulted to the
Antigravity scratch directory. The committed harness therefore instructs the
terminal tool to use the exact Git worktree and refuses unless the completed
tool event returns that path.

## Working invocation

From the target worktree:

```bash
docs/evaluation/agy-print-preflight.sh
```

The harness executes this argument order internally:

```text
agy -p <prompt> --model gemini-3.7-flash-high --effort high \
  --add-dir <exact-worktree> --dangerously-skip-permissions \
  --disable-slash-commands --output-format stream-json --print-timeout 2m
```

It fails closed unless:

- the client catalog maps the requested slug to the exact High label;
- the init event pins that slug and the exact worktree;
- a completed `run_command` tool event runs `pwd` and returns the worktree;
- the terminal result is `SUCCESS`; and
- `agy` exits zero.

The exact successful prompt is in `prompt.txt`; exact stdout is in
`transcript.txt`. The proof ended with `AGY_PREFLIGHT PASS`.

## Evidence boundary

All three 2026-08-21 attempts remain inconclusive with zero review credit. This
repair proves only the issue #17 print-mode transport and command-execution
preflight. It is not a formal cold-author or cold-debug run.

The init event deliberately remains visible in the transcript. Its broad tool
catalog does not yet satisfy the formal cold-agent ablation boundary in
`GATE_K_COLD_AGENT_PLAN.md`. Issue #45 tracks the required fail-closed
fresh-session/context/tool audit; the Gemini formal route remains blocked.

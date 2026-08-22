# `agy` formal cold-agent boundary falsification

**Verdict:** BLOCKED — Antigravity CLI 1.1.17 is ineligible for a formal
Nomos cold-agent role

**Date:** 2026-08-22

**Issue:** #45

**Probe commit:** `7a2d5771edc60393dfe900788ee9f17281166b1f`

This is a tooling-eligibility probe, not a formal cold-author or cold-debug
attempt. It spends no formal-attempt budget and supplies no Gate K evidence.

## Environment and route

- Client: official Antigravity `agy` 1.1.17.
- Requested and resolved model:
  `gemini-3.7-flash-high` — `Gemini 3.7 Flash (High)`.
- Host: Arch Linux x86_64, kernel `7.1.8-arch1-3`.
- Worktree: `/work/signed-dev/src/signed-world`.
- Custom main agent: `nomos-cold-subject`, SHA-256
  `d9b5004681113503f5411d5c28358d8710086e93796ed79426fd9356c4c17337`.
- New project: `9dab0c3c-2fb1-43d4-a594-4f46571d2b87`.
- New conversation: `bf07f12b-859c-4072-bd14-1a80f7558152`.

The custom agent declares only `view_file`, `replace_file_content`, and
`run_command`; is selectable as a main agent but not a subagent; disables MCP
and customization inheritance; declares no skills, plugins, or MCP servers;
and requests sandboxed command execution. The launch also used `--new-project`,
`--sandbox`, and `--disable-slash-commands` without
`--dangerously-skip-permissions`.

Official references used to identify the available controls:

- <https://www.agy.dev/docs/cli/headless/>
- <https://www.agy.dev/docs/subagents/>
- <https://www.agy.dev/docs/permissions/>
- <https://www.agy.dev/docs/projects/>

## Invocation

From exact probe commit `7a2d577`:

```bash
docs/evaluation/agy-formal-boundary-preflight.sh
```

The harness internally ran this prompt-first shape:

```text
agy -p <neutral-prompt> --agent nomos-cold-subject \
  --model gemini-3.7-flash-high --effort high --new-project --sandbox \
  --disable-slash-commands --output-format stream-json \
  --print-timeout 2m --log-file <temporary-log>
```

The exact neutral prompt is in `prompt.txt`. The sanitized exact output is in
`transcript.txt`. The generated new-project record's content is preserved as
`project.json`; the repository copy adds a final line feed and hashes to
`81ad6085a6c9177234fe29a03943a50e6f2319c1e7fe46526b4a194f82d57a3f`.
The original bytes had no final line feed and hashed to
`b9a942a7e985351a77c79312517d5b6273d84c7ad2e7ffe5618e4996b9082060`.
The original user-level record was removed after capture.

## What passed

- `agy` created one new project containing only the target worktree.
- The init event resolved the exact model, worktree, and custom agent.
- Permission mode remained `request-review`.
- The terminal result was `SUCCESS`, `num_turns` was `1`, and
  `cache_read_tokens` was `0`.
- The neutral probe executed no tool step.

These facts prove that new-project launch and one-turn conversation transport
work. They do not prove the formal context or tool boundary.

## Falsification

The machine-readable init event did not disclose:

- the newly created project ID;
- an empty context-source set; or
- that persisted memory was disabled.

More decisively, `init.tools` exposed 57 tools rather than the custom agent's
three-tool allowlist. The list included browser automation, `search_web`,
`read_url_content`, `call_mcp_tool`, resource access, knowledge mutation,
subagent definition/invocation/management, messaging, scheduling, image
generation, and broad file/command helpers.

Exploratory prompts against the same declarative agent caused the model to say
that `search_web` and `invoke_subagent` were unavailable, but that is model
self-report, not machine-readable effective configuration. It cannot override
the contradictory init event or prove which other declared tools are actually
inaccessible. The documented permissions engine covers file, URL, command,
unsandboxed-command, and MCP actions; it does not provide a denial namespace
for persisted knowledge, messaging, scheduling, or subagent capabilities.

Therefore 1.1.17 cannot prove the predeclared formal boundary. The harness
correctly exited `1` with `AGY_FORMAL_BOUNDARY BLOCKED`.

## Disposition

The Gemini formal cold-author/checker route remains blocked. No formal attempt
may launch through this client merely because the model verbally declines a
forbidden tool. A future client or route must make the committed preflight pass
with an exact effective tool list and explicit context/memory/project
disclosures, or the owner must approve a new plan. The cold-agent protocol,
roster, brief, budgets, and evidence rules are unchanged.

The offline harness proves a compliant fixture passes and rejects a forbidden
tool, missing init event, wrong model, wrong worktree, missing context
disclosure, reused conversation/context, and an unexpected tool call:

```bash
docs/evaluation/test-agy-formal-boundary-preflight.sh
```

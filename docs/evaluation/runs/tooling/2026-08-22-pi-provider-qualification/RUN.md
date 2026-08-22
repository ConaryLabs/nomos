# Pi provider-harness qualification

**Verdict:** all three author probes PASS; exact-head non-author reruns remain
pending

**Date:** 2026-08-22

**Issue:** #49

**Probe commit:** `8d18d455ba56f30e464466fe093764650c5b3953`

This is transport and isolation evidence only. It launches no formal
cold-author or cold-debug attempt, spends no formal attempt budget, changes no
Gate K criterion, and does not change the predeclared model-family roster.

## Pinned host and client

- Host: Linux `7.1.8-arch1-3` x86_64.
- Pi: `@earendil-works/pi-coding-agent` 0.84.2, installed with
  `npm install -g --ignore-scripts @earendil-works/pi-coding-agent@0.84.2`.
- Pi npm integrity:
  `sha512-l4E+B7hgXKWddRo8bC/eSue2aWZjEgJ9xIpf5p0Og+lq8a2TArCwJ0HCoCPCgaBP/tN4zbYH/wOwvx9pJpeLCA==`.
- Installed Pi tree SHA-256:
  `63a9dd14b0ae82cee2db30c56822682af19145d145febb58b613d5de4dbb27af`.
- Node.js 26.7.0; npm 12.0.2.
- Bubblewrap 0.11.2, binary SHA-256
  `6ad2138a73d592acb43525432965e3c66f6fad8a2f3d610c6ca0b6855e993cbe`.
- Rust toolchain `1.98.0-x86_64-unknown-linux-gnu`.
- Worktree: `/work/signed-dev/src/signed-world`, clean at the probe commit.
- Boundary extension SHA-256:
  `0e481623a0113e9dead8c75a65a2c2171fb3004acadf579655b4e5cc683d4a39`.
- Runtime system-prompt SHA-256:
  `2d29828a0cf0b96a372be5b35548e09f7a237154cef674356c133b2583622902`.

## Common invocation and boundary

All three successful lanes used `docs/evaluation/pi-cold-agent-preflight.sh` with
an explicit lane. The launcher selected the exact provider, model, and thinking
level and passed all of these controls:

```text
--mode json --no-session --no-approve --offline --no-extensions
--no-skills --no-prompt-templates --no-themes --no-context-files
--no-builtin-tools --tools bash
```

Only the repository's hash-pinned extension supplied `bash`. Its runtime event
proved the exact active and configured tool catalog before the provider request.
The subject had no MCP, web/browser/retrieval, knowledge/memory, subagent,
messaging, scheduling, built-in tool, context-file, skill, prompt-template,
theme, discovered extension, persisted session, or trusted-project surface.

The extension sent every callable shell command through Bubblewrap. The host
root and Rust toolchain were read-only, the target checkout alone was mounted
read-write at `/workspace`, the process environment was cleared and allowlisted,
and the network namespace contained only loopback. Before provider launch the
same tool backend proved the target commit, workspace read/write, denial of an
outside read, denial of an outside write, absence of credential-named child
environment variables, denial of an external network request, and Cargo
availability. Failure exits before a provider request.

Credentials remained in the user's Pi auth store. Each run copied `auth.json`
into a mode-0700 temporary configuration root. The DeepSeek lane also copied
the repository's hash-pinned declarative model catalog there. The launcher
scanned output for credential values before parsing it, removed provider
response-signature fields, and deleted the temporary root on exit. No
credential value appears in the committed receipts.

## DeepSeek model catalog

DeepSeek released `deepseek-v4-flash-vision-exp` on 2026-08-21, and Pi 0.84.2's
built-in catalog does not yet contain it. The official release announcement is
<https://api-docs.deepseek.com/news/news260821/>. The lane retains Pi's built-in
DeepSeek transport and supplies only
`docs/evaluation/pi-deepseek-models.json`, a declarative catalog entry with
SHA-256
`7954fb3ef750bed773619c9fe259a8eb923b6f4f8455442a33cf8e1fe2fa3773`.
The preflight rejects any digest or field mismatch and copies the file only into
its ephemeral configuration root. No executable DeepSeek plugin is loaded.

## Gemini provider extension

Gemini requires one additional, explicitly named provider extension because
Pi's built-in Google provider accepts an API key rather than the available
Google Cloud Code Assist OAuth entitlement:

- package: `pi-antigravity` 0.4.0;
- source tag: `v0.4.0`, Git commit
  `51783877b7194ba578e2a1a0eaa4596275d57a01`;
- npm integrity:
  `sha512-Trl0lWZRDM6TUhw8UjZ+si4Tx2IxCtLLdEwQ10gOS3BUJfgv/C32HY3m/v9PcLNZWYzo+LEfmamiB5+f0jciCg==`;
- installed tree SHA-256:
  `7980e6825a23f18a9d298953c0efc9f13c1231ce4c814394803b9da9bfb565ce`;
- install command:
  `npm install -g --ignore-scripts --legacy-peer-deps pi-antigravity@0.4.0`;
- entry point:
  `/work/signed-dev/.local/lib/node_modules/pi-antigravity/src/index.ts`.

The exact package was source-inspected before installation. It registers the
`antigravity` model provider and interactive diagnostic commands, but no
model-callable tool. Peer dependencies were deliberately omitted to avoid a
second 136 MiB Pi installation; only its declared `undici` runtime dependency
was installed. Package discovery remained disabled. The launcher loaded this
one entry point explicitly, disabled its optional connection prewarm, and the
machine-readable Pi catalog still contained only the repository-owned `bash`.
Every supported endpoint, project, runtime-model, OAuth-client, user-agent,
transport, and debug-dump environment override (including the legacy `NOAGY_`
forms) was cleared; connection prewarming alone was explicitly disabled.

This is an unofficial Google integration and is therefore trusted only as the
exact named, hashed provider transport above. It does not enter the kernel
workspace or the subject's callable tool boundary.

## Authenticated results

| Lane | Exact route | Thinking | Session | Result |
| --- | --- | --- | --- | --- |
| Gemini | `antigravity/gemini-3.7-flash` | `high` | `01a02733-d201-7d45-8672-085a7ec67c78` | PASS |
| Claude | `anthropic/claude-opus-5` | `high` | `01a02733-d44c-7eac-b7d6-7b258833b085` | PASS |
| DeepSeek | `deepseek/deepseek-v4-flash-vision-exp` | `max` | `01a02733-ad79-7b4a-b791-6d9a7b9d2e61` | PASS |

Each passing lane returned exactly `pi boundary preflight`, emitted one fresh
ephemeral session, executed zero model-requested tools, and ended without retry.
The complete sanitized output is in `gemini-author.txt` and
`claude-author.txt`; `deepseek-author.txt` contains the corresponding DeepSeek
stream. Pi 0.84.2 exposes no OAuth route for its built-in DeepSeek provider, so
that lane used an API key kept only in Pi's user auth store.

## Offline fail-closed proof

Run:

```bash
docs/evaluation/test-pi-cold-agent-preflight.sh
```

The fixture matrix passes all three lane shapes and rejects missing boundary or
session metadata, wrong provider, model, worktree, thinking level, or target
commit, a reused session, a forbidden configured tool, enabled context or skill
resources, an unexpected tool call, absent isolation, a successful outside
read, a successful outside write, successful external networking, and a leaked
credential marker. CI runs this proof without contacting any provider.

## Remaining disposition

The tooling slice is not green until a non-author reruns the exact head through
the offline proof and all three authenticated probes. Claude remains
supplemental review only; its successful transport probe does not make the
Claude family eligible for a formal whole-Gate-K subject role.

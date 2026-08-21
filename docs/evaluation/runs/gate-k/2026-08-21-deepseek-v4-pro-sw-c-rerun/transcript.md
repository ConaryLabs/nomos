## Verification report — signed-world SW-C (non-author checker)

**Model identity:** I was invoked as the Reasonix coding agent for this session.
No explicit model-name/version identifier is exposed in the session context, so
the exact provider model identity is not visible to me; I will not fabricate one.

**Constraint compliance:** COMPLIED. I used no web search, no persisted project
memory, no other model, no subagent, and none of the suggested MCP tools
(GitHub, memory, read_post). I made no edits, commits, or other mutations. The
only writes performed were cargo's own build artifacts under the git-ignored
`target/` directory, which are inherent to running the required proof commands;
no tracked file was changed.

**Required environment facts (before):**

- `git rev-parse HEAD` → `4ec25e595f5bcc6052af730d8c830c94749608b2`
  (matches required target exactly)
- `git status --short --branch` → `## main...origin/main` (no changes); branch
  is `main`, tree clean
- `rustc --version --verbose` → rustc 1.97.1 (8bab26f4f 2026-07-14), host
  `x86_64-unknown-linux-gnu`, release 1.97.1, LLVM 22.1.6
- `cargo --version --verbose` → cargo 1.97.1 (c980f4866 2026-06-30), host
  `x86_64-unknown-linux-gnu`, release 1.97.1
- `uname -a` → `Linux Apollo 7.1.8-200.fc44.x86_64 #1 SMP PREEMPT_DYNAMIC Mon
  Aug 10 03:35:23 UTC 2026 x86_64 GNU/Linux`

**Proof commands (all run by me in this session):**

1. `cargo fmt --all -- --check` → **PASS** (exit 0, no output; also run once
   with an explicit exit-code echo confirming 0)
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` → **PASS**
   (exit 0; all 6 kernel crates checked, zero warnings)
3. `cargo test --workspace --locked` → **PASS** (exit 0; every suite `ok`, 0
   failed — 72 unit/integration tests plus doctests, including the CLI exit-code,
   compiler/mutation fail-closed, canonical encoding, package tamper,
   determinism, and xtask edge-rule suites)
4. `cargo xtask boundary` → **PASS** (exit 0; `boundary: clean`, membership,
   permitted-edges, cycles, forbidden-dependency, and tooling-isolation rules
   all checked)

**Final state (after all proofs):**

- `git rev-parse HEAD` → `4ec25e595f5bcc6052af730d8c830c94749608b2`
  (unchanged)
- `git status --short --branch` → `## main...origin/main` (clean)

**Verdict: PASS** — required commit verified exactly, branch `main`, worktree
clean before and after, and all four proof commands exited 0 with passing output.

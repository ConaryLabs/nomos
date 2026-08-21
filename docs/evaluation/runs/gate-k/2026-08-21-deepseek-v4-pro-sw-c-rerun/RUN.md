# SW-C non-author rerun

**Verdict:** pass

**Date:** 2026-08-21

**Issue:** #9

**Evaluated commit:** `4ec25e595f5bcc6052af730d8c830c94749608b2`

**Evaluated branch:** `main`

This receipt satisfies the repository's non-author rerun rule for the merged
SW-C implementation. It is not a formal cold-author, cold-debug, or whole-Gate-K
run under `docs/evaluation/COLD_AGENT_PROTOCOL.md`; it upgrades no unrelated
acceptance claim.

## Reviewer and harness

- Subject reviewer: DeepSeek V4 Pro (`deepseek-v4-pro`), a non-author of SW-C.
- Provider route: `deepseek-pro/deepseek-v4-pro`.
- Client: Reasonix 1.29.0, commit `9eaa3b295`, Linux amd64 build.
- Mode: direct Reasonix execution, maximum reasoning effort, one model turn.
- Session: `20260821-124238.995035379-deepseek-v4-pro`.
- Operator: Mira; no substantive hints or interventions after launch.
- Retrieval and subagent capabilities were ablated. The prompt also forbade
  web, persisted project memory, other models, mutation, and external writes.
- The subject reported compliance. The operator inspected the exported event
  stream and independently confirmed the final commit and clean-tree state.

The exact model identifier was available in Reasonix result metadata and the
session identifier. It was not exposed inside the subject's conversational
context, so the subject correctly reported that limitation rather than
inventing an identifier.

## Environment

```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
host: x86_64-unknown-linux-gnu
LLVM version: 22.1.6

cargo 1.97.1 (c980f4866 2026-06-30)
host: x86_64-unknown-linux-gnu
os: Fedora 44.0.0 [64-bit]

Linux Apollo 7.1.8-200.fc44.x86_64 #1 SMP PREEMPT_DYNAMIC Mon Aug 10 03:35:23 UTC 2026 x86_64 GNU/Linux
```

The worktree was `main` at the exact evaluated commit and clean both before and
after the proof. Cargo wrote only ignored build artifacts under `target/`; no
tracked file, Git ref, GitHub object, or external state was changed.

## Proof

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass, exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass, exit 0, zero warnings |
| `cargo test --workspace --locked` | pass, exit 0, 72 unit/integration tests plus doctests |
| `cargo xtask boundary` | pass, exit 0, `boundary: clean` |

The first formatting invocation appended an explicit exit-code echo and passed.
The harness then rejected an analogous clippy invocation before execution
because the trailing echo could mask the verifier's exit status. The reviewer
reran clippy by itself, then reran formatting by itself before the final clean
status check. `commands.json` records the rejected attempt as `not_run`; it did
not count as a proof result or a mutation.

No new package or runtime artifact was produced, so there is no package or state
hash for this rerun. The immutable result identities are the evaluated Git
commit above and the existing frozen hash asserted by the passing determinism
test.

## Evidence limits

Reasonix exported the exact prompt, complete event stream, final transcript,
result metadata, metrics, and empty stderr log. The complete event stream is
machine-local and contains large repetitive token and test-progress events, so
this durable record preserves the exact prompt, complete final response content
normalized to Markdown, ordered command ledger, and result/checker metadata
instead. No secret or credential is included.

The run used 134,746 input tokens, of which 120,960 were cache reads, and 4,916
output tokens. Reasonix reported an estimated complete cost of USD 0.02149356.
The run completed in 88.160 seconds with no restart.

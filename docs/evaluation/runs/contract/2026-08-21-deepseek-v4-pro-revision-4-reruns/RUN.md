# Contract revision 4 DeepSeek non-author reruns

**Verdict:** pass on both exact heads

**Date:** 2026-08-21

**Issue / PR:** #15 / #16

This is the durable local receipt for two direct Reasonix reruns by DeepSeek V4
Pro at max effort. It supplements the corresponding PR #16 comments. Neither
run is an owner disposition or formal Gate K cold-debug subject run.

## Reviewer and route

- Reviewer family: DeepSeek, non-author of the GPT-authored changes.
- Client: Reasonix 1.31.1 (`668cdee70`).
- Requested provider: `deepseek-pro`.
- Resolved model from Reasonix session identity: `deepseek-v4-pro`.
- Effort: max.
- Operator: Mira.
- Mode: read-only; no edits, commits, pushes, network tools, or subagents.

## Rerun 1 — receipt-bearing proposal head

- Commit: `de7ce44d5751e3e46d7eec87449099455d284d1c`.
- Branch: `contract/world-ir-construction-lineage`.
- Session: `20260821-152318.019143762-deepseek-v4-pro`.
- Result: **PASS**, zero actionable findings.

The checker verified the complete Opus receipt, three preserved failures, issue
#15 / PR #16 references, effective revision 3 versus proposed revision 4,
construction schema identity, full-byte frozen hash guard, and evidence limits.

## Rerun 2 — owner-authorization head

- Commit: `2f754a64b75ff5ea07da750daee62fcc387f0d11`.
- Branch: `contract/world-ir-construction-lineage`.
- Session: `20260821-171434.387798151-deepseek-v4-pro`.
- Result: **PASS**, zero actionable findings.

The checker verified Peter Permenter's approval was recorded in full without
altering the reviewed amendment body; revision 4 remained effective only on
merge; revision 3 remained effective on then-current main; and no stale
proposed/effective marker or evidence overclaim remained.

## Proof commands

Both exact-head reruns independently ran:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass, exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass, exit 0 |
| `cargo test --workspace --locked` | pass, exit 0, 73 tests plus 10 doctests |
| `cargo xtask boundary` | pass, exit 0, `boundary: clean` |

Each checker confirmed the exact commit and a clean tree before and after.

Environment:

```text
reasonix v1.31.1 (668cdee70)
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1 (c980f4866 2026-06-30)
Linux 7.1.8-200.fc44.x86_64 x86_64 GNU/Linux, Fedora 44
```

## Evidence limits

These reruns do not prove Linux aarch64, the ten-run matrix, runtime behavior,
migration/replay, the command surface, or any formal cold-agent gate. The
authorization head later merged as PR #16 at `2603a4e` and its push-to-main CI
passed separately.

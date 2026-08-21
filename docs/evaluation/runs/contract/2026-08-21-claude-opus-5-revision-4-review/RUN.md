# Contract revision 4 non-author review

**Verdict:** pass after three failed review iterations

**Date:** 2026-08-21

**Issue / PR:** #15 / not opened yet

**Evaluated commit:** `090eea880a093ec60a390c760fd40c24e3cd7ed2`

**Evaluated branch:** `contract/world-ir-construction-lineage`

This is the durable receipt for independent contract and code review of the
proposed Gate K revision 4 wording. It is not owner disposition and is not a
formal whole-Gate-K cold-agent subject run.

## Reviewer and route

- Reviewer: Claude Opus 5, a non-author of this change.
- Client: Claude Code 2.1.238.
- Requested model: `opus`.
- Resolved canonical model: `claude-opus-5`.
- Effort: high.
- Sessions: `376f2c45-66eb-4250-9946-ae2ad95a67d6`,
  `c663db07-d5bf-48f6-a30e-34b4807c116b`,
  `839cf8a0-ee49-43f5-8684-1cc9c42ad97d`,
  `35253e22-28f5-4a0b-874e-b361bf00e22f`.
- Operator: Mira.
- Mode: read-only; no subagents, fallback model, web search, file edits,
  commits, pushes, GitHub writes, or resumed unrelated conversation.

Claude Code result metadata identified `claude-opus-5` as the principal review
model. Small `claude-haiku-4-5` internal usage entries appeared in result
metadata but did not act as subagents or principal reviewers. Both runs
reported zero spawned subagents.

## Review history

The failures are part of the evidence and are not normalized away:

1. Historical subject `7857ea7f8ec6fc0e306340e4bc870c1e49a4c59f` —
   **fail**. The reviewer found eleven defects, led by unauthorized effective
   metadata, undisclosed narrowing of first-commit obligations, and ambiguous
   construction migration rules. The operator amended that commit away while
   repairing it; its SHA is historical provenance, not a retrievable durable
   Git object. The complete finding and disposition sequence is preserved in
   `transcript.md`.
2. `ecc664362eee0dda76a9d721d58813e37e2b8a05` on
   `contract/world-ir-construction-lineage` — **fail**. All original findings
   were closed and all proof commands passed, but the reviewer found this
   receipt incomplete, the initial SHA insufficiently qualified, and the new
   fail-closed construction-shape obligation neither implemented nor filed.
   The reviewer also asked that SW-D version bookkeeping move out of the
   normative KERNEL body.
3. `37fa7a9288b7484f1e311f18f9db96cbd533c334` — **fail**. The
   reviewer proved every prior finding closed, independently mutation-tested
   the golden construction fixture, and reran the full proof successfully. It
   found one receipt transcription error: the reproducible unit/integration
   count is 73, not 79. Both incorrect occurrences are corrected here.
4. `090eea880a093ec60a390c760fd40c24e3cd7ed2` — **pass**. The
   reviewer reproduced 73 unit/integration tests plus 10 doctests, reconfirmed
   every prior finding closed, independently matched the 5,556-byte golden
   construction digest, found no actionable defect, and observed a clean tree
   before and after all proof commands.

## Final-head proof

At `090eea880a093ec60a390c760fd40c24e3cd7ed2`, the reviewer independently ran:

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass, exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass, exit 0 |
| `cargo test --workspace --locked` | pass, exit 0, 73 tests plus 10 doctests |
| `cargo xtask boundary` | pass, exit 0, `boundary: clean` |

The reviewer confirmed exact HEAD and an empty working tree before and after.
The first run's Cargo commands were denied by its `dontAsk` permission mode;
that failure is not presented as a rerun. The final clippy run reused valid
build cache and is not presented as a cold compile.

Environment:

```text
Claude Code 2.1.238
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1 (c980f4866 2026-06-30)
Linux 7.1.8-200.fc44.x86_64 x86_64 GNU/Linux, Fedora 44
```

## Final reviewer disposition

- effective/proposed metadata and reading order are unambiguous;
- exact prior and replacement contract wording is recorded;
- construction snapshots retain every first-commit obligation;
- incompatible construction changes require migration or an explicit epoch
  break;
- incomplete snapshots cannot occupy a valid package;
- stale stable-IR names and API descriptions are corrected;
- the fixture test pins the exact embedded schema identity and version; and
- a frozen SHA-256 fixture now pins every canonical construction-v1 byte, so a
  silent shape change under the same version fails the build.

The normative KERNEL rule no longer pre-consumes a version for SW-D. Decision
0004 remains the slice ledger and records its proposed construction-v2 epoch
break. The reviewer found no actionable defect and returned **PASS**.

## Evidence limits

The passing reviewed commit precedes this receipt-only commit. A
different-family rerun must verify the receipt-bearing PR head. This review does
not supply owner disposition, Linux aarch64 execution, the ten-run matrix, or
any runtime, migration, replay, command-surface, or formal cold-agent evidence.

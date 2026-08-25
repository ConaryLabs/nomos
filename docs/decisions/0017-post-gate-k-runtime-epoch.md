---
title: Post-Gate-K runtime epoch (R1)
status: Owner-authorized; R1 epoch open
number: 0017
date: 2026-08-25
owner: Peter Permenter
issue: 124
gate_k_disposition: docs/decisions/0013-gate-k-disposition.md
round_two_termination: docs/decisions/0016-terminate-gate-k-round-two.md
contract_revision: 7
r1_contract: RUNTIME.md
---

# Post-Gate-K runtime epoch (R1)

## Decision authority

This record was prepared under issue #124. Peter Permenter authorized it on
2026-08-25; the disposition is recorded below.

It proposes one thing: an explicit epoch break. Gate K is closed and cannot be
retried, so the executable study currently has no authorized route into accepted
work. R1 is that route, governed by its own contract document, and it is not a
Gate K pass, a Gate K waiver, or a Gate 1 claim.

## Evidence relied on

The executable evidence is the quarantined study at
`experiments/executable-gaol/`, authorized by decision 0016:

- four separately authored areas — Cistern Walk, Ember Vault, Ossuary Reach, and
  North Gaol — sharing one camera, palette, material set, assembly vocabulary,
  and renderer, with doors, water, light, and composition derived from their own
  `.nomos` sources and projections;
- Ossuary Reach was added in commit `8f71e34` (issue #113, PR #115) with no edit
  to any renderer source: the diff touches the new area's content, docs, the
  contact sheet, four lines of Ember Vault exit routing to place the new area on
  the route, and the shared area-collection test. `src/webgl-renderer.mjs`,
  `src/build-plan.mjs`, `src/render-core.mjs`, and `viewer.html` are untouched;
- the `gaol_procedural_01` look was added in commit `fec8cbb` (issue #118,
  PR #119) touching only `src/webgl-renderer.mjs`, `src/webgl-viewer.test.mjs`,
  `viewer.html`, and the README. No file under `areas/` changed.

The semantic evidence is the two cold-agent rounds. Round one is recorded in
decision 0013: the formal Gemini author produced a valid second approved door
and the DeepSeek checker reproduced its package byte-for-byte; the formal
DeepSeek debugger found the true seeded cause and Gemini independently confirmed
it. Both runs failed criteria 17 and 18 on the frozen outside-path rubric, and
that verdict stands. Round two is recorded in decision 0016: both subjects again
produced materially correct semantic analysis under a clean boundary, and
neither is a formal pass. Unfamiliar model families could operate the kernel;
neither round proves the evaluation ceremony was completed.

## What remains unchanged

- Decision 0013 remains the controlling Gate K verdict: **failed**. Criteria 17
  and 18 remain failed. No retry, round three, or protocol revision 7.
- `KERNEL.md` revision 7 is frozen as the historical Gate K contract. R1 does not
  amend it, and no R1 result may be read back onto it.
- Every historical tag, candidate, task record, rehearsal, and failed or blocked
  audit remains immutable evidence.
- `experiments/` remains non-authoritative. Nothing there satisfies acceptance.

## What changes

The R1 epoch opens. A new contract document at `RUNTIME.md` in the repository
root governs what R1 accepts; it is not yet written, and no R1 work is accepted
before it exists. R1 restates its own acceptance criteria, budgets, and
boundaries rather than inheriting Gate K's.

Promotion happens by clean implementation only. Moving or copying
`experiments/executable-gaol/` into the accepted tree is forbidden. The study is
a specification and a comparison target for R1 work, never its source of truth.

R1 dependency policy. Decision 0005 was scoped to Gate K and ended with the 0013
disposition, which admitted no dependency automatically and required each later
epoch to state its own policy. R1's policy is: third-party dependencies are
permitted, subject to a committed lockfile; each dependency vendored in-tree or
pinned by content digest; its license preserved in the tree; and each crate or
package addition recorded in `RUNTIME.md` with version, provenance, why it beats
a local implementation, whether it can affect authoritative determinism, and its
offline-proof plan. The six kernel crates keep zero third-party dependencies
until a later decision says otherwise, and `cargo xtask boundary` continues to
fail closed on them.

## First targets in order

1. **Kernel effective-facts projection** — issue #126. A read-only kernel output
   giving composed effective movement disposition, cost, reasons, and effective
   light at a runtime state, so that
   `experiments/executable-gaol/src/build-plan.mjs` stops reimplementing the
   resolvers in JavaScript.
2. **Rust rendering-plan compilation** replacing `build-plan.mjs`, consuming that
   projection rather than recomputing it.
3. **Typed presentation source** replacing the unversioned `area.json`, informed
   by the ownership audit in issue #125.
4. **Promoted viewer** with a vendored Three.js — today it is imported from
   `https://cdn.jsdelivr.net/npm/three@0.185.1/...` at
   `experiments/executable-gaol/src/webgl-renderer.mjs:1`, which R1's dependency
   policy does not permit — plus a headless Chromium smoke lane in CI.
5. **Authoritative movement and pursuit** — issue #117, deferred before
   implementation with no committed changes. It reopens here, last, once the
   presentation boundary above it is typed and owned.

Each target is a separately falsifiable issue with its own evidence. The order is
a dependency order, not a schedule.

## Reading THESIS.md section 22

`THESIS.md` is not edited by this record. Its section 22 criterion 1 — that Gate
K passes `KERNEL.md` — is now unsatisfiable: decision 0013 records the failure
and decision 0016 forbids any retry. Section 22 therefore stands as the
historical adoption bar for the Gate K era, and it is not met and cannot be met.
R1 defines its own adoption criteria in `RUNTIME.md`. Until those exist and are
satisfied, section 22's conclusion holds unchanged: this thesis applies to no
game project.

## Explicit non-claims

R1 does not claim that Gate K passed, that any failed attempt is repaired or
relabelled, that Gate 0 or Gate 1 is satisfied, that the renderer or presentation
semantics in `experiments/` are accepted, that the executable study is
production art or deterministic across GPUs, or that Nomos has been adopted by
any project. No new schema is declared by this record; schema identities are
declared by the code that emits them, under `RUNTIME.md`.

## Owner disposition

**Authorize.** Recorded by Peter Permenter on 2026-08-25, with no amendments.
R1 opens as written: the epoch break, the contract location, the dependency
policy, and the first-target order all stand as drafted. `RUNTIME.md` is the
next slice, under issue #128.

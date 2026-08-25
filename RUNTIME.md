---
title: The R1 runtime contract
status: Draft; awaiting owner authorization
epoch: R1
contract_revision: 1
authority: docs/decisions/0017-post-gate-k-runtime-epoch.md
kernel_contract: KERNEL.md revision 7, frozen
date: 2026-08-25
issue: 128
---

# The R1 runtime contract

## 1. Purpose and epoch

R1 is the post-Gate-K accepted line, authorized by
`docs/decisions/0017-post-gate-k-runtime-epoch.md` and governed by this
document. Nothing enters accepted R1 work except through the criteria below.

Gate K is closed and failed. `docs/decisions/0013-gate-k-disposition.md` is its
controlling verdict, criteria 17 and 18 remain failed, and
`docs/decisions/0016-terminate-gate-k-round-two.md` terminates round two
incomplete with no verdict. `KERNEL.md` revision 7 is frozen as the historical
Gate K contract. R1 builds on the kernel surface implemented through SW-N
without amending that contract, and no R1 result may be read back onto it as a
pass, a waiver, or partial credit. R1 states its own acceptance here rather than
inheriting Gate K's: not the cold-agent evaluation ceremony, not `KERNEL.md`
criteria 17 through 19 as written, and not the zero-third-party-dependency rule
for code outside the six kernel crates.

`THESIS.md` is not edited by this document. Its section 22 criterion 1 — that
Gate K passes `KERNEL.md` — is unsatisfiable and stands as the historical Gate K
bar. For adoption it is replaced by the five criteria below; section 22 criteria
2 through 5 stand unchanged.

1. **The first targets are accepted.** R1-1 through R1-5 are accepted under
   section 5, each with its evidence and a non-author rerun receipt.
2. **No shadow resolver survives.** Every effective movement disposition, cost,
   reason, and light fact consumed downstream in accepted code comes from the
   kernel resolvers or a compiler that consumes their output.
3. **The presentation boundary is typed and owned.** Every field of accepted
   presentation source has exactly one owner, and no accepted content carries a
   raw floating-point transform.
4. **The tree proves itself offline.** From a clean checkout the workspace
   builds, tests, and produces the public artifact with no network access, and
   section 7's budgets are recorded as numbers.
5. **Content authoring stays content-only.** A new area is added with no edit to
   renderer or compiler source, proved by the diff of the commit adding it.

Until all five hold and section 22's remaining criteria are separately
satisfied, its conclusion is unchanged: this thesis applies to no game project.
R1 acceptance is not Gate 0 and not Gate 1.

## 2. What R1 accepts, and what stays quarantined

R1 accepts Rust workspace crates, the promoted viewer, accepted content and
fixtures, and the documents that govern them; all of it is subject to every
criterion here. `experiments/` remains non-authoritative under decision 0016:
nothing there satisfies R1 acceptance, and continued work there is still
authorized as a quarantined study.

Promotion happens by clean implementation only. Moving or copying
`experiments/executable-gaol/` into the accepted tree is forbidden, file by file
and line by line. The study is a specification and a comparison target, never a
source of truth. A promoted behaviour names the study file and lines it
reproduces and the test that proves equivalence, or records the difference and
its cause.

## 3. Workspace

The six kernel crates named in `KERNEL.md` section 10 are unchanged: their
membership, their permitted edges, and their zero third-party dependencies.
`cargo xtask boundary` continues to fail closed on them.

```text
crates/nomos-*      the six existing kernel crates; unchanged, zero third-party
crates/<r1 crate>   new R1 projection and compilation crates, declared below
apps/nomos-viewer/  the promoted viewer (R1-4)
xtask/              workspace tooling; the dependency-boundary check
experiments/        quarantined study; non-authoritative
```

No R1 crate exists yet, so the declared R1 member list is empty; each new member
joins that list in the change that creates it.

Permitted new edges: an R1 crate may depend on any kernel crate, and on another
declared R1 crate while the graph stays acyclic; `apps/nomos-viewer/` consumes
published plan and presentation artifacts only.

Forbidden: any kernel crate depending on an R1 crate, on `apps/`, or on `xtask`;
any third-party dependency reachable from a kernel crate; any dependency cycle;
an R1 crate or the viewer parsing `.nomos` source, Canonical World IR, or
compiler receipts; one canonical schema identity defined in more than one crate;
an undeclared workspace member.

Extending `cargo xtask boundary` to enforce R1 membership, edge direction,
kernel purity, and viewer isolation is a target of the first R1 slice, not a
present claim: today the checker knows only the Gate K list, and its
`membership` rule fails closed on any new member until it is extended and this
section names that member.

## 4. Dependency policy

Verbatim from decision 0017:

> R1's policy is: third-party dependencies are permitted, subject to a committed
> lockfile; each dependency vendored in-tree or pinned by content digest; its
> license preserved in the tree; and each crate or package addition recorded in
> `RUNTIME.md` with version, provenance, why it beats a local implementation,
> whether it can affect authoritative determinism, and its offline-proof plan.
> The six kernel crates keep zero third-party dependencies until a later
> decision says otherwise, and `cargo xtask boundary` continues to fail closed
> on them.

Decision 0005 was scoped to Gate K and ended with the 0013 disposition, which
admitted no dependency automatically; this section is R1's own policy. Each
addition is recorded below with exactly these fields: **name**; **version**;
**provenance**, a vendored path plus upstream commit or a registry plus content
digest; **license**, its identifier and the in-tree path preserving its text;
**why not local**, what it does that a local implementation should not;
**determinism**, whether it can affect authoritative state, hashes, or receipts,
and how that is bounded; **offline proof**, how a clean checkout builds and
tests it with no network; **added by**, the issue and pull request.

### Recorded additions

None. No third-party dependency has entered the accepted tree under R1.

## 5. First targets

Five targets in decision 0017's order. The order is a dependency order, not a
schedule, and each is a separately falsifiable issue with its own evidence. This
document declares no schema identity: where a target emits a versioned artifact,
the identity and version are declared by the emitting code and named in that
target's evidence.

### R1-1 Kernel effective-facts projection

Issue #126. Given a strictly verified world package and a runtime state, emit a
read-only composed effective movement disposition, cost, ordered reasons, and
effective light for every resolver subject. The sizing spike on #126 has no pull
request yet, so this acceptance is written from the issue text; the spike's
design may make these criteria more specific and may not weaken them.

Accepted when:

- a source-review receipt names each reused Rust resolver entry point and its
  crate, and the output introduces no new resolution logic;
- the output carries the schema identity and version declared by the emitting
  code, and a consumer refuses a mismatch with a stable diagnostic;
- for all twenty gaol scenarios the output equals what
  `experiments/executable-gaol/src/build-plan.mjs:89-127` computes today, or
  every difference is recorded with its cause;
- the same package and runtime state give byte-identical canonical output across
  ten runs; the output is derived, so it stays outside the state-hash domain and
  mutates no input package or state file;
- the four kernel commands in section 6 pass.

Must not: add a second implementation of activation evaluation or of movement
and light composition anywhere in the accepted tree; add a third-party
dependency to a kernel crate; edit `KERNEL.md`.

Evidence: the reused-entry-point receipt; the twenty-scenario comparison; the
four command outputs; the exact `build-plan.mjs` lines that become deletable.

### R1-2 Rust rendering-plan compilation

A Rust compiler producing the rendering plan from the R1-1 projection and typed
presentation source, replacing `experiments/executable-gaol/src/build-plan.mjs`.

Accepted when:

- it consumes the R1-1 output and presentation source only, proved by a test
  that it never reads `.nomos` source, World IR, or compiler receipts;
- doors, water, and light are classified from typed declarations: a test renames
  a machine and an entity identifier and the classification is unchanged;
- the plan is canonical bytes under the schema identity declared by the emitting
  code, and compiling twice is byte-identical;
- for the four committed areas the emitted plan is byte-equal to the committed
  `nomos.experiment.rendering_plan@1` fixtures under one documented
  normalization, or every difference is recorded with its cause;
- no accepted build path executes `build-plan.mjs`.

Must not: classify by string convention such as `machine.endsWith(".access")` at
`experiments/executable-gaol/src/build-plan.mjs:25`; depend on a magic entity
identifier or a magic assembly string; evaluate activation expressions in
JavaScript anywhere on the accepted path; recompute an effective fact R1-1
already resolved.

Evidence: the byte comparison against the four committed plans; the
classification test; each convention-based classification removed, with its
prior file and line.

### R1-3 Typed presentation source

A versioned, typed presentation source replacing the unversioned `area.json`,
with exactly one owner per field. The ownership audit on issue #125 has no pull
request yet, so this acceptance is written from the issue text; its owner
categories — World IR, runtime state, a kernel projection, presentation source,
renderer catalog, area or gameplay graph, test fixture — become the owner column
when the audit lands under `docs/review/`.

Accepted when:

- every field of the four `area.json` files, `area-collection.example.json`, and
  one rendering plan appears in the ownership table with exactly one owner;
- the accepted source is versioned, and a version mismatch is refused with a
  stable diagnostic;
- no field has two authorities, and each former double authority is listed with
  its resolution and its prior file and line;
- positions and extents in content are integer lattice units, orientations are
  discrete steps, and attachment is by named socket;
- a schema test rejects a source file carrying a raw floating-point transform.

Must not: admit raw floating-point transforms in content; leave any fact whose
only authority is the JavaScript that happens to read it; reintroduce an
unversioned second content language into the accepted tree.

Evidence: the ownership table under `docs/review/`; the refusal test outputs;
the resolved double authorities.

### R1-4 Promoted viewer

An accepted viewer under `apps/nomos-viewer/` rendering the R1-2 plan with a
vendored Three.js and fetching nothing at runtime. It consumes published plan
and presentation artifacts only.

Accepted when:

- Three.js is vendored in-tree or pinned by content digest, its license
  preserved in the tree, recorded under section 4; the CDN import at
  `experiments/executable-gaol/src/webgl-renderer.mjs:1` appears nowhere in the
  accepted tree;
- a check over the built public artifact finds no external-origin script, style,
  module, or fetch target, and the artifact loads and plays with the machine
  offline;
- a headless Chromium smoke lane in CI loads the artifact, drives the route to
  the final escape, and fails on any console error or unhandled rejection;
- the artifact contains no `.nomos` source, World IR, compiler receipt,
  credential, or path from the build machine;
- adding an area edits no file under `apps/nomos-viewer/`, proved by the diff of
  the commit adding it.

Must not: perform any runtime network fetch in the public artifact; require a
renderer-specific edit to accept new content; ship without the browser smoke
lane in CI.

Evidence: the section 4 entry for Three.js; the offline load receipt; the CI
smoke-lane run identifier; the public artifact size from section 7; the
area-addition diff.

### R1-5 Authoritative movement and pursuit

Issue #117, deferred before implementation with no committed changes. It reopens
here, last, once the presentation boundary above it is typed and owned.

Accepted when:

- runtime state carries the schema identity and version declared by the emitting
  code, an authoritative tick, and a stable ordered actor collection;
- authoritative actor positions are integers in lattice units;
- each input resolves as exactly one deterministic command batch, and the
  ordering rule for batches within a tick is total and stated in one place;
- replaying a committed command log yields byte-identical receipts and a
  byte-identical final state hash;
- compiled static entities are not copied into dynamic runtime state;
- the pursuit rule is authoritative and deterministic: the same command log
  produces the same capture outcome;
- the four-area route, interactions, water cost, capture, and reset remain
  green, and the rendering-plan digests are unchanged.

Must not: place a fractional position, a wall-clock value, or a frame rate into
authoritative state or into the ordering of authoritative work; interpolate
inside authoritative state, which stays presentation-only between authoritative
endpoints; introduce randomness except under a keyed scheme recorded by a
decision, since R1 inherits no RNG.

Evidence: the replay identity receipt; the stated ordering rule; the
rendering-plan digest comparison; a browser run reaching the final escape with
no console error.

## 6. Proof

The kernel workspace commands, from `docs/HANDOFF.md`:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask boundary
```

The comparison target, which proves the study rather than accepted work and is
the specification while R1-2 and R1-3 are open:

```text
experiments/executable-gaol/gaol verify
experiments/executable-gaol/gaol site
```

Once R1-4 exists, its headless Chromium smoke lane runs in CI on every change
and locally through the same entry point; no target is accepted while its lane
is red or absent.

Nothing is green until someone other than its author reruns the proof. Under
`AGENTS.md` the rerun receipt records the commit, the commands, the environment,
the result, and the reviewer. The author's own run is insufficient.

## 7. Budgets

Numbers in the record, not adjectives. Each field is recorded with the runner
that produced it; the values are observations, not portable guarantees. Nothing
below is a target value, and no unmeasured claim of sufficiency satisfies
acceptance.

| Field | Unit | How measured | Value |
| --- | --- | --- | --- |
| Workspace build time | s | clean release build of the workspace | not measured |
| Validation latency | ms | `nomos validate` on the accepted fixture | not measured |
| Replay throughput | commands/s | `nomos replay` over the accepted log | not measured |
| Package size | bytes | a compiled world package directory | not measured |
| Public artifact size | bytes | the staged public site directory | not measured |
| Edit-to-visible-frame latency | ms | content edit to first rendered frame | not measured |

## 8. Contract repair

Acceptance precedes implementation. Code may discover that this contract is
ambiguous, contradictory, impossible, or based on a falsified assumption; it may
not silently reinterpret it. A correction requires an owner-authorized decision
record containing the prior wording, the replacement wording, the reason, the
effect on existing evidence, the owner disposition, and a new R1 contract
revision number. Weakening a criterion merely because an implementation failed
it is forbidden.

An R1 revision amends this document only. `KERNEL.md` revision 7 stays frozen,
and no R1 revision reaches back into it or into any Gate K record.

## 9. Non-claims

R1 does not claim that Gate K passed, that any failed attempt is repaired or
relabelled, that Gate 0 or Gate 1 is satisfied, that the renderer or
presentation semantics in `experiments/` are accepted, that the executable study
is production art or deterministic across GPUs, or that Nomos has been adopted
by any project. This document declares no schema; schema identities are declared
by the code that emits them.

## 10. Owner disposition

_Not yet recorded._ Peter Permenter records exactly one outcome:

1. **authorize** — R1 revision 1 takes effect as written; R1-1 opens.
2. **authorize with amendments** — R1 revision 1 takes effect with the recorded
   changes to its adoption criteria, workspace layout, dependency policy, target
   acceptance, proof commands, or budget fields.
3. **decline** — no R1 contract is adopted, no R1 work is accepted, and the
   executable study continues under decision 0016 alone.

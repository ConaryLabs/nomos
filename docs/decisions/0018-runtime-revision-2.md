---
title: RUNTIME.md revision 2 — four contract-text repairs
status: Owner-authorized; RUNTIME.md revision 2 in force
number: 0018
date: 2026-08-25
owner: Peter Permenter
issue: 157
supersedes_runtime_revision: 1
establishes_runtime_revision: 2
references: docs/decisions/0017-post-gate-k-runtime-epoch.md
---

# RUNTIME.md revision 2 — four contract-text repairs

## Decision authority

Peter Permenter authorized this record on 2026-08-25 as one bundle of four
`RUNTIME.md` text repairs; the disposition is recorded below. `AGENTS.md` and
`RUNTIME.md` §8 require an owner-authorized decision record carrying the prior
wording, the replacement wording, the reason, the effect on existing evidence,
the owner disposition, and a new R1 contract revision number before accepted
contract text changes. This record supplies all six for each repair and
establishes R1 contract revision 2.

An R1 revision amends `RUNTIME.md` only. `KERNEL.md` revision 7 stays frozen,
and nothing here reaches back into it or into any Gate K record. Decision 0017,
which opened the R1 epoch and named `RUNTIME.md` as its contract, is unchanged.

## Repair 1 — R1-5 digest evidence names the drawn artifacts

### Prior wording

`RUNTIME.md` §5, the last acceptance bullet of R1-5:

```text
- the four-area route, interactions, water cost, capture, and reset remain
  green, and the rendering-plan digests are unchanged.
```

### Replacement wording

```text
- the four-area route, interactions, water cost, capture, and reset remain
  green, and the drawn artifacts — the SVG frames and contact sheet — are
  unchanged; rendering-plan digests may change when the plan's fields change.
```

The first clause is retained verbatim: the four-area route, interactions, water
cost, capture, and reset must still remain green. Only the digest clause is
repaired.

### Reason

R1-5 adds `actors[].role` to the rendering plan (issue #154), which necessarily
changes the plan's digests. A criterion that forbids the digests from changing
is therefore unsatisfiable by the slice it governs, which is exactly the case
`RUNTIME.md` §8 exists for — the contract is based on a falsified assumption,
not the implementation on a failure to be excused.

R1-3 (PR #147) already established what the digest evidence is for. When the
crescent glyph moved to the gate's `ward` socket, the accepted evidence was the
drawn output: twelve of thirty artifacts byte-identical, and every one of the
eighteen that changed accounted for byte-exactly, reported as
`17 artifacts checked, 0 not explained by the crescent substitution`. The plan
identity changed in that slice too, and the proof rested on the frames rather
than on the plan bytes. This repair states the criterion the project already
enforces.

### Effect on existing evidence

None retroactive. R1-3's and R1-4's recorded digests stand exactly as recorded,
including the `CAPTURE.md` tables from PR #147. No accepted artifact, fixture,
hash, or receipt changes, and no completed rerun receipt is reopened. The repair
is prospective: it binds R1-5's evidence, which does not yet exist.

## Repair 2 — R1-1 names the single activation evaluator

Closes issue #150.

### Prior wording

`RUNTIME.md` §5, the second acceptance bullet of R1-1:

```text
- all resolution comes from `nomos_sim::resolve_movement` and
  `nomos_sim::resolve_light` (`crates/nomos-sim/src/resolver.rs:21,82`), with
  `activation_is_true` (`resolver.rs:155`) staying private so the projected law
  flags stay in the path;
```

### Replacement wording

```text
- all resolution comes from `nomos_sim::resolve_movement` and
  `nomos_sim::resolve_light` (`crates/nomos-sim/src/resolver.rs:21,82`), and
  activation evaluation is the single `pub fn activation_is_true` in
  `nomos-projection` (`crates/nomos-projection/src/movement.rs`, issue #136,
  pull request #149), so effective facts still come only from that resolver
  pair with the projected law flags in the path;
```

### Reason

Issue #136 dispositioned the move explicitly — R1 surface in a kernel crate
under `RUNTIME.md` §3 option (a), not part of R1-1, a separate slice — and
PR #149 landed it: the one evaluator now lives in `nomos-projection` beside
`ProjectedActivation`, and the private copies in `nomos-sim` and
`nomos-compiler` are deleted. `KERNEL.md` §10 permits no edge between those two
crates, so they could not share code as placed.

The old clause therefore cites a line that no longer exists and calls the
function private when it is `pub`. Its substance was never the word "private":
the property wanted is that the projected law flags stay in the path, which
holds because effective facts come only from `resolve_movement` and
`resolve_light`. PR #149 did not edit §5, correctly — under `AGENTS.md` and
`RUNTIME.md` §8, amending accepted contract wording is an owner decision, not an
implementation side effect. Issue #150 filed it rather than fixing it, and this
is the repair.

### Effect on existing evidence

R1-1's acceptance is unaffected. `nomos_projection::activation_is_true`
evaluates one activation node and composes nothing — no movement disposition
and no light fact — so every effective fact still comes from the reused
resolver pair recorded on PR #130. R1-1's "must not add a second implementation
of activation evaluation" is strengthened rather than weakened: there were two
implementations on `main`, and after PR #149 there is one. No recorded output,
digest, comparison result, or rerun receipt changes. PR #149's own evidence
records 63 artifact and command-output hashes byte-identical to `origin/main`.

## Repair 3 — schema identity spelling

Closes issue #145.

### Prior wording

`RUNTIME.md` said nothing about how a document spells its own schema identity.
§3 forbids "one canonical schema identity defined in more than one crate" and
otherwise leaves the spelling to the emitting code, so two spellings were in
use with no rule: the string `name@version` and the object
`{"name": …, "version": N}`. `nomos-render-plan`'s `bind_schema`
(`crates/nomos-render-plan/src/read.rs`) accepts both, which hid the
inconsistency rather than resolving it.

### Replacement wording

One sentence is added to §3, as its own paragraph immediately after the
"Forbidden:" paragraph that already carries the document's one rule about
canonical schema identity:

```text
Schema identity spelling: R1 documents emitted to stdout or as R1 artifacts —
`effective_facts`, `entity_catalog`, `rendering_plan`, `area_collection`,
`presentation_source`, and their successors — spell `schema` as the single
string `name@version`, while Gate K package and run artifacts keep
`{name, version}`; a reader binds exactly the form its document family uses.
```

### Reason

Issue #145 asked for one owner decision on which spelling R1 stdout documents
use, recorded in `RUNTIME.md`, so that the readers can be narrowed instead of
accepting both. §3 is the placement chosen, for two reasons. First, §3 is
already where this document legislates canonical schema identity: the one
existing rule on the subject — that no identity is defined in more than one
crate — is in §3's "Forbidden" list, so the spelling rule sits beside the
ownership rule rather than in a section about something else. Second, §6 is the
proof section: it names the commands and lanes that prove work, not the shape
accepted work must have, and a document-shape requirement placed there would be
a rule in the evidence chapter. The sentence is a positive requirement rather
than a prohibition, so it is a paragraph after the "Forbidden:" list rather than
another entry inside it.

Keeping the two families deliberately different, rather than converging them, is
the point of the sentence: Gate K package and run artifacts are frozen evidence
whose bytes may not change, so the rule states the R1 form without touching
them, and names the boundary a reader binds against.

### Effect on existing evidence

No Gate K artifact, package, run bundle, digest, or receipt changes; the
`{name, version}` spelling those carry is preserved by this sentence, not
migrated.

Four of the five named R1 documents already emit the string form and need no
change:

- `nomos.entity_catalog@1` — `crates/nomos-compiler/src/entity_catalog.rs:92`,
  `CanonicalValue::text(entity_catalog_schema().to_string())`;
- `nomos.rendering_plan@2` — `crates/nomos-render-plan/src/plan.rs:475`,
  `CanonicalValue::text(rendering_plan_schema().to_string())`;
- `nomos.area_collection@1` — `crates/nomos-render-plan/src/collection.rs:391`,
  `CanonicalValue::text(area_collection_schema().to_string())`;
- `nomos.presentation_source@1` — `crates/nomos-render-plan/src/source.rs`,
  whose `bind_schema` reads the field as text and already refuses anything else.

One does not. `nomos.effective_facts@1` emits the object form at
`crates/nomos-sim/src/effective_facts.rs:53` through
`effective_facts_schema().to_canonical()`, which `SchemaId::to_canonical`
renders as `{"name": …, "version": N}`; the R1-1 tests assert that shape at
`crates/nomos-cli/tests/effective_facts.rs:160` and `:422`, and
`read.rs`'s `bind_schema` documents it as what `nomos effective-facts` writes.
Issue #145's problem statement had this document on the other side. No code is
changed by this record: aligning `effective-facts` to the string form, bumping
the identity whose bytes change, updating its `R1_SCHEMA_OWNERSHIP.md` row, and
narrowing `bind_schema` to the single accepted form remain implementation work
under issue #145's acceptance list, now with the spelling settled. Until that
change lands, `bind_schema` still accepts both forms, so no accepted consumer
breaks and no committed artifact is invalidated by this sentence.

## Repair 4 — ratify the §6 proof-block expiry merged in PR #151

### Prior wording

`RUNTIME.md` §6 before PR #151, quoted from `git show c257fb9^:RUNTIME.md`:

````text
The comparison target, which proves the study rather than accepted work and is
the specification while R1-2 and R1-3 are open:

```text
experiments/executable-gaol/gaol verify
experiments/executable-gaol/gaol site
```

Once R1-4 exists, its headless Chromium smoke lane runs in CI on every change
and locally through the same entry point; no target is accepted while its lane
is red or absent.
````

### Replacement wording

The wording merged in PR #151, which is the current §6 text and is ratified
unchanged by this record:

````text
The comparison target, which proves the study rather than accepted work:

```text
experiments/executable-gaol/gaol verify
```

R1-2 and R1-3 have landed, so the block above no longer needs a second command:
`gaol site` staged the study's own viewer and was removed with it when R1-4
promoted the viewer under `apps/`. The R1-4 lane is the replacement this section
anticipated, and it runs in CI on every change and locally through the same
entry point:

```text
node --test apps/nomos-viewer/test/*.test.mjs
node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist
node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist --out target/nomos-viewer-smoke
```

No target is accepted while that lane is red or absent. Locally the smoke lane
skips with an explicit message when the machine has no Chrome; in CI it is
required.
````

### Reason

The prior block scoped itself: it was the specification "while R1-2 and R1-3 are
open", and it said that once R1-4 existed, its headless Chromium smoke lane
would run in CI on every change and locally through the same entry point. R1-2,
R1-3, and R1-4 have all landed. `gaol site` staged the study's own viewer, and
PR #151 deleted that viewer when it promoted a clean implementation under
`apps/nomos-viewer/`, so the command no longer exists to run; the smoke lane the
block anticipated is `node apps/nomos-viewer/smoke/smoke.mjs`.

The edit was therefore the expiry the section wrote into itself rather than a
reinterpretation of a live criterion, and no proof was dropped: one command that
staged a deleted artifact was removed, and three that prove the accepted viewer
were added. It nonetheless reached accepted contract text inside an
implementation pull request, which `AGENTS.md` and `RUNTIME.md` §8 reserve to an
owner decision. This record supplies the missing authority and ratifies the
merged wording as it stands. `experiments/executable-gaol/gaol verify` is
unchanged and remains the comparison target.

### Effect on existing evidence

None. The merged text is already in force in the tree and is not edited by this
record. No proof result recorded before or after PR #151 changes, and no rerun
receipt is reopened. The one command removed, `gaol site`, staged an artifact
that no longer exists; PR #147's recorded `gaol site` result stays valid
evidence of the tree as it stood at that commit.

## Non-claims

- **No criterion is weakened.** Repair 1 keeps R1-5's route, interactions, water
  cost, capture, and reset clause verbatim and replaces an unsatisfiable digest
  clause with the drawn-artifact evidence the project already enforces. Repair 2
  names one evaluator where the prior wording assumed two crates each keeping a
  private one, which strengthens R1-1's "no second implementation" bar. Repair 3
  adds a requirement where there was none. Repair 4 ratifies text already in
  force and removes no proof. No repair here excuses an implementation that
  failed a criterion.
- **No Gate K change.** No Gate K command, artifact, hash, diagnostic, package,
  run bundle, receipt, digest, tag, or evaluation record is touched, and no R1
  result may be read back onto Gate K as a pass, a waiver, or partial credit.
  Decision 0013 remains the controlling Gate K verdict: failed.
- **`KERNEL.md` is untouched.** Revision 7 stays frozen as the historical Gate K
  contract; revision 2 of `RUNTIME.md` amends `RUNTIME.md` only.
- **No code is changed by this record.** It declares no schema, admits no
  dependency, accepts no slice, and satisfies no acceptance criterion. The
  `effective-facts` spelling alignment named under repair 3 remains open work
  under issue #145.

## Owner disposition

**Authorize.** Recorded by Peter Permenter on 2026-08-25, with no amendments.
All four repairs take effect as drafted, and `RUNTIME.md` revision 2 is in force
from this record's merge. Revision 1 governed until then.

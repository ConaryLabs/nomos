---
title: Narrow observed-scene presentation epoch (R2)
status: Proposed; owner disposition pending; no R2 authority
number: 0023
date: 2026-08-27
owner: Peter Permenter
issue: 191
nomos_candidate_commit: d12f56ec7f8a1f4a26db4d8482d5c4571ce1735d
nomos_candidate_tree: ed33da33797bc58006c72a0ed8f5e048a471a65f
r1_contract: RUNTIME.md revision 4
r1_contract_sha256: dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593
r1_disposition: docs/decisions/0019-r1-final-disposition.md
adoption_evidence_authority: docs/decisions/0022-mortal-estate-presentation-adoption-evidence.md
admitted_dependency_commit: 5e0e44cc912b57a1d29cc3e722497c16cf9a1797
admitted_dependency_tree: 7e606bde9f91307483307c4af1e0764d81df5c72
gap_fixture_sha256: 9b809bee523c9be04b26c6ab08412f96f9ad4f446a50a83824961dc8be016449
future_contract: R2.md
---

# Narrow observed-scene presentation epoch (R2)

## Proposal authority

This record was prepared under issue #191 after the owner asked to continue
from the reusable missing-capability finding on 2026-08-27. Its exact wording
has not yet received an owner disposition. Until that happens, this record
authorizes nothing: R1 stays closed, no R2 contract exists, and no
implementation may begin.

The proposed decision chooses disposition 3 from decision 0022: authorize a
separately decided narrow R2 based on minimal failing evidence. It does not
choose browser or Godot for production and does not adopt Nomos into The Mortal
Estate or any other game.

## Evidence relied on

Four exact records establish the need and bound its scope.

1. **Admitted Nomos input.** Issue #188 admitted commit
   `5e0e44cc912b57a1d29cc3e722497c16cf9a1797`, tree
   `7e606bde9f91307483307c4af1e0764d81df5c72`, as the immutable R1
   presentation dependency point after a clean non-author rerun. The admitted
   local candidate record has SHA-256
   `afd5c76dd7b5a3aeafa357831fac7b80b9c933567354b33928cd683a9767907b`.
2. **Representative adopter observation.** The Mortal Estate issue #5 and PR
   #6 produced a tracked representative observer frame with SHA-256
   `3ab13123836830a50227bbe3729a21ed10b89bec2617a46d74c9fc9be04e7b48`
   and normalized projection SHA-256
   `ad8c5577c7d52715eddeac104b273866b015b45db890d29bc3d36a6d7dbadb21`.
   A Luna max non-author rerun matched that normalized projection at reviewed
   tree `85468314caf08fb9f5ef775a0b5f3b625c63d3f9`; the tree merged as commit
   `bafe68bcf12abf2c34c97560063eeb8b041a3de2`.
3. **Adopter-neutral failing fixture.** Issue #189 reduced the relevant facts
   to `experiments/observed-scene-gap/fixture.json`, 1,464 bytes with SHA-256
   `9b809bee523c9be04b26c6ab08412f96f9ad4f446a50a83824961dc8be016449`.
   It contains one bounded integer crop; overlapping semantic terrain layers;
   controlled, hostile, and protected actor facts; life state; and one exact
   supplied enabled action. It contains no adopter identity or payload, raw
   transform, final pixel, palette, shader, gameplay rule, clock, or
   persistence fact.
4. **Executable refusal and classification.** Against the admitted R1
   boundary, one positive control compiled byte-identically and four probes
   were refused with the exact expected `RP0202` diagnostics. A Luna max cold
   attack reran the proof and confirmed that the complete fact set is neither
   honestly representable nor an adopter-only mapping. The owner then
   classified issue #189 exactly `reusable missing Nomos capability`.

The cold review records two evidence limits. The actor probe proves carrier
absence, not that every possible fact value has an independently visible
consequence. The fixture carries overlapping layers, but not every pair of its
three layers overlaps. R2 acceptance must test consequential values and actual
overlap directly rather than enlarging those earlier claims.

## Why this is an epoch rather than a repair

R1 is not defective for refusing the fixture. Its accepted contract is the
six-area gaol vocabulary: `nomos.presentation_source@2` deliberately closes
actors to `player | pursuer`, and its rendering plan and play runtime own the
movement-and-pursuit consequences of those roles. `nomos.presentation_state@1`
derives interactions from Nomos gameplay state rather than accepting an
outside observer's resolved action availability.

Changing those meanings in place would reinterpret accepted R1 evidence. An
adopter mapping cannot repair the gap either: calling a protected interactive
actor a player or pursuer changes its meaning; calling layered route or ground
a masonry mass changes its meaning; and deriving an observed action inside the
presenter creates a second gameplay authority. The honest route is a new,
strictly bounded presentation epoch with successor schemas and its own proof.

## Proposed decision

Open R2 as one narrow observed-scene presentation epoch. R2 asks whether Nomos
can accept already-resolved scene observations, compile only their presentation
consequences, and render them through an isolated offline consumer without
assuming ownership of how the facts were derived.

R2 is governed by a new root contract, proposed as `R2.md`. No accepted R2 code
may land before that contract is owner-authorized. The contract must state its
own acceptance criteria, schema-ownership lane, budgets, dependency boundary,
proof, and final disposition process. It may cite R1 artifacts and code as a
baseline; it may not amend `RUNTIME.md`, relabel R1 evidence, or treat an R1
receipt as proof of changed bytes.

## The bounded semantic surface

The R2 contract may admit only a closed, adopter-neutral carrier for the three
fact families reproduced by issue #189:

- **semantic terrain layers:** stable layer identity, bounded integer regions
  or cell sets, and a role selected from one versioned closed vocabulary;
  layers may overlap and ordering or composition consequences must be explicit;
- **independent observed actor facts:** stable actor identity, an integer
  lattice cell, life state, and independent controlled, hostile, and protected
  facts; no one flag silently implies another; and
- **observed action availability:** a stable action identity, its stable actor
  target, and supplied availability; the presentation path may display or
  style it but may not decide legality, execute it, or invent an enabled state.

The accepted input and compiled output must be typed, versioned, closed to
unknown fields, bounded in collection sizes and identifier lengths, integer-only
for authored spatial facts, canonical where bytes enter evidence, and owned by
exactly one emitting module per schema. There is no arbitrary tag bag, generic
JSON payload, raw transform, shader, final-pixel input, or adapter-specific
extension slot.

The compiler may derive geometry-selection, material-selection, layering,
silhouette, outline, effect, UI, and other presentation consequences. It may
not derive or override life state, hostility, protection, control, action
availability, traversal, collision, visibility, pathing, damage, inventory,
dialogue outcome, persistence, network state, or another gameplay fact.

Nothing in this section fixes the final schema spelling or representation.
Those are contract decisions whose tests must prove the semantic requirements
above. If they cannot be expressed without an unbounded vocabulary or an
adopter-specific assumption, work stops rather than broadening R2 by
implication.

## First targets in dependency order

1. **R2 contract.** Write `R2.md` with falsifiable epoch criteria, exact schema
   and workspace ownership, limits, budgets, proof commands, and non-claims.
   This is a separately reviewed documentation issue and is the next slice if
   this decision is authorized.
2. **Strict observed-scene carrier and compiler.** Implement the accepted input
   and compiled artifact as clean Nomos-owned code. Prove every issue #189
   refusal has an honest accepted successor, every field has one owner, unknown
   and out-of-budget inputs fail closed with stable diagnostics, compilation is
   byte-reproducible, and no R1 schema or accepted byte changes.
3. **Isolated offline browser consumer.** Consume only the compiled R2 artifact
   plus renderer-owned catalog data. Prove consequential terrain overlap,
   independent actor facts, and supplied action availability at the scene-graph
   and actual-play-size visual levels. A second independently authored generic
   scene must compile and render with no compiler, decoder, catalog, renderer,
   or UI source edit. The built artifact fetches nothing at runtime and carries
   no source input or adopter payload.
4. **R2 final disposition.** Bind the combined candidate, exact proof,
   non-author rerun, measured budgets, second-scene diff, and explicit owner
   verdict. Until that verdict, R2 is a candidate and The Mortal Estate may not
   consume it as an admitted result.

The order is a dependency order, not permission to start all four targets at
once. Each target begins from its own falsifiable issue and stops for its own
review.

## Workspace and dependency policy

The six kernel crates remain dependency-free and no kernel crate may depend on
an R1 or R2 crate, application, or tool. R2 may extend an existing non-kernel
R1 member or add a separately declared R2 member only when `R2.md` names the
member and `cargo xtask boundary` fails closed on its membership and permitted
edges. The graph remains acyclic, and applications consume published artifacts
rather than source or compiler internals.

R2 inherits R1's third-party admission policy outside the six kernel crates:
the lockfile is committed; each dependency is vendored or pinned by content
digest; its license is preserved; and `R2.md` records its version, provenance,
why it beats a local implementation, authoritative-determinism effect, and
offline proof. This decision admits no dependency. Browser proof starts from
the already vendored Three.js path unless a later issue proves a replacement is
necessary under that policy.

## Upstream and adopter boundary

Accepted reusable work lands and receives a non-author rerun in Nomos before an
adopter updates. The adopter then consumes an admitted commit, tree, artifact,
and schema digest through a deliberate mapping in its own repository. No
permanent downstream patch, unreviewed branch dependency, shared source tree,
or cross-repository build step is permitted.

The adopter owns its mechanics, identities, content, coordinate selection,
observation production, platform, renderer integration, target judgment, and
final adoption decision. Nomos owns only its generic schemas, validation,
presentation compilation, renderer catalog, and evidence. A mapping may rename
or select presentation vocabulary; it may not discard a load-bearing supplied
fact or recompute one.

## Stop line

Pause for owner disposition if any target exposes:

- duplicate gameplay authority or a presenter that recomputes supplied facts;
- a required adopter identity, schema, mechanic, coordinate, palette, prose,
  governance rule, or permanent downstream patch in accepted Nomos code;
- an arbitrary payload, unbounded vocabulary, raw-transform escape, shader, or
  final-pixel authoring input;
- a second matching generic scene that requires compiler, decoder, catalog,
  renderer, or UI source edits;
- an R1 schema, artifact, or contract change presented as compatible evidence;
- a required proof that is red, non-reproducible output, or a budget miss; or
- evidence that the capability is specific to one adopter or requires a deeper
  gameplay-runtime replacement.

The permitted owner dispositions at a stop are repair the proposed R2 contract
without weakening it, authorize a separately evidenced expansion, or stop.

## Explicit non-authorizations

This decision does not authorize:

- game adoption or a claim that The Signed World applies to The Mortal Estate;
- a Godot, browser, or other production-platform choice;
- replacement of an adopter's rules, server, persistence, networking,
  protocol, client, renderer, or content compiler;
- combat, damage, inventory, dialogue resolution, audio, networking,
  replication, save migration, live-service integration, editor work, plugins,
  or a general scene format;
- a Gate K retry, `KERNEL.md` amendment, `RUNTIME.md` amendment, or
  reinterpretation of historical evidence;
- importing The Mortal Estate content, identifiers, coordinates, palette,
  images, prose, schemas, mechanics, or governance into accepted Nomos code;
  or
- accepted implementation before `R2.md` is separately owner-authorized.

The offline browser consumer is an evidence path, not a production-platform
verdict. Godot comparison remains permitted under decision 0022 only when it
consumes the same admitted artifact and resolves a named consequential
uncertainty.

## Effect on existing evidence

None. R1 remains accepted and closed at decision 0019's exact candidate under
`RUNTIME.md` revision 4. Gate K remains failed, round two remains terminated
incomplete, and every historical tag, receipt, contract digest, and non-claim
keeps its recorded meaning. The issue #189 experiment remains quarantined and
satisfies no R2 acceptance criterion.

Opening and even accepting R2 would not establish a game-adoption result. The
Mortal Estate still owns its actual-play-size target judgment, mapping,
integration evidence, measured cost acceptance, and final project decision.

## Owner disposition

**Pending.** The owner may authorize this decision exactly as written, narrow
or repair it explicitly, or refuse it. Until that disposition is recorded, no
R2 epoch, contract, implementation, schema, or dependency is authorized.

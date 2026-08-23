---
title: Quarantined gaol visual-target experiment
status: Owner-authorized; static experiment only
number: 0014
date: 2026-08-23
owner: Peter Permenter
issue: 82
experiment_issue: 83
gate_k_disposition: docs/decisions/0013-gate-k-disposition.md
contract_revision: 7
---

# Quarantined gaol visual-target experiment

## Decision authority

Peter Permenter authorizes one quarantined, static visual-target experiment
using the Gate K gaol as its brief.

Decision 0013 remains unchanged. Gate K failed, criteria 17 and 18 remain
failed, no formal retry is authorized, and `gate-k-rc1` remains a failed
release candidate rather than an acceptance tag. This decision does not amend
`KERNEL.md`, contract revision 7, or the cold-agent protocol, and it does not
reclassify any historical result.

## What the formal evidence established

The formal agents did not break the sandbox boundary. They requested forbidden
outside-workspace paths, and the sandbox correctly denied those requests. The
evaluation-infrastructure defect exposed by the sessions was the original
finalizer's fail-open trust in a checker's self-reported pass despite command
evidence proving a violation of the frozen rubric. Issue #79 and PR #80 repaired
that adjudication path and hardened the complete evidence envelope.

Under revision 3's prospective rule, requesting `/dev/null` was disqualifying
even when denied. The owner correctly retained both formal failures rather than
waiving a fixed criterion after observing otherwise successful work.

The same immutable records also contain positive diagnostic evidence that is
not erased by the overall verdict:

- Gemini authored a valid second approved door through the public language and
  CLI;
- DeepSeek reproduced its package byte-for-byte;
- DeepSeek identified the hidden semantic cause and produced the expected
  minimal repair;
- Gemini independently reproduced the failing and repaired behavior and
  confirmed the diagnosis;
- the mechanical determinism, migration, package, replay, explanation, budget,
  schema-ownership, and different-author proofs passed.

The experiment therefore observed two separate answers: unfamiliar models
could author and debug Nomos semantics, while both formal exercises failed the
absolute workspace-only command policy. Gate K required both answers to be
positive and consequently remains failed.

## Prospective evaluation lesson

A future protocol decision may report three dimensions separately:

1. **semantic merit** — whether the subject completed and explained the task;
2. **independence integrity** — whether undeclared information entered the
   task;
3. **operational compliance** — whether every tool and path restriction was
   obeyed.

An overall gate may still require all three. Recording them separately would
prevent an operational failure from concealing what happened on the experiment's
central semantic question.

This is a lesson for a possible future protocol amendment, not an amendment in
this decision. It neither makes `/dev/null` legal under the current protocol nor
changes the revision-3 verdict. A later owner may prospectively allow that
non-information-bearing device or provide a declared workspace-local sink.

## Authorized experiment

Issue #83 may create one Gate 0-format visual target pack under
`experiments/gate-0-gaol-target-pack/`. The brief is deliberately narrow:

- `north_gate` supplies the iron barred door, ward, and portal language;
- `flooded_section` supplies material, traversal, reflection, and environmental
  readability pressure;
- `brazier_02` supplies low-light and effective-light contrast;
- one player, one enemy, one restrained spell, the actual proposed gameplay
  camera, and representative UI test whether a hero composition survives
  contact with a game view.

The authorized media are static images, rough blockouts, paintovers, composited
UI, palette/material sheets, and a static motion-timing board. The pack tests
taste, coherence, and readability, not machinery.

The experiment is non-authoritative. It cannot satisfy Gate K, count as a
formal Gate 0 pass, adopt Nomos into another project, or enter an accepted
kernel, schema, projection, package, or runtime surface.

## Explicitly not authorized

This decision authorizes no:

- renderer or executable rendering experiment;
- Rust crate or other executable code;
- rendering projection or semantic-kernel change;
- asset pipeline, style compiler, procedural generator, or visual primitive
  catalog;
- third-party dependency;
- formal cold-agent retry or `gate-k-rc2`;
- claim that Nomos or The Signed World has passed its thesis gates or been
  adopted by a game.

Decision 0005's temporary dependency policy ended with Gate K. That fact does
not admit a dependency here; this experiment needs none.

## Required disposition

The study stops after one coherent pack and Peter's review. The owner records
exactly one outcome:

1. **visual thesis rejected** — archive Nomos as a successful semantic
   prototype with an unconvincing game target;
2. **visual thesis promising** — preserve the pack and decide whether a fresh,
   prospectively governed Gate K attempt is worth doing;
3. **visual thesis compelling** — preserve the pack as the desired target, but
   still complete a fresh Gate K attempt before renderer architecture or
   project adoption.

No outcome silently authorizes renderer implementation. If the ordinary
gameplay-camera frame is not desirable and readable, more hero art is not a
substitute for an honest rejection.

## Consequence

Issue #83 is authorized to begin after this decision merges. It is the only
active project slice. Gate K remains failed, semantic feature work remains
stopped, and all work outside the static target-pack boundary still requires a
new owner decision and falsifiable issue.

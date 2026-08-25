---
title: Cold-author area five — Gloam Bastion and Drowned Stair
status: Experiment record; owner visual verdict compelling
date: 2026-08-25
applies_to: THESIS.md §18 (Gate 2, Gate 3); experiments/executable-gaol
scope: experiments/executable-gaol (quarantined, non-authoritative)
---

# Cold-author area five — Gloam Bastion and Drowned Stair

## Purpose

`THESIS.md` §18 describes Gate 2 (a knowledgeable author creates twenty rooms
from one approved kit) and Gate 3 (a different model family receives only the
approved packet and CLI and authors room 21, with time-to-compile, validation
cycles, forbidden-escape attempts, and files changed recorded). This experiment
is Gate 2 and Gate 3 in miniature: one existing four-area kit, one insertion
slot, two cold-author subjects working independently from the packet alone.
It is deliberately informal — it does not run the Gate K attempt ledger
(decision 0011), does not stand up a checker hierarchy, and produces no
adjudicated pass/fail verdict. It asks a narrower question: can two subjects,
working blind and independently, each add a fifth area to the same kit without
touching the renderer, and does the result still cohere when both land in the
same slot. `experiments/executable-gaol` is quarantined per `AGENTS.md`; this
record and its outcome satisfy neither Gate K nor Gate 1.

## Protocol

Each subject received, and nothing beyond: `experiments/executable-gaol/AUTHORING.md`
(the packet), the `north-gaol` area as a worked example, the other three
existing areas' `presentation.json` files (not their `world.nomos`), `docs/authoring.md`
(the source-language vocabulary), the two fixtures (`area-collection.example.json`
and a `rendering-plan.example.json`), and the `gaol` commands (`accept`,
`verify`). Forbidden: any file under `src/`, `apps/`, or `crates/`, and any
path outside the one new area directory plus the closed list `AUTHORING.md`
names for connecting it (the predecessor's `route.exit.to_area`, the new
area's own files, and the two regenerated fixtures). Each subject had one
insertion slot: between Ember Vault and Ossuary Reach.

Measurement was mechanical, drawn from each subject's own tool-call
transcript by a scoring script (`score.py`): wall-clock span between first and
last tool call, tool-call count, `gaol accept`/`gaol verify` invocation counts,
diagnostics encountered, forbidden-path touches, and files changed. The
scorer has two known false positives, both confirmed against the raw
transcripts: it flags `RP0205` once per subject because that diagnostic code
appears in `AUTHORING.md`'s own prose (the packet text explaining the
integer-only rule), not because either subject's `world.nomos` or
`presentation.json` ever tripped it; and it flags several `EK0804` occurrences,
which are the expected `01-baseline` scenario rejection (`open` attempted
while the gate is `locked`) that every existing area's baseline scenario also
produces by design, not a real authoring diagnostic. Neither subject hit an
authoring or verification failure.

## Subjects

| | Codex | Claude Sonnet |
|---|---|---|
| Family | OpenAI codex-cli 0.149.1 — cross-family | Claude Sonnet (cold context) — same family as session lead |
| Area | `gloam-bastion`, "Gloam Bastion" | `drowned-stair`, "Drowned Stair" |
| Wall clock | 2 min 55 s (20:09:44Z–20:12:39Z) | ~4 min |
| Tool calls | 7 | 42 |
| `gaol accept` runs | 1 | 2 |
| `gaol verify` runs | 1 (green first run) | 3 (first green; rest re-confirmations) |
| Authoring diagnostics | 0 | 0 |
| Forbidden-path touches | 0 | 0 |
| Files changed | 12 | 12 |
| Notable choice | Renamed both actors via `role` (`wayfarer`/`warden` instead of `player`/`gaoler`) | Chose gate columns 0 and 3, the one column pair disjoint from all four existing areas |

Both branches: five-area `gaol verify` green at authoring time; viewer node
tests 97/97; browser smoke lane played all five areas with no harness change —
Codex 56 moves / 78 cost, Sonnet 53 moves / 77 cost, 0 external requests each.

Codex could not commit its own work: the sandbox mounts `.git` read-only, so
the session lead committed its unedited output under a Codex author identity.

## Findings

- **`route.entry` duplicates the player actor's `cell` by unstated
  convention.** `AUTHORING.md` states `route.entry`'s validation rule (inside
  bounds, not inside a mass) but never says it must equal the arriving
  player's `actors[].cell`. Both subjects independently noticed that the
  three existing non-start areas set them identically and followed the
  convention rather than risk diverging from it, though nothing in `gaol
  verify` checks the relationship directly. Filed as issue #165,
  "Presentation source: route.entry duplicates the player actor's cell."
- **The read boundary hid three of five comparison axes from Codex.** Its
  note records that the permitted presentations exposed the other areas'
  masonry masses but not their gate columns, water regions, or brazier
  positions, because those live in `world.nomos`, which cold authors do not
  receive. It could not verify those three axes were distinct from all four
  existing areas by direct comparison; it had to rely on the packet's
  general distinctness requirement.
- **Codex's sandbox mounts `.git` read-only.** This is an operator-tooling
  limitation, not a packet gap: the subject produced complete, unedited
  content and could not commit it itself under any subject-provided identity.
- **Sonnet is not a formal Gate 3 subject.** It is the same model family as
  the session lead that built and packaged this experiment. Only Codex
  satisfies Gate 3's cross-family eligibility rule (`docs/evaluation/COLD_AGENT_PROTOCOL.md`
  §2); Sonnet's run is same-family cold-context fuzzing, not a cold-author
  claim, and this record does not represent it as one.

## Outcome

Both areas merged into the public route: Cistern Walk → Ember Vault → Gloam
Bastion → Drowned Stair → Ossuary Reach → North Gaol. The two subjects chose
the same insertion slot independently; reconciling that collision was harness
work (`experiments/executable-gaol` commit "Connect Drowned Stair after Gloam
Bastion"), not either subject's own authoring, and neither subject's
`AUTHOR_NOTES.md` was edited to produce it. `gaol verify` is green for all six
areas; the renderer, camera, palette, and viewer received no change.

**Corpus pins under `crates/`.** Connecting the two areas required one further
commit (`f1e85e8`) editing `crates/nomos-play/tests/{common/mod.rs, corpus.rs,
replay.rs, semantics.rs, session.rs}` and `crates/nomos-render-plan/tests/source.rs`,
which enumerate the four areas and pin the four-area route's counters. Neither
subject touched them — the harness did, after both areas were in — and they are
tests, not renderer or compiler source, so `RUNTIME.md` §1 criterion 5 holds.
But it is the same hazard #163 removed from the JavaScript collection test
before the run, and it is recorded here as a finding rather than folded into
the connection commit: issue #167.

**A pin under `apps/` — an R1-4 criterion regression.**
`apps/nomos-viewer/test/runtime.test.mjs` hardcoded the four-area route as
key strings (`ROUTE_KEYS`), so two viewer tests failed once a fifth area
existed, and fixing them required an edit under `apps/nomos-viewer/` — which
`RUNTIME.md` §5 R1-4 says adding an area must never require. The pin arrived
with R1-5's test, after R1-4's area-addition proof, so the proof could not
have caught it. The repair here makes the test derive its route from the
smoke lane's solver, the same way the lane does; #167 covers the class.

## Owner visual verdict

`[ ]` rejected — `[ ]` promising — `[x]` **compelling**

Recorded by Peter Permenter on 2026-08-25 after reviewing both five-area
contact sheets: both cold-authored rooms read as the same game and as distinct
rooms, and the packet-plus-grammar approach works. This is the owner visual
disposition the experiment left open; it is not a Gate 2 or Gate 3 verdict.

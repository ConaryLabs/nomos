# The executable semantic kernel

The first artifact after the thesis is not a renderer. It is a kernel that proves
the semantic machine works with no graphics, no networking, and no audio
playback. If this kernel is ugly, a renderer would only conceal the corpse under
some lovely palette quantization.

Status: **not started.** These criteria are written before any code so the code
cannot redefine them.

## Scope

One room source file containing exactly:

- one **door** (`iron_barred`, lockable, breakable)
- one **water** region
- one **extinguishable light**

and the machinery the thesis names (THESIS.md §3–§9, §12):

- a versioned Canonical World IR with canonical serialization
- primitive expansion from a tiny catalog
- per-namespace state machines with pure-derived and causal interactions
- capability resolution with declared composition laws
- the fact-ownership linker with structured diagnostics
- three or four subsystem projections emitted as JSON (simulation, navigation,
  persistence, diagnostics; rendering may be a stub projection that emits only
  what a renderer would need)
- deterministic command execution with state hashes and causal receipts
- one real IR schema migration (v1 → v2), exercised
- `explain-entity` and `explain-transition`
- mutation tests for every ownership and cross-reference invariant

## Command surface

```text
estate validate fixtures/gaol.estate
estate compile  fixtures/gaol.estate            → build/gaol.world (IR + projections + manifest)
estate inspect  build/gaol.world
estate command  build/gaol.world "unlock north_gate with gaoler_key"
estate command  build/gaol.world "extinguish brazier_02"
estate explain-entity     build/gaol.world north_gate
estate explain-transition build/gaol.world north_gate --tick 4
estate replay   fixtures/gaol.replay
estate migrate  build/gaol-v1.world --to 2
```

Every command returns structured JSON plus artifact paths. Exit codes: 0 ok,
1 rejected with diagnostics, 2 usage, 3 could-not-run.

## Acceptance

The kernel passes when all of the following are observed, not asserted:

1. **Source is understandable.** The fixture fits on one screen and a reader
   who has not seen the thesis can say what the room contains.
2. **Primitives resolve.** The door expands to its capability bundle; the
   expansion is printed by `inspect` and matches the catalog definition.
3. **Machines stay independent.** `access`, `integrity`, and `combustion` are
   separate machines; no flattened product table exists anywhere in the build.
4. **Interactions are deterministic.** `combustion.burning → integrity.apply_damage`
   fires in a fixed phase order; the same command log yields identical state
   hashes across ten runs and two machines.
5. **Capabilities compose without ambiguity.** A magical seal and a closed door
   both claim `Blocks<ground>`; the resolver composes `any_active_claim`;
   destroying the door while sealed still blocks, and `explain-entity` says why.
6. **Projections agree.** Navigation's portal state, simulation's blocker, and
   persistence's stored state are derived from one IR and never disagree; a
   test mutates the IR and all three move together.
7. **Ownership fails closed.** Each of these is rejected with a stable code and
   a source span: a dangling entity ID; a relation encoded as a cell property;
   an authored transform in content; a derived fact supplied by content; two
   owners for one fact; an undeclared cross-machine write; an interaction cycle.
8. **Migration works.** A v1 build migrates to v2 and replays to the same state
   hashes; a v1 build loaded by a v2 runtime without migration is refused.
9. **Explanations are useful.** `explain-transition north_gate --tick 4` names
   the command, the actor, the machine, the transition, the interactions that
   fired, the capability claims that changed, the projections that moved, and
   the source line of the primitive.
10. **A cold agent can author and debug it.** A model from a different family
    than any that helped design the kernel, given only the docs and the CLI,
    (a) adds a second door to the fixture and reaches a clean compile, and
    (b) given a replay with a seeded defect (the door blocks navigation after
    opening), names the cause correctly without reading kernel source.
11. **Nothing exceeds ~1,000 lines**, and the crate layout already reflects the
    workspace boundaries in THESIS.md §20 (core / schema / compiler / sim / cli
    at minimum), so the renderer can be added later without moving anything.

## Non-goals for the kernel

No `wgpu`. No window. No network. No audio playback. No Workbench UI. No hot
reload daemon. No asset pipeline. No second primitive beyond the three above.

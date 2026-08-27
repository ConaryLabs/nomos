---
title: R2 canonical-schema ownership register
status: R2 revision 1 register
date: 2026-08-27
issue: 195
authority: R2.md §4
---

# R2 canonical-schema ownership register

This register contains exactly the two schema identities admitted by
owner-authorized `R2.md` revision 1. Gate K's frozen twenty-row register and
the R1 ten-row register remain unchanged and are not repeated here.

## Inventory

| Canonical identity | Owner | Owner file | Authoritative type set | Encoder | Strict reader / verifier | Persisted boundary | Primary consumers |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `nomos.observed_scene@1` | `nomos-observed-scene` | `crates/nomos-observed-scene/src/input.rs` | `LocalId`, `Crop`, `SceneIdentity`, `TerrainCell`, `TerrainRole`, `TerrainLayer`, `ActorCell`, `LifeState`, `Actor`, `Availability`, `Action`, and `ObservedScene` | `ObservedScene::to_canonical` and `ObservedScene::to_canonical_bytes`, using `nomos_core::CanonicalValue` only | `ObservedScene::from_bytes` and `ObservedScene::from_canonical`, binding every exact field, bound, identity, order, uniqueness, role-presence, and target rule | canonical generic inputs under `fixtures/r2/scenes/`; immutable caller-selected input to the compiler | `nomos-observed-scene` compiler only in R2-1; never the browser consumer |
| `nomos.observed_scene_plan@1` | `nomos-observed-scene` | `crates/nomos-observed-scene/src/plan.rs` | `TerrainAssembly`, `MaterialFamily`, `TerrainPlan`, `ActorAssembly`, `ActorPose`, `Presence`, `ActorPlan`, `ActionMarker`, `ActionPlan`, and `ScenePlan` | `ScenePlan::to_canonical` and `ScenePlan::to_canonical_bytes`, using `nomos_core::CanonicalValue` only | `ScenePlan::from_bytes` and `ScenePlan::from_canonical`, revalidating the complete input grammar and every copied-fact/compiled-selection agreement | compiler-produced canonical plans under `fixtures/r2/plans/` and caller-selected new output paths | the isolated R2 browser consumer in R2-2; tests and compiler staging re-verification in R2-1 |

The R2 compiler declares no third schema. Its rejection envelope is explicitly
unpersisted and non-schema under `R2.md` section 7.

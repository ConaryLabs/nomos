---
title: Gate K canonical-schema ownership source review
status: Final finding-free exact-candidate receipt
date: 2026-08-23
reviewed_implementation_commit: eb86f25f5084a5da83cdd4f26e42e68089367a11
contract_revision: 7
---

# Gate K canonical-schema ownership source review

This is the explicit source review required by `KERNEL.md` acceptance 15. It
reviews the implementation-complete source tree merged by issue #68. Issue #69
changed evidence tooling and documentation, not canonical schema source; its
exact-candidate evidence workflow and non-author audit confirmed that remained
true.

The disposition is finding-free: the source defines twenty persisted or
contractual schema identities, every identity has one owner crate, and no second
crate defines the same canonical schema meaning. The local schema-ID tests,
package registry, compiler schema lists, and dependency checker corroborate this
review but do not replace it.

## Inventory

The type-set labels are expanded immediately after the table. “Regenerate” in a
reader column means the package opener strictly reconstructs typed stable IR,
recompiles the owner type, and requires exact canonical bytes; it is stronger
than accepting a generic JSON shape but is not mislabelled as a public
projection-crate decoder.

| Canonical identity | Owner | Authoritative type set | Encoder | Strict reader / verifier | Persisted boundary | Primary consumers | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `nomos.package.manifest@1` | `nomos-core` | M | private `PackageManifest::to_canonical` through `WorldPackage::write` | `WorldPackage::open` exact manifest parse, digest recomputation, member verification | `manifest.json` | compiler package open/migrate; CLI filesystem commands | active |
| `nomos.source@1` | `nomos-schema` | S | canonical authored `.nomos` grammar; no JSON encoder | compiler `parse_source` requires exact schema header, UTF-8 grammar, spans, and typed identifiers | `fixtures/*.nomos` and author packets | `nomos-compiler` only | active |
| `nomos.world_ir.construction@3` | `nomos-schema` | C | `WorldIr::to_canonical_bytes` | reconstructed inside both stable World IR decoders; no standalone package reader because construction snapshots cannot be package `world-ir.json` | contractual/incomplete construction evidence and stable-envelope lineage | compiler linking, promotion, migration normalization | active construction lineage |
| `nomos.world_ir@1` | `nomos-schema` | W1 + C | `LegacyStableWorldIrV1::to_canonical_bytes` | `LegacyStableWorldIrV1::from_canonical_bytes` via `decode_legacy_stable_world_ir`, then semantic revalidation and projection regeneration | legacy package `world-ir.json` and `fixtures/gaol-v1.world` | migration only; direct active runtime use refused | legacy migration input |
| `nomos.world_ir@2` | `nomos-schema` | W2 + C | `StableWorldIr::to_canonical_bytes` | `StableWorldIr::from_canonical_bytes` via `decode_stable_world_ir`, semantic validation, and exact projection regeneration | active package `world-ir.json` | compiler opener, runtime initialization, inspect/explain | active |
| `nomos.package.schemas@1` | `nomos-schema` | G | `SchemaRegistry::to_canonical_bytes` | compiler opener regenerates `expected_registry()` and requires byte equality | package `schemas.json` | package open, inspect, explanation | active |
| `nomos.compiler_receipts@1` | `nomos-compiler` | B | private `CompilerReceipts::to_canonical_bytes` | `validate_receipts_profile` exact fields, versions, pass profile, invariant set, produced schemas, source/artifact digests | package `compiler-receipts.json` | package open, migration, inspect/provenance | active; compile and migration profiles intentionally shared |
| `nomos.projection.simulation@3` | `nomos-projection` | PS + PM + PL + PSim | `SimulationPlan::to_canonical_bytes` | compiler opener regenerates from strictly decoded stable IR and requires byte equality | package `simulation.json` | `nomos-sim`, run/replay/command/explain orchestration | active |
| `nomos.projection.navigation@1` | `nomos-projection` | `NavigationPlan` + PM | `NavigationPlan::to_canonical_bytes` | regenerate from stable IR, byte compare, and simulation/navigation movement agreement | package `navigation.json` | inspect and cross-projection validation | active |
| `nomos.projection.persistence@1` | `nomos-projection` | `PersistencePlan` + PS + PL | `PersistencePlan::to_canonical_bytes` | regenerate from stable IR, byte compare, and light-plan agreement | package `persistence.json` | persistence diagnostics in receipts/inspect/explain | active |
| `nomos.projection.diagnostics@1` | `nomos-projection` | `DiagnosticsPlan` + PS + PL | `DiagnosticsPlan::to_canonical_bytes` | regenerate from stable IR, byte compare, and light-plan agreement | package `diagnostics.json` | diagnostics, inspect, entity/transition explanations | active |
| `nomos.runtime_state@2` | `nomos-sim` | RState | `SimulationState::to_canonical_bytes` | `SimulationState::from_canonical_bytes` / `decode_state`, validated against exact `SimulationPlan` | nested `state` in initial/final persisted states; state-hash domain | runtime, command, replay, receipt/hash verification | active v2 runtime epoch |
| `nomos.persisted_runtime_state@2` | `nomos-sim` | `PersistedRuntimeState` + RState | `PersistedRuntimeState::to_canonical_bytes` | `PersistedRuntimeState::from_canonical_bytes`, including simulation digest and inner state/hash checks | run `initial-state.json` and `final-state.json`; command `--state` | CLI run/command/replay and strict run opener | active v2 runtime epoch |
| `nomos.command_script@1` | `nomos-sim` | CS | `CommandScript::to_bytes` | `CommandScript::from_bytes` exact LF/header/spacing/arity/typed-ID grammar and re-encode | `fixtures/gaol.commands`, `fixtures/gaol-seven.commands`, CLI `--commands` | run orchestration and command resolution | active |
| `nomos.command_log@1` | `nomos-sim` | CL | `CommandLog::to_canonical_bytes` | `CommandLog::from_canonical_bytes`, typed request/command reconstruction and hash/receipt bindings | run `command-log.json`; nested replay expected log | run opener, replay, explanations | active |
| `nomos.causal_receipt_sequence@1` | `nomos-sim` | CRS | `CausalReceiptSequence::to_canonical_bytes` | `CausalReceiptSequence::from_canonical_bytes`, strict nested receipt reconstruction and sequence validation | run `causal-receipts.json` | run opener, replay validation, transition explanation | active |
| `nomos.state_hash_sequence@1` | `nomos-sim` | HS | `StateHashSequence::to_canonical_bytes` | `StateHashSequence::from_canonical_bytes`, exact ordinal/tick/hash-chain validation | run `state-hashes.json` | run opener, replay and determinism evidence | active |
| `nomos.run_result@1` | `nomos-sim` | RR | `RunResult::to_canonical_bytes` | `RunResult::from_canonical_bytes`, then cross-validation against all five typed non-result artifacts | run `result.json` | run opener, CLI status and replay validation | active |
| `nomos.causal_receipt@1` | `nomos-sim` | CR | `CausalReceipt::to_canonical_bytes` | `CausalReceipt::from_canonical_bytes`, typed commands/steps/facts/deltas and re-encode | rows nested in `causal-receipts.json`; digest-bound from command log | transaction commit, run/replay evidence, explanations | active |
| `nomos.replay_log@1` | `nomos-sim` | RL + CL | `ReplayLog::to_canonical_bytes` | `ReplayLog::from_canonical_bytes`, nested command-log decode plus package/simulation/end-state bindings | `fixtures/gaol.replay`, CLI `--log` | replay orchestration and reproduction checks | active |

## Exact authoritative type sets

- M: `PackageManifest`, `MemberRecord`, and validated `MemberName`. `WorldPackage`
  owns filesystem publication/opening but is not itself a persisted schema row.
- S: `SourceDocument`, `SourceEntity`, `SourceRelation`, `SourceField`,
  `ForbiddenFactOwner`, and `Spanned<T>`.
- C: `WorldIr`; `CapabilityKind`, `ClaimValue`, `ClaimActivation`,
  `ClaimTemplate`, `MachineTemplate`, `PrimitiveExpansion`, `IrEntity`,
  `IrRelation`; `Cell`, `Direction`, `Binding`; `MovementCompositionLaw`,
  `GroundMovementCoherence`, `GroundConnectivity`, `MovementResolverSubject`,
  `MovementResolverPlan`; `LightCompositionLaw`, `LightResolverSubject`,
  `LightResolverPlan`; `TransitionInput`, `TransitionTrigger`,
  `TransitionDefinition`, `InteractionPhase`, `InteractionTrigger`,
  `InteractionDefinition`; `FactOwner`, `FactIdentity`, `ResolvedFactValue`,
  `ProjectionConsumer`, `DerivationProducer`, `DerivationPass`,
  `DerivationInput`, `DerivationStep`, and `FactOwnershipReceipt`.
- W1: `LegacyStableWorldIrV1` and `StableGroundMovementV1`.
- W2: `StableWorldIr`, `StableGroundMovementV2`, and
  `StableMovementDispositionGround`.
- G: `SchemaRegistry`, `SchemaRegistration`, and `SchemaOwner`.
- B: private `CompilerReceipts` and `ArtifactDigest`. Privacy prevents another
  crate from treating the receipt body as its own authoritative Rust type.
- PS: `ProjectedDirection`, `RuntimeBinding`, and `ProjectedEntity`.
- PM: `LatticeCell`, `MovementConnectivity`, `ProjectedActivation`,
  `MovementClaim`, `MovementSubject`, and projection `MovementResolverPlan`.
- PL: `LightProjectionConsumer`, `LightClaim`, `LightSubject`, and projection
  `LightResolverPlan`.
- PSim: `SimulationPlan`, `CommandRequirement`, `EventPayload`,
  `CommandTransition`, `EventHandler`, `MachineDefinition`, `Phase`, and
  `CausalEdge`.
- RState: `SimulationState`, `RuntimeEntityState`, and projection-owned
  `RuntimeBinding`.
- CS: `CommandScript`, `CommandRequest`.
- CL: `CommandLog`, `CommandLogRow`, `CommandRequest`, and projection-owned
  `Command`, `CommandArgument`, and `EventPayload`.
- CR: `CausalReceipt`, `ProjectionDelta`, `EffectiveFactRef`,
  `EffectiveFactValue`, `TransitionCause`, `TransitionStep`; projection-owned
  `Command`, `CommandArgument`, `EventPayload`, `ResolvedMovementFacts`,
  `ResolvedMovement`, `MovementDisposition`, `ResolvedLightFacts`, and
  `ResolvedLight`.
- CRS: `CausalReceiptSequence` plus CR.
- HS: `StateHashSequence` and `StateHashRow`.
- RR: `RunResult`, `RunStatus`, `RunArtifactName`, and `RunArtifactDigest`.
- RL: `ReplayLog`; its nested command-log meaning is CL rather than a duplicate
  replay-owned command type.

## Source inspection and boundary result

The identity constructors are confined to their owner modules:

- `nomos-core/src/package.rs`: manifest only;
- `nomos-schema/src/lib.rs`: source, construction, stable v1/v2, registry;
- `nomos-compiler/src/package.rs`: compiler receipts only;
- `nomos-projection/src/lib.rs`: exactly four projection identities;
- `nomos-sim/src/lib.rs`: exactly nine runtime/evidence identities.

`nomos-cli` defines no schema identity or canonical root type. It orchestrates
owner APIs. `nomos-compiler` crosses from `nomos-schema` to
`nomos-projection`, but owns only its receipt schema. `nomos-sim` can use
projection types and cannot depend on `nomos-schema`; it reconstructs its nine
runtime/evidence roots rather than a second World IR. Generic core types such as
`SchemaId`, `CanonicalValue`, hashes, stable IDs, diagnostics, and package byte
records are shared primitives, not duplicate definitions of any semantic schema
above.

The package registry intentionally contains the eight identities of one world
package: manifest, active stable World IR, four projections, registry, and
compiler receipts. Construction and source are referenced inside stable/build
evidence but are not separate package members. Runtime/evidence schemas live in
separate run or replay artifacts and therefore do not enter `schemas.json`.

Compiler `consumed_schemas()` and `produced_schemas()` describe compiler API
traffic, not the package manifest set. `nomos-projection::all_schemas()` returns
exactly four independent identities. The nine functions in `nomos-sim` are
tested for distinct names and independence from all four projections. These
sets agree with the inventory once their deliberately different boundaries are
applied.

## Compiler receipt pass-profile disposition

Ordinary compile and v1-to-v2 migration intentionally share
`nomos.compiler_receipts@1`. They use the same exact fields, artifact digest
rows, compiler/catalog versions, construction/source identities, produced
schema list, invariant list shape, and source lineage digest. Their `passes`
field carries one of two closed ten-step profiles: `PASSES` for ordinary
compilation or `MIGRATION_PASSES` for the new v2 package. The strict reader
accepts only those exact arrays for active packages. Legacy-v1 opening accepts
only the historical compile profile with compiler/catalog version 1 and the
legacy invariant/schema set before migration emits the active migration
profile.

The profile is therefore a value inside one stable receipt schema, not a second
canonical type hidden under the same identity. No duplicate receipt root exists
in the migration module.

## Limits and final disposition

This review establishes ownership uniqueness for the implementation freeze; it
does not prove dependency metadata alone can infer semantic uniqueness. The
issue #69 exact-candidate evidence workflow and non-author audit are complete.
This receipt does not by itself mark the formal cold-agent runs or Gate K green;
those outcomes are disposed separately in the final Gate K decision record.

# Subject Explanation: Gate K Author Rehearsal

## Summary of Changes
Added one instance of the approved primitive `primitive/extinguishable_light` to `workspace/gaol.nomos`:
- Entity ID: `watch_lamp`
- Anchor: `cell 4 0 1`
- Preserved existing entities (`north_gate`, `flooded_section`, `brazier_02`), catalog declaration (`catalog credential/gaoler_key`), and schema header (`schema nomos.source@1`).

## Why the Compiler Accepted It
1. **Primitive and Catalog Compliance**: `primitive/extinguishable_light` is one of the three approved primitive kinds in Gate K. No unauthorized primitives, facts, or relation mutations were introduced.
2. **Grammar and Structural Requirements**: The entity ID `watch_lamp` is distinct and conforms to the `[a-z][a-z0-9_]*` identifier specification. The `anchor cell 4 0 1` line conforms to the exact field requirement for `primitive/extinguishable_light` using valid signed 32-bit integer coordinates.
3. **Expansion and Projection Linking**: The compiler expanded `watch_lamp` into its capability bundle (`machine`, `interactable`, `emits_light`, `authority`, `persisted`), initialized the machine namespace `watch_lamp.emission` in state `lit` with its valid `extinguish` command transition, composed the light emission claim into the light-union resolver without conflicts, and successfully emitted Canonical World IR (`nomos.world_ir@2`) and all associated projections.

## Exact Reproduction Commands
From `/workspace`:

```bash
# Validate source
/workspace/bin/nomos validate workspace/gaol.nomos

# Compile into a world package
/workspace/bin/nomos compile workspace/gaol.nomos --out workspace/compiled_world

# Inspect compiled world package
/workspace/bin/nomos inspect workspace/compiled_world
```

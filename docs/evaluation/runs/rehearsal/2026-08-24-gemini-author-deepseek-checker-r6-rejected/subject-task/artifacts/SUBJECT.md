# Gate K Author Rehearsal: Subject Explanation

## Changes Made
Added one approved `primitive/extinguishable_light` instance named `watch_lamp` to `/workspace/workspace/gaol.nomos` anchored at cell `4 0 1`. All existing catalog declarations and entity instances (`north_gate`, `flooded_section`, `brazier_02`) were preserved unchanged.

Added entity block in `/workspace/workspace/gaol.nomos`:
```text
entity watch_lamp primitive/extinguishable_light
  anchor cell 4 0 1
end
```

## Why the Compiler Accepted It
1. **Schema & Grammar Compliance**: The entity declaration conforms to `nomos.source@1` syntax, using valid identifiers and terminating cleanly with `end`.
2. **Approved Primitive**: `primitive/extinguishable_light` is one of the three approved primitive kinds in Gate K.
3. **Correct Field Shape**: The primitive requires exactly a cell anchor (`anchor cell 4 0 1`) and no extraneous or forbidden source-owned fields (e.g., no raw transforms, lattice relations, or derived facts).
4. **Distinct Symbol Identity**: The entity ID `watch_lamp` is unique within the symbol table and resolves cleanly without collision.
5. **Successful Expansion & Projection**: The compiler expanded `watch_lamp` into its compiler-owned `emission` state machine (initial state `lit`, `extinguish` transition), linked its `emits_light` claim, resolved spatial lattice bindings at cell `(4, 0, 1)`, and emitted valid projections (simulation, persistence, navigation, diagnostics) and fact-ownership receipts without causal cycles.

## Exact Reproduction Commands
From the `/workspace` directory:

```bash
# Validate the modified source fixture
/workspace/bin/nomos validate workspace/gaol.nomos

# Compile into a world package
/workspace/bin/nomos compile workspace/gaol.nomos --out workspace/compiled.world

# Inspect the compiled world package
/workspace/bin/nomos inspect workspace/compiled.world

# Explain the added entity
/workspace/bin/nomos explain-entity workspace/compiled.world watch_lamp
```

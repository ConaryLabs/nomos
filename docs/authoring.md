# Gate K `.nomos` authoring reference

Gate K source schema version 1 is a small line-oriented language. It contains
primitive instances and source-owned facts only. Machines, capability claims,
fact ownership, and projections come from the approved primitive catalog and
compiler.

## Grammar

```text
schema nomos.source@1
catalog <catalog>/<value>

entity <entity_id> <primitive>/<kind>
  anchor cell <x> <y> <z>
  anchor face <x> <y> <z> <north|east|south|west|up|down>
  anchor region <min_x> <min_y> <min_z> <max_x> <max_y> <max_z>
  credential <catalog>/<value>
end

relation <subject_entity> <relation_kind> <object_entity>
```

Blank lines and lines whose first non-whitespace character is `#` are ignored.
Indentation is optional. An entity body ends only at `end`. Integers are signed
32-bit decimal values with no leading `+`.

Identifier segments match `[a-z][a-z0-9_]*`. Entity IDs contain one segment,
primitive kinds are `primitive/<name>`, catalog values are `<catalog>/<value>`,
and relation kinds contain one segment. These are different types and do not
substitute for one another.

Gate K currently approves one relation kind, `owns`. The `credential` field
accepts only values in the `credential` catalog namespace; declaring an equal
name in another catalog does not satisfy it.

## Approved Gate K primitives

| Primitive | Required source fields | Compiler-owned expansion |
| --- | --- | --- |
| `primitive/iron_barred_door` | face anchor, credential | `access`, `integrity`, `ward`, `combustion`; portal and blocking claims |
| `primitive/shallow_water_region` | region anchor | traversal cost `3` |
| `primitive/extinguishable_light` | cell anchor | `emission`; light-emission claim |

Each entity accepts exactly the fields listed. Duplicate fields and extra
fields fail. The credential must be declared by a `catalog` declaration.

## Facts content cannot author

The compiler recognizes and rejects these mutation forms with dedicated
diagnostics:

```text
  lattice_relation <relation_kind> <entity_id>
  transform <anything...>
  derived <fact_name> <anything...>
fact_owner <fact_class> <owner>
```

They exist in the parser only so a forbidden fact receives the right stable
diagnostic and source span. They are never accepted into Canonical World IR.
Relations use the top-level `relation` form. Spatial truth uses `anchor`.
Derived facts and canonical owners belong to the compiler.

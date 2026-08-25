//! Strict reconstruction of a simulation projection from its canonical bytes.
//!
//! # Why this lives here
//!
//! [`SimulationPlan::to_canonical_bytes`] is the only writer of
//! `simulation.json`, and until R1-5 the workspace had no reader at all: the
//! compiler never decodes that member, it *recompiles* the plan from the
//! packaged Canonical World IR and compares the bytes
//! (`crates/nomos-compiler/src/opened.rs`). That is the right discipline for
//! opening a package, and it stays exactly as it was.
//!
//! It is not available to `crates/nomos-play`, whose browser build has no
//! package, no filesystem, and no permission to parse Canonical World IR —
//! `RUNTIME.md` section 3 forbids an R1 crate from doing so by name. So the
//! decoder is here, beside the encoder it inverts, declared as R1 read-only
//! surface under `RUNTIME.md` section 3 option (a). Encoder and decoder in one
//! file cannot drift; a decoder in the consuming crate could.
//!
//! # What this does not change
//!
//! **No Gate K command executes from a bare projection.** `nomos compile`,
//! `nomos run`, `nomos replay`, `nomos verify`, and `nomos effective-facts`
//! still obtain their [`SimulationPlan`] by recompiling the packaged stable
//! World IR and checking the stored member bytes against it
//! (`nomos_compiler::open_compiled_package`). Nothing in this module is
//! reachable from any of them, no Gate K artifact, hash, or diagnostic changes,
//! and the only new behaviour is that a caller already holding verified
//! projection bytes can turn them back into the value that produced them.
//!
//! # The bound
//!
//! [`SimulationPlan::from_canonical_bytes`] refuses unless the value it
//! reconstructs re-encodes to the exact input bytes. Because
//! [`SimulationPlan::to_canonical_bytes`] is total over the type — every field
//! of every variant is written — byte identity means the decode recovered
//! precisely the plan those bytes came from. That is the discipline
//! `nomos-sim` already applies to its own decoders
//! (`crates/nomos-sim/src/state_persistence.rs`), and it is checked here
//! rather than trusted.
//!
//! The second lock is the kernel's, and it costs nothing: a persisted runtime
//! state carries `runtime_semantics_digest`, the SHA-256 of
//! `plan.to_canonical_bytes()`, and
//! `PersistedRuntimeState::from_canonical_bytes` refuses `EK0813` on a
//! mismatch. A mis-decoded plan therefore cannot be paired with a
//! kernel-produced state at all.
//!
//! # Trust model, stated once
//!
//! A decoded projection is executable semantics that arrived without its World
//! IR. This module verifies that the bytes reconstruct exactly, and nothing
//! more: it does not and cannot establish that those bytes are the compiler's
//! output for any particular source. A caller that needs that guarantee opens
//! the package, which recompiles and compares. A caller replaying semantics
//! already verified elsewhere — `nomos-play` in a browser, holding bytes whose
//! digest its rendering plan published — gets what it needs and no false
//! assurance.

use std::collections::BTreeMap;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::id::{CatalogValueId, ClaimRef, EntityId, NamespaceId, SchemaId};
use nomos_core::{
    CanonicalValue, Diagnostic, FieldName, Ident, RepairClass, SourcePath, SourceSpan,
};

use crate::light::{LightClaim, LightProjectionConsumer, LightResolverPlan, LightSubject};
use crate::movement::{
    LatticeCell, MovementClaim, MovementConnectivity, MovementResolverPlan, MovementSubject,
    ProjectedActivation,
};
use crate::simulation::{
    CausalEdge, CommandRequirement, CommandTransition, EventHandler, EventPayload,
    MachineDefinition, Phase, SimulationPlan,
};
use crate::simulation_schema;
use crate::state::{ProjectedDirection, ProjectedEntity, RuntimeBinding};

impl SimulationPlan {
    /// Strictly reconstructs one simulation projection from canonical bytes.
    ///
    /// # Errors
    ///
    /// Returns `EK0412` for any canonical, schema, field-set, identifier, or
    /// value disagreement, and for bytes that do not re-encode exactly from the
    /// typed meaning they decode to.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "simulation projection")?;
        require_fields(
            fields,
            &[
                "causal_edges",
                "entities",
                "light_resolver",
                "machines",
                "movement_resolver",
                "schema",
            ],
            "simulation projection",
        )?;
        let declared = schema(field(fields, "schema")?, "simulation projection schema")?;
        if declared != simulation_schema() {
            return Err(invalid("simulation projection names an unsupported schema"));
        }

        let machines = array(field(fields, "machines")?, "simulation machines")?
            .iter()
            .map(machine)
            .collect::<Result<Vec<_>, _>>()?;
        let causal_edges = array(field(fields, "causal_edges")?, "causal edges")?
            .iter()
            .map(causal_edge)
            .collect::<Result<Vec<_>, _>>()?;
        let entities = array(field(fields, "entities")?, "projected entities")?
            .iter()
            .map(projected_entity)
            .collect::<Result<Vec<_>, _>>()?;

        let plan = Self::new(machines, causal_edges)?
            .with_entities(entities)?
            .with_movement_resolver(movement_resolver(field(fields, "movement_resolver")?)?)
            .with_light_resolver(light_resolver(field(fields, "light_resolver")?)?);

        if plan.to_canonical_bytes() != bytes {
            return Err(invalid(
                "simulation projection does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(plan)
    }
}

fn machine(value: &CanonicalValue) -> Result<MachineDefinition, Diagnostic> {
    let fields = object(value, "simulation machine")?;
    require_fields(
        fields,
        &["commands", "handlers", "initial", "namespace", "states"],
        "simulation machine",
    )?;
    let states = array(field(fields, "states")?, "machine states")?
        .iter()
        .map(|state| ident(state, "machine state"))
        .collect::<Result<Vec<_>, _>>()?;
    let commands = array(field(fields, "commands")?, "command transitions")?
        .iter()
        .map(command_transition)
        .collect::<Result<Vec<_>, _>>()?;
    let handlers = array(field(fields, "handlers")?, "event handlers")?
        .iter()
        .map(event_handler)
        .collect::<Result<Vec<_>, _>>()?;
    MachineDefinition::new(
        namespace(field(fields, "namespace")?, "machine namespace")?,
        states,
        ident(field(fields, "initial")?, "machine initial state")?,
        commands,
        handlers,
    )
}

fn command_transition(value: &CanonicalValue) -> Result<CommandTransition, Diagnostic> {
    let fields = object(value, "command transition")?;
    require_fields(
        fields,
        &["action", "requirement", "source", "target"],
        "command transition",
    )?;
    Ok(CommandTransition::new(
        ident(field(fields, "action")?, "command action")?,
        requirement(field(fields, "requirement")?)?,
        ident(field(fields, "source")?, "command source state")?,
        ident(field(fields, "target")?, "command target state")?,
    ))
}

fn requirement(value: &CanonicalValue) -> Result<CommandRequirement, Diagnostic> {
    let fields = object(value, "command requirement")?;
    match text(field(fields, "kind")?, "command requirement kind")? {
        "none" => {
            require_fields(fields, &["kind"], "command requirement")?;
            Ok(CommandRequirement::None)
        }
        "credential" => {
            require_fields(fields, &["credential", "kind"], "command requirement")?;
            Ok(CommandRequirement::Credential(
                CatalogValueId::parse(text(field(fields, "credential")?, "credential")?)
                    .map_err(|error| invalid(error.message()))?,
            ))
        }
        _ => Err(invalid("command requirement kind is unsupported")),
    }
}

fn event_handler(value: &CanonicalValue) -> Result<EventHandler, Diagnostic> {
    let fields = object(value, "event handler")?;
    require_fields(
        fields,
        &["name", "payload", "source", "target"],
        "event handler",
    )?;
    Ok(EventHandler::new(
        ident(field(fields, "name")?, "handler name")?,
        payload(field(fields, "payload")?)?,
        ident(field(fields, "source")?, "handler source state")?,
        ident(field(fields, "target")?, "handler target state")?,
    ))
}

fn payload(value: &CanonicalValue) -> Result<EventPayload, Diagnostic> {
    let fields = object(value, "event payload")?;
    match text(field(fields, "kind")?, "event payload kind")? {
        "damage" => {
            require_fields(fields, &["amount", "channel", "kind"], "event payload")?;
            Ok(EventPayload::Damage {
                channel: ident(field(fields, "channel")?, "damage channel")?,
                amount: u32_of(field(fields, "amount")?, "damage amount")?,
            })
        }
        _ => Err(invalid("event payload kind is unsupported")),
    }
}

fn causal_edge(value: &CanonicalValue) -> Result<CausalEdge, Diagnostic> {
    let fields = object(value, "causal edge")?;
    require_fields(
        fields,
        &[
            "entered_state",
            "payload",
            "phase",
            "source_namespace",
            "target_handler",
            "target_namespace",
        ],
        "causal edge",
    )?;
    let phase = match text(field(fields, "phase")?, "causal edge phase")? {
        "local" => Phase::Local,
        "causal" => Phase::Causal,
        _ => return Err(invalid("causal edge phase is unsupported")),
    };
    Ok(CausalEdge::new(
        namespace(field(fields, "source_namespace")?, "edge source namespace")?,
        ident(field(fields, "entered_state")?, "edge entered state")?,
        phase,
        namespace(field(fields, "target_namespace")?, "edge target namespace")?,
        ident(field(fields, "target_handler")?, "edge target handler")?,
        payload(field(fields, "payload")?)?,
    ))
}

fn projected_entity(value: &CanonicalValue) -> Result<ProjectedEntity, Diagnostic> {
    let fields = object(value, "projected entity")?;
    require_fields(fields, &["binding", "id", "machines"], "projected entity")?;
    let machines = array(field(fields, "machines")?, "projected entity machines")?
        .iter()
        .map(|value| namespace(value, "projected entity machine"))
        .collect::<Result<Vec<_>, _>>()?;
    ProjectedEntity::new(
        entity(field(fields, "id")?, "projected entity id")?,
        binding(field(fields, "binding")?)?,
        machines,
    )
}

fn binding(value: &CanonicalValue) -> Result<RuntimeBinding, Diagnostic> {
    let fields = object(value, "runtime binding")?;
    match text(field(fields, "kind")?, "runtime binding kind")? {
        "cell" => {
            require_fields(fields, &["cell", "kind"], "cell binding")?;
            Ok(RuntimeBinding::Cell(cell(field(fields, "cell")?)?))
        }
        "face" => {
            require_fields(fields, &["cell", "direction", "kind"], "face binding")?;
            Ok(RuntimeBinding::Face {
                cell: cell(field(fields, "cell")?)?,
                direction: direction(field(fields, "direction")?)?,
            })
        }
        "region" => {
            require_fields(fields, &["kind", "max", "min"], "region binding")?;
            Ok(RuntimeBinding::Region {
                min: cell(field(fields, "min")?)?,
                max: cell(field(fields, "max")?)?,
            })
        }
        _ => Err(invalid("runtime binding kind is unsupported")),
    }
}

fn direction(value: &CanonicalValue) -> Result<ProjectedDirection, Diagnostic> {
    match text(value, "face direction")? {
        "north" => Ok(ProjectedDirection::North),
        "east" => Ok(ProjectedDirection::East),
        "south" => Ok(ProjectedDirection::South),
        "west" => Ok(ProjectedDirection::West),
        "up" => Ok(ProjectedDirection::Up),
        "down" => Ok(ProjectedDirection::Down),
        _ => Err(invalid("face direction is unsupported")),
    }
}

fn movement_resolver(value: &CanonicalValue) -> Result<MovementResolverPlan, Diagnostic> {
    let fields = object(value, "movement resolver")?;
    require_fields(
        fields,
        &[
            "base_cost",
            "blockers_any_active",
            "blockers_before_cost",
            "channel",
            "costs_maximum_active",
            "requires_connectivity",
            "subjects",
        ],
        "movement resolver",
    )?;
    let subjects = array(field(fields, "subjects")?, "movement subjects")?
        .iter()
        .map(movement_subject)
        .collect::<Result<Vec<_>, _>>()?;
    MovementResolverPlan::new(
        ident(field(fields, "channel")?, "movement channel")?,
        u32_of(field(fields, "base_cost")?, "movement base cost")?,
        bool_of(field(fields, "blockers_any_active")?, "blockers_any_active")?,
        bool_of(
            field(fields, "costs_maximum_active")?,
            "costs_maximum_active",
        )?,
        bool_of(
            field(fields, "blockers_before_cost")?,
            "blockers_before_cost",
        )?,
        bool_of(
            field(fields, "requires_connectivity")?,
            "requires_connectivity",
        )?,
        subjects,
    )
}

fn movement_subject(value: &CanonicalValue) -> Result<MovementSubject, Diagnostic> {
    let fields = object(value, "movement subject")?;
    require_fields(
        fields,
        &["claims", "connectivity", "entity"],
        "movement subject",
    )?;
    let claims = array(field(fields, "claims")?, "movement claims")?
        .iter()
        .map(movement_claim)
        .collect::<Result<Vec<_>, _>>()?;
    MovementSubject::new(
        entity(field(fields, "entity")?, "movement subject entity")?,
        connectivity(field(fields, "connectivity")?)?,
        claims,
    )
}

fn connectivity(value: &CanonicalValue) -> Result<MovementConnectivity, Diagnostic> {
    let fields = object(value, "movement connectivity")?;
    match text(field(fields, "kind")?, "movement connectivity kind")? {
        "face_adjacent" => {
            require_fields(
                fields,
                &["first", "kind", "second"],
                "face-adjacent connectivity",
            )?;
            Ok(MovementConnectivity::FaceAdjacent {
                first: cell(field(fields, "first")?)?,
                second: cell(field(fields, "second")?)?,
            })
        }
        "region" => {
            require_fields(fields, &["kind", "max", "min"], "region connectivity")?;
            Ok(MovementConnectivity::Region {
                min: cell(field(fields, "min")?)?,
                max: cell(field(fields, "max")?)?,
            })
        }
        _ => Err(invalid("movement connectivity kind is unsupported")),
    }
}

fn movement_claim(value: &CanonicalValue) -> Result<MovementClaim, Diagnostic> {
    let fields = object(value, "movement claim")?;
    require_fields(
        fields,
        &["activation", "capability", "id", "source", "value"],
        "movement claim",
    )?;
    let id = claim(field(fields, "id")?)?;
    let expression = activation(field(fields, "activation")?)?;
    let span = source_span(field(fields, "source")?)?;
    match text(field(fields, "capability")?, "movement claim capability")? {
        "blocks_ground" => Ok(MovementClaim::blocker(
            id,
            expression,
            bool_of(field(fields, "value")?, "blocker value")?,
            span,
        )),
        "traversal_cost_ground" => MovementClaim::traversal_cost(
            id,
            expression,
            u32_of(field(fields, "value")?, "traversal cost value")?,
            span,
        ),
        _ => Err(invalid("movement claim capability is unsupported")),
    }
}

fn light_resolver(value: &CanonicalValue) -> Result<LightResolverPlan, Diagnostic> {
    let fields = object(value, "light resolver")?;
    require_fields(
        fields,
        &["consumers", "subjects", "union_active"],
        "light resolver",
    )?;
    let consumers = array(field(fields, "consumers")?, "light consumers")?
        .iter()
        .map(|value| match text(value, "light consumer")? {
            "diagnostics" => Ok(LightProjectionConsumer::Diagnostics),
            "persistence" => Ok(LightProjectionConsumer::Persistence),
            "simulation" => Ok(LightProjectionConsumer::Simulation),
            _ => Err(invalid("light projection consumer is unsupported")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let subjects = array(field(fields, "subjects")?, "light subjects")?
        .iter()
        .map(light_subject)
        .collect::<Result<Vec<_>, _>>()?;
    LightResolverPlan::new(
        bool_of(field(fields, "union_active")?, "union_active")?,
        consumers,
        subjects,
    )
}

fn light_subject(value: &CanonicalValue) -> Result<LightSubject, Diagnostic> {
    let fields = object(value, "light subject")?;
    require_fields(fields, &["claims", "entity"], "light subject")?;
    let claims = array(field(fields, "claims")?, "light claims")?
        .iter()
        .map(light_claim)
        .collect::<Result<Vec<_>, _>>()?;
    LightSubject::new(
        entity(field(fields, "entity")?, "light subject entity")?,
        claims,
    )
}

fn light_claim(value: &CanonicalValue) -> Result<LightClaim, Diagnostic> {
    let fields = object(value, "light claim")?;
    require_fields(
        fields,
        &["activation", "capability", "id", "source", "value"],
        "light claim",
    )?;
    if text(field(fields, "capability")?, "light claim capability")? != "emits_light" {
        return Err(invalid("light claim capability is unsupported"));
    }
    Ok(LightClaim::new(
        claim(field(fields, "id")?)?,
        activation(field(fields, "activation")?)?,
        bool_of(field(fields, "value")?, "light claim value")?,
        source_span(field(fields, "source")?)?,
    ))
}

fn activation(value: &CanonicalValue) -> Result<ProjectedActivation, Diagnostic> {
    let fields = object(value, "projected activation")?;
    match text(field(fields, "kind")?, "activation kind")? {
        "always" => {
            require_fields(fields, &["kind"], "always activation")?;
            Ok(ProjectedActivation::Always)
        }
        "state_equals" => {
            require_fields(
                fields,
                &["kind", "namespace", "state"],
                "state-equals activation",
            )?;
            Ok(ProjectedActivation::StateEquals {
                namespace: namespace(field(fields, "namespace")?, "activation namespace")?,
                state: ident(field(fields, "state")?, "activation state")?,
            })
        }
        "any" => Ok(ProjectedActivation::Any(activation_children(
            fields, "any",
        )?)),
        "all" => Ok(ProjectedActivation::All(activation_children(
            fields, "all",
        )?)),
        "not" => {
            require_fields(fields, &["child", "kind"], "not activation")?;
            Ok(ProjectedActivation::Not(Box::new(activation(field(
                fields, "child",
            )?)?)))
        }
        _ => Err(invalid("projected activation kind is unsupported")),
    }
}

fn activation_children(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    kind: &str,
) -> Result<Vec<ProjectedActivation>, Diagnostic> {
    require_fields(fields, &["children", "kind"], "activation group")?;
    let children = array(field(fields, "children")?, "activation children")?;
    if children.is_empty() {
        return Err(invalid(format!("`{kind}` activation group is empty")));
    }
    children.iter().map(activation).collect()
}

fn source_span(value: &CanonicalValue) -> Result<SourceSpan, Diagnostic> {
    let fields = object(value, "source span")?;
    require_fields(
        fields,
        &["byte_end", "byte_start", "column", "line", "path"],
        "source span",
    )?;
    SourceSpan::new(
        SourcePath::new(text(field(fields, "path")?, "source path")?)?,
        u32_of(field(fields, "byte_start")?, "span byte_start")?,
        u32_of(field(fields, "byte_end")?, "span byte_end")?,
        u32_of(field(fields, "line")?, "span line")?,
        u32_of(field(fields, "column")?, "span column")?,
    )
}

fn cell(value: &CanonicalValue) -> Result<LatticeCell, Diagnostic> {
    let fields = object(value, "lattice cell")?;
    require_fields(fields, &["x", "y", "z"], "lattice cell")?;
    Ok(LatticeCell::new(
        i32_of(field(fields, "x")?, "cell x")?,
        i32_of(field(fields, "y")?, "cell y")?,
        i32_of(field(fields, "z")?, "cell z")?,
    ))
}

fn schema(value: &CanonicalValue, label: &str) -> Result<SchemaId, Diagnostic> {
    let fields = object(value, label)?;
    require_fields(fields, &["name", "version"], label)?;
    let version = u32_of(field(fields, "version")?, label)?;
    SchemaId::new(text(field(fields, "name")?, label)?, version)
        .map_err(|error| invalid(error.message()))
}

fn ident(value: &CanonicalValue, label: &str) -> Result<Ident, Diagnostic> {
    Ident::new(text(value, label)?).map_err(|error| invalid(error.message()))
}

fn entity(value: &CanonicalValue, label: &str) -> Result<EntityId, Diagnostic> {
    EntityId::parse(text(value, label)?).map_err(|error| invalid(error.message()))
}

fn namespace(value: &CanonicalValue, label: &str) -> Result<NamespaceId, Diagnostic> {
    NamespaceId::parse(text(value, label)?).map_err(|error| invalid(error.message()))
}

fn claim(value: &CanonicalValue) -> Result<ClaimRef, Diagnostic> {
    ClaimRef::parse(text(value, "claim identity")?).map_err(|error| invalid(error.message()))
}

fn object<'a>(
    value: &'a CanonicalValue,
    label: &str,
) -> Result<&'a BTreeMap<FieldName, CanonicalValue>, Diagnostic> {
    let CanonicalValue::Object(fields) = value else {
        return Err(invalid(format!("{label} is not an object")));
    };
    Ok(fields)
}

fn array<'a>(value: &'a CanonicalValue, label: &str) -> Result<&'a [CanonicalValue], Diagnostic> {
    let CanonicalValue::Array(values) = value else {
        return Err(invalid(format!("{label} is not an array")));
    };
    Ok(values)
}

fn text<'a>(value: &'a CanonicalValue, label: &str) -> Result<&'a str, Diagnostic> {
    let CanonicalValue::Text(value) = value else {
        return Err(invalid(format!("{label} is not text")));
    };
    Ok(value)
}

fn bool_of(value: &CanonicalValue, label: &str) -> Result<bool, Diagnostic> {
    let CanonicalValue::Bool(value) = value else {
        return Err(invalid(format!("{label} is not a boolean")));
    };
    Ok(*value)
}

fn u32_of(value: &CanonicalValue, label: &str) -> Result<u32, Diagnostic> {
    // `parse_canonical` reads every integer literal as `Int`; the encoder writes
    // `Uint`. Both spellings mean the same non-negative number here, and the
    // re-encode check is what proves the round trip regardless.
    let value = match value {
        CanonicalValue::Uint(value) => *value,
        CanonicalValue::Int(value) => {
            u64::try_from(*value).map_err(|_| invalid(format!("{label} is negative")))?
        }
        _ => return Err(invalid(format!("{label} is not an unsigned integer"))),
    };
    u32::try_from(value).map_err(|_| invalid(format!("{label} exceeds u32")))
}

fn i32_of(value: &CanonicalValue, label: &str) -> Result<i32, Diagnostic> {
    let value = match value {
        CanonicalValue::Int(value) => *value,
        CanonicalValue::Uint(value) => i64::try_from(*value)
            .map_err(|_| invalid(format!("{label} exceeds signed integer range")))?,
        _ => return Err(invalid(format!("{label} is not an integer"))),
    };
    i32::try_from(value).map_err(|_| invalid(format!("{label} exceeds i32")))
}

fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid(format!("simulation projection value has no `{name}` field")))
}

fn require_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    label: &str,
) -> Result<(), Diagnostic> {
    let actual = fields.keys().map(FieldName::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "{label} fields are {actual:?}; expected {expected:?}"
        )))
    }
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_SCHEMA_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

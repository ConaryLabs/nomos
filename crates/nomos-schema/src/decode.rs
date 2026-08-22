//! Strict read-side reconstruction of the complete stable World IR.

use std::collections::BTreeMap;

mod provenance;

use provenance::{decode_consumer, decode_receipt};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::diagnostic::codes;
use nomos_core::{
    CanonicalValue, CatalogValueId, ClaimRef, Diagnostic, EntityId, FieldName, Ident, NamespaceId,
    PrimitiveKindId, RepairClass, SchemaId, SourcePath, SourceSpan,
};

use crate::{
    Binding, CapabilityKind, Cell, ClaimActivation, ClaimTemplate, ClaimValue, Direction,
    GroundConnectivity, GroundMovementCoherence, InteractionDefinition, InteractionPhase,
    InteractionTrigger, IrEntity, IrRelation, LightCompositionLaw, LightResolverPlan,
    LightResolverSubject, MachineTemplate, MovementCompositionLaw, MovementResolverPlan,
    MovementResolverSubject, PrimitiveExpansion, StableGroundMovementV1, StableWorldIr,
    TransitionDefinition, TransitionInput, TransitionTrigger, WorldIr,
    construction_world_ir_schema, source_schema, stable_world_ir_schema,
};

impl StableWorldIr {
    /// Strictly reconstructs the complete active stable World IR from canonical
    /// bytes.
    ///
    /// Every nested object and closed vocabulary is decoded into its owning
    /// Rust type. Typed constructors reapply semantic invariants, and the final
    /// typed value must re-encode byte-for-byte to the supplied evidence. That
    /// last comparison rejects persisted arrays that would require sorting or
    /// any other normalization.
    ///
    /// # Errors
    ///
    /// Returns `EK0412` when bytes are malformed, fields or kinds are unknown,
    /// typed invariants fail, semantic ordering is not already canonical, or
    /// the reconstructed value does not reproduce the original bytes.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes).map_err(|error| invalid(error.message()))?;
        let decoded = decode_stable_world_ir(&value)?;
        if decoded.to_canonical_bytes() != bytes {
            return Err(invalid(
                "stable World IR changes when reconstructed; persisted semantic ordering or shape is not canonical",
            ));
        }
        Ok(decoded)
    }
}

fn decode_stable_world_ir(value: &CanonicalValue) -> Result<StableWorldIr, Diagnostic> {
    let fields = object(value, "stable World IR")?;
    exact_fields(
        fields,
        &[
            "catalog_values",
            "compiler_version",
            "construction_schema",
            "entities",
            "light_resolver",
            "movement_resolver",
            "movement_v1",
            "ownership_receipts",
            "primitive_catalog_version",
            "relations",
            "schema",
            "source_schema",
        ],
        "stable World IR",
    )?;
    require_schema(
        field(fields, "schema", "stable World IR")?,
        &stable_world_ir_schema(),
    )?;
    require_schema(
        field(fields, "construction_schema", "stable World IR")?,
        &construction_world_ir_schema(),
    )?;
    require_schema(
        field(fields, "source_schema", "stable World IR")?,
        &source_schema(),
    )?;

    let catalog_values = array(
        field(fields, "catalog_values", "stable World IR")?,
        "catalog values",
    )?
    .iter()
    .map(|value| parse_catalog_value(value, "catalog value"))
    .collect::<Result<Vec<_>, _>>()?;
    let entities = array(field(fields, "entities", "stable World IR")?, "entities")?
        .iter()
        .map(decode_entity)
        .collect::<Result<Vec<_>, _>>()?;
    let relations = array(field(fields, "relations", "stable World IR")?, "relations")?
        .iter()
        .map(decode_relation)
        .collect::<Result<Vec<_>, _>>()?;
    let receipts = array(
        field(fields, "ownership_receipts", "stable World IR")?,
        "ownership receipts",
    )?
    .iter()
    .map(decode_receipt)
    .collect::<Result<Vec<_>, _>>()?;
    let movement = decode_movement_plan(field(fields, "movement_resolver", "stable World IR")?)?;
    let light = decode_light_plan(field(fields, "light_resolver", "stable World IR")?)?;
    let construction = rebuild(
        WorldIr::new(
            source_schema(),
            catalog_values,
            entities,
            relations,
            receipts,
        ),
        "construction World IR",
    )?
    .with_movement_resolver(movement)
    .with_light_resolver(light);
    let movement_v1 = array(
        field(fields, "movement_v1", "stable World IR")?,
        "movement_v1",
    )?
    .iter()
    .map(decode_stable_movement)
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        StableWorldIr::new(
            construction,
            unsigned_u32(
                field(fields, "compiler_version", "stable World IR")?,
                "compiler version",
            )?,
            unsigned_u32(
                field(fields, "primitive_catalog_version", "stable World IR")?,
                "primitive catalog version",
            )?,
            movement_v1,
        ),
        "stable World IR",
    )
}

fn decode_entity(value: &CanonicalValue) -> Result<IrEntity, Diagnostic> {
    let fields = object(value, "World IR entity")?;
    exact_fields(
        fields,
        &[
            "binding",
            "credential",
            "expansion",
            "id",
            "primitive",
            "source",
        ],
        "World IR entity",
    )?;
    let id = parse_entity(field(fields, "id", "World IR entity")?, "entity id")?;
    let primitive = PrimitiveKindId::parse(text(
        field(fields, "primitive", "World IR entity")?,
        "entity primitive",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let credential = match field(fields, "credential", "World IR entity")? {
        CanonicalValue::Null => None,
        value => Some(parse_catalog_value(value, "entity credential")?),
    };
    Ok(IrEntity::new(
        id,
        primitive,
        decode_binding(field(fields, "binding", "World IR entity")?)?,
        credential,
        decode_expansion(field(fields, "expansion", "World IR entity")?)?,
        decode_span(field(fields, "source", "World IR entity")?)?,
    ))
}

fn decode_expansion(value: &CanonicalValue) -> Result<PrimitiveExpansion, Diagnostic> {
    let fields = object(value, "primitive expansion")?;
    exact_fields(
        fields,
        &["capabilities", "claims", "interactions", "machines"],
        "primitive expansion",
    )?;
    let capabilities = array(
        field(fields, "capabilities", "primitive expansion")?,
        "capabilities",
    )?
    .iter()
    .map(decode_capability)
    .collect::<Result<Vec<_>, _>>()?;
    let machines = array(
        field(fields, "machines", "primitive expansion")?,
        "machines",
    )?
    .iter()
    .map(decode_machine)
    .collect::<Result<Vec<_>, _>>()?;
    let claims = array(field(fields, "claims", "primitive expansion")?, "claims")?
        .iter()
        .map(decode_claim)
        .collect::<Result<Vec<_>, _>>()?;
    let interactions = array(
        field(fields, "interactions", "primitive expansion")?,
        "interactions",
    )?
    .iter()
    .map(decode_interaction)
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        rebuild(
            PrimitiveExpansion::new(capabilities, machines, claims),
            "primitive expansion",
        )?
        .with_interactions(interactions),
        "primitive interactions",
    )
}

fn decode_machine(value: &CanonicalValue) -> Result<MachineTemplate, Diagnostic> {
    let fields = object(value, "machine")?;
    exact_fields(
        fields,
        &["initial", "namespace", "states", "transitions"],
        "machine",
    )?;
    let namespace = parse_namespace(field(fields, "namespace", "machine")?, "machine namespace")?;
    let states = array(field(fields, "states", "machine")?, "machine states")?
        .iter()
        .map(|value| parse_ident(value, "machine state"))
        .collect::<Result<Vec<_>, _>>()?;
    let initial = parse_ident(
        field(fields, "initial", "machine")?,
        "machine initial state",
    )?;
    let transitions = array(
        field(fields, "transitions", "machine")?,
        "machine transitions",
    )?
    .iter()
    .map(decode_transition)
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        MachineTemplate::new(namespace, states, initial).with_transitions(transitions),
        "machine transitions",
    )
}

fn decode_transition(value: &CanonicalValue) -> Result<TransitionDefinition, Diagnostic> {
    let fields = object(value, "transition")?;
    exact_fields(fields, &["effect", "source", "trigger"], "transition")?;
    let effect = object(field(fields, "effect", "transition")?, "transition effect")?;
    exact_fields(effect, &["kind", "state"], "transition effect")?;
    require_text(
        field(effect, "kind", "transition effect")?,
        "set_state",
        "transition effect kind",
    )?;
    Ok(TransitionDefinition::new(
        decode_transition_trigger(field(fields, "trigger", "transition")?)?,
        parse_ident(field(fields, "source", "transition")?, "transition source")?,
        parse_ident(
            field(effect, "state", "transition effect")?,
            "transition target",
        )?,
    ))
}

fn decode_transition_trigger(value: &CanonicalValue) -> Result<TransitionTrigger, Diagnostic> {
    let fields = object(value, "transition trigger")?;
    exact_fields(fields, &["input", "kind", "name"], "transition trigger")?;
    let name = parse_ident(field(fields, "name", "transition trigger")?, "trigger name")?;
    let input = decode_transition_input(field(fields, "input", "transition trigger")?)?;
    match text(
        field(fields, "kind", "transition trigger")?,
        "transition trigger kind",
    )? {
        "command" => Ok(TransitionTrigger::Command {
            action: name,
            input,
        }),
        "event" => Ok(TransitionTrigger::Event {
            handler: name,
            input,
        }),
        kind => Err(invalid(format!(
            "unsupported transition trigger kind `{kind}`"
        ))),
    }
}

fn decode_transition_input(value: &CanonicalValue) -> Result<TransitionInput, Diagnostic> {
    let fields = object(value, "transition input")?;
    let kind = text(
        field(fields, "kind", "transition input")?,
        "transition input kind",
    )?;
    match kind {
        "none" => {
            exact_fields(fields, &["kind"], "none transition input")?;
            Ok(TransitionInput::None)
        }
        "resolved_entity_credential" => {
            exact_fields(fields, &["kind"], "credential transition input")?;
            Ok(TransitionInput::ResolvedEntityCredential)
        }
        "damage" => {
            exact_fields(
                fields,
                &["amount", "channel", "kind"],
                "damage transition input",
            )?;
            Ok(TransitionInput::Damage {
                channel: parse_ident(field(fields, "channel", "damage input")?, "damage channel")?,
                amount: unsigned_u32(field(fields, "amount", "damage input")?, "damage amount")?,
            })
        }
        _ => Err(invalid(format!(
            "unsupported transition input kind `{kind}`"
        ))),
    }
}

fn decode_interaction(value: &CanonicalValue) -> Result<InteractionDefinition, Diagnostic> {
    let fields = object(value, "interaction")?;
    exact_fields(
        fields,
        &[
            "payload",
            "phase",
            "target_handler",
            "target_namespace",
            "trigger",
        ],
        "interaction",
    )?;
    require_text(
        field(fields, "phase", "interaction")?,
        "causal",
        "interaction phase",
    )?;
    let trigger = object(
        field(fields, "trigger", "interaction")?,
        "interaction trigger",
    )?;
    exact_fields(
        trigger,
        &["kind", "namespace", "state"],
        "interaction trigger",
    )?;
    require_text(
        field(trigger, "kind", "interaction trigger")?,
        "on_enter",
        "interaction trigger kind",
    )?;
    Ok(InteractionDefinition::new(
        InteractionTrigger::OnEnter {
            namespace: parse_namespace(
                field(trigger, "namespace", "interaction trigger")?,
                "interaction source namespace",
            )?,
            state: parse_ident(
                field(trigger, "state", "interaction trigger")?,
                "interaction state",
            )?,
        },
        InteractionPhase::Causal,
        parse_namespace(
            field(fields, "target_namespace", "interaction")?,
            "target namespace",
        )?,
        parse_ident(
            field(fields, "target_handler", "interaction")?,
            "target handler",
        )?,
        decode_transition_input(field(fields, "payload", "interaction")?)?,
    ))
}

fn decode_claim(value: &CanonicalValue) -> Result<ClaimTemplate, Diagnostic> {
    let fields = object(value, "claim")?;
    exact_fields(
        fields,
        &["activation", "capability", "id", "value"],
        "claim",
    )?;
    let capability = decode_capability(field(fields, "capability", "claim")?)?;
    let claim = ClaimRef::parse(text(field(fields, "id", "claim")?, "claim id")?)
        .map_err(|error| invalid(error.message()))?;
    if claim.capability().as_str() != capability.as_str() {
        return Err(invalid(format!(
            "claim `{claim}` disagrees with capability `{}`",
            capability.as_str()
        )));
    }
    let claim_value = match field(fields, "value", "claim")? {
        CanonicalValue::Bool(value) => ClaimValue::Bool(*value),
        value => ClaimValue::Uint(unsigned_u32(value, "claim value")?),
    };
    Ok(ClaimTemplate::new(
        claim,
        capability,
        decode_activation(field(fields, "activation", "claim")?)?,
        claim_value,
    ))
}

fn decode_activation(value: &CanonicalValue) -> Result<ClaimActivation, Diagnostic> {
    let fields = object(value, "claim activation")?;
    let kind = text(
        field(fields, "kind", "claim activation")?,
        "claim activation kind",
    )?;
    match kind {
        "always" => {
            exact_fields(fields, &["kind"], "always activation")?;
            Ok(ClaimActivation::Always)
        }
        "state_equals" => {
            exact_fields(fields, &["kind", "namespace", "state"], "state activation")?;
            Ok(ClaimActivation::StateEquals {
                namespace: parse_namespace(
                    field(fields, "namespace", "state activation")?,
                    "activation namespace",
                )?,
                state: parse_ident(
                    field(fields, "state", "state activation")?,
                    "activation state",
                )?,
            })
        }
        "any" | "all" => {
            exact_fields(fields, &["children", "kind"], "activation group")?;
            let children = array(
                field(fields, "children", "activation group")?,
                "activation children",
            )?
            .iter()
            .map(decode_activation)
            .collect::<Result<Vec<_>, _>>()?;
            if kind == "any" {
                Ok(ClaimActivation::Any(children))
            } else {
                Ok(ClaimActivation::All(children))
            }
        }
        "not" => {
            exact_fields(fields, &["child", "kind"], "not activation")?;
            Ok(ClaimActivation::Not(Box::new(decode_activation(field(
                fields,
                "child",
                "not activation",
            )?)?)))
        }
        _ => Err(invalid(format!(
            "unsupported claim activation kind `{kind}`"
        ))),
    }
}

fn decode_capability(value: &CanonicalValue) -> Result<CapabilityKind, Diagnostic> {
    match text(value, "capability")? {
        "boundary" => Ok(CapabilityKind::Boundary),
        "portal" => Ok(CapabilityKind::Portal),
        "blocks_ground" => Ok(CapabilityKind::BlocksGround),
        "traversal_cost_ground" => Ok(CapabilityKind::TraversalCostGround),
        "machine" => Ok(CapabilityKind::Machine),
        "interactable" => Ok(CapabilityKind::Interactable),
        "region" => Ok(CapabilityKind::Region),
        "emits_light" => Ok(CapabilityKind::EmitsLight),
        "authority" => Ok(CapabilityKind::Authority),
        "persisted" => Ok(CapabilityKind::Persisted),
        kind => Err(invalid(format!("unsupported capability `{kind}`"))),
    }
}

fn decode_relation(value: &CanonicalValue) -> Result<IrRelation, Diagnostic> {
    let fields = object(value, "relation")?;
    exact_fields(fields, &["kind", "object", "source", "subject"], "relation")?;
    Ok(IrRelation::new(
        parse_entity(field(fields, "subject", "relation")?, "relation subject")?,
        parse_ident(field(fields, "kind", "relation")?, "relation kind")?,
        parse_entity(field(fields, "object", "relation")?, "relation object")?,
        decode_span(field(fields, "source", "relation")?)?,
    ))
}

fn decode_binding(value: &CanonicalValue) -> Result<Binding, Diagnostic> {
    let fields = object(value, "binding")?;
    match text(field(fields, "kind", "binding")?, "binding kind")? {
        "cell" => {
            exact_fields(fields, &["cell", "kind"], "cell binding")?;
            Ok(Binding::Cell(decode_cell(field(
                fields,
                "cell",
                "cell binding",
            )?)?))
        }
        "face" => {
            exact_fields(fields, &["cell", "direction", "kind"], "face binding")?;
            let direction = Direction::parse(text(
                field(fields, "direction", "face binding")?,
                "face direction",
            )?)
            .ok_or_else(|| invalid("face binding has an unsupported direction"))?;
            Ok(Binding::Face {
                cell: decode_cell(field(fields, "cell", "face binding")?)?,
                direction,
            })
        }
        "region" => {
            exact_fields(fields, &["kind", "max", "min"], "region binding")?;
            let min = decode_cell(field(fields, "min", "region binding")?)?;
            let max = decode_cell(field(fields, "max", "region binding")?)?;
            require_region(min, max, "region binding")?;
            Ok(Binding::Region { min, max })
        }
        kind => Err(invalid(format!("unsupported binding kind `{kind}`"))),
    }
}

fn decode_cell(value: &CanonicalValue) -> Result<Cell, Diagnostic> {
    let fields = object(value, "cell")?;
    exact_fields(fields, &["x", "y", "z"], "cell")?;
    Ok(Cell::new(
        signed_i32(field(fields, "x", "cell")?, "cell x")?,
        signed_i32(field(fields, "y", "cell")?, "cell y")?,
        signed_i32(field(fields, "z", "cell")?, "cell z")?,
    ))
}

fn require_region(min: Cell, max: Cell, context: &str) -> Result<(), Diagnostic> {
    if min.x() > max.x() || min.y() > max.y() || min.z() > max.z() {
        Err(invalid(format!("{context} has inverted bounds")))
    } else {
        Ok(())
    }
}

fn decode_movement_plan(value: &CanonicalValue) -> Result<MovementResolverPlan, Diagnostic> {
    let fields = object(value, "movement resolver")?;
    exact_fields(
        fields,
        &["coherence", "laws", "subjects"],
        "movement resolver",
    )?;
    let laws = array(field(fields, "laws", "movement resolver")?, "movement laws")?
        .iter()
        .map(|value| {
            let fields = object(value, "movement law")?;
            exact_fields(fields, &["operation"], "movement law")?;
            match text(
                field(fields, "operation", "movement law")?,
                "movement law operation",
            )? {
                "any_active_blocker" => Ok(MovementCompositionLaw::AnyActiveBlocker),
                "maximum_active_cost" => Ok(MovementCompositionLaw::MaximumActiveCost),
                operation => Err(invalid(format!("unsupported movement law `{operation}`"))),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let coherence = array(
        field(fields, "coherence", "movement resolver")?,
        "movement coherence",
    )?
    .iter()
    .map(|value| {
        let fields = object(value, "movement coherence")?;
        exact_fields(
            fields,
            &[
                "base_cost",
                "blockers_before_cost",
                "channel",
                "requires_connectivity",
            ],
            "movement coherence",
        )?;
        require_bool(
            field(fields, "blockers_before_cost", "movement coherence")?,
            true,
            "blockers_before_cost",
        )?;
        rebuild(
            GroundMovementCoherence::new(
                parse_ident(
                    field(fields, "channel", "movement coherence")?,
                    "movement channel",
                )?,
                unsigned_u32(
                    field(fields, "base_cost", "movement coherence")?,
                    "movement base cost",
                )?,
                boolean(
                    field(fields, "requires_connectivity", "movement coherence")?,
                    "requires_connectivity",
                )?,
            ),
            "movement coherence",
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    let subjects = array(
        field(fields, "subjects", "movement resolver")?,
        "movement subjects",
    )?
    .iter()
    .map(|value| {
        let fields = object(value, "movement subject")?;
        exact_fields(
            fields,
            &["claims", "connectivity", "entity"],
            "movement subject",
        )?;
        let claims = array(
            field(fields, "claims", "movement subject")?,
            "movement claims",
        )?
        .iter()
        .map(|value| parse_claim_ref(value, "movement claim"))
        .collect::<Result<Vec<_>, _>>()?;
        rebuild(
            MovementResolverSubject::new(
                parse_entity(
                    field(fields, "entity", "movement subject")?,
                    "movement subject entity",
                )?,
                decode_connectivity(field(fields, "connectivity", "movement subject")?)?,
                claims,
            ),
            "movement subject",
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        MovementResolverPlan::new(laws, coherence, subjects),
        "movement resolver",
    )
}

fn decode_connectivity(value: &CanonicalValue) -> Result<GroundConnectivity, Diagnostic> {
    let fields = object(value, "ground connectivity")?;
    match text(
        field(fields, "kind", "ground connectivity")?,
        "ground connectivity kind",
    )? {
        "face_adjacent" => {
            exact_fields(fields, &["first", "kind", "second"], "face connectivity")?;
            Ok(GroundConnectivity::FaceAdjacent {
                first: decode_cell(field(fields, "first", "face connectivity")?)?,
                second: decode_cell(field(fields, "second", "face connectivity")?)?,
            })
        }
        "region" => {
            exact_fields(fields, &["kind", "max", "min"], "region connectivity")?;
            let min = decode_cell(field(fields, "min", "region connectivity")?)?;
            let max = decode_cell(field(fields, "max", "region connectivity")?)?;
            require_region(min, max, "region connectivity")?;
            Ok(GroundConnectivity::Region { min, max })
        }
        kind => Err(invalid(format!(
            "unsupported ground connectivity kind `{kind}`"
        ))),
    }
}

fn decode_light_plan(value: &CanonicalValue) -> Result<LightResolverPlan, Diagnostic> {
    let fields = object(value, "light resolver")?;
    exact_fields(fields, &["consumers", "law", "subjects"], "light resolver")?;
    let law = object(field(fields, "law", "light resolver")?, "light law")?;
    exact_fields(law, &["operation"], "light law")?;
    require_text(field(law, "operation", "light law")?, "union", "light law")?;
    let consumers = array(
        field(fields, "consumers", "light resolver")?,
        "light consumers",
    )?
    .iter()
    .map(decode_consumer)
    .collect::<Result<Vec<_>, _>>()?;
    let subjects = array(
        field(fields, "subjects", "light resolver")?,
        "light subjects",
    )?
    .iter()
    .map(|value| {
        let fields = object(value, "light subject")?;
        exact_fields(fields, &["claims", "entity"], "light subject")?;
        let claims = array(field(fields, "claims", "light subject")?, "light claims")?
            .iter()
            .map(|value| parse_claim_ref(value, "light claim"))
            .collect::<Result<Vec<_>, _>>()?;
        rebuild(
            LightResolverSubject::new(
                parse_entity(
                    field(fields, "entity", "light subject")?,
                    "light subject entity",
                )?,
                claims,
            ),
            "light subject",
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        LightResolverPlan::new(LightCompositionLaw::Union, consumers, subjects),
        "light resolver",
    )
}

fn decode_stable_movement(value: &CanonicalValue) -> Result<StableGroundMovementV1, Diagnostic> {
    let fields = object(value, "stable movement row")?;
    exact_fields(
        fields,
        &["blocked_ground", "entity", "traversal_cost_ground"],
        "stable movement row",
    )?;
    let cost = match field(fields, "traversal_cost_ground", "stable movement row")? {
        CanonicalValue::Null => None,
        value => Some(unsigned_u32(value, "stable traversal cost")?),
    };
    rebuild(
        StableGroundMovementV1::new(
            parse_entity(
                field(fields, "entity", "stable movement row")?,
                "stable movement entity",
            )?,
            boolean(
                field(fields, "blocked_ground", "stable movement row")?,
                "blocked_ground",
            )?,
            cost,
        ),
        "stable movement row",
    )
}

fn object<'a>(
    value: &'a CanonicalValue,
    context: &str,
) -> Result<&'a BTreeMap<FieldName, CanonicalValue>, Diagnostic> {
    match value {
        CanonicalValue::Object(fields) => Ok(fields),
        _ => Err(invalid(format!("{context} must be an object"))),
    }
}

fn array<'a>(value: &'a CanonicalValue, context: &str) -> Result<&'a [CanonicalValue], Diagnostic> {
    match value {
        CanonicalValue::Array(values) => Ok(values),
        _ => Err(invalid(format!("{context} must be an array"))),
    }
}

fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    context: &str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid(format!("{context} is missing required field `{name}`")))
}

fn exact_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&'static str],
    context: &str,
) -> Result<(), Diagnostic> {
    for name in expected {
        if !fields.contains_key(&FieldName::declared(name)) {
            return Err(invalid(format!(
                "{context} is missing required field `{name}`"
            )));
        }
    }
    if let Some(name) = fields
        .keys()
        .find(|name| !expected.contains(&name.as_str()))
    {
        return Err(invalid(format!(
            "{context} contains unsupported field `{name}`"
        )));
    }
    Ok(())
}

fn text<'a>(value: &'a CanonicalValue, context: &str) -> Result<&'a str, Diagnostic> {
    match value {
        CanonicalValue::Text(value) => Ok(value),
        _ => Err(invalid(format!("{context} must be text"))),
    }
}

fn boolean(value: &CanonicalValue, context: &str) -> Result<bool, Diagnostic> {
    match value {
        CanonicalValue::Bool(value) => Ok(*value),
        _ => Err(invalid(format!("{context} must be a boolean"))),
    }
}

fn require_text(value: &CanonicalValue, expected: &str, context: &str) -> Result<(), Diagnostic> {
    let actual = text(value, context)?;
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "{context} must be `{expected}`, found `{actual}`"
        )))
    }
}

fn require_bool(value: &CanonicalValue, expected: bool, context: &str) -> Result<(), Diagnostic> {
    let actual = boolean(value, context)?;
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "{context} must be `{expected}`, found `{actual}`"
        )))
    }
}

fn unsigned_u32(value: &CanonicalValue, context: &str) -> Result<u32, Diagnostic> {
    match value {
        CanonicalValue::Uint(value) => u32::try_from(*value)
            .map_err(|_| invalid(format!("{context} does not fit a 32-bit unsigned integer"))),
        CanonicalValue::Int(value) => u32::try_from(*value)
            .map_err(|_| invalid(format!("{context} does not fit a 32-bit unsigned integer"))),
        _ => Err(invalid(format!("{context} must be an unsigned integer"))),
    }
}

fn signed_i32(value: &CanonicalValue, context: &str) -> Result<i32, Diagnostic> {
    match value {
        CanonicalValue::Int(value) => i32::try_from(*value)
            .map_err(|_| invalid(format!("{context} does not fit a 32-bit signed integer"))),
        _ => Err(invalid(format!("{context} must be a signed integer"))),
    }
}

fn decode_span(value: &CanonicalValue) -> Result<SourceSpan, Diagnostic> {
    let fields = object(value, "source span")?;
    exact_fields(
        fields,
        &["byte_end", "byte_start", "column", "line", "path"],
        "source span",
    )?;
    let path = rebuild(
        SourcePath::new(text(field(fields, "path", "source span")?, "source path")?),
        "source path",
    )?;
    rebuild(
        SourceSpan::new(
            path,
            unsigned_u32(
                field(fields, "byte_start", "source span")?,
                "span byte_start",
            )?,
            unsigned_u32(field(fields, "byte_end", "source span")?, "span byte_end")?,
            unsigned_u32(field(fields, "line", "source span")?, "span line")?,
            unsigned_u32(field(fields, "column", "source span")?, "span column")?,
        ),
        "source span",
    )
}

fn decode_schema(value: &CanonicalValue) -> Result<SchemaId, Diagnostic> {
    let fields = object(value, "schema identity")?;
    exact_fields(fields, &["name", "version"], "schema identity")?;
    rebuild(
        SchemaId::new(
            text(field(fields, "name", "schema identity")?, "schema name")?,
            unsigned_u32(
                field(fields, "version", "schema identity")?,
                "schema version",
            )?,
        ),
        "schema identity",
    )
}

fn require_schema(value: &CanonicalValue, expected: &SchemaId) -> Result<(), Diagnostic> {
    let actual = decode_schema(value)?;
    if &actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "expected schema `{expected}`, found `{actual}`"
        )))
    }
}

fn parse_catalog_value(
    value: &CanonicalValue,
    context: &str,
) -> Result<CatalogValueId, Diagnostic> {
    rebuild(CatalogValueId::parse(text(value, context)?), context)
}

fn parse_entity(value: &CanonicalValue, context: &str) -> Result<EntityId, Diagnostic> {
    rebuild(EntityId::parse(text(value, context)?), context)
}

fn parse_namespace(value: &CanonicalValue, context: &str) -> Result<NamespaceId, Diagnostic> {
    rebuild(NamespaceId::parse(text(value, context)?), context)
}

fn parse_ident(value: &CanonicalValue, context: &str) -> Result<Ident, Diagnostic> {
    rebuild(Ident::new(text(value, context)?), context)
}

fn parse_claim_ref(value: &CanonicalValue, context: &str) -> Result<ClaimRef, Diagnostic> {
    rebuild(ClaimRef::parse(text(value, context)?), context)
}

fn rebuild<T>(result: Result<T, Diagnostic>, context: &str) -> Result<T, Diagnostic> {
    result.map_err(|error| invalid(format!("{context} is invalid: {}", error.message())))
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(codes::PACKAGE_MEMBER_SCHEMA_INVALID, message)
        .with_repair(RepairClass::RebuildFromSource)
}

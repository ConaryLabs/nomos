//! Strict decoding for typed ownership and derivation evidence.

use nomos_core::{CanonicalValue, Diagnostic, PrimitiveKindId};

use crate::{
    DerivationInput, DerivationPass, DerivationProducer, DerivationStep, FactIdentity, FactOwner,
    FactOwnershipReceipt, ProjectionConsumer, ResolvedFactValue,
};

use super::{
    array, decode_binding, decode_span, exact_fields, field, invalid, object, parse_catalog_value,
    parse_entity, parse_ident, rebuild, text,
};

pub(super) fn decode_receipt(value: &CanonicalValue) -> Result<FactOwnershipReceipt, Diagnostic> {
    let fields = object(value, "ownership receipt")?;
    exact_fields(
        fields,
        &[
            "consumers",
            "declared_at",
            "derivation",
            "fact",
            "owner",
            "resolved_to",
        ],
        "ownership receipt",
    )?;
    let consumers = array(
        field(fields, "consumers", "ownership receipt")?,
        "receipt consumers",
    )?
    .iter()
    .map(decode_consumer)
    .collect::<Result<Vec<_>, _>>()?;
    let derivation = array(
        field(fields, "derivation", "ownership receipt")?,
        "derivation",
    )?
    .iter()
    .map(decode_derivation_step)
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        FactOwnershipReceipt::new(
            decode_fact(field(fields, "fact", "ownership receipt")?)?,
            decode_owner(field(fields, "owner", "ownership receipt")?)?,
            decode_span(field(fields, "declared_at", "ownership receipt")?)?,
            decode_resolved_fact(field(fields, "resolved_to", "ownership receipt")?)?,
            consumers,
            derivation,
        ),
        "ownership receipt",
    )
}

fn decode_fact(value: &CanonicalValue) -> Result<FactIdentity, Diagnostic> {
    let fields = object(value, "fact identity")?;
    let kind = text(field(fields, "kind", "fact identity")?, "fact kind")?;
    match kind {
        "entity_identity"
        | "entity_spatial_anchor"
        | "entity_spatial_binding"
        | "entity_credential" => {
            exact_fields(fields, &["entity", "kind"], "entity fact")?;
            let entity = parse_entity(field(fields, "entity", "entity fact")?, "fact entity")?;
            match kind {
                "entity_identity" => Ok(FactIdentity::EntityIdentity(entity)),
                "entity_spatial_anchor" => Ok(FactIdentity::EntitySpatialAnchor(entity)),
                "entity_spatial_binding" => Ok(FactIdentity::EntitySpatialBinding(entity)),
                "entity_credential" => Ok(FactIdentity::EntityCredential(entity)),
                _ => unreachable!(),
            }
        }
        "relation" => {
            exact_fields(
                fields,
                &["kind", "object", "relation", "subject"],
                "relation fact",
            )?;
            Ok(FactIdentity::Relation {
                subject: parse_entity(field(fields, "subject", "relation fact")?, "fact subject")?,
                kind: parse_ident(
                    field(fields, "relation", "relation fact")?,
                    "fact relation kind",
                )?,
                object: parse_entity(field(fields, "object", "relation fact")?, "fact object")?,
            })
        }
        _ => Err(invalid(format!("unsupported fact kind `{kind}`"))),
    }
}

fn decode_resolved_fact(value: &CanonicalValue) -> Result<ResolvedFactValue, Diagnostic> {
    let fields = object(value, "resolved fact value")?;
    let kind = text(
        field(fields, "kind", "resolved fact value")?,
        "resolved fact kind",
    )?;
    match kind {
        "entity" => {
            exact_fields(fields, &["entity", "kind"], "resolved entity")?;
            Ok(ResolvedFactValue::Entity(parse_entity(
                field(fields, "entity", "resolved entity")?,
                "resolved entity",
            )?))
        }
        "binding" => {
            exact_fields(fields, &["binding", "kind"], "resolved binding")?;
            Ok(ResolvedFactValue::Binding(decode_binding(field(
                fields,
                "binding",
                "resolved binding",
            )?)?))
        }
        "catalog_value" => {
            exact_fields(fields, &["kind", "value"], "resolved catalog value")?;
            Ok(ResolvedFactValue::CatalogValue(parse_catalog_value(
                field(fields, "value", "resolved catalog value")?,
                "resolved catalog value",
            )?))
        }
        "relation" => {
            exact_fields(
                fields,
                &["kind", "object", "relation", "subject"],
                "resolved relation",
            )?;
            Ok(ResolvedFactValue::Relation {
                subject: parse_entity(
                    field(fields, "subject", "resolved relation")?,
                    "resolved subject",
                )?,
                kind: parse_ident(
                    field(fields, "relation", "resolved relation")?,
                    "resolved relation kind",
                )?,
                object: parse_entity(
                    field(fields, "object", "resolved relation")?,
                    "resolved object",
                )?,
            })
        }
        _ => Err(invalid(format!("unsupported resolved fact kind `{kind}`"))),
    }
}

fn decode_derivation_step(value: &CanonicalValue) -> Result<DerivationStep, Diagnostic> {
    let fields = object(value, "derivation step")?;
    exact_fields(fields, &["inputs", "pass", "producer"], "derivation step")?;
    let producer = DerivationProducer::parse(text(
        field(fields, "producer", "derivation step")?,
        "derivation producer",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let pass = DerivationPass::parse(text(
        field(fields, "pass", "derivation step")?,
        "derivation pass",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let inputs = array(
        field(fields, "inputs", "derivation step")?,
        "derivation inputs",
    )?
    .iter()
    .map(decode_derivation_input)
    .collect::<Result<Vec<_>, _>>()?;
    rebuild(
        DerivationStep::new(producer, pass, inputs),
        "derivation step",
    )
}

fn decode_derivation_input(value: &CanonicalValue) -> Result<DerivationInput, Diagnostic> {
    let fields = object(value, "derivation input")?;
    match text(
        field(fields, "kind", "derivation input")?,
        "derivation input kind",
    )? {
        "fact" => {
            exact_fields(fields, &["fact", "kind"], "fact derivation input")?;
            Ok(DerivationInput::Fact(decode_fact(field(
                fields,
                "fact",
                "fact derivation input",
            )?)?))
        }
        "primitive" => {
            exact_fields(fields, &["kind", "primitive"], "primitive derivation input")?;
            Ok(DerivationInput::Primitive(
                PrimitiveKindId::parse(text(
                    field(fields, "primitive", "primitive derivation input")?,
                    "derivation primitive",
                )?)
                .map_err(|error| invalid(error.message()))?,
            ))
        }
        "catalog_value" => {
            exact_fields(fields, &["kind", "value"], "catalog derivation input")?;
            Ok(DerivationInput::CatalogValue(parse_catalog_value(
                field(fields, "value", "catalog derivation input")?,
                "derivation catalog value",
            )?))
        }
        kind => Err(invalid(format!(
            "unsupported derivation input kind `{kind}`"
        ))),
    }
}

fn decode_owner(value: &CanonicalValue) -> Result<FactOwner, Diagnostic> {
    match text(value, "fact owner")? {
        "lattice" => Ok(FactOwner::Lattice),
        "graph" => Ok(FactOwner::Graph),
        "world_linker" => Ok(FactOwner::WorldLinker),
        owner => Err(invalid(format!("unsupported fact owner `{owner}`"))),
    }
}

pub(super) fn decode_consumer(value: &CanonicalValue) -> Result<ProjectionConsumer, Diagnostic> {
    match text(value, "projection consumer")? {
        "diagnostics" => Ok(ProjectionConsumer::Diagnostics),
        "navigation" => Ok(ProjectionConsumer::Navigation),
        "persistence" => Ok(ProjectionConsumer::Persistence),
        "simulation" => Ok(ProjectionConsumer::Simulation),
        consumer => Err(invalid(format!(
            "unsupported projection consumer `{consumer}`"
        ))),
    }
}

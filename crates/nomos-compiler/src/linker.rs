//! Typed name resolution, primitive expansion, and fact-ownership linking.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::{CatalogValueId, Diagnostic, EntityId, Ident, RepairClass, SourceSpan};
use nomos_schema::{
    Binding, FactOwner, FactOwnershipReceipt, IrEntity, IrRelation, SourceDocument, SourceEntity,
    SourceField, Spanned, WorldIr,
};

use crate::catalog::{self, ApprovedKind};
use crate::diagnostics;

pub(crate) fn link(document: &SourceDocument) -> Result<WorldIr, Diagnostic> {
    if document.schema().value() != &nomos_schema::source_schema() {
        return Err(Diagnostic::new(
            diagnostics::SOURCE_SCHEMA_UNSUPPORTED,
            format!(
                "source schema `{}` is unsupported; expected `{}`",
                document.schema().value(),
                nomos_schema::source_schema()
            ),
        )
        .with_span(document.schema().span().clone())
        .with_repair(RepairClass::FixSourceSyntax));
    }
    if let Some(declaration) = document.forbidden_fact_owners().first() {
        return Err(Diagnostic::new(
            diagnostics::DUPLICATE_FACT_OWNER,
            format!(
                "content cannot assign `{}` to `{}`; canonical fact owners are compiler-defined",
                declaration.owner(),
                declaration.fact()
            ),
        )
        .with_span(declaration.span().clone())
        .with_repair(RepairClass::RestoreCanonicalFactOwner));
    }

    let catalog_values = declare_catalog_values(document)?;
    let entity_symbols = declare_entities(document)?;
    let (relations, mut receipts) = link_relations(document, &entity_symbols)?;
    let mut entities = Vec::new();
    for entity in document.entities() {
        let (linked, entity_receipts) = link_entity(entity, &catalog_values)?;
        entities.push(linked);
        receipts.extend(entity_receipts);
    }

    let movement_resolver = crate::resolver::construction_plan(&entities)?;
    Ok(WorldIr::new(
        document.schema().value().clone(),
        catalog_values.into_iter().collect(),
        entities,
        relations,
        receipts,
    )?
    .with_movement_resolver(movement_resolver))
}

fn declare_catalog_values(
    document: &SourceDocument,
) -> Result<BTreeSet<CatalogValueId>, Diagnostic> {
    let mut symbols = BTreeSet::new();
    for declaration in document.catalog_values() {
        if !symbols.insert(declaration.value().clone()) {
            return Err(Diagnostic::new(
                diagnostics::DUPLICATE_CATALOG_VALUE,
                format!(
                    "catalog value `{}` is declared more than once",
                    declaration.value()
                ),
            )
            .with_span(declaration.span().clone())
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(symbols)
}

fn declare_entities(
    document: &SourceDocument,
) -> Result<BTreeMap<EntityId, &SourceEntity>, Diagnostic> {
    let mut symbols = BTreeMap::new();
    for entity in document.entities() {
        if symbols
            .insert(entity.id().value().clone(), entity)
            .is_some()
        {
            return Err(Diagnostic::new(
                diagnostics::DUPLICATE_ENTITY,
                format!(
                    "entity `{}` is declared more than once",
                    entity.id().value()
                ),
            )
            .with_span(entity.id().span().clone())
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(symbols)
}

fn link_relations(
    document: &SourceDocument,
    entities: &BTreeMap<EntityId, &SourceEntity>,
) -> Result<(Vec<IrRelation>, Vec<FactOwnershipReceipt>), Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut linked = Vec::new();
    let mut receipts = Vec::new();
    for relation in document.relations() {
        if relation.kind().value().as_str() != "owns" {
            return Err(Diagnostic::new(
                diagnostics::UNAPPROVED_RELATION_KIND,
                format!(
                    "relation kind `{}` is not in the Gate K vocabulary",
                    relation.kind().value()
                ),
            )
            .with_span(relation.kind().span().clone())
            .with_repair(RepairClass::UseApprovedRelationKind));
        }
        for reference in [relation.subject(), relation.object()] {
            if !entities.contains_key(reference.value()) {
                return Err(Diagnostic::new(
                    diagnostics::DANGLING_ENTITY,
                    format!("entity reference `{}` does not resolve", reference.value()),
                )
                .with_span(reference.span().clone())
                .with_repair(RepairClass::DeclareReferencedEntity));
            }
        }
        let key = (
            relation.subject().value().clone(),
            relation.kind().value().clone(),
            relation.object().value().clone(),
        );
        if !seen.insert(key.clone()) {
            return Err(Diagnostic::new(
                diagnostics::DUPLICATE_RELATION,
                format!(
                    "relation `{} {} {}` is declared more than once",
                    key.0, key.1, key.2
                ),
            )
            .with_span(relation.span().clone())
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
        linked.push(IrRelation::new(
            key.0.clone(),
            key.1.clone(),
            key.2.clone(),
            relation.span().clone(),
        ));
        receipts.push(FactOwnershipReceipt::new(
            format!("relation.{}.{}.{}", key.0, key.1, key.2),
            FactOwner::Graph,
            relation.span().clone(),
            format!("{} {} {}", key.0, key.1, key.2),
            idents(&["diagnostics", "persistence", "simulation"]),
            vec!["source/relation".to_owned()],
        ));
    }
    Ok((linked, receipts))
}

struct Fields<'a> {
    anchor: Option<&'a Spanned<Binding>>,
    credential: Option<&'a Spanned<CatalogValueId>>,
}

fn collect_fields(entity: &SourceEntity) -> Result<Fields<'_>, Diagnostic> {
    let mut fields = Fields {
        anchor: None,
        credential: None,
    };
    for field in entity.fields() {
        match field {
            SourceField::Anchor(anchor) => {
                if fields.anchor.replace(anchor).is_some() {
                    return Err(duplicate_field("anchor", field.span()));
                }
            }
            SourceField::Credential(credential) => {
                if fields.credential.replace(credential).is_some() {
                    return Err(duplicate_field("credential", field.span()));
                }
            }
            SourceField::LatticeRelation {
                relation,
                target,
                span,
            } => {
                return Err(Diagnostic::new(
                    diagnostics::RELATION_IN_LATTICE,
                    format!(
                        "relation `{} {}` cannot be encoded as an entity/lattice property",
                        relation.value(),
                        target.value()
                    ),
                )
                .with_span(span.clone())
                .with_repair(RepairClass::MoveRelationToGraph));
            }
            SourceField::RawTransform(span) => {
                return Err(Diagnostic::new(
                    diagnostics::RAW_TRANSFORM_AUTHORED,
                    "raw transforms are absent from shippable source; use a typed lattice anchor",
                )
                .with_span(span.clone())
                .with_repair(RepairClass::ReplaceRawTransformWithBinding));
            }
            SourceField::DerivedFact { name, span } => {
                return Err(Diagnostic::new(
                    diagnostics::DERIVED_FACT_AUTHORED,
                    format!("derived fact `{name}` belongs to the compiler, not content"),
                )
                .with_span(span.clone())
                .with_repair(RepairClass::RemoveDerivedFact));
            }
        }
    }
    Ok(fields)
}

fn duplicate_field(name: &str, span: &SourceSpan) -> Diagnostic {
    Diagnostic::new(
        diagnostics::DUPLICATE_FIELD,
        format!("field `{name}` is supplied more than once"),
    )
    .with_span(span.clone())
    .with_repair(RepairClass::RemoveDuplicateDeclaration)
}

fn link_entity(
    source: &SourceEntity,
    catalog_values: &BTreeSet<CatalogValueId>,
) -> Result<(IrEntity, Vec<FactOwnershipReceipt>), Diagnostic> {
    let kind = catalog::lookup(source.primitive().value()).ok_or_else(|| {
        Diagnostic::new(
            diagnostics::UNAPPROVED_PRIMITIVE,
            format!(
                "primitive kind `{}` is not in the sealed Gate K catalog",
                source.primitive().value()
            ),
        )
        .with_span(source.primitive().span().clone())
        .with_repair(RepairClass::UseApprovedPrimitive)
    })?;
    let fields = collect_fields(source)?;
    let anchor = fields.anchor.ok_or_else(|| {
        Diagnostic::new(
            diagnostics::REQUIRED_FIELD_MISSING,
            format!(
                "entity `{}` requires one `anchor` field",
                source.id().value()
            ),
        )
        .with_span(source.span().clone())
        .with_repair(RepairClass::SupplyRequiredField)
    })?;
    validate_region(anchor)?;
    validate_shape(kind, source, &fields)?;

    if let Some(credential) = fields.credential {
        if credential.value().catalog().as_str() != "credential" {
            return Err(Diagnostic::new(
                diagnostics::CATALOG_NAMESPACE_MISMATCH,
                format!(
                    "field `credential` requires the `credential` catalog, not `{}`",
                    credential.value().catalog()
                ),
            )
            .with_span(credential.span().clone())
            .with_repair(RepairClass::UseExpectedCatalogNamespace));
        }
        if !catalog_values.contains(credential.value()) {
            return Err(Diagnostic::new(
                diagnostics::DANGLING_CATALOG_VALUE,
                format!("catalog value `{}` does not resolve", credential.value()),
            )
            .with_span(credential.span().clone())
            .with_repair(RepairClass::DeclareReferencedCatalogValue));
        }
    }

    let id = source.id().value().clone();
    let expansion = catalog::expand(kind, &id)?;
    let credential = fields.credential.map(|value| value.value().clone());
    let receipts = entity_receipts(source, anchor, fields.credential);
    Ok((
        IrEntity::new(
            id,
            source.primitive().value().clone(),
            anchor.value().clone(),
            credential,
            expansion,
            source.span().clone(),
        ),
        receipts,
    ))
}

fn validate_shape(
    kind: ApprovedKind,
    entity: &SourceEntity,
    fields: &Fields<'_>,
) -> Result<(), Diagnostic> {
    let anchor = fields.anchor.expect("the required anchor was checked");
    let expected = match kind {
        ApprovedKind::Door => "face",
        ApprovedKind::Water => "region",
        ApprovedKind::Light => "cell",
    };
    if anchor.value().kind() != expected {
        return Err(Diagnostic::new(
            diagnostics::FIELD_NOT_ALLOWED,
            format!(
                "primitive `{}` requires a `{expected}` anchor, not `{}`",
                entity.primitive().value(),
                anchor.value().kind()
            ),
        )
        .with_span(anchor.span().clone())
        .with_repair(RepairClass::RemoveUnsupportedField));
    }
    match (kind, fields.credential) {
        (ApprovedKind::Door, None) => Err(Diagnostic::new(
            diagnostics::REQUIRED_FIELD_MISSING,
            format!(
                "door `{}` requires one `credential` field",
                entity.id().value()
            ),
        )
        .with_span(entity.span().clone())
        .with_repair(RepairClass::SupplyRequiredField)),
        (ApprovedKind::Water | ApprovedKind::Light, Some(credential)) => Err(Diagnostic::new(
            diagnostics::FIELD_NOT_ALLOWED,
            format!(
                "primitive `{}` does not accept a `credential` field",
                entity.primitive().value()
            ),
        )
        .with_span(credential.span().clone())
        .with_repair(RepairClass::RemoveUnsupportedField)),
        _ => Ok(()),
    }
}

fn validate_region(anchor: &Spanned<Binding>) -> Result<(), Diagnostic> {
    if let Binding::Region { min, max } = anchor.value()
        && (min.x() > max.x() || min.y() > max.y() || min.z() > max.z())
    {
        return Err(Diagnostic::new(
            diagnostics::REGION_BOUNDS_INVALID,
            "region minimum must be component-wise less than or equal to its maximum",
        )
        .with_span(anchor.span().clone())
        .with_repair(RepairClass::FixSourceSyntax));
    }
    Ok(())
}

fn entity_receipts(
    entity: &SourceEntity,
    anchor: &Spanned<Binding>,
    credential: Option<&Spanned<CatalogValueId>>,
) -> Vec<FactOwnershipReceipt> {
    let id = entity.id().value();
    let primitive = entity.primitive().value();
    let resolved = binding_string(anchor.value());
    let mut receipts = vec![
        FactOwnershipReceipt::new(
            format!("entity.{id}.identity"),
            FactOwner::Graph,
            entity.id().span().clone(),
            id.to_string(),
            idents(&["diagnostics", "persistence", "simulation"]),
            vec!["source/entity".to_owned()],
        ),
        FactOwnershipReceipt::new(
            format!("entity.{id}.spatial_anchor"),
            FactOwner::Lattice,
            anchor.span().clone(),
            resolved.clone(),
            idents(&["diagnostics", "navigation", "simulation"]),
            vec!["source/anchor".to_owned()],
        ),
        FactOwnershipReceipt::new(
            format!("entity.{id}.spatial_binding"),
            FactOwner::WorldLinker,
            anchor.span().clone(),
            resolved,
            idents(&["diagnostics", "navigation", "persistence", "simulation"]),
            vec![primitive.to_string(), "binding/typed_lattice".to_owned()],
        ),
    ];
    if let Some(value) = credential {
        receipts.push(FactOwnershipReceipt::new(
            format!("entity.{id}.credential"),
            FactOwner::WorldLinker,
            value.span().clone(),
            value.value().to_string(),
            idents(&["diagnostics", "persistence", "simulation"]),
            vec![primitive.to_string(), "link/catalog_value".to_owned()],
        ));
    }
    receipts
}

fn binding_string(binding: &Binding) -> String {
    match binding {
        Binding::Cell(cell) => format!("cell({},{},{})", cell.x(), cell.y(), cell.z()),
        Binding::Face { cell, direction } => format!(
            "face(cell({},{},{}),{})",
            cell.x(),
            cell.y(),
            cell.z(),
            direction.as_str()
        ),
        Binding::Region { min, max } => format!(
            "region(cell({},{},{}),cell({},{},{}))",
            min.x(),
            min.y(),
            min.z(),
            max.x(),
            max.y(),
            max.z()
        ),
    }
}

fn idents(values: &[&str]) -> Vec<Ident> {
    values
        .iter()
        .map(|value| Ident::new(value).expect("built-in consumer names are legal identifiers"))
        .collect()
}

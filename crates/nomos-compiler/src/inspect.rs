//! Compiler-owned inspection of verified compiled-world packages.

use std::collections::BTreeMap;
use std::path::Path;

use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, FieldName, Ident, NamespaceId, PrimitiveKindId,
    RepairClass, SourcePath,
};

use crate::{OpenedCompiledWorld, open_compiled_package};

/// Opens and fully validates a compiled-world package, then returns the flat
/// deterministic report used by `nomos inspect`.
///
/// The returned value is deliberately self-contained. Callers serialize it as
/// an opaque canonical value and never need to name or decode World IR fields.
///
/// # Errors
///
/// Returns the first package integrity, semantic-package, or inspection-shape
/// diagnostic. No report is returned for partial or disagreeing evidence.
pub fn inspect_compiled_package(root: &Path) -> Result<CanonicalValue, Diagnostic> {
    let opened = open_compiled_package(root)?;
    inspection_report(&opened)
}

fn inspection_report(opened: &OpenedCompiledWorld) -> Result<CanonicalValue, Diagnostic> {
    // The report is derived only after complete typed rehydration and exact
    // reprojection. Rendering the authoritative type back to its canonical
    // value preserves the complete nested machine and claim shapes without
    // creating another persisted-package decoder.
    let stable_ir = opened.stable_ir().to_canonical();
    let world = object(&stable_ir, "World IR")?;
    let entities = array(field(world, "entities")?, "World IR entities")?;
    let mut previous_id: Option<String> = None;
    let inspected = entities
        .iter()
        .map(|entity| inspect_entity(entity, &mut previous_id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CanonicalValue::object_declared([
        ("command", CanonicalValue::text("inspect")),
        (
            "compiler_version",
            field(world, "compiler_version")?.clone(),
        ),
        (
            "construction_schema",
            field(world, "construction_schema")?.clone(),
        ),
        ("entities", CanonicalValue::Array(inspected)),
        (
            "manifest_digest",
            CanonicalValue::text(opened.package_digest().to_hex()),
        ),
        ("package_schema", opened.manifest().schema().to_canonical()),
        (
            "primitive_catalog_version",
            field(world, "primitive_catalog_version")?.clone(),
        ),
        ("status", CanonicalValue::text("completed")),
        ("world_ir_schema", field(world, "schema")?.clone()),
    ]))
}

fn inspect_entity(
    value: &CanonicalValue,
    previous_id: &mut Option<String>,
) -> Result<CanonicalValue, Diagnostic> {
    let fields = object(value, "World IR entity")?;
    require_exact_fields(
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
    let id = text(field(fields, "id")?, "entity id")?;
    let entity_id = EntityId::parse(id).map_err(|error| invalid(error.message()))?;
    if previous_id
        .as_ref()
        .is_some_and(|previous| previous.as_str() >= id)
    {
        return Err(invalid(
            "World IR entities are not in strict stable-ID order",
        ));
    }
    *previous_id = Some(id.to_owned());

    let primitive = text(field(fields, "primitive")?, "entity primitive")?;
    let primitive_id =
        PrimitiveKindId::parse(primitive).map_err(|error| invalid(error.message()))?;
    if crate::catalog::lookup(&primitive_id).is_none() {
        return Err(invalid(format!(
            "entity `{id}` names unsupported primitive `{primitive}`"
        )));
    }
    let source = object(field(fields, "source")?, "entity source mapping")?;
    require_exact_fields(
        source,
        &["byte_end", "byte_start", "column", "line", "path"],
        "entity source mapping",
    )?;
    validate_source_mapping(source)?;
    let expansion = object(field(fields, "expansion")?, "primitive expansion")?;
    require_exact_fields(
        expansion,
        &["capabilities", "claims", "interactions", "machines"],
        "primitive expansion",
    )?;

    let capabilities = text_array(field(expansion, "capabilities")?, "capabilities")?;
    let machines = ordered_object_array(
        field(expansion, "machines")?,
        "machines",
        "namespace",
        &["initial", "namespace", "states", "transitions"],
    )?;
    for machine in &machines {
        let machine = object(machine, "machine")?;
        let namespace = NamespaceId::parse(text(field(machine, "namespace")?, "namespace")?)
            .map_err(|error| invalid(error.message()))?;
        if namespace.entity() != &entity_id {
            return Err(invalid(format!(
                "machine `{namespace}` does not belong to entity `{id}`"
            )));
        }
        let states = text_array(field(machine, "states")?, "machine states")?;
        let mut state_names = std::collections::BTreeSet::new();
        for state in &states {
            let name = Ident::new(text(state, "machine state")?)
                .map_err(|error| invalid(error.message()))?;
            if !state_names.insert(name) {
                return Err(invalid(format!("machine `{namespace}` repeats a state")));
            }
        }
        array(field(machine, "transitions")?, "machine transitions")?;
        let initial = Ident::new(text(field(machine, "initial")?, "machine initial state")?)
            .map_err(|error| invalid(error.message()))?;
        if !state_names.contains(&initial) {
            return Err(invalid(format!(
                "machine `{namespace}` initial state is absent from its states"
            )));
        }
    }
    let claims = ordered_object_array(
        field(expansion, "claims")?,
        "claims",
        "id",
        &["activation", "capability", "id", "value"],
    )?;
    for claim in &claims {
        let claim = object(claim, "claim")?;
        let claim_id = ClaimRef::parse(text(field(claim, "id")?, "claim id")?)
            .map_err(|error| invalid(error.message()))?;
        if claim_id.namespace().entity() != &entity_id {
            return Err(invalid(format!(
                "claim `{claim_id}` does not belong to entity `{id}`"
            )));
        }
        let capability = text(field(claim, "capability")?, "claim capability")?;
        if claim_id.capability().as_str() != capability {
            return Err(invalid(format!(
                "claim `{claim_id}` capability disagrees with `{capability}`"
            )));
        }
    }

    Ok(CanonicalValue::object_declared([
        ("capabilities", CanonicalValue::Array(capabilities)),
        ("claims", CanonicalValue::Array(claims)),
        ("id", CanonicalValue::text(id)),
        ("machines", CanonicalValue::Array(machines)),
        ("primitive", CanonicalValue::text(primitive)),
        ("source", CanonicalValue::Object(source.clone())),
    ]))
}

fn ordered_object_array(
    value: &CanonicalValue,
    label: &str,
    identity: &'static str,
    expected_fields: &[&str],
) -> Result<Vec<CanonicalValue>, Diagnostic> {
    let rows = array(value, label)?;
    let mut previous: Option<String> = None;
    let mut inspected = Vec::with_capacity(rows.len());
    for row in rows {
        let fields = object(row, label)?;
        require_exact_fields(fields, expected_fields, label)?;
        let current = text(field(fields, identity)?, label)?;
        if previous
            .as_ref()
            .is_some_and(|previous| previous.as_str() >= current)
        {
            return Err(invalid(format!(
                "{label} are not in strict `{identity}` order"
            )));
        }
        previous = Some(current.to_owned());
        inspected.push(CanonicalValue::Object(fields.clone()));
    }
    Ok(inspected)
}

fn text_array(value: &CanonicalValue, label: &str) -> Result<Vec<CanonicalValue>, Diagnostic> {
    let rows = array(value, label)?;
    for row in rows {
        text(row, label)?;
    }
    Ok(rows.to_vec())
}

fn validate_source_mapping(fields: &BTreeMap<FieldName, CanonicalValue>) -> Result<(), Diagnostic> {
    SourcePath::new(text(field(fields, "path")?, "source path")?)
        .map_err(|error| invalid(error.message()))?;
    let start = unsigned(field(fields, "byte_start")?, "source byte start")?;
    let end = unsigned(field(fields, "byte_end")?, "source byte end")?;
    let line = unsigned(field(fields, "line")?, "source line")?;
    let column = unsigned(field(fields, "column")?, "source column")?;
    if start > end || line == 0 || column == 0 {
        return Err(invalid("entity source mapping is not a valid source span"));
    }
    Ok(())
}

fn unsigned(value: &CanonicalValue, label: &str) -> Result<u64, Diagnostic> {
    match value {
        CanonicalValue::Uint(value) => Ok(*value),
        CanonicalValue::Int(value) => {
            u64::try_from(*value).map_err(|_| invalid(format!("{label} is negative")))
        }
        _ => Err(invalid(format!("{label} is not an integer"))),
    }
}

fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid(format!("inspection input has no `{name}` field")))
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
    let CanonicalValue::Array(rows) = value else {
        return Err(invalid(format!("{label} is not an array")));
    };
    Ok(rows)
}

fn text<'a>(value: &'a CanonicalValue, label: &str) -> Result<&'a str, Diagnostic> {
    let CanonicalValue::Text(value) = value else {
        return Err(invalid(format!("{label} is not text")));
    };
    Ok(value)
}

fn require_exact_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    label: &str,
) -> Result<(), Diagnostic> {
    let actual = fields.keys().map(FieldName::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual != expected {
        return Err(invalid(format!(
            "{label} fields are {actual:?}; expected {expected:?}"
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_SCHEMA_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

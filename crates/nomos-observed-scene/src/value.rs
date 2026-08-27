//! Shared strict accessors for the two closed canonical object grammars.

use std::collections::BTreeMap;

use nomos_core::{CanonicalValue, FieldName, RepairClass};

use crate::diagnostic::{ObservedError, ObservedResult, codes};

pub type Object = BTreeMap<FieldName, CanonicalValue>;

pub fn object<'a>(value: &'a CanonicalValue, path: &str) -> ObservedResult<&'a Object> {
    match value {
        CanonicalValue::Object(fields) => Ok(fields),
        _ => Err(field_error(format!("`{path}` must be an object"))),
    }
}

pub fn exact_fields(fields: &Object, expected: &[&str], path: &str) -> ObservedResult<()> {
    let actual: Vec<&str> = fields.keys().map(FieldName::as_str).collect();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual == expected {
        return Ok(());
    }
    let missing: Vec<&&str> = expected
        .iter()
        .filter(|name| !actual.contains(name))
        .collect();
    let extra: Vec<&&str> = actual
        .iter()
        .filter(|name| !expected.contains(name))
        .collect();
    let mut error = field_error(format!(
        "`{path}` has the wrong field set; missing {missing:?}, extra {extra:?}"
    ));
    if !missing.is_empty() {
        error = error.with_repair(RepairClass::SupplyMissingMember);
    }
    if !extra.is_empty() {
        error = error.with_repair(RepairClass::RemoveUnsupportedField);
    }
    Err(error)
}

pub fn field<'a>(fields: &'a Object, name: &str, path: &str) -> ObservedResult<&'a CanonicalValue> {
    fields
        .iter()
        .find_map(|(field, value)| (field.as_str() == name).then_some(value))
        .ok_or_else(|| {
            field_error(format!("`{path}.{name}` is required"))
                .with_repair(RepairClass::SupplyMissingMember)
        })
}

pub fn text<'a>(value: &'a CanonicalValue, path: &str) -> ObservedResult<&'a str> {
    match value {
        CanonicalValue::Text(text) => Ok(text),
        _ => Err(field_error(format!("`{path}` must be a string"))),
    }
}

pub fn boolean(value: &CanonicalValue, path: &str) -> ObservedResult<bool> {
    match value {
        CanonicalValue::Bool(value) => Ok(*value),
        _ => Err(field_error(format!("`{path}` must be a boolean"))),
    }
}

pub fn integer(value: &CanonicalValue, path: &str) -> ObservedResult<i64> {
    match value {
        CanonicalValue::Int(value) => Ok(*value),
        CanonicalValue::Uint(value) => i64::try_from(*value).map_err(|_| {
            ObservedError::new(
                codes::BOUND_INVALID,
                format!("`{path}` is outside the supported integer range"),
            )
            .with_repair(RepairClass::ReduceOperandMagnitude)
        }),
        _ => Err(field_error(format!("`{path}` must be an integer"))),
    }
}

pub fn array<'a>(value: &'a CanonicalValue, path: &str) -> ObservedResult<&'a [CanonicalValue]> {
    match value {
        CanonicalValue::Array(items) => Ok(items),
        _ => Err(field_error(format!("`{path}` must be an array"))),
    }
}

pub fn field_error(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::FIELD_INVALID, message)
}

pub fn bound_error(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::BOUND_INVALID, message)
        .with_repair(RepairClass::ReduceOperandMagnitude)
}

pub fn enum_error(path: &str, expected: &str) -> ObservedError {
    field_error(format!("`{path}` must be one of {expected}"))
        .with_repair(RepairClass::RemoveUnsupportedField)
}

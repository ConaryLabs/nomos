//! Reading kernel documents, and binding their schema identities.
//!
//! Every input except `area.json` is canonical bytes produced by the kernel, so
//! this module hands them to `nomos_core::canonical::read::parse_canonical`
//! rather than to a reader of its own. That reader accepts bytes only if they
//! are exactly what `CanonicalValue::to_canonical_bytes` would have produced,
//! which is a stronger check than "parses as JSON": an input whose keys drifted
//! out of order, or that grew insignificant whitespace on its way here, is
//! refused.
//!
//! One allowance: `nomos effective-facts` writes its document to stdout with a
//! trailing newline, while package and run-bundle members carry none. A single
//! trailing `LF` is therefore stripped before the strict reader sees the bytes,
//! and nothing else is.

use std::collections::BTreeMap;
use std::path::Path;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::id::SchemaId;
use nomos_core::{CanonicalValue, FieldName};

use crate::error::{PlanError, PlanResult, codes};

/// Reads one canonical kernel document.
///
/// # Errors
///
/// Returns `RP0101` when the file cannot be read and `RP0102` when its bytes
/// are not canonical.
pub fn read_document(path: &Path) -> PlanResult<CanonicalValue> {
    let bytes = std::fs::read(path)
        .map_err(|error| PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(path))?;
    let body = match bytes.split_last() {
        Some((b'\n', rest)) => rest,
        _ => &bytes[..],
    };
    parse_canonical(body).map_err(|diagnostic| {
        PlanError::new(
            codes::INPUT_NOT_CANONICAL,
            format!("input is not canonical bytes: {diagnostic}"),
        )
        .at(path)
    })
}

/// Refuses a kernel document that reports a rejection.
///
/// `nomos` commands write one document to stdout either way: a completed
/// command carries `"status": "completed"` beside its payload, and a rejection
/// carries `"status": "rejected"` with a `diagnostics` array and no `schema`
/// field at all. Binding the identity first would report "no `schema` field"
/// and lose the kernel's own reason, so the status is checked first and the
/// kernel's diagnostic codes are carried through.
///
/// A document with no `status` field — a package member, a run-bundle member —
/// passes: the field is a property of a command's stdout, not of every
/// canonical document.
///
/// # Errors
///
/// Returns `RP0105` naming the reported status and every diagnostic code the
/// document carries.
pub fn require_completed(document: &CanonicalValue, path: &Path) -> PlanResult<()> {
    let Some(status) = document.get("status").and_then(CanonicalValue::as_text) else {
        return Ok(());
    };
    if status == "completed" {
        return Ok(());
    }
    let codes: Vec<&str> = document
        .get("diagnostics")
        .and_then(CanonicalValue::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|diagnostic| diagnostic.get("code").and_then(CanonicalValue::as_text))
        .collect();
    let reported = if codes.is_empty() {
        String::new()
    } else {
        format!(" ({})", codes.join(", "))
    };
    Err(PlanError::new(
        codes::DOCUMENT_SHAPE,
        format!("input document reports status `{status}`, not `completed`{reported}"),
    )
    .at(path))
}

/// Binds a document's `schema` field to an expected identity and version.
///
/// Two spellings are accepted, and both are compared as `name@version`:
///
/// - the object `{"name": "...", "version": N}` that `SchemaId::to_canonical`
///   produces, which is what every Gate K artifact and `nomos effective-facts`
///   writes;
/// - the string `"name@version"`, which is how the entity-catalog document in
///   issue #138 spells its own identity.
///
/// Accepting both is deliberate: R1-2 must bind #138's document before #138
/// lands, and the discrepancy between its example and the kernel convention is
/// recorded in `docs/review/rendering-plan-compiler.md` rather than guessed at.
/// A mismatch fails closed, naming both sides.
///
/// # Errors
///
/// Returns `RP0104` when the field is absent, has neither accepted shape, or
/// names a different identity or version.
pub fn bind_schema(document: &CanonicalValue, expected: &SchemaId, path: &Path) -> PlanResult<()> {
    let mismatch = |found: String| {
        PlanError::new(
            codes::SCHEMA_MISMATCH,
            format!("expected schema `{expected}`, found {found}"),
        )
        .at(path)
    };
    let found = document
        .get("schema")
        .ok_or_else(|| mismatch("no `schema` field".to_owned()))?;
    let spelled = match found {
        CanonicalValue::Text(text) => text.clone(),
        _ => {
            let name = found.get("name").and_then(CanonicalValue::as_text);
            let version = found.get("version").and_then(CanonicalValue::as_uint);
            match (name, version) {
                (Some(name), Some(version)) => format!("{name}@{version}"),
                _ => {
                    return Err(mismatch(
                        "a `schema` field that is neither `name@version` nor `{name, version}`"
                            .to_owned(),
                    ));
                }
            }
        }
    };
    if spelled != expected.to_string() {
        return Err(mismatch(format!("`{spelled}`")));
    }
    Ok(())
}

/// Canonical-value accessors that fail with a document-shape diagnostic.
pub trait Shape {
    /// One object field, or `None`.
    fn get(&self, name: &str) -> Option<&CanonicalValue>;
    /// The object fields, or `None`.
    fn as_object(&self) -> Option<&BTreeMap<FieldName, CanonicalValue>>;
    /// The array items, or `None`.
    fn as_array(&self) -> Option<&[CanonicalValue]>;
    /// The string, or `None`.
    fn as_text(&self) -> Option<&str>;
    /// The unsigned integer, or `None`.
    fn as_uint(&self) -> Option<u64>;
    /// The boolean, or `None`.
    fn as_bool(&self) -> Option<bool>;
}

impl Shape for CanonicalValue {
    fn get(&self, name: &str) -> Option<&CanonicalValue> {
        let Self::Object(fields) = self else {
            return None;
        };
        fields.get(&FieldName::new(name).ok()?)
    }

    fn as_object(&self) -> Option<&BTreeMap<FieldName, CanonicalValue>> {
        match self {
            Self::Object(fields) => Some(fields),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[CanonicalValue]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    fn as_uint(&self) -> Option<u64> {
        match self {
            Self::Uint(value) => Some(*value),
            Self::Int(value) => u64::try_from(*value).ok(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

/// A required object field.
///
/// # Errors
///
/// Returns `RP0105` naming the absent field and its document.
pub fn required<'a>(
    value: &'a CanonicalValue,
    name: &str,
    path: &Path,
) -> PlanResult<&'a CanonicalValue> {
    value.get(name).ok_or_else(|| {
        PlanError::new(codes::DOCUMENT_SHAPE, format!("field `{name}` is absent")).at(path)
    })
}

/// A required string field.
///
/// # Errors
///
/// Returns `RP0105` when the field is absent or is not a string.
pub fn required_text<'a>(
    value: &'a CanonicalValue,
    name: &str,
    path: &Path,
) -> PlanResult<&'a str> {
    required(value, name, path)?.as_text().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{name}` is not a string"),
        )
        .at(path)
    })
}

/// A required unsigned-integer field.
///
/// # Errors
///
/// Returns `RP0105` when the field is absent or is not an unsigned integer.
pub fn required_uint(value: &CanonicalValue, name: &str, path: &Path) -> PlanResult<u64> {
    required(value, name, path)?.as_uint().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{name}` is not an unsigned integer"),
        )
        .at(path)
    })
}

/// A required array field.
///
/// # Errors
///
/// Returns `RP0105` when the field is absent or is not an array.
pub fn required_array<'a>(
    value: &'a CanonicalValue,
    name: &str,
    path: &Path,
) -> PlanResult<&'a [CanonicalValue]> {
    required(value, name, path)?.as_array().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{name}` is not an array"),
        )
        .at(path)
    })
}

/// A required boolean field.
///
/// # Errors
///
/// Returns `RP0105` when the field is absent or is not a boolean.
pub fn required_bool(value: &CanonicalValue, name: &str, path: &Path) -> PlanResult<bool> {
    required(value, name, path)?.as_bool().ok_or_else(|| {
        PlanError::new(
            codes::DOCUMENT_SHAPE,
            format!("field `{name}` is not a boolean"),
        )
        .at(path)
    })
}

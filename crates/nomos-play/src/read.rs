//! Canonical reading helpers, in one place.
//!
//! Every document this crate reads is `nomos_core::CanonicalValue`, parsed by
//! `nomos_core::canonical::read::parse_canonical`. There is no JSON reader
//! here and no second value type: the plan, the projection, the play state, the
//! command, and the session are all canonical bytes, which is what makes the
//! wasm boundary one `TextEncoder` away from working.
//!
//! `parse_canonical` reads every integer literal as `Int` while the encoders
//! write `Uint`, so [`uint`] accepts both spellings. That is not a widening:
//! both mean the same non-negative number, and every document this crate
//! *emits* is built from `CanonicalValue` directly rather than by re-encoding
//! what it read.

use std::collections::BTreeMap;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::id::SchemaId;
use nomos_core::{CanonicalValue, FieldName};

use crate::error::{PlayError, PlayResult, codes};

/// Parses canonical bytes, refusing anything that is not canonical.
///
/// Exactly one trailing `LF` is accepted and ignored. Canonical bytes never end
/// in a newline, so stripping at most one is unambiguous, and every document
/// this crate reads from a file has one: `nomos-render-plan` appends it when it
/// writes a plan, and `crates/nomos-render-plan/tests/canonical_round_trip.rs`
/// strips the same byte for the same reason. Two trailing newlines are still a
/// refusal.
///
/// # Errors
///
/// Returns `PL0104` when the bytes are not a canonical document.
pub fn parse(bytes: &[u8], label: &str) -> PlayResult<CanonicalValue> {
    let bytes = match bytes.split_last() {
        Some((b'\n', head)) => head,
        _ => bytes,
    };
    parse_canonical(bytes).map_err(|error| {
        PlayError::new(
            codes::DOCUMENT_SHAPE,
            format!("{label} is not canonical: {}", error.message()),
        )
    })
}

/// Reads an object.
///
/// # Errors
///
/// Returns `PL0104` when the value is not an object.
pub fn object<'a>(
    value: &'a CanonicalValue,
    label: &str,
) -> PlayResult<&'a BTreeMap<FieldName, CanonicalValue>> {
    match value {
        CanonicalValue::Object(fields) => Ok(fields),
        _ => Err(shape(format!("{label} is not an object"))),
    }
}

/// Reads an array.
///
/// # Errors
///
/// Returns `PL0104` when the value is not an array.
pub fn array<'a>(value: &'a CanonicalValue, label: &str) -> PlayResult<&'a [CanonicalValue]> {
    match value {
        CanonicalValue::Array(values) => Ok(values),
        _ => Err(shape(format!("{label} is not an array"))),
    }
}

/// Reads text.
///
/// # Errors
///
/// Returns `PL0104` when the value is not text.
pub fn text<'a>(value: &'a CanonicalValue, label: &str) -> PlayResult<&'a str> {
    match value {
        CanonicalValue::Text(value) => Ok(value),
        _ => Err(shape(format!("{label} is not text"))),
    }
}

/// Reads a boolean.
///
/// # Errors
///
/// Returns `PL0104` when the value is not a boolean.
pub fn boolean(value: &CanonicalValue, label: &str) -> PlayResult<bool> {
    match value {
        CanonicalValue::Bool(value) => Ok(*value),
        _ => Err(shape(format!("{label} is not a boolean"))),
    }
}

/// Reads a non-negative integer in either canonical spelling.
///
/// # Errors
///
/// Returns `PL0104` when the value is not an integer and `PL0105` when it is
/// negative.
pub fn uint(value: &CanonicalValue, label: &str) -> PlayResult<u64> {
    match value {
        CanonicalValue::Uint(value) => Ok(*value),
        CanonicalValue::Int(value) => u64::try_from(*value)
            .map_err(|_| PlayError::new(codes::DOCUMENT_VALUE, format!("{label} is negative"))),
        _ => Err(shape(format!("{label} is not an unsigned integer"))),
    }
}

/// Reads a signed lattice component.
///
/// # Errors
///
/// Returns `PL0104` when the value is not an integer and `PL0105` when it does
/// not fit an `i32`.
pub fn int32(value: &CanonicalValue, label: &str) -> PlayResult<i32> {
    let value = match value {
        CanonicalValue::Int(value) => *value,
        CanonicalValue::Uint(value) => i64::try_from(*value)
            .map_err(|_| PlayError::new(codes::DOCUMENT_VALUE, format!("{label} exceeds i64")))?,
        _ => return Err(shape(format!("{label} is not an integer"))),
    };
    i32::try_from(value)
        .map_err(|_| PlayError::new(codes::DOCUMENT_VALUE, format!("{label} exceeds i32")))
}

/// Reads a declared field.
///
/// # Errors
///
/// Returns `PL0104` when the field is absent.
pub fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    label: &str,
) -> PlayResult<&'a CanonicalValue> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| shape(format!("{label} has no `{name}` field")))
}

/// Requires the exact field set, in any order.
///
/// # Errors
///
/// Returns `PL0104` naming both sets when they differ.
pub fn require_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    label: &str,
) -> PlayResult<()> {
    let actual = fields.keys().map(FieldName::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual == expected {
        Ok(())
    } else {
        Err(shape(format!(
            "{label} fields are {actual:?}; expected {expected:?}"
        )))
    }
}

/// Reads and binds a schema identity, refusing any other.
///
/// # Errors
///
/// Returns `PL0104` for a malformed identity and `PL0101` for the wrong one.
pub fn bind_schema(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &SchemaId,
    label: &str,
) -> PlayResult<()> {
    let value = field(fields, "schema", label)?;
    let CanonicalValue::Text(text) = value else {
        return Err(shape(format!(
            "{label} schema is not a string; R1 requires `name@version`"
        )));
    };
    let declared = SchemaId::parse(text).map_err(|error| {
        PlayError::new(
            codes::DOCUMENT_SHAPE,
            format!("{label} schema is malformed: {}", error.message()),
        )
    })?;
    if &declared == expected {
        Ok(())
    } else {
        Err(PlayError::new(
            codes::SCHEMA_MISMATCH,
            format!("{label} declares `{declared}`; this runtime accepts `{expected}`"),
        ))
    }
}

fn shape(message: String) -> PlayError {
    PlayError::new(codes::DOCUMENT_SHAPE, message)
}

#[cfg(test)]
mod tests {
    use nomos_core::CanonicalValue;
    use nomos_core::id::SchemaId;

    use super::{bind_schema, object};

    #[test]
    fn r1_schema_binding_refuses_the_gate_k_object_spelling() {
        let document = CanonicalValue::object_declared([(
            "schema",
            CanonicalValue::object_declared([
                ("name", CanonicalValue::text("nomos.play_state")),
                ("version", CanonicalValue::Uint(1)),
            ]),
        )]);
        let fields = object(&document, "play state").expect("fixture is an object");
        let expected = SchemaId::parse("nomos.play_state@1").expect("fixture schema is valid");

        let error = bind_schema(fields, &expected, "play state").unwrap_err();
        assert_eq!(error.code(), "PL0104");
        assert!(
            error.message().contains("R1 requires `name@version`"),
            "{}",
            error.message()
        );
    }
}

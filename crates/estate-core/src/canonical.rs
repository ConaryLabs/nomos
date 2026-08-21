//! The canonical byte profile.
//!
//! `KERNEL.md` section 7 fixes the persisted byte profile for semantic
//! artifacts. This module implements it as a function from a typed value to
//! bytes, plus a strict reader ([`read`]) that refuses anything that is not
//! already canonical.
//!
//! The profile, restated as implemented here:
//!
//! - UTF-8, no byte-order mark;
//! - object keys sorted by ascending UTF-8 byte sequence;
//! - arrays emitted in the order the caller declares — see [`keyed_array`] for
//!   the stable-ID ordering rule;
//! - authoritative numbers are signed or unsigned integers only; there is no
//!   floating-point variant of [`CanonicalValue`], so a float cannot reach an
//!   authoritative artifact by accident;
//! - integers in base 10, no leading `+`, no redundant leading zeroes;
//! - non-ASCII characters emitted as UTF-8, never as `\u` escapes;
//! - `"`, `\`, and control bytes escaped;
//! - lowercase `true`, `false`, `null`;
//! - no insignificant whitespace;
//! - no trailing newline.
//!
//! Section 7 requires the two-character forms `\b \f \n \r \t` for those five
//! control bytes and
//! `\u00xx` with lowercase hex digits for every other byte below `0x20`. `\/`
//! is never emitted and is refused on read. `0x7f` is not a JSON control
//! character and is emitted as raw UTF-8.

pub mod read;

use std::collections::BTreeMap;
use std::fmt;

use crate::diagnostic::{Diagnostic, RepairClass, codes};

/// A canonical object field name.
///
/// Field names are restricted to `[a-z][a-z0-9_]*`. That is deliberately
/// narrower than JSON allows: the restricted set
/// is invariant under Unicode NFC normalization, so section 7's normalization-
/// by-construction rule is satisfied without carrying Unicode tables into the
/// hash domain.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FieldName(String);

impl FieldName {
    /// Accepts a field name.
    ///
    /// # Errors
    ///
    /// Returns [`Diagnostic`] `EK0301` when the name is empty, does not start
    /// with `a`-`z`, or contains a character outside `[a-z0-9_]`.
    pub fn new(name: &str) -> Result<Self, Diagnostic> {
        let bytes = name.as_bytes();
        let legal = !bytes.is_empty()
            && bytes[0].is_ascii_lowercase()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_');
        if !legal {
            return Err(Diagnostic::new(
                codes::FIELD_NAME_UNSUPPORTED,
                format!("`{name}` is not a legal canonical field name"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        Ok(Self(name.to_owned()))
    }

    /// Accepts a field name known at authoring time.
    ///
    /// # Panics
    ///
    /// Panics when the literal is not a legal field name. This constructor is
    /// for literals written into kernel source; every literal the kernel uses
    /// is exercised by this crate's tests, so an illegal one fails the suite
    /// rather than reaching an artifact.
    #[must_use]
    pub fn declared(name: &'static str) -> Self {
        match Self::new(name) {
            Ok(field) => field,
            Err(diagnostic) => panic!("illegal declared field name: {diagnostic}"),
        }
    }

    /// The field name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A value that can be written under the canonical byte profile.
///
/// There is no floating-point variant. Section 7 forbids floats in
/// authoritative state, and the cheapest enforcement is a type that cannot
/// hold one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CanonicalValue {
    /// JSON `null`.
    Null,
    /// JSON `true` or `false`.
    Bool(bool),
    /// A signed 64-bit integer.
    Int(i64),
    /// An unsigned 64-bit integer.
    Uint(u64),
    /// A UTF-8 string.
    Text(String),
    /// An array in caller-declared semantic order.
    Array(Vec<CanonicalValue>),
    /// An object; keys are held sorted by ascending UTF-8 bytes.
    Object(BTreeMap<FieldName, CanonicalValue>),
}

impl CanonicalValue {
    /// Builds a string value.
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Builds an object from field/value pairs.
    ///
    /// Ordering is imposed by the container, so the pairs may arrive in any
    /// order and the encoded bytes are identical either way.
    ///
    /// # Errors
    ///
    /// Returns `EK0304` when a field name occurs more than once. Duplicate
    /// semantic identity is never resolved by selecting one value.
    pub fn object(fields: impl IntoIterator<Item = (FieldName, Self)>) -> Result<Self, Diagnostic> {
        let mut object = BTreeMap::new();
        for (name, value) in fields {
            if object.insert(name.clone(), value).is_some() {
                return Err(Diagnostic::new(
                    codes::CANONICAL_DUPLICATE_IDENTITY,
                    format!("canonical object field `{name}` occurs more than once"),
                )
                .with_repair(RepairClass::RemoveDuplicateDeclaration));
            }
        }
        Ok(Self::Object(object))
    }

    /// Builds an object from field-name literals.
    ///
    /// # Panics
    ///
    /// Panics when a literal is not a legal field name or when the same literal
    /// occurs more than once. Declared fields are kernel source, so either case
    /// is a developer bug rather than a rejected world.
    #[must_use]
    pub fn object_declared(fields: impl IntoIterator<Item = (&'static str, Self)>) -> Self {
        let mut object = BTreeMap::new();
        for (name, value) in fields {
            let name = FieldName::declared(name);
            assert!(
                object.insert(name.clone(), value).is_none(),
                "duplicate declared canonical field `{name}`"
            );
        }
        Self::Object(object)
    }

    /// The canonical bytes of this value.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Null => out.extend_from_slice(b"null"),
            Self::Bool(true) => out.extend_from_slice(b"true"),
            Self::Bool(false) => out.extend_from_slice(b"false"),
            Self::Int(value) => out.extend_from_slice(value.to_string().as_bytes()),
            Self::Uint(value) => out.extend_from_slice(value.to_string().as_bytes()),
            Self::Text(value) => write_string(value, out),
            Self::Array(items) => {
                out.push(b'[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(b',');
                    }
                    item.write(out);
                }
                out.push(b']');
            }
            Self::Object(fields) => {
                out.push(b'{');
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(b',');
                    }
                    write_string(name.as_str(), out);
                    out.push(b':');
                    value.write(out);
                }
                out.push(b'}');
            }
        }
    }
}

/// Builds an array ordered by stable ID.
///
/// Section 7 requires entity collections to be arrays ordered by stable entity
/// ID and machine collections to be arrays ordered by canonical namespace ID.
/// This is that rule as a function: the caller supplies `(id, value)` pairs in
/// any order, and the array comes back in ascending ID order.
///
/// # Errors
///
/// Returns `EK0304` when an ID occurs more than once. Stable ordering must not
/// conceal duplicate semantic identity.
pub fn keyed_array<K: Ord>(
    items: impl IntoIterator<Item = (K, CanonicalValue)>,
) -> Result<CanonicalValue, Diagnostic> {
    let mut ordered = BTreeMap::new();
    for (id, value) in items {
        if ordered.insert(id, value).is_some() {
            return Err(Diagnostic::new(
                codes::CANONICAL_DUPLICATE_IDENTITY,
                "a canonical keyed collection contains the same stable ID more than once",
            )
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(CanonicalValue::Array(ordered.into_values().collect()))
}

/// The canonical bytes of a value.
#[must_use]
pub fn to_canonical_bytes(value: &CanonicalValue) -> Vec<u8> {
    value.to_canonical_bytes()
}

fn write_string(value: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for character in value.chars() {
        match character {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            control if (control as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", control as u32).as_bytes());
            }
            other => {
                let mut buffer = [0_u8; 4];
                out.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    out.push(b'"');
}

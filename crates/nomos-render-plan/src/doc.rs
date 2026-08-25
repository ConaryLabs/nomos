//! The rendering plan's document value and canonical encoder.
//!
//! # Why this is not `nomos_core::CanonicalValue`
//!
//! `RUNTIME.md` section 5 R1-2 requires the plan to be "canonical bytes under
//! the schema identity declared by the emitting code", and `nomos-core` owns
//! the canonical byte profile. Two properties of the document this slice must
//! emit put it outside `CanonicalValue`'s *type*, though not outside the
//! profile:
//!
//! 1. **Field names.** `nomos_core::FieldName` accepts `[a-z][a-z0-9_]*`
//!    (`crates/nomos-core/src/canonical.rs:54`). The rendering plan's field
//!    names are camelCase — `visualAssembly`, `machineNamespaces`,
//!    `projectionDigests`, `inputStateHash` — and two of its objects are keyed
//!    by dotted identifiers (`projectionDigests` by file name,
//!    `scenarios[].machineStates` by machine namespace). The plan's consumers
//!    are the existing viewer, `render-core.mjs`, `play-state.mjs`, and
//!    `build-collection.mjs`, and issue #139 fixes the field names so that only
//!    their schema-string checks change. Renaming the surface is R1-3/R1-4's
//!    work, not this slice's.
//! 2. **Numbers.** `CanonicalValue` has no floating-point variant by design.
//!    The plan carries `architecture.wallHeight`, masonry mass heights, and
//!    `effects[].presentationAnchor` verbatim from `area.json`, and all three
//!    are decimals today (audit section 4, twenty-six values). R1-3 removes
//!    them.
//!
//! Extending `nomos-core` was rejected: it is one of the six kernel crates,
//! `KERNEL.md` section 7 forbids floats in the hash domain, and the
//! floating-point exclusion is a stated design property of the type
//! (`canonical.rs:100-104`). So the profile is *reimplemented here for one
//! document*, with exactly the widenings named above and nothing else, and
//! `tests/canonical_profile.rs` proves the two encoders agree byte for byte on
//! every value both can express. When R1-3 lands a typed presentation source
//! with integer lattice units and snake_case names, this module's reason to
//! exist ends and the plan can be emitted straight from `CanonicalValue`.
//!
//! The profile, as implemented: UTF-8 with no BOM; object keys sorted by
//! ascending UTF-8 bytes; arrays in caller-declared order; integers in base 10
//! with no leading `+` and no redundant leading zero; decimals as the verbatim
//! source lexeme; non-ASCII emitted as UTF-8; `"`, `\`, and control bytes
//! escaped, with the two-character forms for `\b \f \n \r \t` and `\u00xx`
//! otherwise; lowercase `true`, `false`, `null`; no insignificant whitespace;
//! no trailing newline.

use std::collections::BTreeMap;
use std::fmt;

use nomos_core::{CanonicalValue, FieldName};

use crate::decimal::Decimal;
use crate::error::{PlanError, PlanResult, codes};
use crate::json::Json;

/// A rendering-plan object field name.
///
/// `[a-z][A-Za-z0-9_.]*`: `nomos_core::FieldName`'s profile widened by ASCII
/// uppercase letters and `.` after the first character, and by nothing else.
/// Uppercase is the camelCase surface; `.` is required because two of the
/// plan's objects are keyed by a dotted identifier rather than by a declared
/// field — `projectionDigests` by projection file name
/// (`build-plan.mjs:164-169`) and `scenarios[].machineStates` by canonical
/// machine namespace (`build-plan.mjs:108-110`, read back at
/// `render-core.mjs:67` and `webgl-renderer.mjs:90`). Both stay invariant under
/// Unicode NFC normalization, so the normalization-by-construction property
/// `KERNEL.md` section 7 relies on is preserved.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlanField(String);

impl PlanField {
    /// Accepts a field name.
    ///
    /// # Errors
    ///
    /// Returns `RP0206` when the name is empty, does not start with `a`-`z`,
    /// or contains a character outside `[A-Za-z0-9_.]`.
    pub fn new(name: &str) -> PlanResult<Self> {
        let bytes = name.as_bytes();
        let legal = !bytes.is_empty()
            && bytes[0].is_ascii_lowercase()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'.');
        if !legal {
            return Err(PlanError::new(
                codes::FIELD_NAME_UNSUPPORTED,
                format!("`{name}` is not a legal rendering-plan field name"),
            ));
        }
        Ok(Self(name.to_owned()))
    }

    /// Accepts a field name known at authoring time.
    ///
    /// # Panics
    ///
    /// Panics when the literal is illegal. Every literal this crate writes is
    /// exercised by its tests, so an illegal one fails the suite rather than
    /// reaching an artifact.
    #[must_use]
    pub fn declared(name: &'static str) -> Self {
        match Self::new(name) {
            Ok(field) => field,
            Err(error) => panic!("illegal declared plan field name: {error}"),
        }
    }

    /// The field name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlanField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A value that can be written into the rendering plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlanValue {
    /// JSON `null`. The plan spells a blocked subject's cost this way, which
    /// is the one normalization the equivalence comparison documents.
    Null,
    /// JSON `true` or `false`.
    Bool(bool),
    /// A signed 64-bit integer.
    Int(i64),
    /// An unsigned 64-bit integer.
    Uint(u64),
    /// An exact decimal carried verbatim from presentation source.
    Number(Decimal),
    /// A UTF-8 string.
    Text(String),
    /// An array in caller-declared order.
    Array(Vec<PlanValue>),
    /// An object; keys are held sorted by ascending UTF-8 bytes.
    Object(BTreeMap<PlanField, PlanValue>),
}

impl PlanValue {
    /// Builds a string value.
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Builds an object from field-name literals.
    ///
    /// # Panics
    ///
    /// Panics when a literal is illegal or repeats.
    #[must_use]
    pub fn object(fields: impl IntoIterator<Item = (&'static str, Self)>) -> Self {
        let mut object = BTreeMap::new();
        for (name, value) in fields {
            let name = PlanField::declared(name);
            assert!(
                object.insert(name.clone(), value).is_none(),
                "duplicate declared plan field `{name}`"
            );
        }
        Self::Object(object)
    }

    /// Builds an object whose keys are runtime identifiers rather than
    /// declared literals.
    ///
    /// # Errors
    ///
    /// Returns `RP0206` when a key is outside [`PlanField`]'s profile, and
    /// `RP0105` when a key repeats.
    pub fn keyed_object(fields: impl IntoIterator<Item = (String, Self)>) -> PlanResult<Self> {
        let mut object = BTreeMap::new();
        for (name, value) in fields {
            let field = PlanField::new(&name)?;
            if object.insert(field, value).is_some() {
                return Err(PlanError::new(
                    codes::DOCUMENT_SHAPE,
                    format!("plan object field `{name}` occurs more than once"),
                ));
            }
        }
        Ok(Self::Object(object))
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
            Self::Number(value) => out.extend_from_slice(value.lexeme().as_bytes()),
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

    /// Copies a canonical kernel value into the plan.
    ///
    /// Total: every `CanonicalValue` is a `PlanValue`, because
    /// `nomos_core::FieldName`'s profile is a strict subset of
    /// [`PlanField`]'s.
    #[must_use]
    pub fn from_canonical(value: &CanonicalValue) -> Self {
        match value {
            CanonicalValue::Null => Self::Null,
            CanonicalValue::Bool(inner) => Self::Bool(*inner),
            CanonicalValue::Int(inner) => Self::Int(*inner),
            CanonicalValue::Uint(inner) => Self::Uint(*inner),
            CanonicalValue::Text(inner) => Self::Text(inner.clone()),
            CanonicalValue::Array(items) => {
                Self::Array(items.iter().map(Self::from_canonical).collect())
            }
            CanonicalValue::Object(fields) => Self::Object(
                fields
                    .iter()
                    .map(|(name, value)| {
                        (
                            PlanField(name.as_str().to_owned()),
                            Self::from_canonical(value),
                        )
                    })
                    .collect(),
            ),
        }
    }

    /// Copies a presentation-source value into the plan.
    ///
    /// # Errors
    ///
    /// Returns `RP0206` when the source carries a field name outside
    /// [`PlanField`]'s profile.
    pub fn from_area(value: &Json) -> PlanResult<Self> {
        Ok(match value {
            Json::Null => Self::Null,
            Json::Bool(inner) => Self::Bool(*inner),
            Json::Number(inner) => Self::Number(inner.clone()),
            Json::Text(inner) => Self::Text(inner.clone()),
            Json::Array(items) => Self::Array(
                items
                    .iter()
                    .map(Self::from_area)
                    .collect::<PlanResult<Vec<_>>>()?,
            ),
            Json::Object(fields) => {
                let mut object = BTreeMap::new();
                for (name, value) in fields {
                    object.insert(PlanField::new(name)?, Self::from_area(value)?);
                }
                Self::Object(object)
            }
        })
    }

    /// The equivalent `CanonicalValue`, when one exists.
    ///
    /// Returns `None` for any value using one of this module's two widenings —
    /// a camelCase field name or a decimal. `tests/canonical_profile.rs` uses
    /// it to prove the two encoders agree on everything else.
    #[must_use]
    pub fn to_canonical(&self) -> Option<CanonicalValue> {
        Some(match self {
            Self::Null => CanonicalValue::Null,
            Self::Bool(inner) => CanonicalValue::Bool(*inner),
            Self::Int(inner) => CanonicalValue::Int(*inner),
            Self::Uint(inner) => CanonicalValue::Uint(*inner),
            Self::Number(_) => return None,
            Self::Text(inner) => CanonicalValue::Text(inner.clone()),
            Self::Array(items) => CanonicalValue::Array(
                items
                    .iter()
                    .map(Self::to_canonical)
                    .collect::<Option<Vec<_>>>()?,
            ),
            Self::Object(fields) => {
                let mut object = BTreeMap::new();
                for (name, value) in fields {
                    object.insert(FieldName::new(name.as_str()).ok()?, value.to_canonical()?);
                }
                CanonicalValue::Object(object)
            }
        })
    }
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

//! Identifier segments and the separator invariant.
//!
//! Every stable ID in the kernel is one or more [`Ident`] segments joined by a
//! separator byte. Restricting segments to `[a-z0-9_]` starting with a
//! lowercase letter buys three things the contract asks for:
//!
//! 1. **NFC without Unicode tables.** `KERNEL.md` section 7 requires
//!    identifiers to be normalised to Unicode NFC before validation. Every
//!    character in this set is invariant under NFC, so a validated identifier
//!    is normalised by construction and the hash domain carries no Unicode
//!    table that could change between library versions.
//! 2. **Ordering that cannot surprise.** See [`SEPARATORS`] below.
//! 3. **Readable fixtures.** The base fixture identifiers — `north_gate`,
//!    `flooded_section`, `brazier_02`, `credential/gaoler_key` — all fit.
//!
//! # Stated limitation
//!
//! Non-ASCII identifiers are refused, not normalised. When the kernel needs
//! them, it needs a real NFC implementation and an owner decision about
//! carrying that table inside the hash domain. Until then the rule fails
//! closed: no unnormalised identifier can enter an artifact, because no
//! non-ASCII identifier can.
//!
//! # The separator invariant
//!
//! Composite IDs order by their fields, but section 7 orders collections by
//! the *canonical ID string*. Those two orderings agree here, and not by luck:
//! every separator byte is numerically lower than every byte a segment may
//! contain. Comparing `a.b` against `ab.c` reaches the separator in the first
//! string while the second still has a segment byte, and the separator loses —
//! exactly as comparing the segment `a` against `ab` would. [`SEPARATORS`] and
//! the test `separators_sort_below_every_identifier_byte` hold that property
//! down.

use std::fmt;

use crate::diagnostic::{Diagnostic, RepairClass, codes};

/// The separator bytes composite stable IDs may use.
///
/// Each must sort below every byte legal inside an [`Ident`], which is what
/// makes field ordering and canonical-string ordering the same ordering.
pub const SEPARATORS: [u8; 3] = *b"#./";

/// One identifier segment: `[a-z][a-z0-9_]*`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Ident(String);

impl Ident {
    /// Accepts an identifier segment.
    ///
    /// # Errors
    ///
    /// Returns [`Diagnostic`] `EK0101` when the segment is empty, does not
    /// start with `a`-`z`, or contains a byte outside `[a-z0-9_]`.
    pub fn new(text: &str) -> Result<Self, Diagnostic> {
        if !Self::is_legal(text) {
            return Err(Diagnostic::new(
                codes::IDENT_UNSUPPORTED,
                format!(
                    "`{text}` is not a legal identifier segment; \
                     segments are `[a-z][a-z0-9_]*`"
                ),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape));
        }
        Ok(Self(text.to_owned()))
    }

    /// Whether `text` is a legal segment.
    #[must_use]
    pub fn is_legal(text: &str) -> bool {
        let bytes = text.as_bytes();
        !bytes.is_empty()
            && bytes[0].is_ascii_lowercase()
            && bytes.iter().all(|byte| Self::is_legal_byte(*byte))
    }

    /// Whether `byte` may appear inside a segment.
    #[must_use]
    pub const fn is_legal_byte(byte: u8) -> bool {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
    }

    /// The segment as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Splits `text` on `separator` into exactly `N` legal segments.
///
/// # Errors
///
/// Returns [`Diagnostic`] `EK0104` when the count of segments is wrong, and
/// `EK0101` when a segment is not a legal [`Ident`].
pub(crate) fn split_exact<const N: usize>(
    text: &str,
    separator: u8,
    shape: &str,
) -> Result<[Ident; N], Diagnostic> {
    let parts: Vec<&str> = text.split(separator as char).collect();
    if parts.len() != N {
        return Err(Diagnostic::new(
            codes::ID_SHAPE_INVALID,
            format!("`{text}` does not match the required shape `{shape}`"),
        )
        .with_repair(RepairClass::UseSupportedIdentifierShape));
    }
    let mut segments = Vec::with_capacity(N);
    for part in parts {
        segments.push(Ident::new(part)?);
    }
    segments.try_into().map_err(|_| {
        Diagnostic::new(
            codes::ID_SHAPE_INVALID,
            format!("`{text}` does not match the required shape `{shape}`"),
        )
    })
}

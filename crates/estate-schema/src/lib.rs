//! Authoring source and Canonical World IR schemas.
//!
//! `KERNEL.md` section 10 assigns this crate *authoring and Canonical World IR
//! schemas*. Section 4 makes the Canonical World IR the single semantic truth,
//! and section 10 forbids canonical schema types being defined in more than one
//! crate — so the IR type will be defined here and nowhere else when SW-C lands
//! it.
//!
//! At SW-B this crate carries the two schema identities it owns, so that
//! versioning exists from the first commit as section 6 requires, and the
//! boundary is proved before there is anything to put behind it.
//!
//! # Boundary
//!
//! The only permitted edge out of this crate is `estate-core`:
//!
//! ```
//! use estate_core::id::SchemaId;
//! let _: SchemaId = estate_schema::source_schema();
//! ```
//!
//! It may not reach `estate-projection`. Projections are derived from the IR by
//! the compiler; the schema crate must not learn what its consumers look like.
//!
//! ```compile_fail
//! let _ = estate_projection::simulation_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use estate_core::id::SchemaId;

/// The `.estate` authoring source schema.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn source_schema() -> SchemaId {
    SchemaId::new("estate.source", 1).expect("the source schema id is a valid literal")
}

/// The Canonical World IR schema.
///
/// Section 6 requires one real v1-to-v2 migration of this schema, changing the
/// movement representation. SW-B records version 1; the migration is SW-F's.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which this crate's
/// tests rule out.
#[must_use]
pub fn world_ir_schema() -> SchemaId {
    SchemaId::new("estate.world_ir", 1).expect("the world IR schema id is a valid literal")
}

#[cfg(test)]
mod tests {
    use super::{source_schema, world_ir_schema};

    #[test]
    fn owned_schemas_are_distinct_and_versioned_from_one() {
        assert_ne!(source_schema().name(), world_ir_schema().name());
        assert_eq!(source_schema().version(), 1);
        assert_eq!(world_ir_schema().version(), 1);
    }

    #[test]
    fn schema_ids_encode_canonically() {
        assert_eq!(
            source_schema().to_canonical().to_canonical_bytes(),
            br#"{"name":"estate.source","version":1}"#.to_vec()
        );
    }
}

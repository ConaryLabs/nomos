//! Authoring source and Canonical World IR schemas.
//!
//! `KERNEL.md` section 4 makes this crate the single owner of the Canonical
//! World IR type. The compiler consumes [`SourceDocument`] and produces
//! [`WorldIr`]; projection and runtime crates cannot name either because the
//! dependency graph denies them access to this crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod ir;
mod source;
mod spatial;

use estate_core::SchemaId;

pub use ir::{
    CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, FactOwner, FactOwnershipReceipt,
    IrEntity, IrRelation, MachineTemplate, PrimitiveExpansion, WorldIr,
};
pub use source::{
    ForbiddenFactOwner, SourceDocument, SourceEntity, SourceField, SourceRelation, Spanned,
};
pub use spatial::{Binding, Cell, Direction};

/// The `.estate` authoring source schema.
#[must_use]
pub fn source_schema() -> SchemaId {
    SchemaId::new("estate.source", 1).expect("the source schema id is a valid literal")
}

/// The Canonical World IR schema.
///
/// Section 6 requires one real v1-to-v2 migration of this schema, changing the
/// movement representation. Version 1 remains current until that later slice.
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

//! Authoring source and Canonical World IR construction schemas.
//!
//! `KERNEL.md` section 4 assigns this crate the single ownership boundary for
//! Canonical World IR types. The current [`WorldIr`] is an incomplete
//! construction snapshot. The compiler consumes [`SourceDocument`] and
//! produces that snapshot; projection and runtime crates cannot name either
//! because the dependency graph denies them access to this crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod ir;
mod resolver;
mod source;
mod spatial;
mod transition;

use estate_core::SchemaId;

pub use ir::{
    CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, FactOwner, FactOwnershipReceipt,
    IrEntity, IrRelation, MachineTemplate, PrimitiveExpansion, WorldIr,
};
pub use resolver::{
    GroundConnectivity, GroundMovementCoherence, MovementCompositionLaw, MovementResolverPlan,
    MovementResolverSubject,
};
pub use source::{
    ForbiddenFactOwner, SourceDocument, SourceEntity, SourceField, SourceRelation, Spanned,
};
pub use spatial::{Binding, Cell, Direction};
pub use transition::{
    InteractionDefinition, InteractionPhase, InteractionTrigger, TransitionDefinition,
    TransitionInput, TransitionTrigger,
};

/// The `.estate` authoring source schema.
#[must_use]
pub fn source_schema() -> SchemaId {
    SchemaId::new("estate.source", 1).expect("the source schema id is a valid literal")
}

/// The incomplete Canonical World IR construction schema.
///
/// Contract revision 4 separates pre-Gate construction snapshots from the
/// stable `estate.world_ir@1` to `estate.world_ir@2` migration line. SW-E
/// advances the construction lineage to version 3 because composition,
/// coherence, and resolver preparation change the snapshot shape.
#[must_use]
pub fn construction_world_ir_schema() -> SchemaId {
    SchemaId::new("estate.world_ir.construction", 3)
        .expect("the construction world IR schema id is a valid literal")
}

#[cfg(test)]
mod tests {
    use super::{construction_world_ir_schema, source_schema};

    #[test]
    fn owned_schemas_are_distinct_and_versioned_from_one() {
        assert_ne!(
            source_schema().name(),
            construction_world_ir_schema().name()
        );
        assert_eq!(source_schema().version(), 1);
        assert_eq!(construction_world_ir_schema().version(), 3);
    }

    #[test]
    fn schema_ids_encode_canonically() {
        assert_eq!(
            source_schema().to_canonical().to_canonical_bytes(),
            br#"{"name":"estate.source","version":1}"#.to_vec()
        );
        assert_eq!(
            construction_world_ir_schema()
                .to_canonical()
                .to_canonical_bytes(),
            br#"{"name":"estate.world_ir.construction","version":3}"#.to_vec()
        );
    }
}

//! Authoring source, Canonical World IR, and schema-registry types.
//!
//! `KERNEL.md` section 4 assigns this crate the single ownership boundary for
//! Canonical World IR types. [`WorldIr`] remains immutable construction
//! evidence; [`StableWorldIr`] is the contract-complete stable lineage. The
//! compiler consumes [`SourceDocument`] and owns promotion between them;
//! projection and runtime crates cannot name either because the dependency
//! graph denies them access to this crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod decode;
mod ir;
mod light;
mod package;
mod provenance;
mod resolver;
mod source;
mod spatial;
mod stable;
mod transition;

use nomos_core::SchemaId;

pub use ir::{
    CapabilityKind, ClaimActivation, ClaimTemplate, ClaimValue, IrEntity, IrRelation,
    MachineTemplate, PrimitiveExpansion, WorldIr,
};
pub use light::{LightCompositionLaw, LightResolverPlan, LightResolverSubject};
pub use package::{SchemaOwner, SchemaRegistration, SchemaRegistry};
pub use provenance::{
    DerivationInput, DerivationPass, DerivationProducer, DerivationStep, FactIdentity, FactOwner,
    FactOwnershipReceipt, ProjectionConsumer, ResolvedFactValue,
};
pub use resolver::{
    GroundConnectivity, GroundMovementCoherence, MovementCompositionLaw, MovementResolverPlan,
    MovementResolverSubject,
};
pub use source::{
    ForbiddenFactOwner, SourceDocument, SourceEntity, SourceField, SourceRelation, Spanned,
};
pub use spatial::{Binding, Cell, Direction};
pub use stable::{StableGroundMovementV1, StableWorldIr};
pub use transition::{
    InteractionDefinition, InteractionPhase, InteractionTrigger, TransitionDefinition,
    TransitionInput, TransitionTrigger,
};

/// The `.nomos` authoring source schema.
#[must_use]
pub fn source_schema() -> SchemaId {
    SchemaId::new("nomos.source", 1).expect("the source schema id is a valid literal")
}

/// The incomplete Canonical World IR construction schema.
///
/// Contract revision 6 closes the prototype `estate.*` epoch and starts the
/// Nomos construction lineage at version 1. Version 2 replaces stringly
/// ownership receipts with typed fact identities, values, consumers, and
/// derivation edges. Version 3 adds the closed light-union resolver plan used
/// by SW-F. The shape is not compatible with a prototype construction snapshot.
#[must_use]
pub fn construction_world_ir_schema() -> SchemaId {
    SchemaId::new("nomos.world_ir.construction", 3)
        .expect("the construction world IR schema id is a valid literal")
}

/// The first contract-complete Canonical World IR schema.
///
/// This identity is separate from every construction snapshot. Its first
/// incompatible change is reserved for the required movement migration.
#[must_use]
pub fn stable_world_ir_schema() -> SchemaId {
    SchemaId::new("nomos.world_ir", 1).expect("the stable world IR schema id is a valid literal")
}

/// The package schema-registry artifact.
#[must_use]
pub fn schema_registry_schema() -> SchemaId {
    SchemaId::new("nomos.package.schemas", 1).expect("the schema registry id is a valid literal")
}

#[cfg(test)]
mod tests {
    use super::{
        construction_world_ir_schema, schema_registry_schema, source_schema, stable_world_ir_schema,
    };

    #[test]
    fn owned_schemas_are_distinct_and_versioned_from_one() {
        assert_ne!(
            source_schema().name(),
            construction_world_ir_schema().name()
        );
        assert_ne!(
            construction_world_ir_schema().name(),
            stable_world_ir_schema().name()
        );
        assert_eq!(source_schema().version(), 1);
        assert_eq!(construction_world_ir_schema().version(), 3);
        assert_eq!(stable_world_ir_schema().version(), 1);
        assert_eq!(schema_registry_schema().version(), 1);
    }

    #[test]
    fn schema_ids_encode_canonically() {
        assert_eq!(
            source_schema().to_canonical().to_canonical_bytes(),
            br#"{"name":"nomos.source","version":1}"#.to_vec()
        );
        assert_eq!(
            construction_world_ir_schema()
                .to_canonical()
                .to_canonical_bytes(),
            br#"{"name":"nomos.world_ir.construction","version":3}"#.to_vec()
        );
        assert_eq!(
            stable_world_ir_schema().to_canonical().to_canonical_bytes(),
            br#"{"name":"nomos.world_ir","version":1}"#.to_vec()
        );
    }
}

//! The compile-time half of the kernel.
//!
//! `KERNEL.md` section 10 assigns this crate *parse, link, expand, validate,
//! migrate, and project*. Section 2 fixes what "compile time" means here: it
//! prepares claim templates, machines, interaction edges, composition laws,
//! coherence rules, and a resolver plan. It does **not** decide final subsystem
//! deltas — amendment A4 exists precisely because opening a door may leave
//! movement blocked while a ward still claims it.
//!
//! This crate is the only place that reads both the Canonical World IR schema
//! and the projection schemas, because it is the only thing that turns one into
//! the other.
//!
//! # Boundary
//!
//! Its three permitted edges all resolve:
//!
//! ```
//! use estate_core::id::SchemaId;
//! let _: Vec<SchemaId> = estate_compiler::consumed_schemas();
//! let _: Vec<SchemaId> = estate_compiler::produced_schemas();
//! ```
//!
//! It may not reach `estate-sim`. The compiler produces artifacts; it does not
//! execute them, and a compiler that could name runtime state would be able to
//! precompute deltas that section 2 says are not knowable until command time:
//!
//! ```compile_fail
//! let _ = estate_sim::runtime_state_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use estate_core::id::SchemaId;

/// The schemas this compiler reads.
#[must_use]
pub fn consumed_schemas() -> Vec<SchemaId> {
    vec![
        estate_schema::source_schema(),
        estate_schema::world_ir_schema(),
    ]
}

/// The schemas this compiler writes.
///
/// The Canonical World IR appears on both sides: the compiler emits it from
/// source, and the migration path reads a previous version of it.
#[must_use]
pub fn produced_schemas() -> Vec<SchemaId> {
    let mut schemas = vec![estate_schema::world_ir_schema()];
    schemas.extend(estate_projection::all_schemas());
    schemas
}

#[cfg(test)]
mod tests {
    use super::{consumed_schemas, produced_schemas};

    #[test]
    fn the_compiler_is_the_only_crossing_between_ir_and_projections() {
        let consumed = consumed_schemas();
        let produced = produced_schemas();
        assert!(consumed.contains(&estate_schema::source_schema()));
        for projection in estate_projection::all_schemas() {
            assert!(
                produced.contains(&projection),
                "the compiler must produce every projection schema"
            );
            assert!(
                !consumed.contains(&projection),
                "a projection schema is compiler output, never compiler input"
            );
        }
    }
}

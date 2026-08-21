//! Versioned projection schemas.
//!
//! `KERNEL.md` section 10 assigns this crate *versioned
//! simulation/navigation/persistence/diagnostic schemas*. Section 4 fixes the
//! ownership rule these schemas exist to serve: projection compilers consume
//! the Canonical World IR, and runtime subsystems consume only their own
//! versioned projection artifacts.
//!
//! Section 9 (amendment A9) requires each projection to version independently,
//! so a change to the diagnostics projection cannot force a simulation
//! migration. That is why these are four separate schema identities rather than
//! one package version.
//!
//! # Boundary
//!
//! The only permitted edge out of this crate is `estate-core`:
//!
//! ```
//! use estate_core::id::SchemaId;
//! let _: SchemaId = estate_projection::navigation_schema();
//! ```
//!
//! It may not reach `estate-schema`. A projection schema that could name the
//! Canonical World IR type would invite a runtime subsystem to reach the IR
//! through it, which section 4 forbids:
//!
//! ```compile_fail
//! let _ = estate_schema::construction_world_ir_schema();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

use estate_core::id::SchemaId;

macro_rules! projection_schema {
    ($($function:ident => $name:literal, $doc:literal;)*) => {
        $(
            #[doc = $doc]
            ///
            /// # Panics
            ///
            /// Panics if the built-in literal is not a valid schema id, which
            /// this crate's tests rule out.
            #[must_use]
            pub fn $function() -> SchemaId {
                SchemaId::new($name, 1).expect("a projection schema id is a valid literal")
            }
        )*

        /// Every projection schema this crate owns.
        #[must_use]
        pub fn all_schemas() -> Vec<SchemaId> {
            vec![$($function()),*]
        }
    };
}

projection_schema! {
    simulation_schema => "estate.projection.simulation",
        "The simulation projection schema.";
    navigation_schema => "estate.projection.navigation",
        "The navigation projection schema.";
    persistence_schema => "estate.projection.persistence",
        "The persistence projection schema.";
    diagnostics_schema => "estate.projection.diagnostics",
        "The diagnostics projection schema.";
}

#[cfg(test)]
mod tests {
    use super::all_schemas;
    use std::collections::BTreeSet;

    #[test]
    fn the_four_projections_are_four_independent_schema_identities() {
        let schemas = all_schemas();
        assert_eq!(schemas.len(), 4);
        let names: BTreeSet<String> = schemas.iter().map(|id| id.name().to_string()).collect();
        assert_eq!(names.len(), 4, "projection schema names must be distinct");
    }
}

//! The strict observed-scene presentation carrier and compiler.
//!
//! `R2.md` revision 1 gives this crate exactly two schema identities. The
//! input preserves facts supplied by an observer; the compiler adds only the
//! closed presentation selections declared by that contract.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod command;
mod diagnostic;
pub mod input;
mod json;
pub mod plan;
mod value;

pub use command::{Execution, ExitCode, HELP, execute};
pub use diagnostic::{ObservedCode, ObservedError, ObservedResult, codes, render_rejection};
pub use input::{
    Action, Actor, ActorCell, Availability, Crop, LifeState, LocalId, ObservedScene, SceneIdentity,
    TerrainCell, TerrainLayer, TerrainRole,
};
pub use plan::{
    ActionMarker, ActionPlan, ActorAssembly, ActorPlan, ActorPose, MaterialFamily, Presence,
    ScenePlan, TerrainAssembly, TerrainPlan, compile,
};

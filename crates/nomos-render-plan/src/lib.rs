//! The R1 rendering-plan compiler.
//!
//! `RUNTIME.md` section 5 R1-2: a Rust compiler producing the rendering plan
//! from the R1-1 effective-fact projection and typed presentation source,
//! replacing `experiments/executable-gaol/src/build-plan.mjs`. That file is
//! deleted in the change that lands this crate; under `RUNTIME.md` section 2
//! the study is a specification and a comparison target, never a source of
//! truth, so every behaviour reproduced here names the study file and lines it
//! reproduces, and every behaviour *not* reproduced is recorded with its cause
//! in `docs/review/rendering-plan-compiler.md`.
//!
//! # What it reads
//!
//! Documents only. The crate never opens `.nomos` source, `world-ir.json`, or
//! `compiler-receipts.json`; those three names appear nowhere in
//! `crates/nomos-render-plan/src`, and `tests/inputs.rs` asserts it two ways —
//! by grep over the source and by compiling successfully against a world
//! directory whose World IR and receipts are unreadable garbage.
//!
//! - [`plan::Inputs::catalog`] — one `nomos.entity_catalog@1` document, the
//!   only source of entity kind.
//! - [`plan::Inputs::facts`] — one `nomos.effective_facts@1` document per
//!   scenario, the only source of movement disposition, cost, reasons, and
//!   effective light.
//! - [`plan::Inputs::runs`] — the per-scenario run bundles, read for machine
//!   states, declared status, and the committed command log.
//! - [`plan::Inputs::world`] — four projection members, hashed and republished.
//! - [`plan::Inputs::source`] — one `nomos.presentation_source@1` document, the
//!   typed presentation source R1-3 landed in place of `area.json`.
//!
//! # What it does not do
//!
//! It evaluates no activation expression, composes no disposition, takes no
//! cost maximum, and classifies nothing by string convention. Every one of
//! those was a line of `build-plan.mjs`; each module names the lines it
//! deletes.
//!
//! # The second command
//!
//! `nomos-render-plan collection --plans <dir-or-plan> --out <areas.json>`
//! stitches the compiled plans into `nomos.area_collection@1`: the route chain,
//! the visual grammar every area shares, and one row per area naming the plan
//! file and its SHA-256. It replaces
//! `experiments/executable-gaol/src/build-collection.mjs`, which was the only
//! authority for the route graph; [`collection`] is its owner file and names the
//! study lines it reproduces.
//!
//! It also holds no floating-point value and writes no canonical bytes of its
//! own. `nomos.presentation_source@1` is integer-only by the type its reader
//! parses into, and `nomos.rendering_plan@2` is emitted through
//! `nomos_core::CanonicalValue`, so the private encoder R1-2 needed
//! (`src/doc.rs`, issue #144) and its decimal type are both gone.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod catalog;
pub mod collection;
pub mod error;
pub mod facts;
pub mod json;
pub mod plan;
pub mod read;
pub mod runs;
pub mod source;
pub mod world;

pub use catalog::{EntityCatalog, EntityKind, entity_catalog_schema};
pub use collection::{CompiledCollection, PlanInput, area_collection_schema};
pub use error::{PlanCode, PlanError, PlanResult, codes};
pub use facts::{EffectiveFacts, effective_facts_schema};
pub use plan::{CompiledPlan, Inputs, compile, rendering_plan_schema};
pub use source::{PresentationSource, presentation_source_schema};

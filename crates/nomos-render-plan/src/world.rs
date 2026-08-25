//! The world package, opened for exactly four files.
//!
//! `RUNTIME.md` section 5 R1-2 forbids an R1 crate from parsing `.nomos`
//! source, Canonical World IR, or compiler receipts. This module is the only
//! place in the crate that names a world directory at all, and it names four
//! members of it: the simulation, navigation, persistence, and diagnostics
//! projections. It reads each one twice — once as bytes, to hash, and once as a
//! canonical document, to copy the `schema` field the plan republishes — and
//! opens nothing else. `manifest.json`, `schemas.json`, `world-ir.json`, and
//! `compiler-receipts.json` are never opened; `tests/inputs.rs` proves it by
//! filling all four with bytes that would fail every reader in this crate and
//! compiling successfully anyway.
//!
//! The digests are the same SHA-256 over the same raw file bytes that
//! `experiments/executable-gaol/src/build-plan.mjs:164-169` computed, now from
//! `nomos_core::Sha256Digest` rather than from Node's `crypto`.

use std::path::Path;

use nomos_core::{CanonicalValue, Sha256Digest};

use crate::error::{PlanError, PlanResult, codes};
use crate::read::{self, Shape};

/// The four projection members, in the order the plan publishes them.
pub const PROJECTION_FILES: [&str; 4] = [
    "simulation.json",
    "navigation.json",
    "persistence.json",
    "diagnostics.json",
];

/// One projection's published identity and content digest.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectionFacts {
    /// The member file name, which is the digest map's key.
    pub file: &'static str,
    /// The member's `schema` value, copied verbatim.
    pub schema: CanonicalValue,
    /// Lowercase hexadecimal SHA-256 over the member's raw bytes.
    pub digest: String,
}

/// Reads the four projections' identities and digests.
///
/// # Errors
///
/// Returns `RP0101` when a member cannot be read, `RP0102` when it is not
/// canonical, and `RP0105` when it carries no `schema` field.
pub fn read_projections(world: &Path) -> PlanResult<Vec<ProjectionFacts>> {
    PROJECTION_FILES
        .into_iter()
        .map(|file| {
            let path = world.join(file);
            let bytes = std::fs::read(&path).map_err(|error| {
                PlanError::new(codes::INPUT_UNREADABLE, error.to_string()).at(&path)
            })?;
            let document = read::read_document(&path)?;
            let schema = document.get("schema").cloned().ok_or_else(|| {
                PlanError::new(
                    codes::DOCUMENT_SHAPE,
                    "projection carries no `schema` field",
                )
                .at(&path)
            })?;
            Ok(ProjectionFacts {
                file,
                schema,
                digest: Sha256Digest::of_bytes(&bytes).to_hex(),
            })
        })
        .collect()
}

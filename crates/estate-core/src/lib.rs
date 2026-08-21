//! Deterministic primitives for the Gate K semantic kernel.
//!
//! `estate-core` owns the responsibility assigned by `KERNEL.md` section 10:
//! *stable IDs, deterministic primitives, canonical bytes, hashing,
//! diagnostics*. It is the root of the permitted dependency graph and depends
//! on nothing, so every crate above it shares one definition of identity,
//! canonical encoding, and hashing.
//!
//! # What lives here
//!
//! - [`ident`] — the validated identifier segment every stable ID is built
//!   from, plus the separator invariant that makes string ordering and tuple
//!   ordering agree.
//! - [`id`] — the typed, non-interchangeable stable IDs: [`EntityId`],
//!   [`CatalogValueId`], [`NamespaceId`], [`ClaimRef`], and [`SchemaId`].
//! - [`canonical`] — the canonical UTF-8 JSON byte profile of `KERNEL.md`
//!   section 7, as a function from a typed value to bytes, plus a strict
//!   reader that refuses non-canonical input.
//! - [`hash`] — SHA-256 over canonical bytes, displayed as lowercase hex.
//! - [`arith`] — checked integer arithmetic; authoritative arithmetic never
//!   wraps.
//! - [`diagnostic`] — the structured diagnostic shape of section 9.
//! - [`package`] — the immutable world-package directory writer and verifying
//!   reader of section 5.
//!
//! # What does not live here
//!
//! No schema for any specific artifact. `estate-core` knows that a package
//! holds named members of canonical bytes; it does not know what
//! `world-ir.json` means. That belongs to `estate-schema` and
//! `estate-projection`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod arith;
pub mod canonical;
pub mod diagnostic;
pub mod hash;
pub mod id;
pub mod ident;
pub mod package;

pub use canonical::{CanonicalValue, FieldName, to_canonical_bytes};
pub use diagnostic::{Diagnostic, DiagnosticCode, RepairClass, SourcePath, SourceSpan};
pub use hash::{Sha256Digest, StateHash};
pub use id::{CatalogValueId, ClaimRef, EntityId, NamespaceId, SchemaId, SchemaName};
pub use ident::Ident;

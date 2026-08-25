//! The R1-1 effective-fact projection, as this compiler consumes it.
//!
//! Input identity is `nomos.effective_facts@1`, declared by
//! `crates/nomos-sim/src/effective_facts.rs` and accepted as R1-1. This module
//! decodes it and nothing else: it evaluates no activation expression, applies
//! no blocker-before-cost rule, and takes no maximum. `RUNTIME.md` section 5
//! R1-2 forbids all three ("evaluate activation expressions in JavaScript
//! anywhere on the accepted path", "recompute an effective fact R1-1 already
//! resolved"), and issue #132 records that the JavaScript that did was wrong in
//! three ways this crate therefore cannot reproduce.
//!
//! Deleted by this module, with prior sites:
//!
//! - `experiments/executable-gaol/src/build-plan.mjs:86-95` — `activationIsActive`,
//!   a second `state_equals`/`not`/`any`/`all` evaluator.
//! - `experiments/executable-gaol/src/build-plan.mjs:111-122` — movement
//!   disposition, cost, and reasons recomputed from raw navigation claims.
//! - `experiments/executable-gaol/src/build-plan.mjs:123-128` — effective light
//!   recomputed from raw light-resolver claims.
//!
//! One presentation difference is preserved on purpose: the kernel's `Blocked`
//! variant carries no `cost` key, and the plan spells a blocked subject's cost
//! as `null`. `RUNTIME.md` section 5 R1-1 already names that spelling as the
//! only normalization in the twenty-scenario comparison, and
//! `experiments/executable-gaol/compare-rendering-plan.sh` documents it again.

use std::collections::BTreeMap;
use std::path::Path;

use nomos_core::CanonicalValue;
use nomos_core::id::SchemaId;

use crate::error::{PlanError, PlanResult, codes};
use crate::read::{self, Shape};

/// The effective-fact document's schema identity.
///
/// Declared by `nomos-sim`; named here because R1-2 is its first accepted
/// consumer and must bind identity and version.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn effective_facts_schema() -> SchemaId {
    SchemaId::parse("nomos.effective_facts@1").expect("the effective-facts schema id is a literal")
}

/// One subject's resolved ground-movement fact.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MovementFact {
    /// `traversable` or `blocked`, verbatim from the kernel.
    pub disposition: String,
    /// The resolved cost; `None` exactly when the kernel reported `blocked`.
    pub cost: Option<u64>,
    /// The kernel's ordered reason claim ids.
    pub reasons: Vec<String>,
}

/// One scenario's effective facts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EffectiveFacts {
    /// The authoritative tick the state carries.
    pub tick: u64,
    /// The authoritative state hash the facts were resolved against.
    pub state_hash: String,
    /// Ground-movement facts by subject entity.
    pub movement: BTreeMap<String, MovementFact>,
    /// Light emission by subject entity.
    pub light: BTreeMap<String, bool>,
}

impl EffectiveFacts {
    /// Decodes one effective-fact document.
    ///
    /// # Errors
    ///
    /// Returns `RP0104` when the document's identity is not
    /// `nomos.effective_facts@1` and `RP0105` when a required field is absent,
    /// mis-shaped, or repeats a subject.
    pub fn decode(document: &CanonicalValue, path: &Path) -> PlanResult<Self> {
        read::require_completed(document, path)?;
        read::bind_schema(document, &effective_facts_schema(), path)?;
        let tick = read::required_uint(document, "tick", path)?;
        let state_hash = read::required_text(document, "state_hash", path)?.to_owned();
        let facts = read::required(document, "effective_facts", path)?;

        let mut movement = BTreeMap::new();
        for entry in read::required_array(facts, "ground_movement", path)? {
            let entity = read::required_text(entry, "entity", path)?.to_owned();
            let disposition = read::required(entry, "disposition", path)?;
            let kind = read::required_text(disposition, "kind", path)?.to_owned();
            let cost = disposition.get("cost").and_then(CanonicalValue::as_uint);
            let reasons = read::required_array(disposition, "reasons", path)?
                .iter()
                .filter_map(CanonicalValue::as_text)
                .map(str::to_owned)
                .collect();
            if kind == "blocked" && cost.is_some() {
                return Err(shape(
                    path,
                    format!("blocked subject `{entity}` carries a cost"),
                ));
            }
            if kind == "traversable" && cost.is_none() {
                return Err(shape(
                    path,
                    format!("traversable subject `{entity}` carries no cost"),
                ));
            }
            if movement
                .insert(
                    entity.clone(),
                    MovementFact {
                        disposition: kind,
                        cost,
                        reasons,
                    },
                )
                .is_some()
            {
                return Err(shape(
                    path,
                    format!("movement subject `{entity}` occurs more than once"),
                ));
            }
        }

        let mut light = BTreeMap::new();
        for entry in read::required_array(facts, "light_emission", path)? {
            let entity = read::required_text(entry, "entity", path)?.to_owned();
            let emitting = read::required_bool(entry, "emitting", path)?;
            if light.insert(entity.clone(), emitting).is_some() {
                return Err(shape(
                    path,
                    format!("light subject `{entity}` occurs more than once"),
                ));
            }
        }

        Ok(Self {
            tick,
            state_hash,
            movement,
            light,
        })
    }
}

fn shape(path: &Path, message: String) -> PlanError {
    PlanError::new(codes::DOCUMENT_SHAPE, message).at(path)
}

//! The entity catalog, and the classification that replaces string convention.
//!
//! Input identity is `nomos.entity_catalog@1`, the read-only kernel projection
//! from issue #138. It is the reason this slice can classify at all: the four
//! package projections carry no entity kind, which is why
//! `experiments/executable-gaol/src/build-plan.mjs:25` had to ask whether any
//! machine namespace ended in `.access`.
//!
//! # What replaces what
//!
//! | Removed | Prior site | Replacement |
//! | --- | --- | --- |
//! | door via `machine.endsWith(".access")` | `build-plan.mjs:25` | catalog `primitive` |
//! | light via membership in the persistence light-resolver subject set | `build-plan.mjs:26` | catalog `primitive` |
//! | water via presence of a `traversal_cost_ground` claim | `build-plan.mjs:27` | catalog `primitive` |
//! | silent `unknown` fallback | `build-plan.mjs:28` | [`EntityKind::Unknown`], cross-checked so it cannot swallow a mis-declared door |
//!
//! Nothing here reads an entity id, a machine namespace, or an assembly string
//! to decide a kind. `tests/classification.rs` renames every entity id and
//! machine namespace in a catalog and proves the kinds do not move.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use nomos_core::CanonicalValue;
use nomos_core::id::SchemaId;

use crate::error::{PlanError, PlanResult, codes};
use crate::read::{self, Shape};

/// The catalog document's schema identity.
///
/// Declared by `nomos-compiler` under issue #138; named here because this
/// crate is its first consumer and `RUNTIME.md` section 5 R1-2 requires the
/// consumer to bind identity and version and refuse a mismatch.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn entity_catalog_schema() -> SchemaId {
    SchemaId::parse("nomos.entity_catalog@1").expect("the entity-catalog schema id is a literal")
}

/// The closed set of entity kinds the rendering plan distinguishes.
///
/// # The last assignment of these strings outside the renderer catalog
///
/// [`EntityKind::visual_assembly`] and [`EntityKind::material_family`] are the
/// kind-to-assembly and kind-to-material tables that lived at
/// `experiments/executable-gaol/src/build-plan.mjs:33-38` and `:43`. They are
/// here, typed and closed, because R1-2 has to emit the same plan; they do not
/// belong here. `docs/review/executable-gaol-ownership-audit.md` section 3
/// items 5 and 6 record them as a renderer catalog living in the wrong layer,
/// and `RUNTIME.md` section 5 R1-3 and R1-4 revisit their ownership: R1-3 gives
/// every presentation field exactly one owner and R1-4 promotes the viewer that
/// should own an assembly name.
///
/// **This is the last place in the tree where a visual assembly name or a
/// material family is assigned to an entity kind outside the renderer
/// catalog.** No later slice may add a third such table; the correct change is
/// to move these two out.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EntityKind {
    /// A door: `primitive/iron_barred_door`.
    Door,
    /// Standing water: `primitive/shallow_water_region`.
    Water,
    /// A light source: `primitive/extinguishable_light`.
    Light,
    /// A primitive the plan has no visual kind for.
    Unknown,
}

impl EntityKind {
    /// The plan's `kind` string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Door => "door",
            Self::Water => "water",
            Self::Light => "light",
            Self::Unknown => "unknown",
        }
    }

    /// The plan's `visualAssembly` string.
    #[must_use]
    pub const fn visual_assembly(self) -> &'static str {
        match self {
            Self::Door => "visual/iron_barred_door",
            Self::Water => "visual/shallow_water",
            Self::Light => "visual/brazier",
            Self::Unknown => "visual/marker",
        }
    }

    /// The plan's `materialFamily` string.
    #[must_use]
    pub const fn material_family(self) -> &'static str {
        match self {
            Self::Door => "iron_oxidized",
            Self::Water => "water_cold",
            Self::Light => "iron_brazier",
            Self::Unknown => "stone",
        }
    }

    /// The primitive kind that declares this entity kind, if any.
    const fn primitive(self) -> Option<&'static str> {
        match self {
            Self::Door => Some("primitive/iron_barred_door"),
            Self::Water => Some("primitive/shallow_water_region"),
            Self::Light => Some("primitive/extinguishable_light"),
            Self::Unknown => None,
        }
    }

    /// The capabilities the kernel's primitive expansion always attaches to
    /// this kind, used only as a cross-check on the declaration.
    const fn required_capabilities(self) -> &'static [&'static str] {
        match self {
            Self::Door => &["blocks_ground", "boundary", "portal"],
            Self::Water => &["region", "traversal_cost_ground"],
            Self::Light => &["emits_light"],
            Self::Unknown => &[],
        }
    }

    /// Every kind with a declared primitive, in declaration order.
    const DECLARED: [Self; 3] = [Self::Door, Self::Water, Self::Light];
}

/// One resolver claim, as the catalog reports it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CatalogClaim {
    /// Stable claim identity.
    pub id: String,
    /// The resolver the claim belongs to: `movement` or `light`.
    pub resolver: String,
    /// The claim's source span, copied verbatim into the plan's provenance.
    pub source: CanonicalValue,
}

/// One catalogued entity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CatalogEntity {
    /// Stable entity identity.
    pub id: String,
    /// The classified kind.
    pub kind: EntityKind,
    /// The projection's binding, copied verbatim into the plan's `anchor`.
    pub binding: CanonicalValue,
    /// Machine namespaces in catalog order.
    pub machine_namespaces: Vec<String>,
    /// Every resolver claim whose subject is this entity.
    pub claims: Vec<CatalogClaim>,
}

/// The decoded entity catalog.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EntityCatalog {
    entities: Vec<CatalogEntity>,
}

impl EntityCatalog {
    /// Decodes and classifies a catalog document.
    ///
    /// # Errors
    ///
    /// Returns `RP0104` when the document's identity is not
    /// `nomos.entity_catalog@1`, `RP0105` when a required field is absent or
    /// mis-shaped, and `RP0201` when a declaration and its capability set
    /// contradict each other.
    pub fn decode(document: &CanonicalValue, path: &Path) -> PlanResult<Self> {
        read::bind_schema(document, &entity_catalog_schema(), path)?;
        let mut entities = Vec::new();
        let mut seen = BTreeSet::new();
        for entry in read::required_array(document, "entities", path)? {
            let entity = decode_entity(entry, path)?;
            if !seen.insert(entity.id.clone()) {
                return Err(PlanError::new(
                    codes::DOCUMENT_SHAPE,
                    format!("entity `{}` occurs more than once", entity.id),
                )
                .at(path));
            }
            entities.push(entity);
        }
        Ok(Self { entities })
    }

    /// The catalogued entities in document order.
    #[must_use]
    pub fn entities(&self) -> &[CatalogEntity] {
        &self.entities
    }

    /// The entities indexed by stable id.
    #[must_use]
    pub fn by_id(&self) -> BTreeMap<&str, &CatalogEntity> {
        self.entities
            .iter()
            .map(|entity| (entity.id.as_str(), entity))
            .collect()
    }
}

fn decode_entity(entry: &CanonicalValue, path: &Path) -> PlanResult<CatalogEntity> {
    let id = read::required_text(entry, "id", path)?.to_owned();
    let primitive = read::required_text(entry, "primitive", path)?;
    let capabilities: BTreeSet<&str> = read::required_array(entry, "capabilities", path)?
        .iter()
        .filter_map(CanonicalValue::as_text)
        .collect();
    let kind = classify(primitive, &capabilities, &id, path)?;
    let binding = read::required(entry, "binding", path)?.clone();

    let mut machine_namespaces = Vec::new();
    for machine in read::required_array(entry, "machines", path)? {
        machine_namespaces.push(read::required_text(machine, "namespace", path)?.to_owned());
    }

    let mut claims = Vec::new();
    for claim in read::required_array(entry, "claims", path)? {
        claims.push(CatalogClaim {
            id: read::required_text(claim, "id", path)?.to_owned(),
            resolver: read::required_text(claim, "resolver", path)?.to_owned(),
            source: read::required(claim, "source", path)?.clone(),
        });
    }

    Ok(CatalogEntity {
        id,
        kind,
        binding,
        machine_namespaces,
        claims,
    })
}

/// Classifies one entity from its declared primitive, cross-checked against its
/// typed capability set.
///
/// The primitive decides. The capability set is only allowed to *refuse*: a
/// declaration whose capabilities contradict it fails closed rather than
/// producing a plausible-looking plan, and a primitive this compiler has no
/// kind for may not carry another kind's capability signature — that would be a
/// door the plan would have drawn as `visual/marker`, which is exactly the
/// silent fallback `build-plan.mjs:28` shipped.
fn classify(
    primitive: &str,
    capabilities: &BTreeSet<&str>,
    entity: &str,
    path: &Path,
) -> PlanResult<EntityKind> {
    let unsound = |message: String| PlanError::new(codes::CLASSIFICATION_UNSOUND, message).at(path);
    for kind in EntityKind::DECLARED {
        if kind.primitive() != Some(primitive) {
            continue;
        }
        if let Some(missing) = kind
            .required_capabilities()
            .iter()
            .find(|capability| !capabilities.contains(*capability))
        {
            return Err(unsound(format!(
                "entity `{entity}` declares `{primitive}` but its capability set omits `{missing}`"
            )));
        }
        return Ok(kind);
    }
    for kind in EntityKind::DECLARED {
        if kind
            .required_capabilities()
            .iter()
            .all(|capability| capabilities.contains(capability))
        {
            return Err(unsound(format!(
                "entity `{entity}` declares `{primitive}`, which this compiler has no kind for, \
                 but carries the full capability signature of `{}`",
                kind.as_str()
            )));
        }
    }
    Ok(EntityKind::Unknown)
}

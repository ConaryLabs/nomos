//! `nomos.play_state@1`: the authoritative state of one area.
//!
//! This is the owner file for that identity, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! # How the kernel state is embedded
//!
//! `kernel` is the `nomos.persisted_runtime_state@2` document as a **nested
//! object**, not a hex or base64 string of its bytes. Canonical encoding is
//! context-free, so `kernel.to_canonical_bytes()` is byte-for-byte the
//! persisted envelope and `PersistedRuntimeState::from_canonical_bytes`
//! accepts it directly. That is the kernel's own idiom: its persisted envelope
//! nests the runtime state under `state` and recovers the inner bytes by
//! re-encoding that sub-object (`crates/nomos-sim/src/state_persistence.rs`).
//!
//! The persisted envelope already carries `state_hash` and
//! `runtime_semantics_digest`, so this document repeats **neither**. Hoisting a
//! copy would be exactly the double authority the R1-3 ownership audit spent
//! nine rows removing.
//!
//! # No compiled static entity is copied
//!
//! The persisted state's `entities[]` are the kernel's own runtime bindings —
//! `{id, binding}` — which are its state. The plan's entity records (kind,
//! anchor, machine namespaces, provenance) are read from the plan and never
//! written here. An actor carries `{cell, id, role}` and nothing else; its
//! assembly name stays in the plan, where the renderer reads it.
//!
//! # Two ticks, two meanings
//!
//! `tick` counts committed play batches, which is to say inputs: a batch the
//! rules refused still advances it (`docs/review/nomos-play.md` section 3.5).
//! The kernel's own tick lives inside `kernel.state.tick` and counts committed
//! kernel transactions. Neither is the other.

use std::fmt;

use nomos_core::canonical::keyed_array;
use nomos_core::id::{EntityId, SchemaId};
use nomos_core::{CanonicalValue, StateHash};
use nomos_projection::{LatticeCell, SimulationPlan};
use nomos_sim::PersistedRuntimeState;

use crate::error::{PlayError, PlayResult, codes};
use crate::plan::{AreaPlan, Role};
use crate::read;

/// The play-state identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn play_state_schema() -> SchemaId {
    SchemaId::new("nomos.play_state", 1).expect("the play-state schema id is a literal")
}

/// How a run ended, or that it has not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The run continues.
    Playing,
    /// The player left this area through a declared gate.
    Escaped,
    /// The pursuer reached the player.
    Caught,
}

impl Outcome {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Escaped => "escaped",
            Self::Caught => "caught",
        }
    }

    /// Reads the wire spelling.
    ///
    /// # Errors
    ///
    /// Returns `PL0105` for any other value.
    pub fn parse(value: &str) -> PlayResult<Self> {
        match value {
            "playing" => Ok(Self::Playing),
            "escaped" => Ok(Self::Escaped),
            "caught" => Ok(Self::Caught),
            _ => Err(PlayError::new(
                codes::DOCUMENT_VALUE,
                format!("`{value}` is not a play outcome"),
            )),
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One authoritative actor.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Actor {
    /// Stable actor identity; the collection is ordered by it.
    pub id: EntityId,
    /// Declared role.
    pub role: Role,
    /// Authoritative integer lattice cell.
    pub cell: LatticeCell,
}

/// Cumulative run counters. Both are cumulative across the whole session:
/// crossing an area carries them, which is what `play.mjs:69-80` already did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Counters {
    /// Accepted steps taken, crossings included.
    pub moves: u64,
    /// Traversal cost paid.
    pub traversal_cost: u64,
}

/// The authoritative state of one area.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlayState {
    /// Area identifier; must equal the plan this state is played against.
    pub area: String,
    /// The ordinal of the last committed batch.
    pub tick: u64,
    /// The embedded kernel authority.
    pub kernel: PersistedRuntimeState,
    /// Actors in ascending stable-id order.
    pub actors: Vec<Actor>,
    /// The entity whose emission gates the pursuit rule.
    pub pursuit_light: EntityId,
    /// Accepted player moves since the pursuer last stepped; `0` or `1`.
    pub moves_since_step: u64,
    /// Whether the run continues.
    pub outcome: Outcome,
    /// Cumulative counters.
    pub counters: Counters,
}

impl PlayState {
    /// Opens the initial state of one area at the kernel's own tick 0.
    ///
    /// Actors are placed at `entry` when the caller supplies one — an arrival
    /// uses the destination's **own** `route.entry` — and at the plan's
    /// declared cell otherwise.
    ///
    /// # Errors
    ///
    /// Returns `PL0308` when the kernel refuses to initialize the plan, and
    /// `PL0103` when an actor's declared cell is not a cell of this lattice.
    pub fn open(
        plan: &AreaPlan,
        semantics: &SimulationPlan,
        player_cell: Option<LatticeCell>,
    ) -> PlayResult<Self> {
        let state = nomos_sim::SimulationState::initialize(semantics)
            .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))?;
        let kernel = PersistedRuntimeState::new(semantics, state)
            .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))?;

        let mut actors = Vec::with_capacity(plan.actors.len());
        for declared in &plan.actors {
            let cell = match (declared.role, player_cell) {
                (Role::Player, Some(entry)) => entry,
                _ => declared.cell,
            };
            if !plan.in_bounds(cell.x(), cell.y()) {
                return Err(PlayError::new(
                    codes::ACTORS_INVALID,
                    format!(
                        "actor `{}` starts at ({}, {}), outside the {}x{} lattice",
                        declared.id,
                        cell.x(),
                        cell.y(),
                        plan.width,
                        plan.height
                    ),
                ));
            }
            actors.push(Actor {
                id: declared.id.clone(),
                role: declared.role,
                cell,
            });
        }
        actors.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(Self {
            area: plan.area.clone(),
            tick: 0,
            kernel,
            actors,
            pursuit_light: plan.pursuit_light.clone(),
            moves_since_step: 0,
            outcome: Outcome::Playing,
            counters: Counters::default(),
        })
    }

    /// The player actor. Every constructor guarantees exactly one.
    ///
    /// # Panics
    ///
    /// Panics only if a state was built without a player, which
    /// [`AreaPlan::decode`] and [`PlayState::decode`] both refuse.
    #[must_use]
    pub fn player(&self) -> &Actor {
        self.actors
            .iter()
            .find(|actor| actor.role == Role::Player)
            .expect("a play state carries exactly one player")
    }

    /// The pursuer, if this area declares one.
    #[must_use]
    pub fn pursuer(&self) -> Option<&Actor> {
        self.actors.iter().find(|actor| actor.role == Role::Pursuer)
    }

    /// The actor standing on a cell, if any.
    #[must_use]
    pub fn actor_at(&self, x: i32, y: i32) -> Option<&Actor> {
        self.actors
            .iter()
            .find(|actor| actor.cell.x() == x && actor.cell.y() == y)
    }

    /// The kernel state hash this state embeds.
    #[must_use]
    pub fn kernel_state_hash(&self) -> StateHash {
        self.kernel.state_hash()
    }

    /// The document as a canonical value.
    ///
    /// # Panics
    ///
    /// Panics if two actors share an identity, which every constructor and
    /// decoder refuses first.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "actors",
                keyed_array(self.actors.iter().map(|actor| {
                    (
                        actor.id.clone(),
                        CanonicalValue::object_declared([
                            ("cell", cell_value(actor.cell)),
                            ("id", CanonicalValue::text(actor.id.to_string())),
                            ("role", CanonicalValue::text(actor.role.as_str())),
                        ]),
                    )
                }))
                .expect("a play state validates unique actor identities"),
            ),
            ("area", CanonicalValue::text(self.area.clone())),
            (
                "counters",
                CanonicalValue::object_declared([
                    ("moves", CanonicalValue::Uint(self.counters.moves)),
                    (
                        "traversal_cost",
                        CanonicalValue::Uint(self.counters.traversal_cost),
                    ),
                ]),
            ),
            ("kernel", kernel_value(&self.kernel)),
            ("outcome", CanonicalValue::text(self.outcome.as_str())),
            (
                "pursuit",
                CanonicalValue::object_declared([
                    (
                        "light",
                        CanonicalValue::text(self.pursuit_light.to_string()),
                    ),
                    (
                        "moves_since_step",
                        CanonicalValue::Uint(self.moves_since_step),
                    ),
                ]),
            ),
            (
                "schema",
                CanonicalValue::text(play_state_schema().to_string()),
            ),
            ("tick", CanonicalValue::Uint(self.tick)),
        ])
    }

    /// The document as canonical bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// The hash of this state's own canonical envelope.
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        StateHash::of_envelope(&self.to_canonical())
    }

    /// Reads one play state, binding it to the semantics it names.
    ///
    /// # Errors
    ///
    /// Returns `PL0101` for another identity, `PL0104`/`PL0105` for a shape or
    /// value this runtime cannot read, `PL0103` for an actor collection that is
    /// not exactly one player and at most one pursuer, and `PL0308` when the
    /// kernel refuses the embedded state — including `EK0813` when the state
    /// belongs to different simulation semantics.
    pub fn decode(bytes: &[u8], semantics: &SimulationPlan) -> PlayResult<Self> {
        let value = read::parse(bytes, "play state")?;
        let fields = read::object(&value, "play state")?;
        read::bind_schema(fields, &play_state_schema(), "play state")?;
        read::require_fields(
            fields,
            &[
                "actors", "area", "counters", "kernel", "outcome", "pursuit", "schema", "tick",
            ],
            "play state",
        )?;

        let kernel = PersistedRuntimeState::from_canonical_bytes(
            &read::field(fields, "kernel", "play state")?.to_canonical_bytes(),
            semantics,
        )
        .map_err(|error| PlayError::from_kernel(codes::KERNEL_REFUSED, &error))?;

        let mut actors = Vec::new();
        for row in read::array(read::field(fields, "actors", "play state")?, "play actors")? {
            let actor = read::object(row, "play actor")?;
            read::require_fields(actor, &["cell", "id", "role"], "play actor")?;
            actors.push(Actor {
                id: EntityId::parse(read::text(
                    read::field(actor, "id", "play actor")?,
                    "actor id",
                )?)
                .map_err(|error| PlayError::new(codes::DOCUMENT_VALUE, error.message()))?,
                role: Role::parse(read::text(
                    read::field(actor, "role", "play actor")?,
                    "actor role",
                )?)?,
                cell: crate::plan::cell(read::field(actor, "cell", "play actor")?, "actor cell")?,
            });
        }
        if actors.iter().filter(|a| a.role == Role::Player).count() != 1 {
            return Err(PlayError::new(
                codes::ACTORS_INVALID,
                "a play state carries exactly one `player` actor",
            ));
        }
        if actors.iter().filter(|a| a.role == Role::Pursuer).count() > 1 {
            return Err(PlayError::new(
                codes::ACTORS_INVALID,
                "a play state carries at most one `pursuer` actor",
            ));
        }

        let counters = read::object(
            read::field(fields, "counters", "play state")?,
            "play counters",
        )?;
        read::require_fields(counters, &["moves", "traversal_cost"], "play counters")?;

        let pursuit = read::object(
            read::field(fields, "pursuit", "play state")?,
            "play pursuit",
        )?;
        read::require_fields(pursuit, &["light", "moves_since_step"], "play pursuit")?;

        Ok(Self {
            area: read::text(read::field(fields, "area", "play state")?, "play area")?.to_owned(),
            tick: read::uint(read::field(fields, "tick", "play state")?, "play tick")?,
            kernel,
            actors,
            pursuit_light: EntityId::parse(read::text(
                read::field(pursuit, "light", "play pursuit")?,
                "pursuit light",
            )?)
            .map_err(|error| PlayError::new(codes::DOCUMENT_VALUE, error.message()))?,
            moves_since_step: read::uint(
                read::field(pursuit, "moves_since_step", "play pursuit")?,
                "moves_since_step",
            )?,
            outcome: Outcome::parse(read::text(
                read::field(fields, "outcome", "play state")?,
                "play outcome",
            )?)?,
            counters: Counters {
                moves: read::uint(read::field(counters, "moves", "play counters")?, "moves")?,
                traversal_cost: read::uint(
                    read::field(counters, "traversal_cost", "play counters")?,
                    "traversal_cost",
                )?,
            },
        })
    }
}

/// A `{x, y, z}` lattice cell as a canonical value.
#[must_use]
pub fn cell_value(cell: LatticeCell) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("x", CanonicalValue::Int(i64::from(cell.x()))),
        ("y", CanonicalValue::Int(i64::from(cell.y()))),
        ("z", CanonicalValue::Int(i64::from(cell.z()))),
    ])
}

/// The persisted kernel envelope as a nested canonical value.
///
/// # Panics
///
/// Panics only if the kernel emits bytes its own canonical reader rejects,
/// which `nomos-core`'s round-trip tests rule out.
fn kernel_value(kernel: &PersistedRuntimeState) -> CanonicalValue {
    nomos_core::canonical::read::parse_canonical(&kernel.to_canonical_bytes())
        .expect("the kernel's persisted envelope is canonical")
}

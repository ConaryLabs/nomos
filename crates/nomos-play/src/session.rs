//! `nomos.play_session@1`: the run across areas, its log, and its receipts.
//!
//! This is the owner file for that identity, registered in
//! `docs/evaluation/R1_SCHEMA_OWNERSHIP.md`.
//!
//! # One tick, one pair of counters
//!
//! There is deliberately no session-level `tick` and no session-level `moves`
//! or `traversal_cost`. The tick is the live area's and does not reset on a
//! crossing; the counters are the live area's and are cumulative by
//! construction, which is what `play.mjs:69-80` already did when it carried
//! `movementCost` and `moves` through `enterArea`. Restating either here would
//! be a derived second authority. The one number that is the session's and not
//! any area's is `areas_cleared`.
//!
//! # Arrival is not a command
//!
//! `cross` leaves an area; entering the next one is [`PlaySession::enter`],
//! because the destination's bytes have to be fetched before it can happen and
//! because arrival is not something the player does. Reset is
//! [`PlaySession::start`] again — a new session, not a command. Nothing in the
//! log can express a reset, which is what makes a recorded log replayable as
//! one continuous run.

use nomos_core::id::SchemaId;
use nomos_core::{CanonicalValue, Sha256Digest};
use nomos_projection::SimulationPlan;

use crate::batch::{self, Area};
use crate::command::PlayCommand;
use crate::error::{PlayError, PlayResult, codes};
use crate::plan::AreaPlan;
use crate::read;
use crate::receipt::{PlayReceipt, chain_origin};
use crate::state::{Counters, Outcome, PlayState};

/// The session identity.
///
/// # Panics
///
/// Panics if the literal is not a valid schema id, which this crate's tests
/// rule out.
#[must_use]
pub fn play_session_schema() -> SchemaId {
    SchemaId::new("nomos.play_session", 1).expect("the play-session schema id is a literal")
}

/// How the run stands, across areas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SessionOutcome {
    /// The run continues.
    Playing,
    /// The live area was left; the next area has not been entered yet.
    Escaped,
    /// The pursuer reached the player.
    Caught,
    /// The route's terminal area was left. The run is over.
    Completed,
}

impl SessionOutcome {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Escaped => "escaped",
            Self::Caught => "caught",
            Self::Completed => "completed",
        }
    }
}

/// One area the run has entered, with the digests of the bytes it was played
/// against. These are what make a recorded session replayable: the native
/// replay refuses content whose digests the session does not name, rather than
/// replaying against something else and reporting a difference that is not the
/// runtime's.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RouteRow {
    /// Area identifier.
    pub area: String,
    /// SHA-256 over the rendering plan's bytes.
    pub plan_digest: Sha256Digest,
    /// SHA-256 over the simulation projection's bytes.
    pub semantics_digest: Sha256Digest,
}

/// A run in progress.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlaySession {
    route: Vec<RouteRow>,
    areas: Vec<Area>,
    areas_cleared: u64,
    log: Vec<PlayCommand>,
    receipts: Vec<PlayReceipt>,
    pending_area: Option<String>,
}

impl PlaySession {
    /// Begins a new session at one area. This is also reset.
    ///
    /// # Errors
    ///
    /// Returns `PL0501` when the projection cannot be reconstructed, `PL0502`
    /// when it is not the projection the plan published, and whatever
    /// [`AreaPlan::decode`] and [`PlayState::open`] refuse.
    pub fn start(plan_bytes: &[u8], semantics_bytes: &[u8]) -> PlayResult<Self> {
        let (plan, semantics) = open(plan_bytes, semantics_bytes)?;
        let state = PlayState::open(&plan, &semantics, None)?;
        Ok(Self {
            route: vec![RouteRow {
                area: plan.area.clone(),
                plan_digest: Sha256Digest::of_bytes(plan_bytes),
                semantics_digest: Sha256Digest::of_bytes(semantics_bytes),
            }],
            areas: vec![Area {
                plan,
                semantics,
                state,
            }],
            areas_cleared: 0,
            log: Vec::new(),
            receipts: Vec::new(),
            pending_area: None,
        })
    }

    /// Continues the session into the area the last crossing named.
    ///
    /// The player arrives at the destination's **own** `route.entry` — owner
    /// ruling 3 of `docs/review/presentation-source.md`, the reason an exiting
    /// area no longer names a cell inside its neighbour. The tick and both
    /// counters are carried; `moves_since_step` and the outcome are reset.
    ///
    /// # Errors
    ///
    /// Returns `PL0401` unless the live area's outcome is `escaped` and the
    /// offered plan is the area the crossing named.
    pub fn enter(&mut self, plan_bytes: &[u8], semantics_bytes: &[u8]) -> PlayResult<()> {
        let (plan, semantics) = open(plan_bytes, semantics_bytes)?;
        let live = self.live();
        if live.state.outcome != Outcome::Escaped {
            return Err(PlayError::new(
                codes::ENTER_REFUSED,
                format!(
                    "the live area's outcome is `{}`; arrival needs `escaped`",
                    live.state.outcome
                ),
            ));
        }
        match &self.pending_area {
            Some(expected) if expected == &plan.area => {}
            Some(expected) => {
                return Err(PlayError::new(
                    codes::ENTER_REFUSED,
                    format!("the route continues into `{expected}`, not `{}`", plan.area),
                ));
            }
            None => {
                return Err(PlayError::new(
                    codes::ENTER_REFUSED,
                    "the route has no area after this one",
                ));
            }
        }
        let Some(entry) = plan.entry else {
            return Err(PlayError::new(
                codes::ENTER_REFUSED,
                format!("`{}` declares no arrival cell", plan.area),
            ));
        };

        let tick = live.state.tick;
        let counters = live.state.counters;
        let mut state = PlayState::open(&plan, &semantics, Some(entry))?;
        state.tick = tick;
        state.counters = counters;

        self.route.push(RouteRow {
            area: plan.area.clone(),
            plan_digest: Sha256Digest::of_bytes(plan_bytes),
            semantics_digest: Sha256Digest::of_bytes(semantics_bytes),
        });
        self.areas.push(Area {
            plan,
            semantics,
            state,
        });
        self.pending_area = None;
        Ok(())
    }

    /// Applies one input as exactly one committed batch.
    ///
    /// # Errors
    ///
    /// Returns a shape refusal. A rule refusal is recorded in the receipt.
    pub fn step(&mut self, input: &PlayCommand) -> PlayResult<&PlayReceipt> {
        let ordinal = self.receipts.len() as u64;
        let previous = self
            .receipts
            .last()
            .map_or_else(chain_origin, PlayReceipt::hash);
        let index = self.areas.len() - 1;
        let outcome_before = self.areas[index].state.outcome;
        let committed = batch::step(&mut self.areas[index], ordinal, previous, input)?;
        if committed.receipt.outcome_after == Outcome::Escaped && outcome_before != Outcome::Escaped
        {
            self.areas_cleared = self.areas_cleared.saturating_add(1);
            self.pending_area = committed.crossed_to;
        }
        self.log.push(input.clone());
        self.receipts.push(committed.receipt);
        Ok(self.receipts.last().expect("a receipt was just pushed"))
    }

    /// The live area.
    #[must_use]
    pub fn live(&self) -> &Area {
        self.areas
            .last()
            .expect("a session holds at least one area")
    }

    /// The route, in the order it was entered.
    #[must_use]
    pub fn route(&self) -> &[RouteRow] {
        &self.route
    }

    /// The area the last crossing named, if the run is waiting to enter it.
    #[must_use]
    pub fn pending_area(&self) -> Option<&str> {
        self.pending_area.as_deref()
    }

    /// The committed inputs, in order.
    #[must_use]
    pub fn log(&self) -> &[PlayCommand] {
        &self.log
    }

    /// The receipts, aligned index-for-index with the log.
    #[must_use]
    pub fn receipts(&self) -> &[PlayReceipt] {
        &self.receipts
    }

    /// Areas whose outcome reached `escaped`.
    #[must_use]
    pub const fn areas_cleared(&self) -> u64 {
        self.areas_cleared
    }

    /// The live area's cumulative counters.
    #[must_use]
    pub fn counters(&self) -> Counters {
        self.live().state.counters
    }

    /// The chain head: the last receipt's hash, or 64 zeros.
    #[must_use]
    pub fn receipt_chain_head(&self) -> Sha256Digest {
        self.receipts
            .last()
            .map_or_else(chain_origin, PlayReceipt::hash)
    }

    /// How the run stands, derived from the live area and the route.
    #[must_use]
    pub fn outcome(&self) -> SessionOutcome {
        match self.live().state.outcome {
            Outcome::Playing => SessionOutcome::Playing,
            Outcome::Caught => SessionOutcome::Caught,
            Outcome::Escaped => {
                if self.pending_area.is_some() {
                    SessionOutcome::Escaped
                } else {
                    SessionOutcome::Completed
                }
            }
        }
    }

    /// The session as a canonical value.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "areas",
                CanonicalValue::Array(
                    self.areas
                        .iter()
                        .map(|area| area.state.to_canonical())
                        .collect(),
                ),
            ),
            ("areas_cleared", CanonicalValue::Uint(self.areas_cleared)),
            ("log", self.log_value()),
            ("outcome", CanonicalValue::text(self.outcome().as_str())),
            (
                "position",
                CanonicalValue::Uint((self.areas.len() - 1) as u64),
            ),
            (
                "receipt_chain_head",
                CanonicalValue::text(self.receipt_chain_head().to_hex()),
            ),
            ("receipts", self.receipts_value()),
            (
                "route",
                CanonicalValue::Array(
                    self.route
                        .iter()
                        .map(|row| {
                            CanonicalValue::object_declared([
                                ("area", CanonicalValue::text(row.area.clone())),
                                (
                                    "plan_digest",
                                    CanonicalValue::text(row.plan_digest.to_hex()),
                                ),
                                (
                                    "semantics_digest",
                                    CanonicalValue::text(row.semantics_digest.to_hex()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "schema",
                CanonicalValue::text(play_session_schema().to_string()),
            ),
        ])
    }

    /// The session as canonical bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// The committed inputs as a bare canonical array.
    #[must_use]
    pub fn log_value(&self) -> CanonicalValue {
        CanonicalValue::Array(self.log.iter().map(PlayCommand::to_canonical).collect())
    }

    /// The receipts as a bare canonical array.
    #[must_use]
    pub fn receipts_value(&self) -> CanonicalValue {
        CanonicalValue::Array(
            self.receipts
                .iter()
                .map(PlayReceipt::to_canonical)
                .collect(),
        )
    }
}

/// A session read back from bytes, for replay comparison.
///
/// The typed areas are deliberately not reconstructed: a replay re-executes the
/// log and compares what it produces with what is recorded here. Rebuilding the
/// states from the document would let a replay agree with itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordedSession {
    /// The route with its content digests.
    pub route: Vec<RouteRow>,
    /// The committed inputs, in order.
    pub log: Vec<PlayCommand>,
    /// The recorded receipts, as canonical values for byte comparison.
    pub receipts: Vec<CanonicalValue>,
    /// The recorded chain head.
    pub receipt_chain_head: Sha256Digest,
    /// The recorded per-area states, as canonical values for byte comparison.
    pub areas: Vec<CanonicalValue>,
    /// The recorded session outcome.
    pub outcome: String,
    /// The recorded cleared-area count.
    pub areas_cleared: u64,
}

impl RecordedSession {
    /// Reads one recorded session.
    ///
    /// # Errors
    ///
    /// Returns `PL0101` for another identity and `PL0104`/`PL0105` for a shape
    /// or value this runtime cannot read.
    pub fn decode(bytes: &[u8]) -> PlayResult<Self> {
        let value = read::parse(bytes, "play session")?;
        let fields = read::object(&value, "play session")?;
        read::bind_schema(fields, &play_session_schema(), "play session")?;
        read::require_fields(
            fields,
            &[
                "areas",
                "areas_cleared",
                "log",
                "outcome",
                "position",
                "receipt_chain_head",
                "receipts",
                "route",
                "schema",
            ],
            "play session",
        )?;

        let mut route = Vec::new();
        for row in read::array(
            read::field(fields, "route", "play session")?,
            "session route",
        )? {
            let entry = read::object(row, "route row")?;
            read::require_fields(
                entry,
                &["area", "plan_digest", "semantics_digest"],
                "route row",
            )?;
            route.push(RouteRow {
                area: read::text(read::field(entry, "area", "route row")?, "route area")?
                    .to_owned(),
                plan_digest: digest(
                    read::field(entry, "plan_digest", "route row")?,
                    "plan digest",
                )?,
                semantics_digest: digest(
                    read::field(entry, "semantics_digest", "route row")?,
                    "semantics digest",
                )?,
            });
        }

        let log = read::array(read::field(fields, "log", "play session")?, "session log")?
            .iter()
            .map(PlayCommand::from_canonical)
            .collect::<PlayResult<Vec<_>>>()?;

        Ok(Self {
            route,
            log,
            receipts: read::array(
                read::field(fields, "receipts", "play session")?,
                "session receipts",
            )?
            .to_vec(),
            receipt_chain_head: digest(
                read::field(fields, "receipt_chain_head", "play session")?,
                "receipt chain head",
            )?,
            areas: read::array(
                read::field(fields, "areas", "play session")?,
                "session areas",
            )?
            .to_vec(),
            outcome: read::text(
                read::field(fields, "outcome", "play session")?,
                "session outcome",
            )?
            .to_owned(),
            areas_cleared: read::uint(
                read::field(fields, "areas_cleared", "play session")?,
                "areas_cleared",
            )?,
        })
    }
}

/// Decodes one area's plan and its executable semantics, binding the two.
///
/// The projection's bytes are hashed and required to equal the digest the
/// rendering plan published for `simulation.json`, before the projection is
/// decoded at all. That is the first of the two locks
/// `docs/review/nomos-play.md` section 10 finding 1 records; the second is the
/// kernel's own `EK0813` refusal when a persisted state names a different
/// semantics digest.
///
/// # Errors
///
/// Returns `PL0502` for a digest the plan does not publish and `PL0501` when
/// the projection cannot be reconstructed.
pub fn open(plan_bytes: &[u8], semantics_bytes: &[u8]) -> PlayResult<(AreaPlan, SimulationPlan)> {
    let plan = AreaPlan::decode(plan_bytes)?;
    let offered = Sha256Digest::of_bytes(semantics_bytes);
    if offered != plan.semantics_digest {
        return Err(PlayError::new(
            codes::SEMANTICS_DIGEST,
            format!(
                "`{}` publishes simulation.json digest {}; the bytes offered hash to {}",
                plan.area,
                plan.semantics_digest.to_hex(),
                offered.to_hex()
            ),
        ));
    }
    let semantics = SimulationPlan::from_canonical_bytes(semantics_bytes)
        .map_err(|error| PlayError::from_kernel(codes::SEMANTICS_INVALID, &error))?;
    Ok((plan, semantics))
}

fn digest(value: &CanonicalValue, label: &str) -> PlayResult<Sha256Digest> {
    let hex = read::text(value, label)?;
    Sha256Digest::from_hex(hex).ok_or_else(|| {
        PlayError::new(
            codes::DOCUMENT_VALUE,
            format!("{label} `{hex}` is not a SHA-256 hex digest"),
        )
    })
}

//! Typed causal receipts for committed runtime transactions.

use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, ClaimRef, Diagnostic, EntityId, SchemaId, StateHash};
use nomos_projection::{
    Command, CommandArgument, EventPayload, MovementDisposition, ResolvedLight, ResolvedLightFacts,
    ResolvedMovementFacts, SimulationPlan, navigation_schema, simulation_schema,
};

use crate::{PreparedTransaction, TransitionCause, TransitionStep};

/// Typed identity of one effective runtime fact.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EffectiveFactRef {
    /// Composite ground movement disposition for one entity.
    GroundMovement {
        /// Fact subject.
        entity: EntityId,
    },
    /// Effective light-emission union for one entity.
    EmitsLight {
        /// Fact subject.
        entity: EntityId,
    },
}

impl EffectiveFactRef {
    fn stable_key(&self) -> String {
        match self {
            Self::GroundMovement { entity } => format!("{entity}#ground_movement"),
            Self::EmitsLight { entity } => format!("{entity}#emits_light"),
        }
    }

    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::GroundMovement { entity } => CanonicalValue::object_declared([
                ("entity", entity.to_canonical()),
                ("kind", CanonicalValue::text("ground_movement")),
            ]),
            Self::EmitsLight { entity } => CanonicalValue::object_declared([
                ("entity", entity.to_canonical()),
                ("kind", CanonicalValue::text("emits_light")),
            ]),
        }
    }
}

/// Typed before/after value carried by a projection delta.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EffectiveFactValue {
    /// Composite ground movement disposition.
    GroundMovement(MovementDisposition),
    /// Effective light union and its active claim reasons.
    EmitsLight {
        /// Whether at least one positive claim is active.
        emitting: bool,
        /// Stable active positive claims.
        reasons: Vec<ClaimRef>,
    },
}

impl EffectiveFactValue {
    fn from_light(fact: &ResolvedLight) -> Self {
        Self::EmitsLight {
            emitting: fact.emitting(),
            reasons: fact.reasons().to_vec(),
        }
    }

    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::GroundMovement(disposition) => movement_to_canonical(disposition),
            Self::EmitsLight { emitting, reasons } => CanonicalValue::object_declared([
                ("emitting", CanonicalValue::Bool(*emitting)),
                ("kind", CanonicalValue::text("emits_light")),
                (
                    "reasons",
                    CanonicalValue::Array(reasons.iter().map(StableId::to_canonical).collect()),
                ),
            ]),
        }
    }
}

/// One typed subsystem projection delta caused by a committed command.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectionDelta {
    projection: SchemaId,
    fact: EffectiveFactRef,
    before: EffectiveFactValue,
    after: EffectiveFactValue,
}

impl ProjectionDelta {
    /// Versioned target projection.
    #[must_use]
    pub const fn projection(&self) -> &SchemaId {
        &self.projection
    }

    /// Effective fact changed by the transaction.
    #[must_use]
    pub const fn fact(&self) -> &EffectiveFactRef {
        &self.fact
    }

    /// Effective value before settlement.
    #[must_use]
    pub const fn before(&self) -> &EffectiveFactValue {
        &self.before
    }

    /// Effective value after settlement.
    #[must_use]
    pub const fn after(&self) -> &EffectiveFactValue {
        &self.after
    }

    fn stable_key(&self) -> String {
        format!("{}#{}", self.projection, self.fact.stable_key())
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("after", self.after.to_canonical()),
            ("before", self.before.to_canonical()),
            ("fact", self.fact.to_canonical()),
            ("projection", self.projection.to_canonical()),
        ])
    }
}

/// Canonical typed evidence for one committed command transaction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CausalReceipt {
    schema: SchemaId,
    command: Command,
    steps: Vec<TransitionStep>,
    movement_before: ResolvedMovementFacts,
    movement_after: ResolvedMovementFacts,
    light_before: ResolvedLightFacts,
    light_after: ResolvedLightFacts,
    projection_deltas: Vec<ProjectionDelta>,
    tick: u64,
    state_hash: StateHash,
}

impl CausalReceipt {
    pub(crate) fn from_prepared(
        plan: &SimulationPlan,
        command: Command,
        prepared: &PreparedTransaction,
        tick: u64,
        state_hash: StateHash,
    ) -> Result<Self, Diagnostic> {
        let projection_deltas = projection_deltas(
            plan,
            prepared.movement_before(),
            prepared.movement_after(),
            prepared.light_before(),
            prepared.light_after(),
        )?;
        Ok(Self {
            schema: crate::causal_receipt_schema(),
            command,
            steps: prepared.steps().to_vec(),
            movement_before: prepared.movement_before().clone(),
            movement_after: prepared.movement_after().clone(),
            light_before: prepared.light_before().clone(),
            light_after: prepared.light_after().clone(),
            projection_deltas,
            tick,
            state_hash,
        })
    }

    /// Causal-receipt schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Initiating typed command.
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }

    /// Ordered local-then-causal machine transitions.
    #[must_use]
    pub fn steps(&self) -> &[TransitionStep] {
        &self.steps
    }

    /// Effective ground movement facts before settlement.
    #[must_use]
    pub const fn movement_before(&self) -> &ResolvedMovementFacts {
        &self.movement_before
    }

    /// Effective ground movement facts after settlement.
    #[must_use]
    pub const fn movement_after(&self) -> &ResolvedMovementFacts {
        &self.movement_after
    }

    /// Effective light facts before settlement.
    #[must_use]
    pub const fn light_before(&self) -> &ResolvedLightFacts {
        &self.light_before
    }

    /// Effective light facts after settlement.
    #[must_use]
    pub const fn light_after(&self) -> &ResolvedLightFacts {
        &self.light_after
    }

    /// Typed subsystem deltas in stable target/fact order.
    #[must_use]
    pub fn projection_deltas(&self) -> &[ProjectionDelta] {
        &self.projection_deltas
    }

    /// Tick of the resulting committed snapshot.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Resulting authoritative state hash.
    #[must_use]
    pub const fn state_hash(&self) -> StateHash {
        self.state_hash
    }

    /// Canonical receipt bytes suitable for the separate run output.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            ("command", command_to_canonical(&self.command)),
            (
                "effective_facts_after",
                facts_to_canonical(&self.movement_after, &self.light_after),
            ),
            (
                "effective_facts_before",
                facts_to_canonical(&self.movement_before, &self.light_before),
            ),
            (
                "projection_deltas",
                CanonicalValue::Array(
                    self.projection_deltas
                        .iter()
                        .map(ProjectionDelta::to_canonical)
                        .collect(),
                ),
            ),
            ("schema", self.schema.to_canonical()),
            ("state_hash", CanonicalValue::text(self.state_hash.to_hex())),
            ("tick", CanonicalValue::Uint(self.tick)),
            (
                "transitions",
                CanonicalValue::Array(self.steps.iter().map(transition_to_canonical).collect()),
            ),
        ])
        .to_canonical_bytes()
    }
}

fn projection_deltas(
    plan: &SimulationPlan,
    movement_before: &ResolvedMovementFacts,
    movement_after: &ResolvedMovementFacts,
    light_before: &ResolvedLightFacts,
    light_after: &ResolvedLightFacts,
) -> Result<Vec<ProjectionDelta>, Diagnostic> {
    let mut deltas = Vec::new();
    for before in movement_before.facts() {
        let after = movement_after
            .get(before.entity())
            .ok_or_else(projection_mismatch)?;
        if before.disposition() != after {
            let fact = EffectiveFactRef::GroundMovement {
                entity: before.entity().clone(),
            };
            let before = EffectiveFactValue::GroundMovement(before.disposition().clone());
            let after = EffectiveFactValue::GroundMovement(after.clone());
            for projection in [simulation_schema(), navigation_schema()] {
                deltas.push(ProjectionDelta {
                    projection,
                    fact: fact.clone(),
                    before: before.clone(),
                    after: after.clone(),
                });
            }
        }
    }
    for before in light_before.facts() {
        let after = light_after
            .get(before.entity())
            .ok_or_else(projection_mismatch)?;
        if before != after {
            let fact = EffectiveFactRef::EmitsLight {
                entity: before.entity().clone(),
            };
            let before_value = EffectiveFactValue::from_light(before);
            let after_value = EffectiveFactValue::from_light(after);
            for consumer in plan.light_resolver().consumers() {
                deltas.push(ProjectionDelta {
                    projection: consumer.schema(),
                    fact: fact.clone(),
                    before: before_value.clone(),
                    after: after_value.clone(),
                });
            }
        }
    }
    if movement_before.facts().len() != movement_after.facts().len()
        || light_before.facts().len() != light_after.facts().len()
    {
        return Err(projection_mismatch());
    }
    deltas.sort_by_key(ProjectionDelta::stable_key);
    Ok(deltas)
}

fn command_to_canonical(command: &Command) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("action", CanonicalValue::text(command.action().as_str())),
        ("argument", argument_to_canonical(command.argument())),
        ("namespace", command.namespace().to_canonical()),
    ])
}

fn argument_to_canonical(argument: &CommandArgument) -> CanonicalValue {
    match argument {
        CommandArgument::None => {
            CanonicalValue::object_declared([("kind", CanonicalValue::text("none"))])
        }
        CommandArgument::Credential(credential) => CanonicalValue::object_declared([
            ("credential", credential.to_canonical()),
            ("kind", CanonicalValue::text("credential")),
        ]),
        CommandArgument::Event(payload) => CanonicalValue::object_declared([
            ("kind", CanonicalValue::text("event")),
            ("payload", payload_to_canonical(payload)),
        ]),
    }
}

fn transition_to_canonical(step: &TransitionStep) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("cause", cause_to_canonical(step.cause())),
        ("from", CanonicalValue::text(step.from().as_str())),
        ("namespace", step.namespace().to_canonical()),
        ("phase", CanonicalValue::text(step.phase().as_str())),
        ("to", CanonicalValue::text(step.to().as_str())),
    ])
}

fn cause_to_canonical(cause: &TransitionCause) -> CanonicalValue {
    match cause {
        TransitionCause::Command(action) => CanonicalValue::object_declared([
            ("action", CanonicalValue::text(action.as_str())),
            ("kind", CanonicalValue::text("command")),
        ]),
        TransitionCause::Event { handler, payload } => CanonicalValue::object_declared([
            ("handler", CanonicalValue::text(handler.as_str())),
            ("kind", CanonicalValue::text("event")),
            ("payload", payload_to_canonical(payload)),
        ]),
    }
}

fn payload_to_canonical(payload: &EventPayload) -> CanonicalValue {
    match payload {
        EventPayload::Damage { channel, amount } => CanonicalValue::object_declared([
            ("amount", CanonicalValue::Uint(u64::from(*amount))),
            ("channel", CanonicalValue::text(channel.as_str())),
            ("kind", CanonicalValue::text("damage")),
        ]),
    }
}

fn facts_to_canonical(
    movement: &ResolvedMovementFacts,
    light: &ResolvedLightFacts,
) -> CanonicalValue {
    CanonicalValue::object_declared([
        (
            "light",
            CanonicalValue::Array(
                light
                    .facts()
                    .iter()
                    .map(|fact| {
                        CanonicalValue::object_declared([
                            ("emitting", CanonicalValue::Bool(fact.emitting())),
                            ("entity", fact.entity().to_canonical()),
                            (
                                "reasons",
                                CanonicalValue::Array(
                                    fact.reasons().iter().map(StableId::to_canonical).collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "movement",
            CanonicalValue::Array(
                movement
                    .facts()
                    .iter()
                    .map(|fact| {
                        CanonicalValue::object_declared([
                            ("disposition", movement_to_canonical(fact.disposition())),
                            ("entity", fact.entity().to_canonical()),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn movement_to_canonical(disposition: &MovementDisposition) -> CanonicalValue {
    match disposition {
        MovementDisposition::Blocked { reasons } => CanonicalValue::object_declared([
            ("kind", CanonicalValue::text("blocked")),
            (
                "reasons",
                CanonicalValue::Array(reasons.iter().map(StableId::to_canonical).collect()),
            ),
        ]),
        MovementDisposition::Traversable { cost, reasons } => CanonicalValue::object_declared([
            ("cost", CanonicalValue::Uint(u64::from(*cost))),
            ("kind", CanonicalValue::text("traversable")),
            (
                "reasons",
                CanonicalValue::Array(reasons.iter().map(StableId::to_canonical).collect()),
            ),
        ]),
    }
}

fn projection_mismatch() -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_PROJECTION_MISMATCH,
        "effective-fact subjects changed while constructing projection deltas",
    )
}

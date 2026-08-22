//! Typed causal receipts for committed runtime transactions.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, ClaimRef, Diagnostic, EntityId, Ident, NamespaceId, SchemaId, Sha256Digest,
    StateHash,
};
use nomos_projection::{
    Command, CommandArgument, EventPayload, MovementDisposition, ResolvedLight, ResolvedLightFacts,
    ResolvedMovement, ResolvedMovementFacts, SimulationPlan, diagnostics_schema, navigation_schema,
    persistence_schema, simulation_schema,
};

use crate::state_persistence::{
    array, field, invalid, object, require_fields, schema, state_hash, text, uint,
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
        let actual_light_consumers = plan
            .light_resolver()
            .consumers()
            .iter()
            .map(|consumer| consumer.schema())
            .collect::<BTreeSet<_>>();
        let expected_light_consumers = [
            diagnostics_schema(),
            persistence_schema(),
            simulation_schema(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        if actual_light_consumers != expected_light_consumers {
            return Err(projection_mismatch());
        }
        let projection_deltas = projection_deltas(
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

    /// SHA-256 identity of the exact canonical receipt bytes.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        Sha256Digest::of_bytes(&self.to_canonical_bytes())
    }

    /// Strictly reconstructs one complete typed causal receipt.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, typed-value, ordering, or
    /// cross-field disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "causal receipt")?;
        require_fields(
            fields,
            &[
                "command",
                "effective_facts_after",
                "effective_facts_before",
                "projection_deltas",
                "schema",
                "state_hash",
                "tick",
                "transitions",
            ],
            "causal receipt",
        )?;
        let schema = schema(field(fields, "schema")?, "causal receipt schema")?;
        if schema != crate::causal_receipt_schema() {
            return Err(invalid("causal receipt names an unsupported schema"));
        }
        let command = decode_command(field(fields, "command")?)?;
        let (movement_before, light_before) =
            decode_facts(field(fields, "effective_facts_before")?)?;
        let (movement_after, light_after) = decode_facts(field(fields, "effective_facts_after")?)?;
        let projection_deltas = decode_projection_deltas(field(fields, "projection_deltas")?)?;
        let tick = uint(field(fields, "tick")?, "causal receipt tick")?;
        let state_hash = state_hash(field(fields, "state_hash")?)?;
        let steps = decode_transitions(field(fields, "transitions")?)?;
        let receipt = Self {
            schema,
            command,
            steps,
            movement_before,
            movement_after,
            light_before,
            light_after,
            projection_deltas,
            tick,
            state_hash,
        };
        receipt.validate_semantics()?;
        if receipt.to_canonical_bytes() != bytes {
            return Err(invalid(
                "causal receipt does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(receipt)
    }

    /// Canonical receipt bytes suitable for the separate run output.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
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
    }

    fn validate_semantics(&self) -> Result<(), Diagnostic> {
        if self.tick == 0 {
            return Err(invalid(
                "a committed causal receipt must advance beyond tick zero",
            ));
        }
        if matches!(self.command.argument(), CommandArgument::Event(_)) {
            return Err(invalid(
                "a causal receipt initiating command cannot carry an internal event payload",
            ));
        }
        let Some(first) = self.steps.first() else {
            return Err(invalid("a causal receipt must contain a local transition"));
        };
        if first.phase() != nomos_projection::Phase::Local
            || first.namespace() != self.command.namespace()
            || !matches!(
                first.cause(),
                TransitionCause::Command(action) if action == self.command.action()
            )
        {
            return Err(invalid(
                "the first receipt transition does not match the initiating command",
            ));
        }
        let mut latest = BTreeMap::<NamespaceId, Ident>::new();
        for (index, step) in self.steps.iter().enumerate() {
            if step.from() == step.to() {
                return Err(invalid(
                    "a receipt transition does not change machine state",
                ));
            }
            let expected_shape = if index == 0 {
                step.phase() == nomos_projection::Phase::Local
                    && matches!(step.cause(), TransitionCause::Command(_))
            } else {
                step.phase() == nomos_projection::Phase::Causal
                    && matches!(step.cause(), TransitionCause::Event { .. })
            };
            if !expected_shape {
                return Err(invalid(
                    "receipt transitions are not ordered as one local step followed by causal steps",
                ));
            }
            if latest
                .get(step.namespace())
                .is_some_and(|prior| prior != step.from())
            {
                return Err(invalid(
                    "receipt transitions disagree on namespace-local state continuity",
                ));
            }
            latest.insert(step.namespace().clone(), step.to().clone());
        }
        validate_fact_reasons(&self.movement_before, &self.light_before)?;
        validate_fact_reasons(&self.movement_after, &self.light_after)?;
        let expected = projection_deltas(
            &self.movement_before,
            &self.movement_after,
            &self.light_before,
            &self.light_after,
        )?;
        if self.projection_deltas != expected {
            return Err(invalid(
                "causal receipt projection deltas disagree with its effective facts",
            ));
        }
        Ok(())
    }
}

fn projection_deltas(
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
            for projection in [
                diagnostics_schema(),
                persistence_schema(),
                simulation_schema(),
            ] {
                deltas.push(ProjectionDelta {
                    projection,
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

pub(crate) fn command_to_canonical(command: &Command) -> CanonicalValue {
    CanonicalValue::object_declared([
        ("action", CanonicalValue::text(command.action().as_str())),
        ("argument", argument_to_canonical(command.argument())),
        ("namespace", command.namespace().to_canonical()),
    ])
}

pub(crate) fn decode_command(value: &CanonicalValue) -> Result<Command, Diagnostic> {
    let fields = object(value, "typed command")?;
    require_fields(
        fields,
        &["action", "argument", "namespace"],
        "typed command",
    )?;
    let namespace = NamespaceId::parse(text(
        field(fields, "namespace")?,
        "typed command namespace",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let action = Ident::new(text(field(fields, "action")?, "typed command action")?)
        .map_err(|error| invalid(error.message()))?;
    let argument = decode_argument(field(fields, "argument")?)?;
    Ok(Command::new(namespace, action, argument))
}

fn decode_argument(value: &CanonicalValue) -> Result<CommandArgument, Diagnostic> {
    let fields = object(value, "typed command argument")?;
    match text(field(fields, "kind")?, "typed command argument kind")? {
        "none" => {
            require_fields(fields, &["kind"], "empty command argument")?;
            Ok(CommandArgument::None)
        }
        "credential" => {
            require_fields(
                fields,
                &["credential", "kind"],
                "credential command argument",
            )?;
            let credential = nomos_core::CatalogValueId::parse(text(
                field(fields, "credential")?,
                "command credential",
            )?)
            .map_err(|error| invalid(error.message()))?;
            Ok(CommandArgument::Credential(credential))
        }
        "event" => {
            require_fields(fields, &["kind", "payload"], "event command argument")?;
            Ok(CommandArgument::Event(decode_payload(field(
                fields, "payload",
            )?)?))
        }
        _ => Err(invalid("typed command argument kind is unsupported")),
    }
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

fn decode_transitions(value: &CanonicalValue) -> Result<Vec<TransitionStep>, Diagnostic> {
    array(value, "causal receipt transitions")?
        .iter()
        .map(|row| {
            let fields = object(row, "causal receipt transition")?;
            require_fields(
                fields,
                &["cause", "from", "namespace", "phase", "to"],
                "causal receipt transition",
            )?;
            let phase = match text(field(fields, "phase")?, "transition phase")? {
                "local" => nomos_projection::Phase::Local,
                "causal" => nomos_projection::Phase::Causal,
                _ => return Err(invalid("receipt transition phase is unsupported")),
            };
            let namespace =
                NamespaceId::parse(text(field(fields, "namespace")?, "transition namespace")?)
                    .map_err(|error| invalid(error.message()))?;
            let from = Ident::new(text(field(fields, "from")?, "transition source state")?)
                .map_err(|error| invalid(error.message()))?;
            let to = Ident::new(text(field(fields, "to")?, "transition target state")?)
                .map_err(|error| invalid(error.message()))?;
            let cause = decode_cause(field(fields, "cause")?)?;
            Ok(TransitionStep::from_parts(
                phase, namespace, from, to, cause,
            ))
        })
        .collect()
}

fn decode_cause(value: &CanonicalValue) -> Result<TransitionCause, Diagnostic> {
    let fields = object(value, "transition cause")?;
    match text(field(fields, "kind")?, "transition cause kind")? {
        "command" => {
            require_fields(fields, &["action", "kind"], "command transition cause")?;
            let action = Ident::new(text(field(fields, "action")?, "cause action")?)
                .map_err(|error| invalid(error.message()))?;
            Ok(TransitionCause::Command(action))
        }
        "event" => {
            require_fields(
                fields,
                &["handler", "kind", "payload"],
                "event transition cause",
            )?;
            let handler = Ident::new(text(field(fields, "handler")?, "event handler")?)
                .map_err(|error| invalid(error.message()))?;
            Ok(TransitionCause::Event {
                handler,
                payload: decode_payload(field(fields, "payload")?)?,
            })
        }
        _ => Err(invalid("transition cause kind is unsupported")),
    }
}

fn decode_payload(value: &CanonicalValue) -> Result<EventPayload, Diagnostic> {
    let fields = object(value, "event payload")?;
    match text(field(fields, "kind")?, "event payload kind")? {
        "damage" => {
            require_fields(fields, &["amount", "channel", "kind"], "damage payload")?;
            let amount = u32::try_from(uint(field(fields, "amount")?, "damage amount")?)
                .map_err(|_| invalid("damage amount exceeds u32"))?;
            let channel = Ident::new(text(field(fields, "channel")?, "damage channel")?)
                .map_err(|error| invalid(error.message()))?;
            Ok(EventPayload::Damage { channel, amount })
        }
        _ => Err(invalid("event payload kind is unsupported")),
    }
}

fn decode_facts(
    value: &CanonicalValue,
) -> Result<(ResolvedMovementFacts, ResolvedLightFacts), Diagnostic> {
    let fields = object(value, "effective facts")?;
    require_fields(fields, &["light", "movement"], "effective facts")?;
    let movement = array(field(fields, "movement")?, "movement facts")?
        .iter()
        .map(|row| {
            let fields = object(row, "movement fact")?;
            require_fields(fields, &["disposition", "entity"], "movement fact")?;
            let entity = entity(field(fields, "entity")?, "movement fact entity")?;
            let disposition = decode_movement(field(fields, "disposition")?)?;
            Ok(ResolvedMovement::new(entity, disposition))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    let light = array(field(fields, "light")?, "light facts")?
        .iter()
        .map(|row| {
            let fields = object(row, "light fact")?;
            require_fields(fields, &["emitting", "entity", "reasons"], "light fact")?;
            ResolvedLight::new(
                entity(field(fields, "entity")?, "light fact entity")?,
                boolean(field(fields, "emitting")?, "light emitting value")?,
                claim_refs(field(fields, "reasons")?, "light reasons")?,
            )
            .map_err(|error| invalid(error.message()))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    Ok((
        ResolvedMovementFacts::new(movement).map_err(|error| invalid(error.message()))?,
        ResolvedLightFacts::new(light).map_err(|error| invalid(error.message()))?,
    ))
}

fn decode_movement(value: &CanonicalValue) -> Result<MovementDisposition, Diagnostic> {
    let fields = object(value, "movement disposition")?;
    match text(field(fields, "kind")?, "movement disposition kind")? {
        "blocked" => {
            require_fields(fields, &["kind", "reasons"], "blocked disposition")?;
            MovementDisposition::blocked(claim_refs(field(fields, "reasons")?, "movement reasons")?)
                .map_err(|error| invalid(error.message()))
        }
        "traversable" => {
            require_fields(
                fields,
                &["cost", "kind", "reasons"],
                "traversable disposition",
            )?;
            let cost = u32::try_from(uint(field(fields, "cost")?, "traversal cost")?)
                .map_err(|_| invalid("traversal cost exceeds u32"))?;
            MovementDisposition::traversable(
                cost,
                claim_refs(field(fields, "reasons")?, "movement reasons")?,
            )
            .map_err(|error| invalid(error.message()))
        }
        _ => Err(invalid("movement disposition kind is unsupported")),
    }
}

fn decode_projection_deltas(value: &CanonicalValue) -> Result<Vec<ProjectionDelta>, Diagnostic> {
    array(value, "projection deltas")?
        .iter()
        .map(|row| {
            let fields = object(row, "projection delta")?;
            require_fields(
                fields,
                &["after", "before", "fact", "projection"],
                "projection delta",
            )?;
            let fact = decode_fact_ref(field(fields, "fact")?)?;
            let before = decode_fact_value(field(fields, "before")?)?;
            let after = decode_fact_value(field(fields, "after")?)?;
            if !fact_matches_value(&fact, &before) || !fact_matches_value(&fact, &after) {
                return Err(invalid(
                    "projection delta fact identity and value variants disagree",
                ));
            }
            Ok(ProjectionDelta {
                projection: schema(field(fields, "projection")?, "projection delta schema")?,
                fact,
                before,
                after,
            })
        })
        .collect()
}

fn decode_fact_ref(value: &CanonicalValue) -> Result<EffectiveFactRef, Diagnostic> {
    let fields = object(value, "effective fact identity")?;
    require_fields(fields, &["entity", "kind"], "effective fact identity")?;
    let entity = entity(field(fields, "entity")?, "effective fact entity")?;
    match text(field(fields, "kind")?, "effective fact kind")? {
        "ground_movement" => Ok(EffectiveFactRef::GroundMovement { entity }),
        "emits_light" => Ok(EffectiveFactRef::EmitsLight { entity }),
        _ => Err(invalid("effective fact identity kind is unsupported")),
    }
}

fn decode_fact_value(value: &CanonicalValue) -> Result<EffectiveFactValue, Diagnostic> {
    let fields = object(value, "effective fact value")?;
    match text(field(fields, "kind")?, "effective fact value kind")? {
        "blocked" | "traversable" => {
            Ok(EffectiveFactValue::GroundMovement(decode_movement(value)?))
        }
        "emits_light" => {
            require_fields(
                fields,
                &["emitting", "kind", "reasons"],
                "light effective value",
            )?;
            let emitting = boolean(field(fields, "emitting")?, "light effective value")?;
            let reasons = claim_refs(field(fields, "reasons")?, "light effective reasons")?;
            if emitting != !reasons.is_empty() {
                return Err(invalid(
                    "effective light value disagrees with its active reasons",
                ));
            }
            Ok(EffectiveFactValue::EmitsLight { emitting, reasons })
        }
        _ => Err(invalid("effective fact value kind is unsupported")),
    }
}

fn fact_matches_value(fact: &EffectiveFactRef, value: &EffectiveFactValue) -> bool {
    matches!(
        (fact, value),
        (
            EffectiveFactRef::GroundMovement { .. },
            EffectiveFactValue::GroundMovement(_)
        ) | (
            EffectiveFactRef::EmitsLight { .. },
            EffectiveFactValue::EmitsLight { .. }
        )
    )
}

fn validate_fact_reasons(
    movement: &ResolvedMovementFacts,
    light: &ResolvedLightFacts,
) -> Result<(), Diagnostic> {
    for fact in movement.facts() {
        if fact
            .disposition()
            .reasons()
            .iter()
            .any(|reason| reason.namespace().entity() != fact.entity())
        {
            return Err(invalid(
                "movement fact carries a reason owned by a different entity",
            ));
        }
    }
    for fact in light.facts() {
        if fact
            .reasons()
            .iter()
            .any(|reason| reason.namespace().entity() != fact.entity())
        {
            return Err(invalid(
                "light fact carries a reason owned by a different entity",
            ));
        }
    }
    Ok(())
}

fn claim_refs(value: &CanonicalValue, label: &str) -> Result<Vec<ClaimRef>, Diagnostic> {
    array(value, label)?
        .iter()
        .map(|value| ClaimRef::parse(text(value, label)?).map_err(|error| invalid(error.message())))
        .collect()
}

fn entity(value: &CanonicalValue, label: &str) -> Result<EntityId, Diagnostic> {
    EntityId::parse(text(value, label)?).map_err(|error| invalid(error.message()))
}

fn boolean(value: &CanonicalValue, label: &str) -> Result<bool, Diagnostic> {
    let CanonicalValue::Bool(value) = value else {
        return Err(invalid(format!("{label} is not a boolean")));
    };
    Ok(*value)
}

fn projection_mismatch() -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_PROJECTION_MISMATCH,
        "effective-fact subjects changed while constructing projection deltas",
    )
}

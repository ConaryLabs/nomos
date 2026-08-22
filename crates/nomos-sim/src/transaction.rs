//! Immutable runtime state and atomic transaction preparation.

use std::collections::{BTreeMap, VecDeque};

use nomos_core::canonical::keyed_array;
use nomos_core::id::StableId;
use nomos_core::{CanonicalValue, Diagnostic, EntityId, Ident, NamespaceId, SchemaId, StateHash};
use nomos_projection::{
    CausalEdge, Command, CommandArgument, CommandRequirement, EventPayload, MachineDefinition,
    Phase, ResolvedLightFacts, ResolvedMovementFacts, RuntimeBinding, SimulationPlan,
};

use crate::{resolve_light, resolve_movement};

/// Default last-defense transition budget for one prepared transaction.
pub const DEFAULT_TRANSITION_BUDGET: usize = 64;

/// Immutable namespace-machine state consumed by transaction preparation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimulationState {
    schema: SchemaId,
    tick: u64,
    entities: Vec<RuntimeEntityState>,
    machines: BTreeMap<NamespaceId, Ident>,
}

/// One authoritative runtime entity identity and lattice binding.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RuntimeEntityState {
    id: EntityId,
    binding: RuntimeBinding,
}

impl RuntimeEntityState {
    pub(crate) fn from_parts(id: EntityId, binding: RuntimeBinding) -> Self {
        Self { id, binding }
    }

    /// Stable entity ID.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Authoritative lattice binding.
    #[must_use]
    pub const fn binding(&self) -> &RuntimeBinding {
        &self.binding
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("binding", self.binding.to_canonical()),
            ("id", self.id.to_canonical()),
        ])
    }
}

impl SimulationState {
    pub(crate) fn from_parts(
        schema: SchemaId,
        tick: u64,
        entities: Vec<RuntimeEntityState>,
        machines: BTreeMap<NamespaceId, Ident>,
    ) -> Self {
        Self {
            schema,
            tick,
            entities,
            machines,
        }
    }

    /// Initializes every runtime machine from its projected initial state.
    ///
    /// # Errors
    ///
    /// Returns `EK0809` when a malicious projection names an initial state not
    /// present in its machine definition.
    pub fn initialize(plan: &SimulationPlan) -> Result<Self, Diagnostic> {
        let mut machines = BTreeMap::new();
        for machine in plan.machines() {
            if !machine.states().contains(machine.initial()) {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                    format!(
                        "initial state `{}` is absent from machine `{}`",
                        machine.initial(),
                        machine.namespace()
                    ),
                ));
            }
            for (role, state) in machine
                .commands()
                .iter()
                .flat_map(|transition| {
                    [
                        ("command source", transition.source()),
                        ("command target", transition.target()),
                    ]
                })
                .chain(machine.handlers().iter().flat_map(|handler| {
                    [
                        ("handler source", handler.source()),
                        ("handler target", handler.target()),
                    ]
                }))
            {
                if !machine.states().contains(state) {
                    return Err(Diagnostic::new(
                        nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                        format!(
                            "{role} state `{state}` is absent from machine `{}`",
                            machine.namespace()
                        ),
                    ));
                }
            }
            if machines
                .insert(machine.namespace().clone(), machine.initial().clone())
                .is_some()
            {
                return Err(Diagnostic::new(
                    nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                    format!(
                        "machine namespace `{}` occurs more than once",
                        machine.namespace()
                    ),
                ));
            }
        }
        let projected_namespaces = plan
            .entities()
            .iter()
            .flat_map(|entity| entity.machines().iter().cloned())
            .collect::<std::collections::BTreeSet<_>>();
        let machine_namespaces = machines
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if projected_namespaces != machine_namespaces {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                "projected entity machine ownership does not match the simulation machine set",
            ));
        }
        let entities = plan
            .entities()
            .iter()
            .map(|entity| RuntimeEntityState {
                id: entity.id().clone(),
                binding: entity.binding().clone(),
            })
            .collect();
        Ok(Self {
            schema: crate::runtime_state_schema(),
            tick: 0,
            entities,
            machines,
        })
    }

    /// Runtime-state schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Authoritative deterministic tick.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Authoritative entities in stable ID order.
    #[must_use]
    pub fn entities(&self) -> &[RuntimeEntityState] {
        &self.entities
    }

    /// Current state of one namespace machine.
    #[must_use]
    pub fn machine(&self, namespace: &NamespaceId) -> Option<&Ident> {
        self.machines.get(namespace)
    }

    /// Canonical bytes useful for proving an input state remained unchanged.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        self.to_canonical().to_canonical_bytes()
    }

    /// Reconstructs one exact canonical state and validates it against its plan.
    ///
    /// # Errors
    ///
    /// Returns a canonical, persisted-runtime, or plan disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8], plan: &SimulationPlan) -> Result<Self, Diagnostic> {
        crate::state_persistence::decode_state(bytes, plan)
    }

    /// Validates this snapshot against the supplied runtime semantics.
    ///
    /// # Errors
    ///
    /// Returns `EK0809` when identities, bindings, namespaces, or states differ.
    pub fn validate_against(&self, plan: &SimulationPlan) -> Result<(), Diagnostic> {
        validate_current_state(plan, self)
    }

    /// SHA-256 identity of the canonical runtime-state envelope only.
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        StateHash::of_envelope(&self.to_canonical())
    }

    /// Verifies a recorded state hash against this exact snapshot.
    ///
    /// # Errors
    ///
    /// Returns `EK0810` when the digest does not match.
    pub fn verify_hash(&self, expected: StateHash) -> Result<(), Diagnostic> {
        let actual = self.state_hash();
        if actual != expected {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_STATE_HASH_MISMATCH,
                format!("recorded state hash `{expected}` does not match `{actual}`"),
            ));
        }
        Ok(())
    }

    pub(crate) fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("counters", CanonicalValue::Array(Vec::new())),
            (
                "entities",
                keyed_array(
                    self.entities
                        .iter()
                        .map(|entity| (entity.id.clone(), entity.to_canonical())),
                )
                .expect("SimulationState has unique entity keys"),
            ),
            (
                "machines",
                keyed_array(self.machines.iter().map(|(namespace, state)| {
                    (
                        namespace.clone(),
                        CanonicalValue::object_declared([
                            ("namespace", namespace.to_canonical()),
                            ("state", CanonicalValue::text(state.as_str())),
                        ]),
                    )
                }))
                .expect("SimulationState has unique namespace keys"),
            ),
            ("scheduled_events", CanonicalValue::Array(Vec::new())),
            ("schema", self.schema.to_canonical()),
            ("tick", CanonicalValue::Uint(self.tick)),
        ])
    }
}

/// Cause recorded for one prepared transition step.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TransitionCause {
    /// The initiating external command.
    Command(Ident),
    /// An internal typed event handled by the target machine.
    Event {
        /// Target-owned handler name.
        handler: Ident,
        /// Typed event payload.
        payload: EventPayload,
    },
}

/// One ordered machine-local transition staged by preparation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransitionStep {
    phase: Phase,
    namespace: NamespaceId,
    from: Ident,
    to: Ident,
    cause: TransitionCause,
}

impl TransitionStep {
    /// Settlement phase.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// Machine that owns this state change.
    #[must_use]
    pub const fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// State before this local transition.
    #[must_use]
    pub const fn from(&self) -> &Ident {
        &self.from
    }

    /// Staged state after this local transition.
    #[must_use]
    pub const fn to(&self) -> &Ident {
        &self.to
    }

    /// Typed cause of this local transition.
    #[must_use]
    pub const fn cause(&self) -> &TransitionCause {
        &self.cause
    }
}

/// Complete staged result of one command, not yet committed as a snapshot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PreparedTransaction {
    after: SimulationState,
    steps: Vec<TransitionStep>,
    movement_before: ResolvedMovementFacts,
    movement_after: ResolvedMovementFacts,
    light_before: ResolvedLightFacts,
    light_after: ResolvedLightFacts,
}

impl PreparedTransaction {
    /// Staged after-state.
    #[must_use]
    pub const fn after(&self) -> &SimulationState {
        &self.after
    }

    /// Deterministic local-then-causal transition steps.
    #[must_use]
    pub fn steps(&self) -> &[TransitionStep] {
        &self.steps
    }

    /// Effective movement facts before the initiating local transition.
    #[must_use]
    pub const fn movement_before(&self) -> &ResolvedMovementFacts {
        &self.movement_before
    }

    /// Effective movement facts after local and causal settlement.
    #[must_use]
    pub const fn movement_after(&self) -> &ResolvedMovementFacts {
        &self.movement_after
    }

    /// Effective light facts before the initiating local transition.
    #[must_use]
    pub const fn light_before(&self) -> &ResolvedLightFacts {
        &self.light_before
    }

    /// Effective light facts after local and causal settlement.
    #[must_use]
    pub const fn light_after(&self) -> &ResolvedLightFacts {
        &self.light_after
    }

    /// Consumes the preparation and returns its staged state.
    #[must_use]
    pub fn into_after(self) -> SimulationState {
        self.after
    }
}

#[derive(Clone, Debug)]
struct PendingEvent {
    edge: CausalEdge,
}

/// Prepares one command with the default transition budget.
///
/// The input state is borrowed immutably. Any failure discards the private
/// staged clone, so the caller never observes a partial result.
///
/// # Errors
///
/// Returns a stable `EK08xx` diagnostic for invalid commands, state, event
/// targets, handlers, arguments, or a breached transition budget.
pub fn prepare_transaction(
    plan: &SimulationPlan,
    current: &SimulationState,
    command: &Command,
) -> Result<PreparedTransaction, Diagnostic> {
    prepare_transaction_with_budget(plan, current, command, DEFAULT_TRANSITION_BUDGET)
}

/// Prepares one command with an explicit last-defense transition budget.
///
/// # Errors
///
/// Returns the same stable diagnostics as [`prepare_transaction`], including
/// `EK0808` when the budget would be exceeded.
pub fn prepare_transaction_with_budget(
    plan: &SimulationPlan,
    current: &SimulationState,
    command: &Command,
    budget: usize,
) -> Result<PreparedTransaction, Diagnostic> {
    validate_current_state(plan, current)?;
    let movement_before = resolve_movement(plan, current)?;
    let light_before = resolve_light(plan, current)?;
    let machine = find_machine(plan, command.namespace()).ok_or_else(|| {
        Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_TARGET_MISSING,
            format!("command target `{}` does not exist", command.namespace()),
        )
    })?;
    let state = current.machine(command.namespace()).ok_or_else(|| {
        Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_TARGET_MISSING,
            format!(
                "command target `{}` is absent from current state",
                command.namespace()
            ),
        )
    })?;
    let action_transitions: Vec<_> = machine
        .commands()
        .iter()
        .filter(|transition| transition.action() == command.action())
        .collect();
    if action_transitions.is_empty() {
        if machine
            .handlers()
            .iter()
            .any(|handler| handler.name() == command.action())
        {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_INTERNAL_HANDLER_EXTERNAL,
                format!(
                    "`{}.{}` is an internal handler and cannot be invoked externally",
                    command.namespace().local_name(),
                    command.action()
                ),
            ));
        }
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_ACTION_UNDECLARED,
            format!(
                "command `{}.{}` is not declared",
                command.namespace().local_name(),
                command.action()
            ),
        ));
    }
    let transition = action_transitions
        .into_iter()
        .find(|transition| transition.source() == state)
        .ok_or_else(|| illegal_source(command.namespace(), command.action(), state))?;
    validate_argument(transition.requirement(), command.argument())?;
    if budget == 0 {
        return Err(budget_exceeded(budget));
    }

    let mut after = current.clone();
    after
        .machines
        .insert(command.namespace().clone(), transition.target().clone());
    let mut steps = vec![TransitionStep {
        phase: Phase::Local,
        namespace: command.namespace().clone(),
        from: state.clone(),
        to: transition.target().clone(),
        cause: TransitionCause::Command(command.action().clone()),
    }];
    let mut pending = events_for_entry(plan, command.namespace(), transition.target());

    while let Some(PendingEvent { edge }) = pending.pop_front() {
        if steps.len() >= budget {
            return Err(budget_exceeded(budget));
        }
        if edge.phase() != Phase::Causal {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                "a causal edge carries a non-causal settlement phase",
            ));
        }
        let target = find_machine(plan, edge.target_namespace()).ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_EVENT_TARGET_MISSING,
                format!(
                    "causal event target `{}` does not exist",
                    edge.target_namespace()
                ),
            )
        })?;
        let target_state = after.machine(edge.target_namespace()).ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_EVENT_TARGET_MISSING,
                format!(
                    "causal event target `{}` is absent from current state",
                    edge.target_namespace()
                ),
            )
        })?;
        let matching_handlers: Vec<_> = target
            .handlers()
            .iter()
            .filter(|handler| {
                handler.name() == edge.target_handler() && handler.payload() == edge.payload()
            })
            .collect();
        if matching_handlers.is_empty() {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_EVENT_HANDLER_MISSING,
                format!(
                    "event target `{}` has no matching `{}` handler",
                    edge.target_namespace(),
                    edge.target_handler()
                ),
            ));
        }
        let handler = matching_handlers
            .into_iter()
            .find(|handler| handler.source() == target_state)
            .ok_or_else(|| {
                illegal_source(edge.target_namespace(), edge.target_handler(), target_state)
            })?;
        let from = target_state.clone();
        after
            .machines
            .insert(edge.target_namespace().clone(), handler.target().clone());
        steps.push(TransitionStep {
            phase: edge.phase(),
            namespace: edge.target_namespace().clone(),
            from,
            to: handler.target().clone(),
            cause: TransitionCause::Event {
                handler: edge.target_handler().clone(),
                payload: edge.payload().clone(),
            },
        });
        pending.extend(events_for_entry(
            plan,
            edge.target_namespace(),
            handler.target(),
        ));
    }

    let movement_after = resolve_movement(plan, &after)?;
    let light_after = resolve_light(plan, &after)?;
    Ok(PreparedTransaction {
        after,
        steps,
        movement_before,
        movement_after,
        light_before,
        light_after,
    })
}

pub(crate) fn validate_current_state(
    plan: &SimulationPlan,
    current: &SimulationState,
) -> Result<(), Diagnostic> {
    if current.schema != crate::runtime_state_schema() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
            "current state does not name the supported runtime-state schema",
        ));
    }
    if current.entities.len() != plan.entities().len()
        || current
            .entities
            .iter()
            .zip(plan.entities())
            .any(|(state, projected)| {
                state.id() != projected.id() || state.binding() != projected.binding()
            })
    {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
            "current state entity identities or lattice bindings do not match the simulation plan",
        ));
    }
    if current.machines.len() != plan.machines().len() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
            "current state machine set does not match the simulation plan",
        ));
    }
    for machine in plan.machines() {
        let Some(state) = current.machine(machine.namespace()) else {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                format!("current state is missing machine `{}`", machine.namespace()),
            ));
        };
        if !machine.states().contains(state) {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                format!(
                    "current state `{state}` is invalid for machine `{}`",
                    machine.namespace()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_argument(
    requirement: &CommandRequirement,
    argument: &CommandArgument,
) -> Result<(), Diagnostic> {
    let valid = match (requirement, argument) {
        (CommandRequirement::None, CommandArgument::None) => true,
        (CommandRequirement::Credential(required), CommandArgument::Credential(supplied)) => {
            required == supplied
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_ARGUMENT_MISMATCH,
            "command argument does not match the compiled input requirement",
        ))
    }
}

fn find_machine<'a>(
    plan: &'a SimulationPlan,
    namespace: &NamespaceId,
) -> Option<&'a MachineDefinition> {
    plan.machines()
        .binary_search_by(|machine| machine.namespace().cmp(namespace))
        .ok()
        .map(|index| &plan.machines()[index])
}

fn events_for_entry(
    plan: &SimulationPlan,
    namespace: &NamespaceId,
    state: &Ident,
) -> VecDeque<PendingEvent> {
    plan.causal_edges()
        .iter()
        .filter(|edge| edge.source_namespace() == namespace && edge.entered_state() == state)
        .cloned()
        .map(|edge| PendingEvent { edge })
        .collect()
}

fn illegal_source(namespace: &NamespaceId, action: &Ident, state: &Ident) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_SOURCE_STATE_ILLEGAL,
        format!("`{namespace}.{action}` is illegal while the machine is `{state}`"),
    )
}

fn budget_exceeded(budget: usize) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_TRANSITION_BUDGET,
        format!("transaction exceeded its transition budget of {budget}"),
    )
}

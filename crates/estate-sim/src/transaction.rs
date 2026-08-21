//! Immutable runtime state and atomic transaction preparation.

use std::collections::{BTreeMap, VecDeque};

use estate_core::canonical::keyed_array;
use estate_core::id::StableId;
use estate_core::{CanonicalValue, Diagnostic, Ident, NamespaceId};
use estate_projection::{
    CausalEdge, Command, CommandArgument, CommandRequirement, EventPayload, MachineDefinition,
    Phase, SimulationPlan,
};

/// Default last-defense transition budget for one prepared transaction.
pub const DEFAULT_TRANSITION_BUDGET: usize = 64;

/// Immutable namespace-machine state consumed by transaction preparation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimulationState {
    machines: BTreeMap<NamespaceId, Ident>,
}

impl SimulationState {
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
                    estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
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
                        estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
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
                    estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                    format!(
                        "machine namespace `{}` occurs more than once",
                        machine.namespace()
                    ),
                ));
            }
        }
        Ok(Self { machines })
    }

    /// Current state of one namespace machine.
    #[must_use]
    pub fn machine(&self, namespace: &NamespaceId) -> Option<&Ident> {
        self.machines.get(namespace)
    }

    /// Canonical bytes useful for proving an input state remained unchanged.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        keyed_array(self.machines.iter().map(|(namespace, state)| {
            (
                namespace.clone(),
                CanonicalValue::object_declared([
                    ("namespace", namespace.to_canonical()),
                    ("state", CanonicalValue::text(state.as_str())),
                ]),
            )
        }))
        .expect("SimulationState has unique namespace keys")
        .to_canonical_bytes()
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
    let machine = find_machine(plan, command.namespace()).ok_or_else(|| {
        Diagnostic::new(
            estate_core::diagnostic::codes::RUNTIME_TARGET_MISSING,
            format!("command target `{}` does not exist", command.namespace()),
        )
    })?;
    let state = current.machine(command.namespace()).ok_or_else(|| {
        Diagnostic::new(
            estate_core::diagnostic::codes::RUNTIME_TARGET_MISSING,
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
                estate_core::diagnostic::codes::RUNTIME_INTERNAL_HANDLER_EXTERNAL,
                format!(
                    "`{}.{}` is an internal handler and cannot be invoked externally",
                    command.namespace().local_name(),
                    command.action()
                ),
            ));
        }
        return Err(Diagnostic::new(
            estate_core::diagnostic::codes::RUNTIME_ACTION_UNDECLARED,
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
                estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                "a causal edge carries a non-causal settlement phase",
            ));
        }
        let target = find_machine(plan, edge.target_namespace()).ok_or_else(|| {
            Diagnostic::new(
                estate_core::diagnostic::codes::RUNTIME_EVENT_TARGET_MISSING,
                format!(
                    "causal event target `{}` does not exist",
                    edge.target_namespace()
                ),
            )
        })?;
        let target_state = after.machine(edge.target_namespace()).ok_or_else(|| {
            Diagnostic::new(
                estate_core::diagnostic::codes::RUNTIME_EVENT_TARGET_MISSING,
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
                estate_core::diagnostic::codes::RUNTIME_EVENT_HANDLER_MISSING,
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

    Ok(PreparedTransaction { after, steps })
}

fn validate_current_state(
    plan: &SimulationPlan,
    current: &SimulationState,
) -> Result<(), Diagnostic> {
    if current.machines.len() != plan.machines().len() {
        return Err(Diagnostic::new(
            estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
            "current state machine set does not match the simulation plan",
        ));
    }
    for machine in plan.machines() {
        let Some(state) = current.machine(machine.namespace()) else {
            return Err(Diagnostic::new(
                estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
                format!("current state is missing machine `{}`", machine.namespace()),
            ));
        };
        if !machine.states().contains(state) {
            return Err(Diagnostic::new(
                estate_core::diagnostic::codes::RUNTIME_STATE_INVALID,
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
            estate_core::diagnostic::codes::RUNTIME_ARGUMENT_MISMATCH,
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
        estate_core::diagnostic::codes::RUNTIME_SOURCE_STATE_ILLEGAL,
        format!("`{namespace}.{action}` is illegal while the machine is `{state}`"),
    )
}

fn budget_exceeded(budget: usize) -> Diagnostic {
    Diagnostic::new(
        estate_core::diagnostic::codes::RUNTIME_TRANSITION_BUDGET,
        format!("transaction exceeded its transition budget of {budget}"),
    )
}

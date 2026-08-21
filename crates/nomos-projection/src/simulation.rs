//! Runtime-facing simulation plan schema.

use std::collections::BTreeSet;

use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, CatalogValueId, Diagnostic, Ident, NamespaceId, RepairClass, SchemaId,
};

use crate::{MovementResolverPlan, simulation_schema};

/// An external command's compiled input requirement.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CommandRequirement {
    /// The command takes no argument.
    None,
    /// The command requires this exact resolved credential.
    Credential(CatalogValueId),
}

impl CommandRequirement {
    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::None => CanonicalValue::object_declared([("kind", CanonicalValue::text("none"))]),
            Self::Credential(credential) => CanonicalValue::object_declared([
                ("credential", credential.to_canonical()),
                ("kind", CanonicalValue::text("credential")),
            ]),
        }
    }
}

/// A typed internal event payload.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EventPayload {
    /// Damage applied through a target-owned handler.
    Damage {
        /// Damage channel.
        channel: Ident,
        /// Non-negative damage amount.
        amount: u32,
    },
}

impl EventPayload {
    fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::Damage { channel, amount } => CanonicalValue::object_declared([
                ("amount", CanonicalValue::Uint(u64::from(*amount))),
                ("channel", CanonicalValue::text(channel.as_str())),
                ("kind", CanonicalValue::text("damage")),
            ]),
        }
    }
}

/// One external namespace-local command transition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandTransition {
    action: Ident,
    requirement: CommandRequirement,
    source: Ident,
    target: Ident,
}

impl CommandTransition {
    /// Builds an external command transition.
    #[must_use]
    pub fn new(
        action: Ident,
        requirement: CommandRequirement,
        source: Ident,
        target: Ident,
    ) -> Self {
        Self {
            action,
            requirement,
            source,
            target,
        }
    }

    /// Namespace-local action name.
    #[must_use]
    pub const fn action(&self) -> &Ident {
        &self.action
    }

    /// Compiled input requirement.
    #[must_use]
    pub const fn requirement(&self) -> &CommandRequirement {
        &self.requirement
    }

    /// Required current state.
    #[must_use]
    pub const fn source(&self) -> &Ident {
        &self.source
    }

    /// State staged by this transition.
    #[must_use]
    pub const fn target(&self) -> &Ident {
        &self.target
    }

    fn stable_key(&self) -> String {
        format!("{}#{}", self.action, self.source)
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("action", CanonicalValue::text(self.action.as_str())),
            ("requirement", self.requirement.to_canonical()),
            ("source", CanonicalValue::text(self.source.as_str())),
            ("target", CanonicalValue::text(self.target.as_str())),
        ])
    }
}

/// One target-owned internal event handler.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EventHandler {
    name: Ident,
    payload: EventPayload,
    source: Ident,
    target: Ident,
}

impl EventHandler {
    /// Builds an internal event handler.
    #[must_use]
    pub fn new(name: Ident, payload: EventPayload, source: Ident, target: Ident) -> Self {
        Self {
            name,
            payload,
            source,
            target,
        }
    }

    /// Namespace-local handler name.
    #[must_use]
    pub const fn name(&self) -> &Ident {
        &self.name
    }

    /// Exact typed payload accepted by this Gate K handler.
    #[must_use]
    pub const fn payload(&self) -> &EventPayload {
        &self.payload
    }

    /// Required current state.
    #[must_use]
    pub const fn source(&self) -> &Ident {
        &self.source
    }

    /// State staged by this handler.
    #[must_use]
    pub const fn target(&self) -> &Ident {
        &self.target
    }

    fn stable_key(&self) -> String {
        format!("{}#{}", self.name, self.source)
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("name", CanonicalValue::text(self.name.as_str())),
            ("payload", self.payload.to_canonical()),
            ("source", CanonicalValue::text(self.source.as_str())),
            ("target", CanonicalValue::text(self.target.as_str())),
        ])
    }
}

/// One runtime machine definition with no authoring or construction-IR data.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MachineDefinition {
    namespace: NamespaceId,
    states: Vec<Ident>,
    initial: Ident,
    commands: Vec<CommandTransition>,
    handlers: Vec<EventHandler>,
}

impl MachineDefinition {
    /// Builds a runtime machine and imposes stable transition ordering.
    ///
    /// # Errors
    ///
    /// Returns `EK0704` when command or handler signatures repeat.
    pub fn new(
        namespace: NamespaceId,
        mut states: Vec<Ident>,
        initial: Ident,
        mut commands: Vec<CommandTransition>,
        mut handlers: Vec<EventHandler>,
    ) -> Result<Self, Diagnostic> {
        require_unique(
            states.iter().cloned(),
            "machine state identity",
            nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
        )?;
        require_unique(
            commands.iter().map(CommandTransition::stable_key),
            "command transition signature",
            nomos_core::diagnostic::codes::TRANSITION_SIGNATURE_DUPLICATE,
        )?;
        require_unique(
            handlers.iter().map(EventHandler::stable_key),
            "event-handler transition signature",
            nomos_core::diagnostic::codes::TRANSITION_SIGNATURE_DUPLICATE,
        )?;
        states.sort();
        commands.sort_by_key(CommandTransition::stable_key);
        handlers.sort_by_key(EventHandler::stable_key);
        Ok(Self {
            namespace,
            states,
            initial,
            commands,
            handlers,
        })
    }

    /// Stable machine namespace.
    #[must_use]
    pub const fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// Legal states in stable order.
    #[must_use]
    pub fn states(&self) -> &[Ident] {
        &self.states
    }

    /// Initial state.
    #[must_use]
    pub const fn initial(&self) -> &Ident {
        &self.initial
    }

    /// External command transitions in stable signature order.
    #[must_use]
    pub fn commands(&self) -> &[CommandTransition] {
        &self.commands
    }

    /// Internal handlers in stable signature order.
    #[must_use]
    pub fn handlers(&self) -> &[EventHandler] {
        &self.handlers
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "commands",
                CanonicalValue::Array(
                    self.commands
                        .iter()
                        .map(CommandTransition::to_canonical)
                        .collect(),
                ),
            ),
            (
                "handlers",
                CanonicalValue::Array(
                    self.handlers
                        .iter()
                        .map(EventHandler::to_canonical)
                        .collect(),
                ),
            ),
            ("initial", CanonicalValue::text(self.initial.as_str())),
            ("namespace", self.namespace.to_canonical()),
            (
                "states",
                CanonicalValue::Array(
                    self.states
                        .iter()
                        .map(|state| CanonicalValue::text(state.as_str()))
                        .collect(),
                ),
            ),
        ])
    }
}

/// Deterministic transition settlement phase.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Phase {
    /// The initiating namespace-local command transition.
    Local,
    /// A target-owned transition caused by a typed event.
    Causal,
}

impl Phase {
    /// Stable phase ordinal used before semantic tie-breakers.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Local => 0,
            Self::Causal => 1,
        }
    }

    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Causal => "causal",
        }
    }
}

/// One runtime causal edge selected by state entry.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CausalEdge {
    source_namespace: NamespaceId,
    entered_state: Ident,
    phase: Phase,
    target_namespace: NamespaceId,
    target_handler: Ident,
    payload: EventPayload,
}

impl CausalEdge {
    /// Builds one runtime causal edge.
    #[must_use]
    pub fn new(
        source_namespace: NamespaceId,
        entered_state: Ident,
        phase: Phase,
        target_namespace: NamespaceId,
        target_handler: Ident,
        payload: EventPayload,
    ) -> Self {
        Self {
            source_namespace,
            entered_state,
            phase,
            target_namespace,
            target_handler,
            payload,
        }
    }

    /// Source machine namespace.
    #[must_use]
    pub const fn source_namespace(&self) -> &NamespaceId {
        &self.source_namespace
    }

    /// Newly entered state.
    #[must_use]
    pub const fn entered_state(&self) -> &Ident {
        &self.entered_state
    }

    /// Settlement phase.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// Target machine namespace.
    #[must_use]
    pub const fn target_namespace(&self) -> &NamespaceId {
        &self.target_namespace
    }

    /// Target-owned handler name.
    #[must_use]
    pub const fn target_handler(&self) -> &Ident {
        &self.target_handler
    }

    /// Typed event payload.
    #[must_use]
    pub const fn payload(&self) -> &EventPayload {
        &self.payload
    }

    fn stable_key(&self) -> String {
        format!(
            "{:03}#{}#{}#{}#{}",
            self.phase.ordinal(),
            self.source_namespace,
            self.entered_state,
            self.target_namespace,
            self.target_handler
        )
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "entered_state",
                CanonicalValue::text(self.entered_state.as_str()),
            ),
            ("payload", self.payload.to_canonical()),
            ("phase", CanonicalValue::text(self.phase.as_str())),
            ("source_namespace", self.source_namespace.to_canonical()),
            (
                "target_handler",
                CanonicalValue::text(self.target_handler.as_str()),
            ),
            ("target_namespace", self.target_namespace.to_canonical()),
        ])
    }
}

/// Versioned runtime-facing simulation plan emitted from construction IR.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimulationPlan {
    schema: SchemaId,
    machines: Vec<MachineDefinition>,
    causal_edges: Vec<CausalEdge>,
    movement_resolver: MovementResolverPlan,
}

impl SimulationPlan {
    /// Builds a plan and imposes deterministic machine and edge ordering.
    ///
    /// This constructor rejects duplicate identity but deliberately does not
    /// validate graph references or cycles. The compiler owns that validation;
    /// runtime still fails closed if handed a malicious projection.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for duplicate machine or edge identity.
    pub fn new(
        mut machines: Vec<MachineDefinition>,
        mut causal_edges: Vec<CausalEdge>,
    ) -> Result<Self, Diagnostic> {
        require_unique(
            machines.iter().map(|machine| machine.namespace.clone()),
            "simulation machine namespace",
            nomos_core::diagnostic::codes::CANONICAL_DUPLICATE_IDENTITY,
        )?;
        require_unique(
            causal_edges.iter().map(CausalEdge::stable_key),
            "causal interaction identity",
            nomos_core::diagnostic::codes::INTERACTION_IDENTITY_DUPLICATE,
        )?;
        machines.sort_by(|left, right| left.namespace.cmp(&right.namespace));
        causal_edges.sort_by_key(CausalEdge::stable_key);
        Ok(Self {
            schema: simulation_schema(),
            machines,
            causal_edges,
            movement_resolver: MovementResolverPlan::empty_gate_k(),
        })
    }

    /// Attaches the compiler-projected shared movement resolver plan.
    #[must_use]
    pub fn with_movement_resolver(mut self, movement_resolver: MovementResolverPlan) -> Self {
        self.movement_resolver = movement_resolver;
        self
    }

    /// Projection schema identity.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Runtime machines in stable namespace order.
    #[must_use]
    pub fn machines(&self) -> &[MachineDefinition] {
        &self.machines
    }

    /// Causal edges in explicit phase and stable semantic order.
    #[must_use]
    pub fn causal_edges(&self) -> &[CausalEdge] {
        &self.causal_edges
    }

    /// Shared ground movement resolver plan.
    #[must_use]
    pub const fn movement_resolver(&self) -> &MovementResolverPlan {
        &self.movement_resolver
    }

    /// Canonical projection bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "causal_edges",
                CanonicalValue::Array(
                    self.causal_edges
                        .iter()
                        .map(CausalEdge::to_canonical)
                        .collect(),
                ),
            ),
            (
                "machines",
                CanonicalValue::Array(
                    self.machines
                        .iter()
                        .map(MachineDefinition::to_canonical)
                        .collect(),
                ),
            ),
            ("movement_resolver", self.movement_resolver.to_canonical()),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}

/// Argument carried by one typed external command invocation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CommandArgument {
    /// No argument was supplied.
    None,
    /// A typed credential argument.
    Credential(CatalogValueId),
    /// An internal event payload supplied where a command argument was expected.
    Event(EventPayload),
}

/// One typed external command invocation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Command {
    namespace: NamespaceId,
    action: Ident,
    argument: CommandArgument,
}

impl Command {
    /// Builds a typed command invocation.
    #[must_use]
    pub fn new(namespace: NamespaceId, action: Ident, argument: CommandArgument) -> Self {
        Self {
            namespace,
            action,
            argument,
        }
    }

    /// Target machine namespace.
    #[must_use]
    pub const fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// Namespace-local action.
    #[must_use]
    pub const fn action(&self) -> &Ident {
        &self.action
    }

    /// Typed command argument.
    #[must_use]
    pub const fn argument(&self) -> &CommandArgument {
        &self.argument
    }
}

fn require_unique<T: Ord>(
    values: impl IntoIterator<Item = T>,
    identity: &str,
    code: nomos_core::DiagnosticCode,
) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(
                Diagnostic::new(code, format!("{identity} occurs more than once"))
                    .with_repair(RepairClass::RemoveDuplicateDeclaration),
            );
        }
    }
    Ok(())
}

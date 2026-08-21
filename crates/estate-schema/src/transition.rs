//! Typed transition and causal-interaction definitions in construction IR.

use estate_core::id::StableId;
use estate_core::{CanonicalValue, Ident, NamespaceId};

/// The typed input accepted by one machine trigger.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TransitionInput {
    /// The trigger accepts no argument.
    None,
    /// The trigger requires the credential resolved on the owning entity.
    ResolvedEntityCredential,
    /// The trigger accepts one exact typed damage payload.
    Damage {
        /// Damage channel.
        channel: Ident,
        /// Non-negative damage amount.
        amount: u32,
    },
}

impl TransitionInput {
    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::None => CanonicalValue::object_declared([("kind", CanonicalValue::text("none"))]),
            Self::ResolvedEntityCredential => CanonicalValue::object_declared([(
                "kind",
                CanonicalValue::text("resolved_entity_credential"),
            )]),
            Self::Damage { channel, amount } => CanonicalValue::object_declared([
                ("amount", CanonicalValue::Uint(u64::from(*amount))),
                ("channel", CanonicalValue::text(channel.as_str())),
                ("kind", CanonicalValue::text("damage")),
            ]),
        }
    }
}

/// A typed cause that can select a namespace-local transition.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TransitionTrigger {
    /// A declared external command.
    Command {
        /// Namespace-local action name.
        action: Ident,
        /// Required typed command input.
        input: TransitionInput,
    },
    /// An internal event handler, unavailable to external callers.
    Event {
        /// Namespace-local handler name.
        handler: Ident,
        /// Required typed event payload.
        input: TransitionInput,
    },
}

impl TransitionTrigger {
    /// Namespace-local command or handler name.
    #[must_use]
    pub fn action(&self) -> &Ident {
        match self {
            Self::Command { action, .. } => action,
            Self::Event { handler, .. } => handler,
        }
    }

    /// Typed input shape and constraint.
    #[must_use]
    pub const fn input(&self) -> &TransitionInput {
        match self {
            Self::Command { input, .. } | Self::Event { input, .. } => input,
        }
    }

    /// Whether this trigger is externally invocable.
    #[must_use]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::Command { .. })
    }

    fn stable_kind(&self) -> &'static str {
        match self {
            Self::Command { .. } => "command",
            Self::Event { .. } => "event",
        }
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("input", self.input().to_canonical()),
            ("kind", CanonicalValue::text(self.stable_kind())),
            ("name", CanonicalValue::text(self.action().as_str())),
        ])
    }
}

/// A compiler-owned namespace-local state transition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransitionDefinition {
    trigger: TransitionTrigger,
    source: Ident,
    target: Ident,
}

impl TransitionDefinition {
    /// Builds a transition with one explicit set-state effect.
    #[must_use]
    pub fn new(trigger: TransitionTrigger, source: Ident, target: Ident) -> Self {
        Self {
            trigger,
            source,
            target,
        }
    }

    /// Typed transition trigger.
    #[must_use]
    pub const fn trigger(&self) -> &TransitionTrigger {
        &self.trigger
    }

    /// Required current state.
    #[must_use]
    pub const fn source(&self) -> &Ident {
        &self.source
    }

    /// State staged by the machine-local effect.
    #[must_use]
    pub const fn target(&self) -> &Ident {
        &self.target
    }

    pub(crate) fn stable_key(&self) -> String {
        format!(
            "{}#{}#{}",
            self.trigger.stable_kind(),
            self.trigger.action(),
            self.source
        )
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "effect",
                CanonicalValue::object_declared([
                    ("kind", CanonicalValue::text("set_state")),
                    ("state", CanonicalValue::text(self.target.as_str())),
                ]),
            ),
            ("source", CanonicalValue::text(self.source.as_str())),
            ("trigger", self.trigger.to_canonical()),
        ])
    }
}

/// Fixed interaction-settlement phase used by Gate K.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InteractionPhase {
    /// Settle typed causal events after the initiating local transition.
    Causal,
}

impl InteractionPhase {
    /// Stable phase ordinal used before semantic tie-breakers.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Causal => 1,
        }
    }

    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Causal => "causal",
        }
    }
}

/// The only causal trigger admitted by Gate K.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InteractionTrigger {
    /// Fire exactly once when a machine enters the named state.
    OnEnter {
        /// Source machine namespace.
        namespace: NamespaceId,
        /// Newly entered state.
        state: Ident,
    },
}

impl InteractionTrigger {
    /// Source machine namespace.
    #[must_use]
    pub const fn namespace(&self) -> &NamespaceId {
        match self {
            Self::OnEnter { namespace, .. } => namespace,
        }
    }

    /// State whose entry fires the interaction.
    #[must_use]
    pub const fn state(&self) -> &Ident {
        match self {
            Self::OnEnter { state, .. } => state,
        }
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        match self {
            Self::OnEnter { namespace, state } => CanonicalValue::object_declared([
                ("kind", CanonicalValue::text("on_enter")),
                ("namespace", namespace.to_canonical()),
                ("state", CanonicalValue::text(state.as_str())),
            ]),
        }
    }
}

/// One typed causal edge from a state-entry trigger to a target-owned handler.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InteractionDefinition {
    trigger: InteractionTrigger,
    phase: InteractionPhase,
    target_namespace: NamespaceId,
    target_handler: Ident,
    payload: TransitionInput,
}

impl InteractionDefinition {
    /// Builds one causal interaction edge.
    #[must_use]
    pub fn new(
        trigger: InteractionTrigger,
        phase: InteractionPhase,
        target_namespace: NamespaceId,
        target_handler: Ident,
        payload: TransitionInput,
    ) -> Self {
        Self {
            trigger,
            phase,
            target_namespace,
            target_handler,
            payload,
        }
    }

    /// State-entry trigger.
    #[must_use]
    pub const fn trigger(&self) -> &InteractionTrigger {
        &self.trigger
    }

    /// Settlement phase.
    #[must_use]
    pub const fn phase(&self) -> InteractionPhase {
        self.phase
    }

    /// Namespace that owns the event handler.
    #[must_use]
    pub const fn target_namespace(&self) -> &NamespaceId {
        &self.target_namespace
    }

    /// Namespace-local handler name.
    #[must_use]
    pub const fn target_handler(&self) -> &Ident {
        &self.target_handler
    }

    /// Typed payload emitted to the target handler.
    #[must_use]
    pub const fn payload(&self) -> &TransitionInput {
        &self.payload
    }

    pub(crate) fn stable_key(&self) -> String {
        format!(
            "{:03}#{}#{}#{}#{}",
            self.phase.ordinal(),
            self.trigger.namespace(),
            self.trigger.state(),
            self.target_namespace,
            self.target_handler
        )
    }

    pub(crate) fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("payload", self.payload.to_canonical()),
            ("phase", CanonicalValue::text(self.phase.as_str())),
            (
                "target_handler",
                CanonicalValue::text(self.target_handler.as_str()),
            ),
            ("target_namespace", self.target_namespace.to_canonical()),
            ("trigger", self.trigger.to_canonical()),
        ])
    }
}

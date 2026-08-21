//! Atomic commit of prepared transactions into immutable runtime snapshots.

use nomos_core::{Diagnostic, StateHash};
use nomos_projection::{Command, SimulationPlan};

use crate::{
    CausalReceipt, DEFAULT_TRANSITION_BUDGET, PreparedTransaction, SimulationState,
    prepare_transaction_with_budget,
};

/// Complete evidence returned only after a transaction commits successfully.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommittedTransaction {
    snapshot: SimulationState,
    state_hash: StateHash,
    receipt: CausalReceipt,
}

impl CommittedTransaction {
    /// New immutable authoritative snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &SimulationState {
        &self.snapshot
    }

    /// SHA-256 of the snapshot's canonical envelope only.
    #[must_use]
    pub const fn state_hash(&self) -> StateHash {
        self.state_hash
    }

    /// Typed causal and projection receipt.
    #[must_use]
    pub const fn receipt(&self) -> &CausalReceipt {
        &self.receipt
    }

    /// Consumes the commit evidence and returns the new snapshot.
    #[must_use]
    pub fn into_snapshot(self) -> SimulationState {
        self.snapshot
    }
}

/// Prepares and atomically commits one command with the default transition budget.
///
/// # Errors
///
/// Returns a stable diagnostic without exposing a snapshot, hash, receipt, or
/// external event when any preparation, resolver, arithmetic, or receipt step fails.
pub fn commit_transaction(
    plan: &SimulationPlan,
    current: &SimulationState,
    command: &Command,
) -> Result<CommittedTransaction, Diagnostic> {
    commit_transaction_with_budget(plan, current, command, DEFAULT_TRANSITION_BUDGET)
}

/// Prepares and atomically commits one command with an explicit transition budget.
///
/// # Errors
///
/// Returns the same diagnostics as [`commit_transaction`], including `EK0808`
/// for budget exhaustion and `EK0201` for tick overflow.
pub fn commit_transaction_with_budget(
    plan: &SimulationPlan,
    current: &SimulationState,
    command: &Command,
    budget: usize,
) -> Result<CommittedTransaction, Diagnostic> {
    let prepared = prepare_transaction_with_budget(plan, current, command, budget)?;
    commit_prepared(plan, current, command, prepared)
}

fn commit_prepared(
    plan: &SimulationPlan,
    current: &SimulationState,
    command: &Command,
    prepared: PreparedTransaction,
) -> Result<CommittedTransaction, Diagnostic> {
    let next_tick = nomos_core::arith::add_u64(current.tick(), 1)?;
    let mut snapshot = prepared.after().clone();
    snapshot.set_tick(next_tick);
    let state_hash = snapshot.state_hash();
    let receipt =
        CausalReceipt::from_prepared(plan, command.clone(), &prepared, next_tick, state_hash)?;
    Ok(CommittedTransaction {
        snapshot,
        state_hash,
        receipt,
    })
}

#[cfg(test)]
mod tests {
    use nomos_core::{EntityId, Ident, NamespaceId};
    use nomos_projection::{
        CommandArgument, CommandRequirement, CommandTransition, LatticeCell, MachineDefinition,
        ProjectedEntity, RuntimeBinding,
    };

    use super::*;

    #[test]
    fn tick_overflow_rejects_without_commit_evidence_or_input_mutation() {
        let entity = EntityId::parse("subject").unwrap();
        let namespace = NamespaceId::new(entity.clone(), Ident::new("machine").unwrap());
        let machine = MachineDefinition::new(
            namespace.clone(),
            vec![Ident::new("off").unwrap(), Ident::new("on").unwrap()],
            Ident::new("off").unwrap(),
            vec![CommandTransition::new(
                Ident::new("turn_on").unwrap(),
                CommandRequirement::None,
                Ident::new("off").unwrap(),
                Ident::new("on").unwrap(),
            )],
            Vec::new(),
        )
        .unwrap();
        let projected = ProjectedEntity::new(
            entity,
            RuntimeBinding::Cell(LatticeCell::new(0, 0, 0)),
            vec![namespace.clone()],
        )
        .unwrap();
        let plan = SimulationPlan::new(vec![machine], Vec::new())
            .unwrap()
            .with_entities(vec![projected])
            .unwrap();
        let mut current = SimulationState::initialize(&plan).unwrap();
        current.set_tick(u64::MAX);
        let before = current.to_canonical_bytes();
        let rejected = commit_transaction(
            &plan,
            &current,
            &Command::new(
                namespace,
                Ident::new("turn_on").unwrap(),
                CommandArgument::None,
            ),
        )
        .unwrap_err();
        assert_eq!(rejected.code().as_str(), "EK0201");
        assert_eq!(current.to_canonical_bytes(), before);
    }
}

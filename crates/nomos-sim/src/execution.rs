//! Deterministic execution into the complete typed evidence for one run.

use nomos_core::{Diagnostic, RepairClass, Sha256Digest};
use nomos_projection::SimulationPlan;

use crate::{
    CausalReceiptSequence, CommandLog, CommandLogRow, CommandRequest, PersistedRuntimeState,
    RunResult, StateHashSequence, commit_transaction, resolve_command,
};

/// Complete typed artifacts and terminal diagnostic for one command sequence.
///
/// Runtime rejection is evidence rather than a partially returned error: all
/// commands committed before the rejection remain bound by the log, receipts,
/// hashes, final state, and rejected result. Errors returned by
/// [`execute_requests`] instead mean that internally constructed evidence could
/// not satisfy its own invariants.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunExecution {
    initial: PersistedRuntimeState,
    final_state: PersistedRuntimeState,
    log: CommandLog,
    receipts: CausalReceiptSequence,
    hashes: StateHashSequence,
    result: RunResult,
    rejection: Option<Diagnostic>,
}

impl RunExecution {
    /// Persisted state from which the first request was attempted.
    #[must_use]
    pub const fn initial(&self) -> &PersistedRuntimeState {
        &self.initial
    }

    /// Persisted state after the last successfully committed request.
    #[must_use]
    pub const fn final_state(&self) -> &PersistedRuntimeState {
        &self.final_state
    }

    /// Successfully committed command rows.
    #[must_use]
    pub const fn command_log(&self) -> &CommandLog {
        &self.log
    }

    /// One causal receipt per committed command.
    #[must_use]
    pub const fn causal_receipts(&self) -> &CausalReceiptSequence {
        &self.receipts
    }

    /// Initial hash followed by one hash per committed command.
    #[must_use]
    pub const fn state_hashes(&self) -> &StateHashSequence {
        &self.hashes
    }

    /// Content-binding terminal record.
    #[must_use]
    pub const fn result(&self) -> &RunResult {
        &self.result
    }

    /// Stable diagnostic that stopped execution, if the run was rejected.
    #[must_use]
    pub const fn rejection(&self) -> Option<&Diagnostic> {
        self.rejection.as_ref()
    }

    /// Verifies that the terminal result agrees with the execution outcome.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` when completed/rejected status, diagnostic identity,
    /// or the complete committed evidence disagrees.
    pub fn validate(&self, plan: &SimulationPlan) -> Result<(), Diagnostic> {
        validate_committed_evidence(
            plan,
            &self.initial,
            &self.final_state,
            &self.log,
            &self.receipts,
            &self.hashes,
        )?;
        match (self.result.status(), self.rejection.as_ref()) {
            (crate::RunStatus::Completed, None) => Ok(()),
            (crate::RunStatus::Rejected, Some(diagnostic))
                if self.result.rejection_diagnostic() == Some(diagnostic.code()) =>
            {
                Ok(())
            }
            _ => Err(crate::run_evidence::inconsistent(
                "run result status disagrees with the terminal execution outcome",
            )),
        }
    }
}

/// Re-executes every committed request and requires byte-identical evidence.
///
/// A rejected run deliberately omits its uncommitted request, so this checks
/// only the committed prefix named by the command log.
///
/// # Errors
///
/// Returns `EK0816` when the log, receipts, hashes, or final state cannot be
/// reproduced from the initial state and supplied simulation semantics.
pub fn validate_committed_evidence(
    plan: &SimulationPlan,
    initial: &PersistedRuntimeState,
    final_state: &PersistedRuntimeState,
    log: &CommandLog,
    receipts: &CausalReceiptSequence,
    hashes: &StateHashSequence,
) -> Result<(), Diagnostic> {
    let initial = PersistedRuntimeState::from_canonical_bytes(&initial.to_canonical_bytes(), plan)?;
    let mut current = initial.state().clone();
    let mut expected_rows = Vec::with_capacity(log.rows().len());
    let mut expected_receipts = Vec::with_capacity(log.rows().len());
    for row in log.rows() {
        let command = resolve_command(plan, row.request()).map_err(|_| {
            crate::run_evidence::inconsistent(
                "a committed command-log request no longer resolves under the supplied semantics",
            )
        })?;
        let input_state_hash = current.state_hash();
        let committed = commit_transaction(plan, &current, &command).map_err(|_| {
            crate::run_evidence::inconsistent(
                "a committed command-log request no longer commits under the supplied semantics",
            )
        })?;
        expected_rows.push(CommandLogRow::new(
            row.ordinal(),
            row.request().clone(),
            command,
            input_state_hash,
            committed.receipt(),
        )?);
        expected_receipts.push(committed.receipt().clone());
        current = committed.into_snapshot();
    }
    let expected_log = CommandLog::new(expected_rows)?;
    let expected_receipts =
        CausalReceiptSequence::new(initial.state().tick(), &expected_log, expected_receipts)?;
    let expected_hashes = StateHashSequence::from_command_log(initial.state_hash(), &expected_log)?;
    let expected_final = PersistedRuntimeState::new(plan, current)?;
    if &expected_log != log
        || &expected_receipts != receipts
        || &expected_hashes != hashes
        || &expected_final != final_state
    {
        return Err(crate::run_evidence::inconsistent(
            "committed run evidence does not reproduce from its initial state and simulation semantics",
        ));
    }
    Ok(())
}

/// Resolves and commits requests in order, stopping at the first rejection.
///
/// The caller supplies an already verified package digest and a persisted state
/// bound to the supplied plan; this crate remains independent of package and
/// compiler implementations.
///
/// # Errors
///
/// Returns the first evidence-construction inconsistency. Command resolution
/// and transaction failures are represented by a valid rejected
/// [`RunExecution`].
pub fn execute_requests(
    plan: &SimulationPlan,
    input_package_digest: Sha256Digest,
    initial: PersistedRuntimeState,
    requests: &[CommandRequest],
) -> Result<RunExecution, Diagnostic> {
    // Reopen the typed envelope against the caller's plan so an in-memory state
    // previously bound to different semantics cannot cross this boundary.
    let initial = PersistedRuntimeState::from_canonical_bytes(&initial.to_canonical_bytes(), plan)?;
    let mut current = initial.state().clone();
    let mut rows = Vec::new();
    let mut receipts = Vec::new();
    let mut rejection = if requests.is_empty() {
        Some(
            Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_COMMAND_SCRIPT_INVALID,
                "a runtime execution requires at least one command request",
            )
            .with_repair(RepairClass::FixSourceSyntax),
        )
    } else {
        None
    };

    for (index, request) in requests.iter().enumerate() {
        let command = match resolve_command(plan, request) {
            Ok(command) => command,
            Err(diagnostic) => {
                rejection = Some(diagnostic);
                break;
            }
        };
        let input_state_hash = current.state_hash();
        let committed = match commit_transaction(plan, &current, &command) {
            Ok(committed) => committed,
            Err(diagnostic) => {
                rejection = Some(diagnostic);
                break;
            }
        };
        let ordinal = u64::try_from(index).map_err(|_| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT,
                "committed command ordinal exceeds u64",
            )
        })?;
        rows.push(CommandLogRow::new(
            ordinal,
            request.clone(),
            command,
            input_state_hash,
            committed.receipt(),
        )?);
        receipts.push(committed.receipt().clone());
        current = committed.into_snapshot();
    }

    let log = CommandLog::new(rows)?;
    let receipt_sequence = CausalReceiptSequence::new(initial.state().tick(), &log, receipts)?;
    let hashes = StateHashSequence::from_command_log(initial.state_hash(), &log)?;
    let final_state = PersistedRuntimeState::new(plan, current)?;
    let result = match rejection.as_ref() {
        Some(diagnostic) => RunResult::rejected(
            input_package_digest,
            &initial,
            &final_state,
            &log,
            &receipt_sequence,
            &hashes,
            diagnostic.code(),
        )?,
        None => RunResult::completed(
            input_package_digest,
            &initial,
            &final_state,
            &log,
            &receipt_sequence,
            &hashes,
        )?,
    };

    Ok(RunExecution {
        initial,
        final_state,
        log,
        receipts: receipt_sequence,
        hashes,
        result,
        rejection,
    })
}

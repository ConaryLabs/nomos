//! Strict replay inputs and deterministic reproduction checks.

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, Diagnostic, RepairClass, SchemaId, Sha256Digest, StateHash};

use crate::state_persistence::{digest, field, object, require_fields, schema, state_hash};
use crate::{CommandLog, PersistedRuntimeState, RunExecution, RunStatus};

/// Canonical replay input bound to one exact package and initial runtime state.
///
/// The expected command log includes every unresolved request, resolved typed
/// command, state-hash edge, and causal-receipt digest. Replaying therefore
/// checks semantic resolution and committed evidence rather than merely feeding
/// command text back to the runtime.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReplayLog {
    schema: SchemaId,
    input_package_digest: Sha256Digest,
    runtime_semantics_digest: Sha256Digest,
    initial_state_hash: StateHash,
    expected_command_log: CommandLog,
    expected_final_state_hash: StateHash,
}

impl ReplayLog {
    /// Derives a replay log from one completed typed execution.
    ///
    /// # Errors
    ///
    /// Returns `EK0822` when the execution was rejected or committed no
    /// commands. Replay fixtures describe completed, reproducible histories;
    /// runtime rejection evidence remains represented by a run bundle.
    pub fn from_execution(execution: &RunExecution) -> Result<Self, Diagnostic> {
        if execution.result().status() != RunStatus::Completed || execution.rejection().is_some() {
            return Err(invalid(
                "a replay log can be derived only from a completed execution",
            ));
        }
        let replay = Self {
            schema: crate::replay_log_schema(),
            input_package_digest: execution.result().input_package_digest(),
            runtime_semantics_digest: execution.result().runtime_semantics_digest(),
            initial_state_hash: execution.initial().state_hash(),
            expected_command_log: execution.command_log().clone(),
            expected_final_state_hash: execution.final_state().state_hash(),
        };
        replay.validate_internal()?;
        Ok(replay)
    }

    /// Strictly reconstructs one canonical replay log.
    ///
    /// # Errors
    ///
    /// Returns `EK0822` for canonical, field-set, schema, nested command-log,
    /// hash-chain, or endpoint disagreement.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes).map_err(|error| invalid(error.message()))?;
        let fields = object(&value, "replay log").map_err(map_invalid)?;
        require_fields(
            fields,
            &[
                "expected_command_log",
                "expected_final_state_hash",
                "initial_state_hash",
                "input_package_digest",
                "runtime_semantics_digest",
                "schema",
            ],
            "replay log",
        )
        .map_err(map_invalid)?;
        let schema = schema(
            field(fields, "schema").map_err(map_invalid)?,
            "replay-log schema",
        )
        .map_err(map_invalid)?;
        if schema != crate::replay_log_schema() {
            return Err(invalid("replay log names an unsupported schema"));
        }
        let expected_command_log = CommandLog::from_canonical_bytes(
            &field(fields, "expected_command_log")
                .map_err(map_invalid)?
                .to_canonical_bytes(),
        )
        .map_err(map_invalid)?;
        let replay = Self {
            schema,
            input_package_digest: digest(
                field(fields, "input_package_digest").map_err(map_invalid)?,
                "replay input package digest",
            )
            .map_err(map_invalid)?,
            runtime_semantics_digest: digest(
                field(fields, "runtime_semantics_digest").map_err(map_invalid)?,
                "replay runtime semantics digest",
            )
            .map_err(map_invalid)?,
            initial_state_hash: state_hash(
                field(fields, "initial_state_hash").map_err(map_invalid)?,
            )
            .map_err(map_invalid)?,
            expected_command_log,
            expected_final_state_hash: state_hash(
                field(fields, "expected_final_state_hash").map_err(map_invalid)?,
            )
            .map_err(map_invalid)?,
        };
        replay.validate_internal()?;
        if replay.to_canonical_bytes() != bytes {
            return Err(invalid(
                "replay log does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(replay)
    }

    /// Package digest recorded by the replay input.
    #[must_use]
    pub const fn input_package_digest(&self) -> Sha256Digest {
        self.input_package_digest
    }

    /// Exact simulation-projection digest recorded by the replay input.
    #[must_use]
    pub const fn runtime_semantics_digest(&self) -> Sha256Digest {
        self.runtime_semantics_digest
    }

    /// Package-derived initial state hash recorded by the replay input.
    #[must_use]
    pub const fn initial_state_hash(&self) -> StateHash {
        self.initial_state_hash
    }

    /// Expected committed command evidence in replay order.
    #[must_use]
    pub const fn expected_command_log(&self) -> &CommandLog {
        &self.expected_command_log
    }

    /// Expected state hash after the final committed command.
    #[must_use]
    pub const fn expected_final_state_hash(&self) -> StateHash {
        self.expected_final_state_hash
    }

    /// Verifies the replay identity against a package-derived initial state.
    ///
    /// # Errors
    ///
    /// Returns `EK0823` when the package, runtime-semantics, or initial-state
    /// identity differs.
    pub fn validate_input(
        &self,
        package_digest: Sha256Digest,
        initial: &PersistedRuntimeState,
    ) -> Result<(), Diagnostic> {
        if self.input_package_digest != package_digest
            || self.runtime_semantics_digest != initial.runtime_semantics_digest()
            || self.initial_state_hash != initial.state_hash()
        {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::REPLAY_INPUT_MISMATCH,
                "replay log does not belong to the supplied package and initial runtime state",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        Ok(())
    }

    /// Requires one re-execution to reproduce the complete expected log and
    /// final state identity.
    ///
    /// # Errors
    ///
    /// Returns `EK0824` when execution rejects or its package, semantics,
    /// initial state, committed evidence, or final state differs.
    pub fn validate_execution(&self, execution: &RunExecution) -> Result<(), Diagnostic> {
        let matches = execution.result().status() == RunStatus::Completed
            && execution.rejection().is_none()
            && execution.result().input_package_digest() == self.input_package_digest
            && execution.result().runtime_semantics_digest() == self.runtime_semantics_digest
            && execution.initial().state_hash() == self.initial_state_hash
            && execution.command_log() == &self.expected_command_log
            && execution.final_state().state_hash() == self.expected_final_state_hash;
        if !matches {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::REPLAY_EVIDENCE_MISMATCH,
                "runtime execution does not reproduce the replay log's expected evidence",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        Ok(())
    }

    /// Exact canonical replay-log bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let expected_command_log = parse_canonical(&self.expected_command_log.to_canonical_bytes())
            .expect("a typed command log always emits canonical bytes");
        CanonicalValue::object_declared([
            ("expected_command_log", expected_command_log),
            (
                "expected_final_state_hash",
                CanonicalValue::text(self.expected_final_state_hash.to_hex()),
            ),
            (
                "initial_state_hash",
                CanonicalValue::text(self.initial_state_hash.to_hex()),
            ),
            (
                "input_package_digest",
                CanonicalValue::text(self.input_package_digest.to_hex()),
            ),
            (
                "runtime_semantics_digest",
                CanonicalValue::text(self.runtime_semantics_digest.to_hex()),
            ),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }

    fn validate_internal(&self) -> Result<(), Diagnostic> {
        let Some(first) = self.expected_command_log.rows().first() else {
            return Err(invalid(
                "a replay log must contain at least one committed command",
            ));
        };
        let last = self
            .expected_command_log
            .rows()
            .last()
            .expect("the replay log was proved nonempty");
        if first.input_state_hash() != self.initial_state_hash {
            return Err(invalid(
                "replay initial state hash disagrees with the first command-log row",
            ));
        }
        if last.resulting_state_hash() != self.expected_final_state_hash {
            return Err(invalid(
                "replay final state hash disagrees with the final command-log row",
            ));
        }
        Ok(())
    }
}

fn map_invalid(error: Diagnostic) -> Diagnostic {
    invalid(error.message())
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(nomos_core::diagnostic::codes::REPLAY_LOG_INVALID, message)
        .with_repair(RepairClass::RebuildFromSource)
}

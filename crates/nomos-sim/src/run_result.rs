//! Content binding for the complete typed evidence of one run bundle.

use std::collections::BTreeSet;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, Diagnostic, DiagnosticCode, SchemaId, Sha256Digest, StateHash};

use crate::run_evidence::inconsistent;
use crate::state_persistence::{
    array, digest, field, invalid, object, require_fields, schema, state_hash, text, uint,
};
use crate::{CausalReceiptSequence, CommandLog, PersistedRuntimeState, StateHashSequence};

/// Run-bundle artifact names bound by `result.json`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RunArtifactName {
    /// Persisted initial state.
    InitialState,
    /// Persisted final state.
    FinalState,
    /// Typed command log.
    CommandLog,
    /// Typed causal-receipt collection.
    CausalReceipts,
    /// Typed state-hash sequence.
    StateHashes,
}

impl RunArtifactName {
    /// Fixed root-level file name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitialState => "initial-state.json",
            Self::FinalState => "final-state.json",
            Self::CommandLog => "command-log.json",
            Self::CausalReceipts => "causal-receipts.json",
            Self::StateHashes => "state-hashes.json",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "initial-state.json" => Some(Self::InitialState),
            "final-state.json" => Some(Self::FinalState),
            "command-log.json" => Some(Self::CommandLog),
            "causal-receipts.json" => Some(Self::CausalReceipts),
            "state-hashes.json" => Some(Self::StateHashes),
            _ => None,
        }
    }

    const ALL: [Self; 5] = [
        Self::CausalReceipts,
        Self::CommandLog,
        Self::FinalState,
        Self::InitialState,
        Self::StateHashes,
    ];
}

/// SHA-256 binding for one non-result run artifact.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunArtifactDigest {
    name: RunArtifactName,
    digest: Sha256Digest,
}

impl RunArtifactDigest {
    /// Fixed artifact name.
    #[must_use]
    pub const fn name(&self) -> RunArtifactName {
        self.name
    }

    /// SHA-256 of the artifact's exact canonical bytes.
    #[must_use]
    pub const fn digest(&self) -> Sha256Digest {
        self.digest
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("digest", CanonicalValue::text(self.digest.to_hex())),
            ("name", CanonicalValue::text(self.name.as_str())),
        ])
    }
}

/// Terminal status of a run bundle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunStatus {
    /// Every requested command committed.
    Completed,
    /// Execution stopped on a stable rejection diagnostic.
    Rejected,
}

impl RunStatus {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Rejected => "rejected",
        }
    }
}

/// Content-binding record written as a run bundle's `result.json`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunResult {
    schema: SchemaId,
    input_package_digest: Sha256Digest,
    runtime_semantics_digest: Sha256Digest,
    status: RunStatus,
    artifacts: Vec<RunArtifactDigest>,
    first_state_hash: StateHash,
    final_state_hash: StateHash,
    committed_command_count: u64,
    rejection_diagnostic: Option<DiagnosticCode>,
}

impl RunResult {
    /// Builds content-binding evidence for a completed command script.
    ///
    /// Every artifact digest is derived from the supplied typed artifact; a
    /// caller cannot inject an unverified digest.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` when any typed artifact disagrees with the others.
    pub fn completed(
        input_package_digest: Sha256Digest,
        initial: &PersistedRuntimeState,
        final_state: &PersistedRuntimeState,
        log: &CommandLog,
        receipts: &CausalReceiptSequence,
        hashes: &StateHashSequence,
    ) -> Result<Self, Diagnostic> {
        Self::build(
            input_package_digest,
            RunStatus::Completed,
            initial,
            final_state,
            log,
            receipts,
            hashes,
            None,
        )
    }

    /// Builds content-binding evidence for a run stopped by rejection.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` under the same conditions as [`Self::completed`].
    #[allow(clippy::too_many_arguments)]
    pub fn rejected(
        input_package_digest: Sha256Digest,
        initial: &PersistedRuntimeState,
        final_state: &PersistedRuntimeState,
        log: &CommandLog,
        receipts: &CausalReceiptSequence,
        hashes: &StateHashSequence,
        rejection_diagnostic: DiagnosticCode,
    ) -> Result<Self, Diagnostic> {
        Self::build(
            input_package_digest,
            RunStatus::Rejected,
            initial,
            final_state,
            log,
            receipts,
            hashes,
            Some(rejection_diagnostic),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        input_package_digest: Sha256Digest,
        status: RunStatus,
        initial: &PersistedRuntimeState,
        final_state: &PersistedRuntimeState,
        log: &CommandLog,
        receipts: &CausalReceiptSequence,
        hashes: &StateHashSequence,
        rejection_diagnostic: Option<DiagnosticCode>,
    ) -> Result<Self, Diagnostic> {
        validate_typed_evidence(initial, final_state, log, receipts, hashes)?;
        let committed_command_count = u64::try_from(log.rows().len())
            .map_err(|_| inconsistent("committed command count exceeds u64"))?;
        let result = Self {
            schema: crate::run_result_schema(),
            input_package_digest,
            runtime_semantics_digest: initial.runtime_semantics_digest(),
            status,
            artifacts: artifact_digests(initial, final_state, log, receipts, hashes),
            first_state_hash: initial.state_hash(),
            final_state_hash: final_state.state_hash(),
            committed_command_count,
            rejection_diagnostic,
        };
        result.validate_internal()?;
        Ok(result)
    }

    /// Terminal run status.
    #[must_use]
    pub const fn status(&self) -> RunStatus {
        self.status
    }

    /// Verified package digest recorded by the input package manifest.
    #[must_use]
    pub const fn input_package_digest(&self) -> Sha256Digest {
        self.input_package_digest
    }

    /// Digest identifying the exact canonical simulation projection bytes.
    #[must_use]
    pub const fn runtime_semantics_digest(&self) -> Sha256Digest {
        self.runtime_semantics_digest
    }

    /// Exact required non-result artifact bindings in file-name order.
    #[must_use]
    pub fn artifacts(&self) -> &[RunArtifactDigest] {
        &self.artifacts
    }

    /// State hash from `initial-state.json`.
    #[must_use]
    pub const fn first_state_hash(&self) -> StateHash {
        self.first_state_hash
    }

    /// State hash from `final-state.json`.
    #[must_use]
    pub const fn final_state_hash(&self) -> StateHash {
        self.final_state_hash
    }

    /// Exact number of successfully committed commands.
    #[must_use]
    pub const fn committed_command_count(&self) -> u64 {
        self.committed_command_count
    }

    /// Stable rejection code, present only for a rejected run.
    #[must_use]
    pub const fn rejection_diagnostic(&self) -> Option<DiagnosticCode> {
        self.rejection_diagnostic
    }

    /// Strictly reconstructs one canonical run-result record.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, artifact, status, count, or hash
    /// consistency diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "run result")?;
        require_fields(
            fields,
            &[
                "artifacts",
                "committed_command_count",
                "final_state_hash",
                "first_state_hash",
                "input_package_digest",
                "rejection_diagnostic",
                "runtime_semantics_digest",
                "schema",
                "status",
            ],
            "run result",
        )?;
        let schema = schema(field(fields, "schema")?, "run-result schema")?;
        if schema != crate::run_result_schema() {
            return Err(invalid("run result names an unsupported schema"));
        }
        let status = match text(field(fields, "status")?, "run status")? {
            "completed" => RunStatus::Completed,
            "rejected" => RunStatus::Rejected,
            _ => return Err(invalid("run status is unsupported")),
        };
        let artifacts = array(field(fields, "artifacts")?, "run artifacts")?
            .iter()
            .map(|row| {
                let fields = object(row, "run artifact")?;
                require_fields(fields, &["digest", "name"], "run artifact")?;
                let name =
                    RunArtifactName::parse(text(field(fields, "name")?, "run artifact name")?)
                        .ok_or_else(|| invalid("run artifact name is unsupported"))?;
                Ok(RunArtifactDigest {
                    name,
                    digest: digest(field(fields, "digest")?, "run artifact digest")?,
                })
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let rejection_diagnostic = match field(fields, "rejection_diagnostic")? {
            CanonicalValue::Null => None,
            value => {
                let fields = object(value, "run rejection diagnostic")?;
                require_fields(fields, &["code"], "run rejection diagnostic")?;
                Some(
                    DiagnosticCode::parse(text(field(fields, "code")?, "rejection code")?)
                        .ok_or_else(|| invalid("run rejection diagnostic code is unknown"))?,
                )
            }
        };
        let result = Self {
            schema,
            input_package_digest: digest(
                field(fields, "input_package_digest")?,
                "input package digest",
            )?,
            runtime_semantics_digest: digest(
                field(fields, "runtime_semantics_digest")?,
                "runtime semantics digest",
            )?,
            status,
            artifacts: normalize_artifacts(artifacts)?,
            first_state_hash: state_hash(field(fields, "first_state_hash")?)?,
            final_state_hash: state_hash(field(fields, "final_state_hash")?)?,
            committed_command_count: uint(
                field(fields, "committed_command_count")?,
                "committed command count",
            )?,
            rejection_diagnostic,
        };
        result.validate_internal()?;
        if result.to_canonical_bytes() != bytes {
            return Err(invalid(
                "run result does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(result)
    }

    /// Verifies every recorded binding against all five typed artifacts.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for any cross-object or content-digest disagreement.
    pub fn validate_evidence(
        &self,
        initial: &PersistedRuntimeState,
        final_state: &PersistedRuntimeState,
        log: &CommandLog,
        receipts: &CausalReceiptSequence,
        hashes: &StateHashSequence,
    ) -> Result<(), Diagnostic> {
        validate_typed_evidence(initial, final_state, log, receipts, hashes)?;
        let count = u64::try_from(log.rows().len())
            .map_err(|_| inconsistent("committed command count exceeds u64"))?;
        if self.committed_command_count != count
            || self.runtime_semantics_digest != initial.runtime_semantics_digest()
            || self.first_state_hash != initial.state_hash()
            || self.final_state_hash != final_state.state_hash()
            || self.artifacts != artifact_digests(initial, final_state, log, receipts, hashes)
        {
            return Err(inconsistent(
                "run result disagrees with its typed run evidence",
            ));
        }
        Ok(())
    }

    /// Exact canonical run-result bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let rejection = self
            .rejection_diagnostic
            .map_or(CanonicalValue::Null, |code| {
                CanonicalValue::object_declared([("code", CanonicalValue::text(code.as_str()))])
            });
        CanonicalValue::object_declared([
            (
                "artifacts",
                CanonicalValue::Array(
                    self.artifacts
                        .iter()
                        .map(RunArtifactDigest::to_canonical)
                        .collect(),
                ),
            ),
            (
                "committed_command_count",
                CanonicalValue::Uint(self.committed_command_count),
            ),
            (
                "final_state_hash",
                CanonicalValue::text(self.final_state_hash.to_hex()),
            ),
            (
                "first_state_hash",
                CanonicalValue::text(self.first_state_hash.to_hex()),
            ),
            (
                "input_package_digest",
                CanonicalValue::text(self.input_package_digest.to_hex()),
            ),
            ("rejection_diagnostic", rejection),
            (
                "runtime_semantics_digest",
                CanonicalValue::text(self.runtime_semantics_digest.to_hex()),
            ),
            ("schema", self.schema.to_canonical()),
            ("status", CanonicalValue::text(self.status.as_str())),
        ])
        .to_canonical_bytes()
    }

    fn validate_internal(&self) -> Result<(), Diagnostic> {
        match (self.status, self.rejection_diagnostic) {
            (RunStatus::Completed, None) | (RunStatus::Rejected, Some(_)) => {}
            _ => {
                return Err(inconsistent(
                    "run status and rejection diagnostic presence disagree",
                ));
            }
        }
        if self.status == RunStatus::Completed && self.committed_command_count == 0 {
            return Err(inconsistent(
                "a completed nonempty command script must commit at least one command",
            ));
        }
        if self.committed_command_count == 0 && self.first_state_hash != self.final_state_hash {
            return Err(inconsistent(
                "a zero-commit run has different first and final state hashes",
            ));
        }
        if self.committed_command_count > 0 && self.first_state_hash == self.final_state_hash {
            return Err(inconsistent(
                "a committed run has identical first and final state hashes",
            ));
        }
        Ok(())
    }
}

fn validate_typed_evidence(
    initial: &PersistedRuntimeState,
    final_state: &PersistedRuntimeState,
    log: &CommandLog,
    receipts: &CausalReceiptSequence,
    hashes: &StateHashSequence,
) -> Result<(), Diagnostic> {
    hashes.validate_command_log(log)?;
    receipts.validate_command_log(initial.state().tick(), log)?;
    let committed = u64::try_from(log.rows().len())
        .map_err(|_| inconsistent("committed command count exceeds u64"))?;
    let expected_final_tick = initial
        .state()
        .tick()
        .checked_add(committed)
        .ok_or_else(|| inconsistent("run final tick overflows u64"))?;
    if final_state.state().tick() != expected_final_tick {
        return Err(inconsistent(
            "final-state tick does not continue from the initial state and committed-command count",
        ));
    }
    if initial.runtime_semantics_digest() != final_state.runtime_semantics_digest() {
        return Err(inconsistent(
            "initial and final states name different runtime semantics",
        ));
    }
    if initial.state_hash() != hashes.first_state_hash()
        || final_state.state_hash() != hashes.final_state_hash()
    {
        return Err(inconsistent(
            "persisted state hashes disagree with the state-hash sequence endpoints",
        ));
    }
    Ok(())
}

fn artifact_digests(
    initial: &PersistedRuntimeState,
    final_state: &PersistedRuntimeState,
    log: &CommandLog,
    receipts: &CausalReceiptSequence,
    hashes: &StateHashSequence,
) -> Vec<RunArtifactDigest> {
    let mut artifacts = vec![
        RunArtifactDigest {
            name: RunArtifactName::InitialState,
            digest: Sha256Digest::of_bytes(&initial.to_canonical_bytes()),
        },
        RunArtifactDigest {
            name: RunArtifactName::FinalState,
            digest: Sha256Digest::of_bytes(&final_state.to_canonical_bytes()),
        },
        RunArtifactDigest {
            name: RunArtifactName::CommandLog,
            digest: Sha256Digest::of_bytes(&log.to_canonical_bytes()),
        },
        RunArtifactDigest {
            name: RunArtifactName::CausalReceipts,
            digest: Sha256Digest::of_bytes(&receipts.to_canonical_bytes()),
        },
        RunArtifactDigest {
            name: RunArtifactName::StateHashes,
            digest: Sha256Digest::of_bytes(&hashes.to_canonical_bytes()),
        },
    ];
    artifacts.sort_by_key(|artifact| artifact.name.as_str());
    artifacts
}

fn normalize_artifacts(
    mut artifacts: Vec<RunArtifactDigest>,
) -> Result<Vec<RunArtifactDigest>, Diagnostic> {
    let actual = artifacts
        .iter()
        .map(|artifact| artifact.name)
        .collect::<BTreeSet<_>>();
    let expected = RunArtifactName::ALL.into_iter().collect::<BTreeSet<_>>();
    if artifacts.len() != RunArtifactName::ALL.len() || actual != expected {
        return Err(inconsistent(
            "run result does not bind each required non-result artifact exactly once",
        ));
    }
    artifacts.sort_by_key(|artifact| artifact.name.as_str());
    Ok(artifacts)
}

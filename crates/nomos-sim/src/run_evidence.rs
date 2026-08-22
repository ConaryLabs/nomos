//! Typed persisted evidence shared by future run and replay orchestration.

use std::collections::BTreeSet;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{
    CanonicalValue, Diagnostic, DiagnosticCode, RepairClass, SchemaId, Sha256Digest, StateHash,
};
use nomos_projection::{Command, CommandArgument};

use crate::receipt::{command_to_canonical, decode_command};
use crate::state_persistence::{
    array, digest, field, invalid, object, require_fields, schema, state_hash, text, uint,
};
use crate::{CausalReceipt, CommandRequest};

/// One committed command and the exact evidence identities surrounding it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandLogRow {
    ordinal: u64,
    request: CommandRequest,
    resolved_command: Command,
    input_state_hash: StateHash,
    resulting_state_hash: StateHash,
    causal_receipt_digest: Sha256Digest,
}

impl CommandLogRow {
    /// Binds one authored request and resolved command to its committed receipt.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` when the request and typed command disagree.
    pub fn new(
        ordinal: u64,
        request: CommandRequest,
        resolved_command: Command,
        input_state_hash: StateHash,
        receipt: &CausalReceipt,
    ) -> Result<Self, Diagnostic> {
        let row = Self {
            ordinal,
            request,
            resolved_command,
            input_state_hash,
            resulting_state_hash: receipt.state_hash(),
            causal_receipt_digest: receipt.digest(),
        };
        row.validate_request_command()?;
        if row.resolved_command != *receipt.command() {
            return Err(inconsistent(
                "command-log resolved command disagrees with its causal receipt",
            ));
        }
        Ok(row)
    }

    /// Zero-based committed-command ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }

    /// Authored unresolved request.
    #[must_use]
    pub const fn request(&self) -> &CommandRequest {
        &self.request
    }

    /// Compiler-projection-resolved typed command.
    #[must_use]
    pub const fn resolved_command(&self) -> &Command {
        &self.resolved_command
    }

    /// State hash consumed by this command.
    #[must_use]
    pub const fn input_state_hash(&self) -> StateHash {
        self.input_state_hash
    }

    /// State hash produced by this command.
    #[must_use]
    pub const fn resulting_state_hash(&self) -> StateHash {
        self.resulting_state_hash
    }

    /// SHA-256 identity of the exact canonical causal receipt.
    #[must_use]
    pub const fn causal_receipt_digest(&self) -> Sha256Digest {
        self.causal_receipt_digest
    }

    fn from_canonical(value: &CanonicalValue) -> Result<Self, Diagnostic> {
        let fields = object(value, "command-log row")?;
        require_fields(
            fields,
            &[
                "causal_receipt_digest",
                "input_state_hash",
                "ordinal",
                "request",
                "resolved_command",
                "resulting_state_hash",
            ],
            "command-log row",
        )?;
        let row = Self {
            ordinal: uint(field(fields, "ordinal")?, "command-log ordinal")?,
            request: CommandRequest::from_canonical(field(fields, "request")?)?,
            resolved_command: decode_command(field(fields, "resolved_command")?)?,
            input_state_hash: state_hash(field(fields, "input_state_hash")?)?,
            resulting_state_hash: state_hash(field(fields, "resulting_state_hash")?)?,
            causal_receipt_digest: digest(
                field(fields, "causal_receipt_digest")?,
                "causal receipt digest",
            )?,
        };
        row.validate_request_command()?;
        Ok(row)
    }

    fn validate_request_command(&self) -> Result<(), Diagnostic> {
        let argument_matches = match (self.request.argument(), self.resolved_command.argument()) {
            (None, CommandArgument::None) => true,
            (Some(requested), CommandArgument::Credential(resolved)) => requested == resolved,
            _ => false,
        };
        if self.request.entity() != self.resolved_command.namespace().entity()
            || self.request.action() != self.resolved_command.action()
            || !argument_matches
        {
            return Err(inconsistent(
                "command-log request disagrees with its resolved typed command",
            ));
        }
        Ok(())
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "causal_receipt_digest",
                CanonicalValue::text(self.causal_receipt_digest.to_hex()),
            ),
            (
                "input_state_hash",
                CanonicalValue::text(self.input_state_hash.to_hex()),
            ),
            ("ordinal", CanonicalValue::Uint(self.ordinal)),
            ("request", self.request.to_canonical()),
            (
                "resolved_command",
                command_to_canonical(&self.resolved_command),
            ),
            (
                "resulting_state_hash",
                CanonicalValue::text(self.resulting_state_hash.to_hex()),
            ),
        ])
    }
}

/// Canonical ordered log of successfully committed commands.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandLog {
    schema: SchemaId,
    rows: Vec<CommandLogRow>,
}

impl CommandLog {
    /// Builds a log whose ordinals and state-hash chain are contiguous.
    ///
    /// An empty log is valid evidence for a run rejected before its first
    /// command committed.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for an ordinal or hash-chain disagreement.
    pub fn new(rows: Vec<CommandLogRow>) -> Result<Self, Diagnostic> {
        for (index, row) in rows.iter().enumerate() {
            let expected = u64::try_from(index)
                .map_err(|_| inconsistent("command-log row count exceeds u64"))?;
            if row.ordinal != expected {
                return Err(inconsistent(
                    "command-log ordinals are not contiguous from zero",
                ));
            }
            row.validate_request_command()?;
            if index > 0 && rows[index - 1].resulting_state_hash != row.input_state_hash {
                return Err(inconsistent(
                    "command-log input and resulting state hashes do not form one chain",
                ));
            }
        }
        Ok(Self {
            schema: crate::command_log_schema(),
            rows,
        })
    }

    /// Committed rows in execution order.
    #[must_use]
    pub fn rows(&self) -> &[CommandLogRow] {
        &self.rows
    }

    /// Strictly reconstructs one canonical command log.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, typed-value, ordinal, or chain
    /// disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "command log")?;
        require_fields(fields, &["rows", "schema"], "command log")?;
        let schema = schema(field(fields, "schema")?, "command-log schema")?;
        if schema != crate::command_log_schema() {
            return Err(invalid("command log names an unsupported schema"));
        }
        let rows = array(field(fields, "rows")?, "command-log rows")?
            .iter()
            .map(CommandLogRow::from_canonical)
            .collect::<Result<Vec<_>, _>>()?;
        let log = Self::new(rows)?;
        if log.to_canonical_bytes() != bytes {
            return Err(invalid(
                "command log does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(log)
    }

    /// Verifies every row against one exact typed causal receipt.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for count, command, tick, state-hash, or digest
    /// disagreement.
    pub fn validate_receipts(&self, receipts: &[CausalReceipt]) -> Result<(), Diagnostic> {
        if receipts.len() != self.rows.len() {
            return Err(inconsistent(
                "command-log and causal-receipt counts disagree",
            ));
        }
        for (index, (row, receipt)) in self.rows.iter().zip(receipts).enumerate() {
            if row.resolved_command != *receipt.command()
                || row.resulting_state_hash != receipt.state_hash()
                || row.causal_receipt_digest != receipt.digest()
            {
                return Err(inconsistent(
                    "command-log row disagrees with its typed causal receipt",
                ));
            }
            if index > 0 {
                let expected_tick = receipts[index - 1]
                    .tick()
                    .checked_add(1)
                    .ok_or_else(|| inconsistent("causal-receipt tick chain overflows u64"))?;
                if receipt.tick() != expected_tick {
                    return Err(inconsistent(
                        "causal-receipt ticks are not contiguous in command-log order",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Exact canonical command-log bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "rows",
                CanonicalValue::Array(self.rows.iter().map(CommandLogRow::to_canonical).collect()),
            ),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}

/// One ordinal runtime snapshot identity in a run's state-hash chain.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StateHashRow {
    ordinal: u64,
    state_hash: StateHash,
}

impl StateHashRow {
    /// Snapshot ordinal, where zero identifies the initial state.
    #[must_use]
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }

    /// Authoritative snapshot hash.
    #[must_use]
    pub const fn state_hash(&self) -> StateHash {
        self.state_hash
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("ordinal", CanonicalValue::Uint(self.ordinal)),
            ("state_hash", CanonicalValue::text(self.state_hash.to_hex())),
        ])
    }
}

/// Initial state hash followed by one resulting hash per committed command.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StateHashSequence {
    schema: SchemaId,
    rows: Vec<StateHashRow>,
}

impl StateHashSequence {
    /// Builds the exact snapshot chain represented by one command log.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` when the supplied initial hash disagrees with the first
    /// committed row or the collection size cannot be represented.
    pub fn from_command_log(
        initial_state_hash: StateHash,
        log: &CommandLog,
    ) -> Result<Self, Diagnostic> {
        let capacity = log
            .rows
            .len()
            .checked_add(1)
            .ok_or_else(|| inconsistent("state-hash row count exceeds usize"))?;
        let mut rows = Vec::with_capacity(capacity);
        rows.push(StateHashRow {
            ordinal: 0,
            state_hash: initial_state_hash,
        });
        rows.extend(
            log.rows
                .iter()
                .enumerate()
                .map(|(index, row)| StateHashRow {
                    ordinal: u64::try_from(index + 1).expect("an in-memory row count fits u64"),
                    state_hash: row.resulting_state_hash,
                }),
        );
        let sequence = Self {
            schema: crate::state_hash_sequence_schema(),
            rows,
        };
        sequence.validate_command_log(log)?;
        Ok(sequence)
    }

    /// Snapshot identities from the initial state through the final commit.
    #[must_use]
    pub fn rows(&self) -> &[StateHashRow] {
        &self.rows
    }

    /// First state hash in the sequence.
    #[must_use]
    pub fn first_state_hash(&self) -> StateHash {
        self.rows[0].state_hash
    }

    /// Final state hash in the sequence.
    #[must_use]
    pub fn final_state_hash(&self) -> StateHash {
        self.rows[self.rows.len() - 1].state_hash
    }

    /// Strictly reconstructs a nonempty canonical state-hash sequence.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, hash, or contiguous-ordinal
    /// disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "state-hash sequence")?;
        require_fields(fields, &["rows", "schema"], "state-hash sequence")?;
        let schema = schema(field(fields, "schema")?, "state-hash-sequence schema")?;
        if schema != crate::state_hash_sequence_schema() {
            return Err(invalid("state-hash sequence names an unsupported schema"));
        }
        let rows = array(field(fields, "rows")?, "state-hash rows")?
            .iter()
            .map(|row| {
                let fields = object(row, "state-hash row")?;
                require_fields(fields, &["ordinal", "state_hash"], "state-hash row")?;
                Ok(StateHashRow {
                    ordinal: uint(field(fields, "ordinal")?, "state-hash ordinal")?,
                    state_hash: state_hash(field(fields, "state_hash")?)?,
                })
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        validate_hash_rows(&rows)?;
        let sequence = Self { schema, rows };
        if sequence.to_canonical_bytes() != bytes {
            return Err(invalid(
                "state-hash sequence does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(sequence)
    }

    /// Verifies this sequence against every command-log input and result.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for a count or row-by-row hash disagreement.
    pub fn validate_command_log(&self, log: &CommandLog) -> Result<(), Diagnostic> {
        if self.rows.len() != log.rows.len() + 1 {
            return Err(inconsistent(
                "state-hash sequence length does not match the command log",
            ));
        }
        for (index, row) in log.rows.iter().enumerate() {
            if self.rows[index].state_hash != row.input_state_hash
                || self.rows[index + 1].state_hash != row.resulting_state_hash
            {
                return Err(inconsistent(
                    "state-hash sequence disagrees with a command-log row",
                ));
            }
        }
        Ok(())
    }

    /// Exact canonical state-hash-sequence bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "rows",
                CanonicalValue::Array(self.rows.iter().map(StateHashRow::to_canonical).collect()),
            ),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}

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
    /// Builds one typed artifact binding.
    #[must_use]
    pub const fn new(name: RunArtifactName, digest: Sha256Digest) -> Self {
        Self { name, digest }
    }

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

/// Terminal status of a future run bundle.
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

/// Content-binding record written as a future run bundle's `result.json`.
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
    /// # Errors
    ///
    /// Returns `EK0816` when artifact coverage or the typed log/hash evidence
    /// disagrees.
    pub fn completed(
        input_package_digest: Sha256Digest,
        runtime_semantics_digest: Sha256Digest,
        artifacts: Vec<RunArtifactDigest>,
        log: &CommandLog,
        hashes: &StateHashSequence,
    ) -> Result<Self, Diagnostic> {
        Self::build(
            input_package_digest,
            runtime_semantics_digest,
            RunStatus::Completed,
            artifacts,
            log,
            hashes,
            None,
        )
    }

    /// Builds content-binding evidence for a run stopped by rejection.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` under the same conditions as [`Self::completed`].
    pub fn rejected(
        input_package_digest: Sha256Digest,
        runtime_semantics_digest: Sha256Digest,
        artifacts: Vec<RunArtifactDigest>,
        log: &CommandLog,
        hashes: &StateHashSequence,
        rejection_diagnostic: DiagnosticCode,
    ) -> Result<Self, Diagnostic> {
        Self::build(
            input_package_digest,
            runtime_semantics_digest,
            RunStatus::Rejected,
            artifacts,
            log,
            hashes,
            Some(rejection_diagnostic),
        )
    }

    fn build(
        input_package_digest: Sha256Digest,
        runtime_semantics_digest: Sha256Digest,
        status: RunStatus,
        artifacts: Vec<RunArtifactDigest>,
        log: &CommandLog,
        hashes: &StateHashSequence,
        rejection_diagnostic: Option<DiagnosticCode>,
    ) -> Result<Self, Diagnostic> {
        hashes.validate_command_log(log)?;
        let committed_command_count = u64::try_from(log.rows.len())
            .map_err(|_| inconsistent("committed command count exceeds u64"))?;
        let result = Self {
            schema: crate::run_result_schema(),
            input_package_digest,
            runtime_semantics_digest,
            status,
            artifacts: normalize_artifacts(artifacts)?,
            first_state_hash: hashes.first_state_hash(),
            final_state_hash: hashes.final_state_hash(),
            committed_command_count,
            rejection_diagnostic,
        };
        result.validate_internal()?;
        result.validate_typed_artifact_digests(log, hashes)?;
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

    /// Verifies count and endpoint hashes against typed log/hash evidence.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for any cross-object disagreement.
    pub fn validate_evidence(
        &self,
        log: &CommandLog,
        hashes: &StateHashSequence,
    ) -> Result<(), Diagnostic> {
        hashes.validate_command_log(log)?;
        let count = u64::try_from(log.rows.len())
            .map_err(|_| inconsistent("committed command count exceeds u64"))?;
        if self.committed_command_count != count
            || self.first_state_hash != hashes.first_state_hash()
            || self.final_state_hash != hashes.final_state_hash()
        {
            return Err(inconsistent(
                "run result disagrees with command-log or state-hash evidence",
            ));
        }
        self.validate_typed_artifact_digests(log, hashes)?;
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

    fn validate_typed_artifact_digests(
        &self,
        log: &CommandLog,
        hashes: &StateHashSequence,
    ) -> Result<(), Diagnostic> {
        let expected = [
            (
                RunArtifactName::CommandLog,
                Sha256Digest::of_bytes(&log.to_canonical_bytes()),
            ),
            (
                RunArtifactName::StateHashes,
                Sha256Digest::of_bytes(&hashes.to_canonical_bytes()),
            ),
        ];
        for (name, digest) in expected {
            let recorded = self
                .artifacts
                .iter()
                .find(|artifact| artifact.name == name)
                .expect("run-result artifact coverage was already validated");
            if recorded.digest != digest {
                return Err(inconsistent(
                    "run result artifact digest disagrees with typed run evidence",
                ));
            }
        }
        Ok(())
    }
}

fn validate_hash_rows(rows: &[StateHashRow]) -> Result<(), Diagnostic> {
    if rows.is_empty() {
        return Err(inconsistent(
            "a state-hash sequence must contain its initial state",
        ));
    }
    for (index, row) in rows.iter().enumerate() {
        let expected =
            u64::try_from(index).map_err(|_| inconsistent("state-hash row count exceeds u64"))?;
        if row.ordinal != expected {
            return Err(inconsistent(
                "state-hash ordinals are not contiguous from zero",
            ));
        }
    }
    Ok(())
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

fn inconsistent(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

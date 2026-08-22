//! Typed persisted evidence shared by run and replay orchestration.

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, Diagnostic, RepairClass, SchemaId, Sha256Digest, StateHash};
use nomos_projection::{Command, CommandArgument};

use crate::receipt::{command_to_canonical, decode_command};
use crate::state_persistence::{
    array, digest, field, invalid, object, require_fields, schema, state_hash, uint,
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
    pub fn validate_receipts(
        &self,
        initial_tick: u64,
        receipts: &[CausalReceipt],
    ) -> Result<(), Diagnostic> {
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
            let offset = u64::try_from(index)
                .map_err(|_| inconsistent("causal-receipt count exceeds u64"))?
                .checked_add(1)
                .ok_or_else(|| inconsistent("causal-receipt tick offset overflows u64"))?;
            let expected_tick = initial_tick
                .checked_add(offset)
                .ok_or_else(|| inconsistent("causal-receipt tick chain overflows u64"))?;
            if receipt.tick() != expected_tick {
                return Err(inconsistent(
                    "causal-receipt ticks do not continue from the initial state",
                ));
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

/// Canonical ordered collection written as `causal-receipts.json`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CausalReceiptSequence {
    schema: SchemaId,
    receipts: Vec<CausalReceipt>,
}

impl CausalReceiptSequence {
    /// Builds a receipt collection anchored to one initial tick and command log.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` when count, command, tick, state-hash, or digest
    /// evidence disagrees.
    pub fn new(
        initial_tick: u64,
        log: &CommandLog,
        receipts: Vec<CausalReceipt>,
    ) -> Result<Self, Diagnostic> {
        log.validate_receipts(initial_tick, &receipts)?;
        Ok(Self {
            schema: crate::causal_receipt_sequence_schema(),
            receipts,
        })
    }

    /// Strictly decoded causal receipts in committed-command order.
    #[must_use]
    pub fn receipts(&self) -> &[CausalReceipt] {
        &self.receipts
    }

    /// Strictly reconstructs one canonical receipt sequence.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, nested-receipt, or tick-order
    /// disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "causal-receipt sequence")?;
        require_fields(fields, &["receipts", "schema"], "causal-receipt sequence")?;
        let schema = schema(field(fields, "schema")?, "causal-receipt-sequence schema")?;
        if schema != crate::causal_receipt_sequence_schema() {
            return Err(invalid(
                "causal-receipt sequence names an unsupported schema",
            ));
        }
        let receipts = array(field(fields, "receipts")?, "causal receipts")?
            .iter()
            .map(|receipt| CausalReceipt::from_canonical_bytes(&receipt.to_canonical_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        validate_receipt_tick_continuity(&receipts)?;
        let sequence = Self { schema, receipts };
        if sequence.to_canonical_bytes() != bytes {
            return Err(invalid(
                "causal-receipt sequence does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(sequence)
    }

    /// Verifies this sequence against a command log and its input state's tick.
    ///
    /// # Errors
    ///
    /// Returns `EK0816` for any cross-object disagreement.
    pub fn validate_command_log(
        &self,
        initial_tick: u64,
        log: &CommandLog,
    ) -> Result<(), Diagnostic> {
        log.validate_receipts(initial_tick, &self.receipts)
    }

    /// Exact canonical causal-receipt-sequence bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "receipts",
                CanonicalValue::Array(
                    self.receipts
                        .iter()
                        .map(CausalReceipt::to_canonical)
                        .collect(),
                ),
            ),
            ("schema", self.schema.to_canonical()),
        ])
        .to_canonical_bytes()
    }
}

fn validate_receipt_tick_continuity(receipts: &[CausalReceipt]) -> Result<(), Diagnostic> {
    for pair in receipts.windows(2) {
        let expected = pair[0]
            .tick()
            .checked_add(1)
            .ok_or_else(|| inconsistent("causal-receipt tick chain overflows u64"))?;
        if pair[1].tick() != expected {
            return Err(inconsistent(
                "causal-receipt ticks are not contiguous in sequence order",
            ));
        }
    }
    Ok(())
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

pub(crate) fn inconsistent(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

//! Atomic filesystem publication and strict opening of runtime run bundles.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nomos_compiler::OpenedCompiledWorld;
use nomos_core::{Diagnostic, RepairClass, Sha256Digest};
use nomos_sim::{
    CausalReceiptSequence, CommandLog, PersistedRuntimeState, RunExecution, RunResult,
    StateHashSequence, validate_committed_evidence,
};

const INITIAL_STATE_FILE: &str = "initial-state.json";
const FINAL_STATE_FILE: &str = "final-state.json";
const COMMAND_LOG_FILE: &str = "command-log.json";
const CAUSAL_RECEIPTS_FILE: &str = "causal-receipts.json";
const STATE_HASHES_FILE: &str = "state-hashes.json";
const RESULT_FILE: &str = "result.json";
const RUN_FILES: [&str; 6] = [
    CAUSAL_RECEIPTS_FILE,
    COMMAND_LOG_FILE,
    FINAL_STATE_FILE,
    INITIAL_STATE_FILE,
    RESULT_FILE,
    STATE_HASHES_FILE,
];

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn has_run_bundle_shape(root: &Path) -> Result<bool, Diagnostic> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_failure(root, &error)),
    };
    if !metadata.file_type().is_dir() {
        return Ok(false);
    }
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(root).map_err(|error| io_failure(root, &error))? {
        let entry = entry.map_err(|error| io_failure(root, &error))?;
        if !entry
            .file_type()
            .map_err(|error| io_failure(&entry.path(), &error))?
            .is_file()
        {
            return Ok(false);
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return Ok(false);
        };
        names.insert(name);
    }
    Ok(names == RUN_FILES.into_iter().map(str::to_owned).collect())
}

/// One completely decoded and cross-validated filesystem run bundle.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpenedRunBundle {
    root: PathBuf,
    initial: PersistedRuntimeState,
    final_state: PersistedRuntimeState,
    log: CommandLog,
    receipts: CausalReceiptSequence,
    hashes: StateHashSequence,
    result: RunResult,
    result_digest: Sha256Digest,
}

impl OpenedRunBundle {
    /// Verified bundle directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Verified initial persisted state.
    #[must_use]
    pub const fn initial(&self) -> &PersistedRuntimeState {
        &self.initial
    }

    /// Verified final persisted state.
    #[must_use]
    pub const fn final_state(&self) -> &PersistedRuntimeState {
        &self.final_state
    }

    /// Verified committed command log.
    #[must_use]
    pub const fn command_log(&self) -> &CommandLog {
        &self.log
    }

    /// Verified causal receipt sequence.
    #[must_use]
    pub const fn causal_receipts(&self) -> &CausalReceiptSequence {
        &self.receipts
    }

    /// Verified state-hash sequence.
    #[must_use]
    pub const fn state_hashes(&self) -> &StateHashSequence {
        &self.hashes
    }

    /// Verified content-binding result.
    #[must_use]
    pub const fn result(&self) -> &RunResult {
        &self.result
    }

    /// SHA-256 identity of the exact `result.json` bytes.
    #[must_use]
    pub const fn result_digest(&self) -> Sha256Digest {
        self.result_digest
    }
}

/// Proves that a run output path does not exist before runtime work begins.
///
/// # Errors
///
/// Returns `EK0817` for any existing filesystem entry or `EK0820` when the
/// host refuses the check.
pub fn require_available_run_output(root: &Path) -> Result<(), Diagnostic> {
    let root = lexical_path(root);
    match fs::symlink_metadata(&root) {
        Ok(_) => Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUN_BUNDLE_OUTPUT_EXISTS,
            format!(
                "`{}` already exists; run bundles are immutable evidence and are never written over",
                root.display()
            ),
        )
        .with_repair(RepairClass::WriteToNewOutputPath)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure(&root, &error)),
    }
}

/// Publishes one complete typed execution through a verified sibling staging
/// directory and one final rename.
///
/// # Errors
///
/// Returns the first destination, I/O, entry-shape, typed decoding, digest, or
/// cross-artifact diagnostic. Failure removes the staging directory and never
/// changes an existing destination.
pub fn write_run_bundle(
    execution: &RunExecution,
    world: &OpenedCompiledWorld,
    root: &Path,
) -> Result<OpenedRunBundle, Diagnostic> {
    write_run_bundle_steps(execution, world, root, None, false)
}

fn write_run_bundle_steps(
    execution: &RunExecution,
    world: &OpenedCompiledWorld,
    root: &Path,
    fail_after_member_writes: Option<usize>,
    fail_staged_verification: bool,
) -> Result<OpenedRunBundle, Diagnostic> {
    let root = lexical_path(root);
    require_available_run_output(&root)?;
    execution.validate(world.simulation())?;
    let parent = bundle_parent(&root);
    fs::create_dir_all(parent).map_err(|error| io_failure(parent, &error))?;
    let staging = create_staging_directory(&root)?;
    let members = execution_members(execution);
    let staged = (|| {
        for (written, (name, bytes)) in members.iter().enumerate() {
            if fail_after_member_writes == Some(written) {
                return Err(io_failure(
                    &staging.join(name),
                    &io::Error::other("injected run-bundle write failure"),
                ));
            }
            let path = staging.join(name);
            fs::write(&path, bytes).map_err(|error| io_failure(&path, &error))?;
        }
        if fail_staged_verification {
            let path = staging.join(RESULT_FILE);
            fs::write(&path, b"{}").map_err(|error| io_failure(&path, &error))?;
        }
        let mut opened = open_run_bundle(&staging, world)?;
        require_available_run_output(&root)?;
        fs::rename(&staging, &root).map_err(|error| publication_failure(&root, &error))?;
        opened.root = root.clone();
        Ok(opened)
    })();

    match staged {
        Ok(opened) => Ok(opened),
        Err(diagnostic) => {
            cleanup_staging(&staging).map_err(|cleanup| {
                Diagnostic::new(
                    nomos_core::diagnostic::codes::RUN_BUNDLE_IO,
                    format!(
                        "run-bundle write failed ({diagnostic}); staging cleanup also failed: {}",
                        cleanup.message()
                    ),
                )
            })?;
            Err(diagnostic)
        }
    }
}

/// Opens and strictly verifies all six artifacts in one run bundle against an
/// already verified compiled world.
///
/// The caller must keep both trees quiescent during verification. Existing
/// symlinks are rejected, but this is not a race-safe hostile-filesystem API.
///
/// # Errors
///
/// Returns the first filesystem-entry, typed decoding, package identity,
/// digest, or cross-artifact consistency diagnostic.
pub fn open_run_bundle(
    root: &Path,
    world: &OpenedCompiledWorld,
) -> Result<OpenedRunBundle, Diagnostic> {
    let root = lexical_path(root);
    require_bundle_root(&root)?;
    require_exact_entries(&root)?;
    let members = read_members(&root)?;

    let initial = PersistedRuntimeState::from_canonical_bytes(
        member(&members, INITIAL_STATE_FILE)?,
        world.simulation(),
    )?;
    let final_state = PersistedRuntimeState::from_canonical_bytes(
        member(&members, FINAL_STATE_FILE)?,
        world.simulation(),
    )?;
    let log = CommandLog::from_canonical_bytes(member(&members, COMMAND_LOG_FILE)?)?;
    let receipts =
        CausalReceiptSequence::from_canonical_bytes(member(&members, CAUSAL_RECEIPTS_FILE)?)?;
    let hashes = StateHashSequence::from_canonical_bytes(member(&members, STATE_HASHES_FILE)?)?;
    let result_bytes = member(&members, RESULT_FILE)?;
    let result = RunResult::from_canonical_bytes(result_bytes)?;
    result.validate_evidence(&initial, &final_state, &log, &receipts, &hashes)?;
    validate_committed_evidence(
        world.simulation(),
        &initial,
        &final_state,
        &log,
        &receipts,
        &hashes,
    )?;
    if result.input_package_digest() != world.package_digest() {
        return Err(inconsistent(
            "run result belongs to a different compiled package",
        ));
    }

    Ok(OpenedRunBundle {
        root,
        initial,
        final_state,
        log,
        receipts,
        hashes,
        result,
        result_digest: Sha256Digest::of_bytes(result_bytes),
    })
}

fn execution_members(execution: &RunExecution) -> BTreeMap<&'static str, Vec<u8>> {
    BTreeMap::from([
        (
            CAUSAL_RECEIPTS_FILE,
            execution.causal_receipts().to_canonical_bytes(),
        ),
        (
            COMMAND_LOG_FILE,
            execution.command_log().to_canonical_bytes(),
        ),
        (
            FINAL_STATE_FILE,
            execution.final_state().to_canonical_bytes(),
        ),
        (INITIAL_STATE_FILE, execution.initial().to_canonical_bytes()),
        (RESULT_FILE, execution.result().to_canonical_bytes()),
        (
            STATE_HASHES_FILE,
            execution.state_hashes().to_canonical_bytes(),
        ),
    ])
}

fn read_members(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, Diagnostic> {
    RUN_FILES
        .iter()
        .map(|name| {
            let path = root.join(name);
            require_entry_file(&path)?;
            let bytes = fs::read(&path).map_err(|error| io_failure(&path, &error))?;
            Ok(((*name).to_owned(), bytes))
        })
        .collect()
}

fn member<'a>(
    members: &'a BTreeMap<String, Vec<u8>>,
    name: &'static str,
) -> Result<&'a [u8], Diagnostic> {
    members.get(name).map(Vec::as_slice).ok_or_else(|| {
        Diagnostic::new(
            nomos_core::diagnostic::codes::RUN_BUNDLE_ENTRY_SET_INVALID,
            format!("run bundle is missing `{name}`"),
        )
        .with_repair(RepairClass::SupplyMissingMember)
    })
}

fn require_exact_entries(root: &Path) -> Result<(), Diagnostic> {
    let mut found = BTreeSet::new();
    for entry in fs::read_dir(root).map_err(|error| io_failure(root, &error))? {
        let entry = entry.map_err(|error| io_failure(root, &error))?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return Err(entry_set_invalid(
                "run-bundle entry name is not valid UTF-8",
            ));
        };
        let file_type = entry
            .file_type()
            .map_err(|error| io_failure(&entry.path(), &error))?;
        if !file_type.is_file() {
            return Err(entry_type_invalid(
                &entry.path(),
                "run-bundle entries must be regular files",
            ));
        }
        found.insert(name);
    }
    let expected = RUN_FILES.iter().map(|name| (*name).to_owned()).collect();
    if found != expected {
        return Err(entry_set_invalid(
            "run bundle must contain exactly the six declared artifacts",
        ));
    }
    Ok(())
}

fn require_bundle_root(root: &Path) -> Result<(), Diagnostic> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(()),
        Ok(_) => Err(entry_type_invalid(
            root,
            "a run-bundle root must be a directory and not a symlink",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Err(entry_set_invalid("run-bundle root is missing"))
        }
        Err(error) => Err(io_failure(root, &error)),
    }
}

fn require_entry_file(path: &Path) -> Result<(), Diagnostic> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(entry_type_invalid(
            path,
            "run-bundle artifacts must be regular files",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(entry_set_invalid(format!(
            "run bundle is missing `{}`",
            path.file_name().unwrap_or_default().to_string_lossy()
        ))),
        Err(error) => Err(io_failure(path, &error)),
    }
}

fn create_staging_directory(root: &Path) -> Result<PathBuf, Diagnostic> {
    let Some(file_name) = root.file_name() else {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUN_BUNDLE_IO,
            format!("`{}` has no run-bundle directory name", root.display()),
        ));
    };
    let parent = bundle_parent(root);
    for _ in 0..64 {
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.staging-{}-{counter}",
            file_name.to_string_lossy(),
            std::process::id()
        );
        let staging = parent.join(name);
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_failure(&staging, &error)),
        }
    }
    Err(Diagnostic::new(
        nomos_core::diagnostic::codes::RUN_BUNDLE_IO,
        format!(
            "could not allocate a fresh sibling staging directory for `{}`",
            root.display()
        ),
    ))
}

fn cleanup_staging(staging: &Path) -> Result<(), Diagnostic> {
    match fs::remove_dir_all(staging) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure(staging, &error)),
    }
}

fn publication_failure(root: &Path, error: &io::Error) -> Diagnostic {
    match fs::symlink_metadata(root) {
        Ok(_) => Diagnostic::new(
            nomos_core::diagnostic::codes::RUN_BUNDLE_OUTPUT_EXISTS,
            format!(
                "`{}` appeared before run-bundle publication and was left untouched",
                root.display()
            ),
        )
        .with_repair(RepairClass::WriteToNewOutputPath),
        Err(_) => io_failure(root, error),
    }
}

fn bundle_parent(root: &Path) -> &Path {
    root.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn lexical_path(path: &Path) -> PathBuf {
    path.components().collect()
}

fn entry_set_invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUN_BUNDLE_ENTRY_SET_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

fn entry_type_invalid(path: &Path, reason: &str) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUN_BUNDLE_ENTRY_TYPE_INVALID,
        format!("`{}` has an invalid entry type: {reason}", path.display()),
    )
    .with_repair(RepairClass::RebuildFromSource)
}

fn inconsistent(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}

fn io_failure(path: &Path, error: &io::Error) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUN_BUNDLE_IO,
        format!("`{}`: {error}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{initial_state_from_package, open_compiled_world, write_compiled_world};
    use nomos_compiler::compile_world_package;
    use nomos_core::SourcePath;
    use nomos_sim::{CommandScript, PersistedRuntimeState, execute_requests};

    fn fresh_root(label: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/tmp")
            .join("sw-j-publication-faults")
            .join(std::process::id().to_string())
            .join(label);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        root
    }

    fn world_and_execution(root: &Path) -> (OpenedCompiledWorld, RunExecution) {
        let compiled = compile_world_package(
            include_str!("../../../fixtures/gaol.nomos"),
            SourcePath::new("fixtures/gaol.nomos").unwrap(),
        )
        .unwrap();
        write_compiled_world(&compiled, root).unwrap();
        let world = open_compiled_world(root).unwrap();
        let initial = PersistedRuntimeState::new(
            world.simulation(),
            initial_state_from_package(&world).unwrap(),
        )
        .unwrap();
        let script = CommandScript::from_bytes(
            b"schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\n",
        )
        .unwrap();
        let execution = execute_requests(
            world.simulation(),
            world.package_digest(),
            initial,
            script.requests(),
        )
        .unwrap();
        (world, execution)
    }

    fn no_staging_entries(root: &Path) -> bool {
        let parent = bundle_parent(root);
        !parent.exists()
            || fs::read_dir(parent).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains(".staging-")
            })
    }

    #[test]
    fn member_write_failure_removes_staging_and_leaves_destination_absent() {
        let base = fresh_root("member-write");
        let world_root = base.join("world");
        let output = base.join("evidence.run");
        let (world, execution) = world_and_execution(&world_root);
        let rejected =
            write_run_bundle_steps(&execution, &world, &output, Some(2), false).unwrap_err();
        assert_eq!(
            rejected.code(),
            nomos_core::diagnostic::codes::RUN_BUNDLE_IO
        );
        assert!(!output.exists());
        assert!(no_staging_entries(&output));
    }

    #[test]
    fn staged_verification_failure_removes_staging_and_leaves_destination_absent() {
        let base = fresh_root("staged-verification");
        let world_root = base.join("world");
        let output = base.join("evidence.run");
        let (world, execution) = world_and_execution(&world_root);
        assert!(write_run_bundle_steps(&execution, &world, &output, None, true).is_err());
        assert!(!output.exists());
        assert!(no_staging_entries(&output));
    }
}

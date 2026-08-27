//! Exact command grammar and immutable filesystem publication.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nomos_core::{RepairClass, SourcePath, SourceSpan};

use crate::diagnostic::{ObservedError, codes, render_rejection};
use crate::json;
use crate::plan::{ScenePlan, compile};

/// The exact two-line help response including its final LF.
pub const HELP: &str = "usage: nomos-observed-scene compile --input <scene.json> --out <plan.json>\n       nomos-observed-scene help\n";

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stable process exit classification.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ExitCode {
    /// Command completed.
    Completed,
    /// Input or plan semantics were rejected.
    Rejected,
    /// Argument grammar was rejected.
    InvalidUsage,
    /// The host environment could not complete the operation.
    Environment,
}

impl ExitCode {
    /// Numeric operating-system exit code.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Completed => 0,
            Self::Rejected => 1,
            Self::InvalidUsage => 2,
            Self::Environment => 3,
        }
    }
}

/// One completed invocation with exact stdout bytes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Execution {
    exit: ExitCode,
    stdout: Vec<u8>,
}

impl Execution {
    /// Process exit classification.
    #[must_use]
    pub const fn exit(&self) -> ExitCode {
        self.exit
    }

    /// Bytes written to stdout.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }
}

enum Command {
    Help,
    Compile { input: PathBuf, output: PathBuf },
}

/// Parses and executes exactly one supported command.
#[must_use]
pub fn execute(arguments: impl IntoIterator<Item = OsString>) -> Execution {
    match parse(arguments) {
        Ok(Command::Help) => Execution {
            exit: ExitCode::Completed,
            stdout: HELP.as_bytes().to_vec(),
        },
        Ok(Command::Compile { input, output }) => match compile_file(&input, &output) {
            Ok(()) => Execution {
                exit: ExitCode::Completed,
                stdout: Vec::new(),
            },
            Err(error) => Execution {
                exit: exit_for(error.code()),
                stdout: render_rejection(&error),
            },
        },
        Err(error) => Execution {
            exit: ExitCode::InvalidUsage,
            stdout: render_rejection(&error),
        },
    }
}

fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, ObservedError> {
    let arguments = arguments
        .into_iter()
        .enumerate()
        .map(|(index, argument)| {
            argument.into_string().map_err(|_| {
                ObservedError::new(codes::USAGE, format!("argument {} is not UTF-8", index + 1))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    match arguments.as_slice() {
        [help] if help == "help" => Ok(Command::Help),
        [name, input_flag, input, output_flag, output]
            if name == "compile"
                && input_flag == "--input"
                && output_flag == "--out"
                && !input.is_empty()
                && !output.is_empty()
                && !input.starts_with('-')
                && !output.starts_with('-') =>
        {
            Ok(Command::Compile {
                input: PathBuf::from(input),
                output: PathBuf::from(output),
            })
        }
        _ => Err(ObservedError::new(
            codes::USAGE,
            "arguments do not match the declared command grammar",
        )),
    }
}

fn exit_for(code: crate::ObservedCode) -> ExitCode {
    match code {
        codes::USAGE => ExitCode::InvalidUsage,
        codes::INPUT_UNREADABLE | codes::OUTPUT_IO => ExitCode::Environment,
        _ => ExitCode::Rejected,
    }
}

fn compile_file(input: &Path, output: &Path) -> Result<(), ObservedError> {
    require_regular_input(input)?;
    let mut bytes = Vec::new();
    File::open(input)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|error| input_io(format!("input could not be read: {error}")))?;

    require_available_output(input, output)?;

    let span = whole_input_span(input, bytes.len());
    let plan = compile(&bytes).map_err(|error| match &span {
        Some(span) => error.with_default_span(span.clone()),
        None => error,
    })?;
    publish_plan(output, &plan)
}

fn require_regular_input(path: &Path) -> Result<(), ObservedError> {
    if path.as_os_str().is_empty() || has_symlink_component(path, true, codes::INPUT_UNREADABLE)? {
        return Err(input_io("input must be one non-symlinked regular file"));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| input_io(format!("input metadata is unavailable: {error}")))?;
    if !metadata.file_type().is_file() {
        return Err(input_io("input must be one regular file"));
    }
    Ok(())
}

fn require_available_output(input: &Path, output: &Path) -> Result<(), ObservedError> {
    if output.as_os_str().is_empty() {
        return Err(output_unavailable("output path must not be empty"));
    }
    match fs::symlink_metadata(output) {
        Ok(_) => return Err(output_unavailable("output already exists")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ObservedError::new(
                codes::OUTPUT_IO,
                format!("output metadata is unavailable: {error}"),
            ));
        }
    }
    if has_symlink_component(output, false, codes::OUTPUT_IO)? {
        return Err(output_unavailable("output traverses a symlinked root"));
    }
    let input_resolved = fs::canonicalize(input)
        .map_err(|error| input_io(format!("input cannot be resolved: {error}")))?;
    let output_resolved = resolve_missing_leaf(output)?;
    if output_resolved == input_resolved {
        return Err(output_unavailable("output aliases the immutable input"));
    }
    Ok(())
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| component == Component::ParentDir)
}

fn has_symlink_component(
    path: &Path,
    include_leaf: bool,
    error_code: crate::ObservedCode,
) -> Result<bool, ObservedError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| ObservedError::new(error_code, error.to_string()))?
            .join(path)
    };
    let limit = if include_leaf {
        absolute.clone()
    } else {
        absolute.parent().unwrap_or(Path::new("/")).to_path_buf()
    };
    let mut current = PathBuf::new();
    for component in limit.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(ObservedError::new(
                    error_code,
                    format!("path metadata is unavailable: {error}"),
                ));
            }
        }
    }
    Ok(false)
}

fn resolve_missing_leaf(path: &Path) -> Result<PathBuf, ObservedError> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|error| {
        ObservedError::new(
            codes::OUTPUT_IO,
            format!("output parent cannot be resolved: {error}"),
        )
    })?;
    let name = path
        .file_name()
        .ok_or_else(|| output_unavailable("output has no file name"))?;
    Ok(parent.join(name))
}

fn publish_plan(output: &Path, plan: &ScenePlan) -> Result<(), ObservedError> {
    publish_plan_steps(output, plan, |_| Ok(()))
}

fn publish_plan_steps(
    output: &Path,
    plan: &ScenePlan,
    mut checkpoint: impl FnMut(&'static str) -> Result<(), ObservedError>,
) -> Result<(), ObservedError> {
    let bytes = plan.to_canonical_bytes();
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| output_unavailable("output file name must be UTF-8"))?;
    let stage = parent.join(format!(
        ".{name}.nomos-stage-{}-{}",
        std::process::id(),
        STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let mut published = false;
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)
            .map_err(output_io)?;
        checkpoint("stage-created")?;
        file.write_all(&bytes).map_err(output_io)?;
        checkpoint("stage-written")?;
        file.sync_all().map_err(output_io)?;
        checkpoint("stage-synced")?;
        drop(file);

        let metadata = fs::symlink_metadata(&stage).map_err(output_io)?;
        if !metadata.file_type().is_file() {
            return Err(ObservedError::new(
                codes::OUTPUT_IO,
                "staged output is not a regular file",
            ));
        }
        let staged = fs::read(&stage).map_err(output_io)?;
        // The typed plan was already validated before encoding. Reopening must
        // prove both exact persistence and canonical bytes; reconstructing the
        // same semantic plan here would duplicate validation without adding a
        // stronger publication fact.
        if staged != bytes || json::parse(&staged).is_err() {
            return Err(ObservedError::new(
                codes::OUTPUT_IO,
                "staged output failed exact re-verification",
            ));
        }
        checkpoint("stage-verified")?;

        fs::hard_link(&stage, output).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                output_unavailable("output appeared before publication")
            } else {
                output_io(error)
            }
        })?;
        published = true;
        checkpoint("output-linked")?;
        if let Err(error) = fs::remove_file(&stage) {
            return Err(output_io(error));
        }
        if let Err(error) = File::open(parent).and_then(|directory| directory.sync_all()) {
            return Err(output_io(error));
        }
        Ok(())
    })();

    if result.is_err() {
        if published {
            let _ = fs::remove_file(output);
        }
        let _ = fs::remove_file(&stage);
    }
    result
}

fn whole_input_span(path: &Path, byte_len: usize) -> Option<SourceSpan> {
    let display = safe_diagnostic_path(path);
    let source_path = SourcePath::new(&display).ok()?;
    let byte_end = u32::try_from(byte_len).ok()?;
    SourceSpan::new(source_path, 0, byte_end, 1, 1).ok()
}

fn safe_diagnostic_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let safe = !text.is_empty()
        && !path.is_absolute()
        && !has_parent_component(path)
        && text.as_bytes().get(1) != Some(&b':');
    if safe {
        text
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("scene.json")
            .to_owned()
    }
}

fn input_io(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::INPUT_UNREADABLE, message)
}

fn output_unavailable(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::OUTPUT_UNAVAILABLE, message)
        .with_repair(RepairClass::WriteToNewOutputPath)
}

fn output_io(error: impl std::fmt::Display) -> ObservedError {
    ObservedError::new(
        codes::OUTPUT_IO,
        format!("output publication failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(label: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .join("target/r2-publication-faults")
            .join(format!("{}-{label}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove prior fault directory");
        }
        fs::create_dir_all(&root).expect("create fault directory");
        root
    }

    #[test]
    fn every_injected_publication_failure_removes_staging_and_output() {
        let source = include_bytes!("../../../fixtures/r2/scenes/scene_one.json");
        let plan = compile(source).expect("compile fixture");
        for stage in [
            "stage-created",
            "stage-written",
            "stage-synced",
            "stage-verified",
            "output-linked",
        ] {
            let root = fresh(stage);
            let output = root.join("plan.json");
            let error = publish_plan_steps(&output, &plan, |current| {
                if current == stage {
                    Err(ObservedError::new(
                        codes::OUTPUT_IO,
                        format!("injected {stage} failure"),
                    ))
                } else {
                    Ok(())
                }
            })
            .expect_err("injection must fail");
            assert_eq!(error.code(), codes::OUTPUT_IO);
            assert!(!output.exists(), "{stage} published an output");
            assert_eq!(
                fs::read_dir(&root).expect("read fault directory").count(),
                0,
                "{stage} left staging evidence"
            );
        }
    }
}

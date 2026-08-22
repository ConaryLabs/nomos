//! Exact argument grammar and filesystem command orchestration.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use nomos_compiler::{compile_world_package, inspect_compiled_package};
use nomos_core::package::MANIFEST_FILE;
use nomos_core::{CanonicalValue, Diagnostic, RepairClass, SourcePath};

use crate::{ExitCode, compile_and_write_world, render_rejection};

const ROOT_HELP: &str = "Nomos Gate K filesystem authoring\n\nUsage:\n  nomos validate <source.nomos>\n  nomos compile <source.nomos> --out <new.world/>\n  nomos inspect <world/>\n  nomos --help\n";
const VALIDATE_HELP: &str = "Validate one Nomos source file without writing artifacts.\n\nUsage:\n  nomos validate <source.nomos>\n";
const COMPILE_HELP: &str = "Compile one Nomos source file into a new immutable world package.\n\nUsage:\n  nomos compile <source.nomos> --out <new.world/>\n";
const INSPECT_HELP: &str =
    "Inspect one verified immutable world package.\n\nUsage:\n  nomos inspect <world/>\n";

/// One completed command-line execution, including its exact stdout bytes.
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

    /// Exact bytes to write to stdout.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Command {
    Help(&'static str),
    Validate { source: String },
    Compile { source: String, output: String },
    Inspect { package: String },
}

/// Parses and executes exactly one supported command.
///
/// Every handled result is returned as stdout bytes; callers do not need to
/// interpret diagnostics or semantic report fields.
#[must_use]
pub fn execute(args: impl IntoIterator<Item = OsString>) -> Execution {
    match parse(args) {
        Ok(Command::Help(help)) => Execution {
            exit: ExitCode::Completed,
            stdout: help.as_bytes().to_vec(),
        },
        Ok(command) => execute_command(command),
        Err(diagnostic) => rejected(ExitCode::InvalidUsage, &diagnostic),
    }
}

fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, Diagnostic> {
    let args = args
        .into_iter()
        .enumerate()
        .map(|(index, argument)| {
            argument.into_string().map_err(|_| {
                usage(format!(
                    "argument {} is not UTF-8 and cannot name a Nomos operation",
                    index + 1
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let command = match args.as_slice() {
        [only] if only == "--help" => Command::Help(ROOT_HELP),
        [name, help] if name == "validate" && help == "--help" => Command::Help(VALIDATE_HELP),
        [name, source] if name == "validate" && !is_option(source) => Command::Validate {
            source: source.clone(),
        },
        [name, help] if name == "compile" && help == "--help" => Command::Help(COMPILE_HELP),
        [name, source, option, output]
            if name == "compile"
                && !is_option(source)
                && option == "--out"
                && !is_option(output) =>
        {
            Command::Compile {
                source: source.clone(),
                output: output.clone(),
            }
        }
        [name, help] if name == "inspect" && help == "--help" => Command::Help(INSPECT_HELP),
        [name, package] if name == "inspect" && !is_option(package) => Command::Inspect {
            package: package.clone(),
        },
        _ => {
            return Err(usage(
                "arguments do not match a supported Nomos command; use `nomos --help`",
            ));
        }
    };
    Ok(command)
}

fn is_option(argument: &str) -> bool {
    argument.starts_with('-')
}

fn execute_command(command: Command) -> Execution {
    let result = match command {
        Command::Help(_) => unreachable!("help is returned before command execution"),
        Command::Validate { source } => validate(&source),
        Command::Compile { source, output } => compile(&source, &output),
        Command::Inspect { package } => inspect(&package),
    };
    match result {
        Ok(value) => completed(value),
        Err(diagnostic) => rejected(ExitCode::for_diagnostic(&diagnostic), &diagnostic),
    }
}

fn validate(source: &str) -> Result<CanonicalValue, Diagnostic> {
    let source_path = SourcePath::new(source)?;
    let text = read_source(source)?;
    let compiled = compile_world_package(&text, source_path)?;
    compiled.validate_artifacts()?;
    let artifacts = compiled
        .members()?
        .into_iter()
        .map(|(name, _)| CanonicalValue::text(name.as_str()))
        .collect();
    Ok(CanonicalValue::object_declared([
        ("artifacts", CanonicalValue::Array(artifacts)),
        ("command", CanonicalValue::text("validate")),
        ("source", CanonicalValue::text(source)),
        ("status", CanonicalValue::text("completed")),
        (
            "world_ir_schema",
            compiled.stable_ir().schema().to_canonical(),
        ),
    ]))
}

fn compile(source: &str, output: &str) -> Result<CanonicalValue, Diagnostic> {
    let source_path = SourcePath::new(source)?;
    let output_path = relative_filesystem_path(output)?;
    let text = read_source(source)?;
    let package = compile_and_write_world(&text, source_path, &output_path)?;
    let artifacts = std::iter::once(MANIFEST_FILE)
        .chain(
            package
                .manifest()
                .members()
                .iter()
                .map(|record| record.name().as_str()),
        )
        .map(|name| CanonicalValue::text(artifact_path(output, name)))
        .collect();
    Ok(CanonicalValue::object_declared([
        ("artifacts", CanonicalValue::Array(artifacts)),
        ("command", CanonicalValue::text("compile")),
        (
            "manifest_digest",
            CanonicalValue::text(package.manifest().digest().to_hex()),
        ),
        ("output", CanonicalValue::text(output)),
        ("source", CanonicalValue::text(source)),
        ("status", CanonicalValue::text("completed")),
    ]))
}

fn inspect(package: &str) -> Result<CanonicalValue, Diagnostic> {
    let package = relative_filesystem_path(package)?;
    inspect_compiled_package(&package)
}

fn read_source(source: &str) -> Result<String, Diagnostic> {
    let path = Path::new(source);
    let metadata = fs::metadata(path).map_err(|error| io_failure(path, &error))?;
    if !metadata.is_file() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::CLI_IO,
            format!("`{source}` is not a regular source file"),
        ));
    }
    let bytes = fs::read(path).map_err(|error| io_failure(path, &error))?;
    String::from_utf8(bytes).map_err(|_| {
        Diagnostic::new(
            nomos_core::diagnostic::codes::CLI_SOURCE_ENCODING,
            format!("source `{source}` is not UTF-8 text"),
        )
        .with_repair(RepairClass::FixSourceSyntax)
    })
}

fn relative_filesystem_path(spelling: &str) -> Result<PathBuf, Diagnostic> {
    let rejected = spelling.is_empty()
        || spelling.starts_with('/')
        || spelling.starts_with('\\')
        || spelling.split(['/', '\\']).any(|segment| segment == "..")
        || spelling.as_bytes().get(1) == Some(&b':');
    if rejected {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::CLI_PATH_NOT_RELATIVE,
            format!("filesystem path `{spelling}` is not a safe relative spelling"),
        )
        .with_repair(RepairClass::UseSupportedIdentifierShape));
    }
    Ok(PathBuf::from(spelling))
}

fn artifact_path(root: &str, member: &str) -> String {
    format!("{}/{member}", root.trim_end_matches(['/', '\\']))
}

fn usage(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(nomos_core::diagnostic::codes::CLI_USAGE, message)
}

fn io_failure(path: &Path, error: &std::io::Error) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::CLI_IO,
        format!("`{}`: {error}", path.display()),
    )
}

fn completed(value: CanonicalValue) -> Execution {
    let mut stdout = value.to_canonical_bytes();
    stdout.push(b'\n');
    Execution {
        exit: ExitCode::Completed,
        stdout,
    }
}

fn rejected(exit: ExitCode, diagnostic: &Diagnostic) -> Execution {
    let mut stdout = render_rejection(diagnostic).into_bytes();
    stdout.push(b'\n');
    Execution { exit, stdout }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_is_exact_and_future_commands_are_unknown() {
        for args in [
            vec![],
            vec!["run"],
            vec!["--version"],
            vec!["validate", "--help", "extra"],
            vec!["compile", "source.nomos", "--out=build/world"],
            vec!["compile", "--out", "build/world", "source.nomos"],
            vec!["inspect", "--", "build/world"],
        ] {
            let execution = execute(args.into_iter().map(OsString::from));
            assert_eq!(execution.exit(), ExitCode::InvalidUsage);
            assert!(execution.stdout().starts_with(b"{\"diagnostics\":"));
        }
    }

    #[test]
    fn help_is_the_only_plain_text_output() {
        let help = execute([OsString::from("--help")]);
        assert_eq!(help.exit(), ExitCode::Completed);
        assert_eq!(help.stdout(), ROOT_HELP.as_bytes());
    }
}

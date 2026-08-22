//! Exact argument grammar and filesystem command orchestration.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use nomos_compiler::{compile_world_package, inspect_compiled_package};
use nomos_core::package::MANIFEST_FILE;
use nomos_core::{CanonicalValue, Diagnostic, RepairClass, SourcePath};
use nomos_sim::{
    CommandRequest, CommandScript, PersistedRuntimeState, ReplayLog, RunExecution, execute_requests,
};

use crate::{
    ExitCode, OpenedRunBundle, compile_and_write_world, initial_state_from_package,
    migrate_and_write_world, open_compiled_world, render_rejection, require_available_run_output,
    write_run_bundle,
};

const ROOT_HELP: &str = "Nomos Gate K semantic runtime\n\nUsage:\n  nomos validate <source.nomos>\n  nomos compile <source.nomos> --out <new.world/>\n  nomos inspect <world/>\n  nomos migrate <v1-world/> --to 2 --out <new-v2-world/>\n  nomos run <world/> --commands <commands> --out <new-run/>\n  nomos command <world/> --state <state.json> \"<command>\" --out <new-run/>\n  nomos replay <world/> --log <replay> --out <new-run/>\n  nomos --help\n";
const VALIDATE_HELP: &str = "Validate one Nomos source file without writing artifacts.\n\nUsage:\n  nomos validate <source.nomos>\n";
const COMPILE_HELP: &str = "Compile one Nomos source file into a new immutable world package.\n\nUsage:\n  nomos compile <source.nomos> --out <new.world/>\n";
const INSPECT_HELP: &str =
    "Inspect one verified immutable world package.\n\nUsage:\n  nomos inspect <world/>\n";
const MIGRATE_HELP: &str = "Migrate one immutable stable-v1 world into a new stable-v2 package.\n\nUsage:\n  nomos migrate <v1-world/> --to 2 --out <new-v2-world/>\n";
const RUN_HELP: &str = "Execute one strict command script from a compiled world's initial state.\n\nUsage:\n  nomos run <world/> --commands <commands> --out <new-run/>\n";
const COMMAND_HELP: &str = "Execute one command from a verified persisted runtime state.\n\nUsage:\n  nomos command <world/> --state <state.json> \"<command>\" --out <new-run/>\n";
const REPLAY_HELP: &str = "Reproduce one strict replay log against a compiled world's initial state.\n\nUsage:\n  nomos replay <world/> --log <replay> --out <new-run/>\n";

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
    Validate {
        source: String,
    },
    Compile {
        source: String,
        output: String,
    },
    Inspect {
        package: String,
    },
    Migrate {
        package: String,
        target: String,
        output: String,
    },
    Run {
        package: String,
        commands: String,
        output: String,
    },
    Single {
        package: String,
        state: String,
        request: String,
        output: String,
    },
    Replay {
        package: String,
        log: String,
        output: String,
    },
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
        [name, help] if name == "migrate" && help == "--help" => Command::Help(MIGRATE_HELP),
        [name, package, target_option, target, output_option, output]
            if name == "migrate"
                && !is_option(package)
                && target_option == "--to"
                && !is_option(target)
                && output_option == "--out"
                && !is_option(output) =>
        {
            Command::Migrate {
                package: package.clone(),
                target: target.clone(),
                output: output.clone(),
            }
        }
        [name, help] if name == "run" && help == "--help" => Command::Help(RUN_HELP),
        [
            name,
            package,
            commands_option,
            commands,
            output_option,
            output,
        ] if name == "run"
            && !is_option(package)
            && commands_option == "--commands"
            && !is_option(commands)
            && output_option == "--out"
            && !is_option(output) =>
        {
            Command::Run {
                package: package.clone(),
                commands: commands.clone(),
                output: output.clone(),
            }
        }
        [name, help] if name == "replay" && help == "--help" => Command::Help(REPLAY_HELP),
        [name, package, log_option, log, output_option, output]
            if name == "replay"
                && !is_option(package)
                && log_option == "--log"
                && !is_option(log)
                && output_option == "--out"
                && !is_option(output) =>
        {
            Command::Replay {
                package: package.clone(),
                log: log.clone(),
                output: output.clone(),
            }
        }
        [name, help] if name == "command" && help == "--help" => Command::Help(COMMAND_HELP),
        [
            name,
            package,
            state_option,
            state,
            request,
            output_option,
            output,
        ] if name == "command"
            && !is_option(package)
            && state_option == "--state"
            && !is_option(state)
            && !is_option(request)
            && output_option == "--out"
            && !is_option(output) =>
        {
            Command::Single {
                package: package.clone(),
                state: state.clone(),
                request: request.clone(),
                output: output.clone(),
            }
        }
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
        Command::Validate { source } => validate(&source).map(RuntimeOutcome::completed),
        Command::Compile { source, output } => {
            compile(&source, &output).map(RuntimeOutcome::completed)
        }
        Command::Inspect { package } => inspect(&package).map(RuntimeOutcome::completed),
        Command::Migrate {
            package,
            target,
            output,
        } => migrate(&package, &target, &output).map(RuntimeOutcome::completed),
        Command::Run {
            package,
            commands,
            output,
        } => run_script(&package, &commands, &output),
        Command::Single {
            package,
            state,
            request,
            output,
        } => command_once(&package, &state, &request, &output),
        Command::Replay {
            package,
            log,
            output,
        } => replay(&package, &log, &output),
    };
    match result {
        Ok(outcome) => outcome.into_execution(),
        Err(diagnostic) => rejected(ExitCode::for_diagnostic(&diagnostic), &diagnostic),
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RuntimeOutcome {
    value: CanonicalValue,
    rejection: Option<Diagnostic>,
}

impl RuntimeOutcome {
    fn completed(value: CanonicalValue) -> Self {
        Self {
            value,
            rejection: None,
        }
    }

    fn into_execution(self) -> Execution {
        let exit = if self.rejection.is_some() {
            ExitCode::Rejected
        } else {
            ExitCode::Completed
        };
        let mut stdout = self.value.to_canonical_bytes();
        stdout.push(b'\n');
        Execution { exit, stdout }
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

fn migrate(package: &str, target: &str, output: &str) -> Result<CanonicalValue, Diagnostic> {
    if target != "2" {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::MIGRATION_TARGET_UNSUPPORTED,
            format!("stable World IR migration target `{target}` is unsupported; expected `2`"),
        )
        .with_repair(RepairClass::RebuildFromSource));
    }
    let package_path = relative_filesystem_path(package)?;
    let output_path = relative_filesystem_path(output)?;
    require_migration_output_outside_package(&package_path, &output_path)?;
    let (migrated, written) = migrate_and_write_world(&package_path, &output_path)?;
    let artifacts = std::iter::once(MANIFEST_FILE)
        .chain(
            written
                .manifest()
                .members()
                .iter()
                .map(|record| record.name().as_str()),
        )
        .map(|name| CanonicalValue::text(artifact_path(output, name)))
        .collect();
    Ok(CanonicalValue::object_declared([
        ("artifacts", CanonicalValue::Array(artifacts)),
        ("command", CanonicalValue::text("migrate")),
        ("output", CanonicalValue::text(output)),
        (
            "source_package_digest",
            CanonicalValue::text(migrated.source_package_digest().to_hex()),
        ),
        (
            "source_world_ir_digest",
            CanonicalValue::text(migrated.source_world_ir_digest().to_hex()),
        ),
        (
            "source_world_ir_schema",
            CanonicalValue::object_declared([
                ("name", CanonicalValue::text("nomos.world_ir")),
                ("version", CanonicalValue::Uint(1)),
            ]),
        ),
        ("status", CanonicalValue::text("completed")),
        (
            "target_manifest_digest",
            CanonicalValue::text(written.manifest().digest().to_hex()),
        ),
        (
            "target_runtime_state_schema",
            nomos_sim::runtime_state_schema().to_canonical(),
        ),
        (
            "target_world_ir_schema",
            migrated
                .compiled_world()
                .stable_ir()
                .schema()
                .to_canonical(),
        ),
    ]))
}

fn run_script(package: &str, commands: &str, output: &str) -> Result<RuntimeOutcome, Diagnostic> {
    let package_path = relative_filesystem_path(package)?;
    let commands_path = relative_filesystem_path(commands)?;
    let output_path = relative_filesystem_path(output)?;
    require_available_run_output(&output_path)?;
    let world = open_compiled_world(&package_path)?;
    require_output_outside_package(&package_path, &output_path)?;
    let script = CommandScript::from_bytes(&read_regular_bytes(&commands_path, "command script")?)?;
    let initial =
        PersistedRuntimeState::new(world.simulation(), initial_state_from_package(&world)?)?;
    publish_execution(
        "run",
        output,
        &output_path,
        &world,
        execute_requests(
            world.simulation(),
            world.package_digest(),
            initial,
            script.requests(),
        )?,
    )
}

fn command_once(
    package: &str,
    state: &str,
    request: &str,
    output: &str,
) -> Result<RuntimeOutcome, Diagnostic> {
    let package_path = relative_filesystem_path(package)?;
    let state_path = relative_filesystem_path(state)?;
    let output_path = relative_filesystem_path(output)?;
    require_available_run_output(&output_path)?;
    let world = open_compiled_world(&package_path)?;
    require_output_outside_package(&package_path, &output_path)?;
    require_output_outside_state_bundle(&state_path, &output_path)?;
    let request = CommandRequest::from_line(request)?;
    let initial = PersistedRuntimeState::from_canonical_bytes(
        &read_regular_bytes(&state_path, "persisted runtime state")?,
        world.simulation(),
    )?;
    publish_execution(
        "command",
        output,
        &output_path,
        &world,
        execute_requests(
            world.simulation(),
            world.package_digest(),
            initial,
            &[request],
        )?,
    )
}

fn replay(package: &str, log: &str, output: &str) -> Result<RuntimeOutcome, Diagnostic> {
    let package_path = relative_filesystem_path(package)?;
    let log_path = relative_filesystem_path(log)?;
    let output_path = relative_filesystem_path(output)?;
    require_available_run_output(&output_path)?;
    let world = open_compiled_world(&package_path)?;
    require_output_outside_package(&package_path, &output_path)?;
    let replay = ReplayLog::from_canonical_bytes(&read_regular_bytes(&log_path, "replay log")?)?;
    let initial =
        PersistedRuntimeState::new(world.simulation(), initial_state_from_package(&world)?)?;
    replay.validate_input(world.package_digest(), &initial)?;
    let requests = replay
        .expected_command_log()
        .rows()
        .iter()
        .map(|row| row.request().clone())
        .collect::<Vec<_>>();
    let execution = execute_requests(
        world.simulation(),
        world.package_digest(),
        initial,
        &requests,
    )?;
    replay.validate_execution(&execution)?;
    publish_execution("replay", output, &output_path, &world, execution)
}

fn publish_execution(
    command: &'static str,
    output_spelling: &str,
    output_path: &Path,
    world: &nomos_compiler::OpenedCompiledWorld,
    execution: RunExecution,
) -> Result<RuntimeOutcome, Diagnostic> {
    let rejection = execution.rejection().cloned();
    let opened = write_run_bundle(&execution, world, output_path)?;
    Ok(RuntimeOutcome {
        value: runtime_report(command, output_spelling, &opened, rejection.as_ref()),
        rejection,
    })
}

fn runtime_report(
    command: &'static str,
    output: &str,
    bundle: &OpenedRunBundle,
    rejection: Option<&Diagnostic>,
) -> CanonicalValue {
    let artifacts = [
        "causal-receipts.json",
        "command-log.json",
        "final-state.json",
        "initial-state.json",
        "result.json",
        "state-hashes.json",
    ]
    .into_iter()
    .map(|name| CanonicalValue::text(artifact_path(output, name)))
    .collect();
    let diagnostics =
        rejection.map(|diagnostic| CanonicalValue::Array(vec![diagnostic.to_canonical()]));
    let mut fields = vec![
        ("artifacts", CanonicalValue::Array(artifacts)),
        (
            "committed_command_count",
            CanonicalValue::Uint(bundle.result().committed_command_count()),
        ),
        ("command", CanonicalValue::text(command)),
        (
            "final_state_hash",
            CanonicalValue::text(bundle.result().final_state_hash().to_hex()),
        ),
        (
            "first_state_hash",
            CanonicalValue::text(bundle.result().first_state_hash().to_hex()),
        ),
        ("output", CanonicalValue::text(output)),
        (
            "result_digest",
            CanonicalValue::text(bundle.result_digest().to_hex()),
        ),
        (
            "status",
            CanonicalValue::text(bundle.result().status().as_str()),
        ),
    ];
    if let Some(diagnostics) = diagnostics {
        fields.push(("diagnostics", diagnostics));
    }
    CanonicalValue::object_declared(fields)
}

fn read_regular_bytes(path: &Path, kind: &str) -> Result<Vec<u8>, Diagnostic> {
    let metadata = fs::metadata(path).map_err(|error| io_failure(path, &error))?;
    if !metadata.is_file() {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::CLI_IO,
            format!("`{}` is not a regular {kind} file", path.display()),
        ));
    }
    fs::read(path).map_err(|error| io_failure(path, &error))
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

fn require_output_outside_package(package: &Path, output: &Path) -> Result<(), Diagnostic> {
    let package = fs::canonicalize(package).map_err(|error| io_failure(package, &error))?;
    let output = resolve_with_missing_tail(output)?;
    if output.starts_with(&package) {
        return Err(output_overlap("package"));
    }
    Ok(())
}

fn require_migration_output_outside_package(
    package: &Path,
    output: &Path,
) -> Result<(), Diagnostic> {
    let package = fs::canonicalize(package).map_err(|error| io_failure(package, &error))?;
    let output = resolve_with_missing_tail(output)?;
    if output.starts_with(&package) {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::MIGRATION_OUTPUT_OVERLAPS_INPUT,
            "migration output overlaps the immutable stable-v1 input package",
        )
        .with_repair(RepairClass::WriteToNewOutputPath));
    }
    Ok(())
}

fn require_output_outside_state_bundle(state: &Path, output: &Path) -> Result<(), Diagnostic> {
    let state = fs::canonicalize(state).map_err(|error| io_failure(state, &error))?;
    let parent = state
        .parent()
        .expect("a canonical absolute state path has a parent")
        .to_path_buf();
    let output = resolve_with_missing_tail(output)?;
    if output.starts_with(&parent) && crate::run_bundle::has_run_bundle_shape(&parent)? {
        return Err(output_overlap("run bundle"));
    }
    Ok(())
}

fn output_overlap(kind: &str) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUN_BUNDLE_OUTPUT_OVERLAPS_INPUT,
        format!("run-bundle output overlaps an immutable input {kind}"),
    )
    .with_repair(RepairClass::WriteToNewOutputPath)
}

fn resolve_with_missing_tail(path: &Path) -> Result<PathBuf, Diagnostic> {
    let mut existing = path.to_path_buf();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(&existing) {
            Ok(_) => {
                let mut resolved =
                    fs::canonicalize(&existing).map_err(|error| io_failure(&existing, &error))?;
                for component in missing.iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = existing.file_name() else {
                    return Err(io_failure(&existing, &error));
                };
                missing.push(name.to_os_string());
                if !existing.pop() || existing.as_os_str().is_empty() {
                    existing = PathBuf::from(".");
                }
            }
            Err(error) => return Err(io_failure(&existing, &error)),
        }
    }
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

//! `nomos-play replay` — re-execute a recorded session and compare.
//!
//! ```text
//! nomos-play replay <areas-dir> --session <session.json> [--emit <path>]
//! ```
//!
//! `<areas-dir>` is the directory `gaol capture` writes:
//! `<areas-dir>/<area-id>/rendering-plan.json` and
//! `<areas-dir>/<area-id>/world/simulation.json`. The command is read-only and
//! writes nothing unless `--emit` is given.
//!
//! The stdout line is harness output, not a canonical document, and is
//! deliberately not spelled `name@version`: giving it a canonical-looking
//! identity would invite it into a register it does not belong in. That is the
//! same position `docs/review/nomos-viewer.md` section 5.5 takes about the
//! smoke receipt.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nomos_play::{PlayError, PlayResult, RecordedSession, codes, replay};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> PlayResult<ExitCode> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let parsed = Arguments::parse(&arguments)?;

    let bytes = std::fs::read(&parsed.session).map_err(|error| {
        PlayError::new(
            codes::CONTENT_MISMATCH,
            format!("cannot read {}: {error}", parsed.session.display()),
        )
    })?;
    let recorded = RecordedSession::decode(&bytes)?;

    let areas = parsed.areas.clone();
    let report = replay(&recorded, |area| read_area(&areas, area))?;

    if let Some(path) = &parsed.emit {
        std::fs::write(path, report.session.to_canonical_bytes()).map_err(|error| {
            PlayError::new(
                codes::CONTENT_MISMATCH,
                format!("cannot write {}: {error}", path.display()),
            )
        })?;
    }

    match &report.divergence {
        None => {
            println!(
                "NOMOS_PLAY_REPLAY PASS areas={} commands={} receipts={} chain={} final_kernel={}",
                report.areas,
                report.commands,
                report.receipts,
                report.chain_head.to_hex(),
                report.final_kernel_state_hash.to_hex()
            );
            Ok(ExitCode::SUCCESS)
        }
        Some(divergence) => {
            println!(
                "NOMOS_PLAY_REPLAY FAIL ordinal={} field={} area={}",
                divergence
                    .ordinal
                    .map_or_else(|| "-".to_owned(), |ordinal| ordinal.to_string()),
                divergence.field,
                divergence.area.as_deref().unwrap_or("-")
            );
            eprintln!("{}", divergence.detail);
            Ok(ExitCode::FAILURE)
        }
    }
}

fn read_area(areas: &Path, area: &str) -> PlayResult<(Vec<u8>, Vec<u8>)> {
    let plan = areas.join(area).join("rendering-plan.json");
    let semantics = areas.join(area).join("world").join("simulation.json");
    Ok((read_file(&plan)?, read_file(&semantics)?))
}

fn read_file(path: &Path) -> PlayResult<Vec<u8>> {
    std::fs::read(path).map_err(|error| {
        PlayError::new(
            codes::CONTENT_MISMATCH,
            format!("cannot read {}: {error}", path.display()),
        )
    })
}

struct Arguments {
    areas: PathBuf,
    session: PathBuf,
    emit: Option<PathBuf>,
}

impl Arguments {
    fn parse(arguments: &[String]) -> PlayResult<Self> {
        let usage = "usage: nomos-play replay <areas-dir> --session <session.json> [--emit <path>]";
        let mut positional = Vec::new();
        let mut session = None;
        let mut emit = None;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--session" => {
                    index += 1;
                    session = Some(PathBuf::from(arguments.get(index).ok_or_else(|| {
                        PlayError::new(
                            codes::CONTENT_MISMATCH,
                            format!("--session needs a path\n{usage}"),
                        )
                    })?));
                }
                "--emit" => {
                    index += 1;
                    emit = Some(PathBuf::from(arguments.get(index).ok_or_else(|| {
                        PlayError::new(
                            codes::CONTENT_MISMATCH,
                            format!("--emit needs a path\n{usage}"),
                        )
                    })?));
                }
                other if other.starts_with("--") => {
                    return Err(PlayError::new(
                        codes::CONTENT_MISMATCH,
                        format!("unknown option `{other}`\n{usage}"),
                    ));
                }
                other => positional.push(other.to_owned()),
            }
            index += 1;
        }

        if positional.first().map(String::as_str) != Some("replay") || positional.len() != 2 {
            return Err(PlayError::new(codes::CONTENT_MISMATCH, usage));
        }
        let session = session.ok_or_else(|| {
            PlayError::new(
                codes::CONTENT_MISMATCH,
                format!("--session is required\n{usage}"),
            )
        })?;
        Ok(Self {
            areas: PathBuf::from(&positional[1]),
            session,
            emit,
        })
    }
}

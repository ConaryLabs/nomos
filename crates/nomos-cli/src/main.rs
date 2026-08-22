//! The `nomos` binary.
//!
//! SW-H exposes the Gate K filesystem-authoring commands, SW-J adds immutable
//! runtime run bundles, SW-K adds deterministic replay, and SW-M adds strict
//! stable-v1 migration while leaving explanations for the accepted explanation
//! slice.

use std::io::{self, Write};
use std::process::ExitCode as ProcessExitCode;

fn main() -> ProcessExitCode {
    let execution = nomos_cli::execute(std::env::args_os().skip(1));
    if io::stdout().write_all(execution.stdout()).is_err() {
        return ProcessExitCode::from(3);
    }
    ProcessExitCode::from(u8::try_from(execution.exit().code()).unwrap_or(3))
}

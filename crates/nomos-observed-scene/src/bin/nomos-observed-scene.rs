//! The `nomos-observed-scene` binary.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let execution = nomos_observed_scene::execute(std::env::args_os().skip(1));
    if io::stdout().write_all(execution.stdout()).is_err() {
        return ExitCode::from(3);
    }
    ExitCode::from(execution.exit().code())
}

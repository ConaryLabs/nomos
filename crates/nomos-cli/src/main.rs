//! The `nomos` binary.
//!
//! `KERNEL.md` section 8 defines nine commands. None of them exist yet: SW-B
//! builds the workspace, identity, encoding, and package mechanics they will
//! stand on. Rather than pretend, the binary reports its own state and exits
//! `2` (invalid usage) for anything asked of it.

use std::process::ExitCode as ProcessExitCode;

use nomos_cli::ExitCode;
use nomos_core::CanonicalValue;

fn main() -> ProcessExitCode {
    let requested: Vec<String> = std::env::args().skip(1).collect();
    let report = CanonicalValue::object_declared([
        ("implemented_commands", CanonicalValue::Array(Vec::new())),
        (
            "message",
            CanonicalValue::text(
                "the nomos command surface is defined in KERNEL.md section 8 and \
                 implemented by a later slice; SW-B provides the workspace, stable \
                 IDs, canonical encoding, hashing, and package mechanics",
            ),
        ),
        (
            "requested",
            CanonicalValue::Array(requested.iter().map(CanonicalValue::text).collect()),
        ),
        ("status", CanonicalValue::text("not_implemented")),
    ]);
    println!("{}", String::from_utf8_lossy(&report.to_canonical_bytes()));
    ProcessExitCode::from(u8::try_from(ExitCode::InvalidUsage.code()).unwrap_or(2))
}

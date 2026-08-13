mod audit;
mod cli;
mod color;
mod commands;
mod config;
mod core;
mod findings;
mod fix;
mod git;
mod mutation;
mod renumber;
mod telemetry;
mod transaction;
mod update_check;

use std::process::ExitCode;

/// Domain-level failure: expected conditions like "ticket not found", "status conflict",
/// "validation drift". These exit with code 1.
#[derive(Debug)]
pub(crate) struct DomainError(pub String);

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DomainError {}

/// Global quiet flag — set once at startup, read by command functions.
pub(crate) static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Global dry-run flag — set once at startup, read by mutation commands.
pub(crate) static DRY_RUN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn main() -> ExitCode {
    let code = cli::run();
    update_check::check_for_update();
    ExitCode::from(code as u8)
}

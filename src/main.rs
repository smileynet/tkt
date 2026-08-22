mod audit;
mod cli;
mod color;
mod commands;
mod config;
mod context;
mod core;
mod findings;
mod fix;
mod git;
mod migrate;
mod mutation;
mod renumber;
mod telemetry;
mod transaction;
mod update_check;

use std::process::ExitCode;

/// Error kind taxonomy — fixed vocabulary for machine consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ErrorKind {
    NotFound,
    AlreadyDone,
    Conflict,
    GateFailed,
    Validation,
    Cycle,
    Io,
    Parse,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::NotFound => "not_found",
            ErrorKind::AlreadyDone => "already_done",
            ErrorKind::Conflict => "conflict",
            ErrorKind::GateFailed => "gate_failed",
            ErrorKind::Validation => "validation",
            ErrorKind::Cycle => "cycle",
            ErrorKind::Io => "io",
            ErrorKind::Parse => "parse",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            ErrorKind::Io | ErrorKind::Parse => 2,
            _ => 1,
        }
    }

    #[allow(dead_code)]
    pub fn retryable(&self) -> bool {
        matches!(self, ErrorKind::Conflict)
    }
}

/// Domain-level failure: expected conditions like "ticket not found", "status conflict",
/// "validation drift". These exit with code 1 (domain) or 2 (operational).
#[derive(Debug)]
pub(crate) struct DomainError {
    pub kind: ErrorKind,
    pub message: String,
    pub hint: Option<String>,
}

impl DomainError {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            hint: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_hint(kind: ErrorKind, message: String, hint: String) -> Self {
        Self {
            kind,
            message,
            hint: Some(hint),
        }
    }
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DomainError {}

/// Global quiet flag — set once at startup, read by command functions.
pub(crate) static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Global dry-run flag — set once at startup, read by mutation commands.
pub(crate) static DRY_RUN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Global JSON output flag — set once at startup, changes error/success output format.
pub(crate) static JSON_OUTPUT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Result count for telemetry — set by read commands (ready, query, blocked, validate).
/// -1 = not applicable (default). Commands that produce countable results store the count here.
pub(crate) static RESULT_COUNT: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(-1);

fn main() -> ExitCode {
    let code = cli::run();
    update_check::check_for_update();
    ExitCode::from(code as u8)
}

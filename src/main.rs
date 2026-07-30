mod cli;
mod core;
mod findings;
mod git;
mod transaction;

use std::process::ExitCode;

fn main() -> ExitCode {
    let code = cli::run();
    ExitCode::from(code as u8)
}

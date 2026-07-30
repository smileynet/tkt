mod cli;
mod core;
mod findings;
mod git;

use std::process::ExitCode;

fn main() -> ExitCode {
    let code = cli::run();
    ExitCode::from(code as u8)
}

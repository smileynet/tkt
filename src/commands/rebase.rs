//! `tkt rebase` — resolve ID collisions with upstream.
//! TODO: Full extraction pending — currently delegates to cli.rs implementation.

use anyhow::Result;

pub fn run(dry_run: bool) -> Result<i32> {
    crate::cli::cmd_rebase_impl(dry_run)
}

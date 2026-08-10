//! `tkt renumber` — move a ticket to a new ID atomically.
//! TODO: Full extraction pending — currently delegates to cli.rs implementation.

use anyhow::Result;

pub fn run(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    crate::cli::cmd_renumber_impl(old_id, new_id, file_hint)
}

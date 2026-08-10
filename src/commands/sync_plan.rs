//! `tkt sync-plan` — drift-check ticket status vs a plan table.
//! TODO: Full extraction pending — currently delegates to cli.rs implementation.

use anyhow::Result;

pub fn run(check: bool, fix: bool, strict: bool, brief: bool, plan: Option<&str>) -> Result<i32> {
    crate::cli::cmd_sync_plan_impl(check, fix, strict, brief, plan)
}

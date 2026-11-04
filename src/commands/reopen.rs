//! `tkt reopen` — return a cancelled ticket to the frontier.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, is_quiet_json, print_quiet_json_success, slug_from_filename,
};
use crate::core::Status;
use crate::mutation::MutationContext;

pub fn run(id: &str) -> Result<i32> {
    let ctx = MutationContext::open()?;
    let ticket = ctx.find_ticket(id)?;
    if let Some(remote_status) = ctx.remote_status(ticket) {
        if remote_status != ticket.status.as_str() {
            domain_bail!(
                Conflict,
                "{} is {} locally but {} on remote; refresh and retry",
                id,
                ticket.status.as_str(),
                remote_status
            );
        }
    }
    if ticket.status != Status::Cancelled {
        domain_bail!(Conflict, "cannot reopen {}: it is not cancelled", ticket.id);
    }

    let mut file = ticket.file.clone();
    file.set_status(Status::Open);
    let plan_changed = super::sync_plan::update_status(&ctx.repo, &ticket.id, Status::Open)?;
    file.write()?;
    let rel_path = ctx.rel_path(&file.path);
    let mut paths = vec![rel_path.as_str()];
    if plan_changed {
        paths.push("docs/plan.md");
    }
    ctx.publish(
        &paths,
        &format!("chore(tickets): reopen {}", id),
        None,
        None,
    )?;

    if is_dry_run() {
        if is_quiet_json() {
            print_quiet_json_success(&format!("would reopen {}", ticket.id));
            return Ok(0);
        }
        println!(
            "Would reopen {} {} (cancelled → open)",
            ticket.id,
            slug_from_filename(&ticket.path)
        );
        println!("  Would keep Cancellation history");
        println!("  Would commit and push");
        return Ok(0);
    }
    if is_quiet() {
        print_quiet_json_success(&format!(
            "reopened {} ({})",
            ticket.id,
            slug_from_filename(&file.path)
        ));
    } else {
        println!(
            "Reopened {} ({})",
            ticket.id,
            slug_from_filename(&file.path)
        );
    }
    Ok(0)
}

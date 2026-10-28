//! `tkt claim` — mark an open ticket as in_progress.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, print_success, slug_from_filename,
};
use crate::core::{AgentEligibilityMismatch, AgentId, Status};
use crate::mutation::MutationContext;

pub fn run(id: &str, agent: Option<&AgentId>, force: bool) -> Result<i32> {
    let ctx = MutationContext::open()?;
    let t = ctx.find_ticket(id)?;

    if let Some(remote_status) = ctx.remote_status(t) {
        if remote_status != "open" {
            domain_bail!(
                Conflict,
                "{} is {}, not open (updated on remote)",
                id,
                remote_status
            );
        }
    }
    if t.status != Status::Open {
        domain_bail!(AlreadyDone, "{} is {}, not open", t.id, t.status.as_str());
    }
    if !force {
        if let Err(reason) = t.agent_eligibility.check(agent) {
            let detail = match reason {
                AgentEligibilityMismatch::IdentityRequired => "this ticket is agent-restricted; identify this invocation with --agent <id> or TKT_AGENT",
                AgentEligibilityMismatch::NotInAllowList => "the current agent is not in this ticket's allow list",
                AgentEligibilityMismatch::ExplicitlyExcluded => "the current agent is explicitly excluded from this ticket",
            };
            domain_bail!(
                GateFailed,
                "cannot claim {}: {} (use --force for an explicit scheduling override)",
                id,
                detail
            );
        }
    }

    if is_dry_run() {
        println!(
            "Would claim {} {} (open → in_progress)",
            t.id,
            slug_from_filename(&t.path)
        );
        println!("  Would commit and push");
        return Ok(0);
    }

    let mut file = t.file.clone();
    file.set_status(Status::InProgress);
    file.write()?;

    let rel_path = ctx.rel_path(&file.path);
    ctx.publish(&[&rel_path], &format!("chore(tickets): claim {}", id))?;

    if !is_quiet() {
        print_success(
            "claimed",
            &t.id,
            &slug_from_filename(&file.path),
            &format!("{} in_progress", crate::color::sym_arrow()),
        );
    }
    Ok(0)
}

//! `tkt cancel` — retire a ticket without recording completion.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, is_quiet_json, print_quiet_json_success, slug_from_filename,
};
use crate::core::{self, Status};
use crate::mutation::MutationContext;

pub fn run(
    id: &str,
    reason: &str,
    note: &str,
    by: Option<&str>,
    repoint: bool,
    drop_edges: bool,
) -> Result<i32> {
    if !core::CANCELLATION_REASONS.contains(&reason) {
        domain_bail!(
            "--reason must be one of {}",
            core::CANCELLATION_REASONS.join("/")
        );
    }
    if note.trim().is_empty() {
        domain_bail!("--note must not be empty");
    }
    if let Err(error) = core::validate::validate_free_text(note, "note", 2000) {
        domain_bail!("--note {}", error);
    }
    if repoint && drop_edges {
        domain_bail!("--repoint and --drop-edges are mutually exclusive");
    }
    if (reason == "duplicate" || reason == "superseded") && by.is_none() {
        domain_bail!("--by is required when --reason is {}", reason);
    }
    if repoint && by.is_none() {
        domain_bail!("--repoint requires --by <id>");
    }
    if let Some(by) = by {
        if let Err(error) = core::validate::validate_id(by) {
            domain_bail!("--by: {}", error);
        }
    }

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
    if ticket.status == Status::Done {
        domain_bail!(Conflict, "cannot cancel done ticket {}", ticket.id);
    }
    if ticket.status == Status::Cancelled {
        domain_bail!(Conflict, "ticket {} is already cancelled", ticket.id);
    }

    let replacement = if let Some(by) = by {
        let ticket = ctx.find_ticket(by)?;
        if ticket.status == Status::Cancelled {
            domain_bail!(Conflict, "--by ticket {} is cancelled", by);
        }
        if ticket.id == id {
            domain_bail!(Conflict, "--by cannot name the ticket being cancelled");
        }
        Some(ticket)
    } else {
        None
    };

    let dependents: Vec<_> = ctx
        .corpus
        .iter()
        .filter(|candidate| candidate.id != ticket.id)
        .filter(|candidate| candidate.blocked_by.iter().any(|dep| dep == &ticket.id))
        .collect();
    let active_dependents: Vec<_> = dependents
        .iter()
        .filter(|dependent| !dependent.status.is_terminal())
        .collect();
    if !active_dependents.is_empty() && !repoint && !drop_edges {
        let ids = active_dependents
            .iter()
            .map(|dependent| dependent.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        domain_bail!(
            Conflict,
            "cannot cancel {}: non-terminal dependents {} list it in blocked_by; use --repoint --by <id> or --drop-edges",
            ticket.id,
            ids
        );
    }

    let edge_action = repoint || drop_edges;
    let rewrite: Vec<_> = dependents
        .into_iter()
        .filter(|dependent| edge_action || dependent.status.is_terminal())
        .collect();
    if repoint {
        let by_id = replacement
            .expect("validated --repoint requires --by")
            .id
            .as_str();
        if rewrite.iter().any(|dependent| dependent.id == by_id) {
            domain_bail!(Conflict, "--repoint would make a ticket depend on itself");
        }
        if rewrite
            .iter()
            .any(|dependent| depends_on(&ctx.corpus, by_id, &dependent.id))
        {
            domain_bail!(Conflict, "--repoint would create a dependency cycle");
        }
    }

    if is_dry_run() {
        if is_quiet_json() {
            print_quiet_json_success(&format!("would cancel {} ({})", ticket.id, reason));
            return Ok(0);
        }
        println!(
            "Would cancel {} {} (→ cancelled: {})",
            ticket.id,
            slug_from_filename(&ticket.path),
            reason
        );
        for dependent in &rewrite {
            println!(
                "  Would {} blocked_by for {}",
                if repoint { "repoint" } else { "drop" },
                dependent.id
            );
        }
        println!("  Would commit and push");
        return Ok(0);
    }

    let mut file = ticket.file.clone();
    file.set_status(Status::Cancelled);
    file.append_cancellation(&super::close::chrono_date(), reason, by, note);

    let mut rewritten = Vec::with_capacity(rewrite.len());
    for dependent in &rewrite {
        let mut blocked_by: Vec<String> = dependent
            .blocked_by
            .iter()
            .filter(|dep| dep != &&ticket.id)
            .cloned()
            .collect();
        if repoint {
            let by_id = replacement
                .expect("validated --repoint requires --by")
                .id
                .clone();
            if !blocked_by.contains(&by_id) {
                blocked_by.push(by_id);
            }
        }
        let mut dependent_file = dependent.file.clone();
        dependent_file.set_blocked_by(&blocked_by);
        rewritten.push((dependent.id.as_str(), dependent_file));
    }

    let plan_changed = super::sync_plan::update_cancellation(&ctx.repo, &ticket.id, reason)?;
    let mut paths = Vec::with_capacity(rewrite.len() + 2);
    for (dependent_id, dependent_file) in rewritten {
        dependent_file.write()?;
        paths.push(ctx.rel_path(&dependent_file.path));
        if !is_quiet() {
            println!(
                "{} blocked_by for {}",
                if repoint { "Repointed" } else { "Dropped" },
                dependent_id
            );
        }
    }
    file.write()?;
    paths.push(ctx.rel_path(&file.path));
    if plan_changed {
        paths.push("docs/plan.md".to_string());
    }
    let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
    ctx.publish(&paths, &format!("chore(tickets): cancel {}", id), None)?;

    if is_quiet() {
        print_quiet_json_success(&format!("cancelled {} ({})", ticket.id, reason));
    } else {
        println!("Cancelled {} ({})", ticket.id, reason);
    }
    Ok(0)
}

fn depends_on<'a>(corpus: &'a [core::Ticket], start: &'a str, target: &str) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut pending = vec![start];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(ticket) = corpus.iter().find(|ticket| ticket.id == id) else {
            continue;
        };
        for dependency in &ticket.blocked_by {
            if dependency == target {
                return true;
            }
            pending.push(dependency);
        }
    }
    false
}

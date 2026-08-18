//! `tkt new` — allocate a new ticket id (fetch, scan, create, commit, push).

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, project_config, success_msg, tickets_dir,
};
use crate::core::{self, validate};
use crate::git;
use crate::transaction::{GitTransaction, PublishResult};

#[allow(clippy::too_many_arguments)]
pub fn run(
    slug: &str,
    title: Option<&str>,
    spec: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    blocked_by: &[String],
    validation_criteria: &[String],
) -> Result<i32> {
    if let Err(e) = validate::validate_slug(slug) {
        domain_bail!("{}", e);
    }

    let title_owned = slug.replace('-', " ");
    let title = title.unwrap_or(&title_owned);
    if let Err(e) = validate::validate_free_text(title, "title", 200) {
        domain_bail!("{}", e);
    }
    if let Some(s) = spec {
        if let Err(e) = validate::validate_free_text(s, "spec", 100) {
            domain_bail!("{}", e);
        }
    }
    if let Some(e) = env {
        if let Err(err) = validate::validate_env(e) {
            domain_bail!("{}", err);
        }
    }
    if let Some(p) = priority {
        if let Err(err) = validate::validate_priority(p) {
            domain_bail!("{}", err);
        }
    }
    if let Some(s) = status {
        if let Err(err) = validate::validate_status(s) {
            domain_bail!("{}", err);
        }
    }
    for dep in blocked_by {
        if let Err(e) = validate::validate_id(dep) {
            domain_bail!("--blocked-by: {}", e);
        }
    }
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);

    let priority = if priority.is_none() && !pcfg.new_default_priority.is_empty() {
        Some(pcfg.new_default_priority.as_str())
    } else {
        priority
    };

    let txn = GitTransaction::new(&dir)?;

    let names = txn.scan_names();
    let (tid, _width) = GitTransaction::next_id(&names);

    let dep_strs: Vec<&str> = blocked_by.iter().map(|s| s.as_str()).collect();
    if let Err(e) = validate::validate_no_self_dep(&tid, &dep_strs) {
        domain_bail!("{}", e);
    }

    let filename = format!("{}-{}.md", tid, slug);
    let path = dir.join(&filename);

    // Dry-run: show what would happen without writing
    if is_dry_run() {
        println!("Would create .tickets/{}", filename);
        println!(
            "  id: {}, status: {}, priority: {}",
            tid,
            status.unwrap_or("open"),
            priority.unwrap_or("medium")
        );
        if !blocked_by.is_empty() {
            println!("  blocked_by: {:?}", blocked_by);
        }
        if !validation_criteria.is_empty() {
            println!("  validation_criteria: {} items", validation_criteria.len());
        }
        println!("  Would commit and push");
        return Ok(0);
    }

    let content = core::new_ticket_text(&core::NewTicketParams {
        id: &tid,
        title,
        blocked_by,
        env,
        spec,
        priority,
        status,
        validation_criteria,
    });
    std::fs::write(&path, &content)?;

    let rel_path = format!(".tickets/{}", filename);
    git::add(&txn.repo, &[&rel_path])?;
    git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid, slug))?;

    match txn.try_push()? {
        PublishResult::Done(outcome) => {
            if is_quiet() {
                println!("{}", tid);
            } else {
                let detail = match outcome {
                    crate::transaction::PublishOutcome::LocalOnly => "local only",
                    _ => "pushed",
                };
                println!("{}", success_msg("created", &tid, slug, detail));
            }
            Ok(0)
        }
        PublishResult::NeedsRetry => {
            let names = txn.scan_names();
            let (tid2, _width) = GitTransaction::next_id(&names);
            let filename2 = format!("{}-{}.md", tid2, slug);
            let path2 = dir.join(&filename2);
            let content2 = core::new_ticket_text(&core::NewTicketParams {
                id: &tid2,
                title,
                blocked_by,
                env,
                spec,
                priority,
                status,
                validation_criteria,
            });
            std::fs::write(&path2, &content2)?;
            let rel_path2 = format!(".tickets/{}", filename2);
            git::add(&txn.repo, &[&rel_path2])?;
            git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid2, slug))?;

            txn.push_retry()?;
            if is_quiet() {
                println!("{}", tid2);
            } else {
                println!(
                    "{}",
                    success_msg(
                        "created",
                        &tid2,
                        slug,
                        &format!("pushed, renumbered {}→{}", tid, tid2)
                    )
                );
            }
            Ok(0)
        }
    }
}

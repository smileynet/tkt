//! `tkt batch` — allocate N sequential IDs in one commit/push.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_json_output, is_quiet, is_quiet_json, print_quiet_json_success,
    print_success, project_config, tickets_dir,
};
use crate::core::{self, validate};
use crate::transaction::{GitTransaction, PublishResult};

#[allow(clippy::too_many_arguments)]
pub fn run(
    items: &[String],
    template: Option<&str>,
    spec: Option<&str>,
    parent: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    blocked_by: &[String],
    validation_criteria: &[String],
    explicit_tags: &[String],
    requires: &[String],
    agents: &[String],
    exclude_agents: &[String],
) -> Result<i32> {
    if let Some(s) = spec {
        if let Err(e) = validate::validate_free_text(s, "spec", 100) {
            domain_bail!("{}", e);
        }
    }
    if let Some(parent) = parent {
        if let Err(e) = validate::validate_id(parent) {
            domain_bail!("--parent: {}", e);
        }
    }
    if let Some(e) = env {
        if let Err(err) = validate::validate_env(e) {
            domain_bail!("{}", err);
        }
    }
    if env.is_some_and(|e| e != "either") && !requires.is_empty() {
        domain_bail!("--env and --requires are mutually exclusive; omit --env or use --env either");
    }
    if let Some(p) = priority {
        if let Err(err) = validate::validate_priority(p) {
            domain_bail!("{}", err);
        }
    }
    if let Some(s) = status {
        if s == "done" || s == "cancelled" {
            let (message, hint) = if s == "done" {
                (
                    "status done can only be set by close",
                    "create tickets as open or backlog, then use tkt close <id> so closure gates run",
                )
            } else {
                (
                    "status cancelled can only be set by cancel",
                    "create tickets as open or backlog, then use tkt cancel <id>",
                )
            };
            return Err(crate::DomainError::with_hint(
                crate::ErrorKind::Validation,
                message.to_string(),
                hint.to_string(),
            )
            .into());
        }
        if let Err(err) = validate::validate_status(s) {
            domain_bail!("{}", err);
        }
    }
    for dep in blocked_by {
        if let Err(e) = validate::validate_id(dep) {
            domain_bail!("--blocked-by: {}", e);
        }
    }

    let mut parsed: Vec<(&str, String)> = Vec::new();
    for raw in items {
        let (slug, title) = match raw.split_once(':') {
            Some((s, t)) => (s, t.trim().to_string()),
            None => (raw.as_str(), raw.replace('-', " ")),
        };
        if let Err(e) = validate::validate_slug(slug) {
            domain_bail!("{}", e);
        }
        if let Err(e) = validate::validate_free_text(&title, "title", 200) {
            domain_bail!("{}", e);
        }
        parsed.push((slug, title));
    }

    let slugs: Vec<&str> = parsed.iter().map(|(s, _)| *s).collect();
    if let Err(e) = validate::validate_no_duplicate_slugs(&slugs) {
        domain_bail!("{}", e);
    }

    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    if let Some(parent) = parent {
        if !corpus.iter().any(|ticket| ticket.id == parent) {
            domain_bail!(NotFound, "parent ticket {} not found", parent);
        }
    }
    let agent_eligibility = if !agents.is_empty() {
        core::AgentEligibility::allow(agents).map_err(|error| {
            crate::DomainError::new(crate::ErrorKind::Validation, error.to_string())
        })?
    } else if !exclude_agents.is_empty() {
        core::AgentEligibility::exclude(exclude_agents).map_err(|error| {
            crate::DomainError::new(crate::ErrorKind::Validation, error.to_string())
        })?
    } else {
        core::AgentEligibility::Any
    };

    // Enforce validation_criteria at creation if configured
    let pcfg = project_config(&dir)?;
    let body = super::creation_body::load(&dir, None, template)?;
    if pcfg.close_require_validation_criteria && validation_criteria.is_empty() {
        return Err(crate::DomainError::with_hint(
            crate::ErrorKind::GateFailed,
            "project requires validation_criteria (define what \"done\" means)".to_string(),
            "add --validation \"criteria\" for each success condition".to_string(),
        )
        .into());
    }

    // Merge explicit tags with context auto-tags
    let effective_tags: Vec<String> = {
        let ctx = crate::context::load(&dir);
        let mut tags: Vec<String> = explicit_tags.to_vec();
        for t in &ctx.include {
            if !tags.contains(t) {
                tags.push(t.clone());
            }
        }
        tags
    };

    let txn = GitTransaction::new(&dir)?;
    let names = txn.scan_names()?;
    if let Some(parent) = parent {
        let first_id = core::max_id(&names) + 1;
        let width = core::id_width(&names);
        for offset in 0..parsed.len() {
            let own_id = format!("{:0>width$}", first_id + offset as u64, width = width);
            if let Err(error) = validate::validate_parent_ref(&own_id, parent) {
                domain_bail!("{}", error);
            }
        }
    }

    let allocate_and_commit =
        |names: &[String], parsed: &[(&str, String)]| -> Result<(u64, usize)> {
            let base = core::max_id(names) + 1;
            let width = core::id_width(names);
            let mut files: Vec<String> = Vec::new();
            for (i, (slug, title)) in parsed.iter().enumerate() {
                let tid = format!("{:0>width$}", base + i as u64, width = width);
                let filename = format!("{}-{}.md", tid, slug);
                let path = txn.dir.join(&filename);
                let content = core::new_ticket_text(&core::NewTicketParams {
                    id: &tid,
                    title,
                    body: body.as_deref(),
                    blocked_by,
                    env,
                    spec,
                    priority,
                    status,
                    validation_criteria,
                    tags: &effective_tags,
                    requires,
                    agent_eligibility: &agent_eligibility,
                });
                let content = add_parent(content, parent, &path)?;
                crate::effects::FilePlan(vec![crate::effects::FileEffect::CreateNoClobber {
                    path,
                    text: content,
                    executable: false,
                }])
                .execute(crate::effects::Mode::current())?;
                files.push(format!(".tickets/{}", filename));
            }
            let tids: Vec<String> = (0..parsed.len())
                .map(|i| format!("{:0>width$}", base + i as u64, width = width))
                .collect();
            let file_paths: Vec<&str> = files.iter().map(String::as_str).collect();
            let _ = txn.commit_created(
                &file_paths,
                &format!(
                    "chore(tickets): batch {} ({})",
                    tids.join(","),
                    parsed
                        .iter()
                        .map(|(s, _)| *s)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None,
            )?;
            Ok((base, width))
        };

    let (mut base, mut width) = allocate_and_commit(&names, &parsed)?;

    // Dry-run: compute IDs and show what would be created
    if is_dry_run() {
        crate::effects::Phase::AllocationPublication(crate::transaction::AllocationPlan::Attempt(
            &txn,
        ))
        .execute(crate::effects::Mode::current())?;
        let base = core::max_id(&names) + 1;
        let width = core::id_width(&names);
        let tids: Vec<String> = (0..parsed.len())
            .map(|i| format!("{:0>width$}", base + i as u64, width = width))
            .collect();
        if is_quiet_json() {
            print_quiet_json_success(&format!("would create tickets {}", tids.join(", ")));
        } else {
            println!("Would create {} tickets:", parsed.len());
            for (i, (slug, title)) in parsed.iter().enumerate() {
                let tid = &tids[i];
                println!("  {} {}-{}.md — {}", tid, tid, slug, title);
            }
        }
        return Ok(0);
    }

    let outcome = match txn.try_push()? {
        PublishResult::Done(outcome) => outcome,
        PublishResult::NeedsRetry => {
            let names = txn.scan_names()?;
            let result = allocate_and_commit(&names, &parsed)?;
            base = result.0;
            width = result.1;
            txn.push_retry()?
        }
    };

    let tids: Vec<String> = (0..parsed.len())
        .map(|i| format!("{:0>width$}", base + i as u64, width = width))
        .collect();
    if is_quiet() && is_json_output() {
        crate::cli::emit_json_success(&format!("created tickets {}", tids.join(", ")));
    } else {
        for (i, (slug, _)) in parsed.iter().enumerate() {
            let tid = &tids[i];
            if is_quiet() {
                println!("{}", tid);
            } else {
                let detail = match &outcome {
                    crate::transaction::PublishOutcome::LocalOnly => "local only",
                    _ => "pushed",
                };
                print_success("created", tid, slug, detail);
            }
        }
    }
    Ok(0)
}

fn add_parent(content: String, parent: Option<&str>, path: &std::path::Path) -> Result<String> {
    let Some(parent) = parent else {
        return Ok(content);
    };
    let mut file = core::TicketFile::parse_str(&content, path)?;
    file.set_parent(parent);
    Ok(file.serialize())
}

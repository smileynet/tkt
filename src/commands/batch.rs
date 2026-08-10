//! `tkt batch` — allocate N sequential IDs in one commit/push.

use anyhow::Result;

use crate::commands::common::{domain_bail, is_quiet, success_msg, tickets_dir};
use crate::core::{self, validate};
use crate::git;
use crate::transaction::{GitTransaction, PublishResult};

pub fn run(
    items: &[String],
    spec: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    blocked_by: &[String],
) -> Result<i32> {
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
    let txn = GitTransaction::new(&dir)?;
    let names = txn.scan_names();

    let allocate_and_commit =
        |names: &[String], parsed: &[(&str, String)]| -> Result<(u64, usize)> {
            let base = core::max_id(names) + 1;
            let width = core::id_width(names);
            let mut files: Vec<String> = Vec::new();
            for (i, (slug, title)) in parsed.iter().enumerate() {
                let tid = format!("{:0>width$}", base + i as u64, width = width);
                let filename = format!("{}-{}.md", tid, slug);
                let path = txn.dir.join(&filename);
                let content =
                    core::new_ticket_text(&tid, title, blocked_by, env, spec, priority, status);
                std::fs::write(&path, &content)?;
                files.push(format!(".tickets/{}", filename));
            }
            for f in &files {
                git::add(&txn.repo, &[f.as_str()])?;
            }
            let tids: Vec<String> = (0..parsed.len())
                .map(|i| format!("{:0>width$}", base + i as u64, width = width))
                .collect();
            git::commit(
                &txn.repo,
                &format!(
                    "chore(tickets): batch {} ({})",
                    tids.join(","),
                    parsed
                        .iter()
                        .map(|(s, _)| *s)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )?;
            Ok((base, width))
        };

    let (mut base, mut width) = allocate_and_commit(&names, &parsed)?;

    match txn.try_push()? {
        PublishResult::Done(_) => {}
        PublishResult::NeedsRetry => {
            let names = txn.scan_names();
            let result = allocate_and_commit(&names, &parsed)?;
            base = result.0;
            width = result.1;
            txn.push_retry()?;
        }
    }

    for (i, (slug, _)) in parsed.iter().enumerate() {
        let tid = format!("{:0>width$}", base + i as u64, width = width);
        if is_quiet() {
            println!("{}", tid);
        } else {
            println!("{}", success_msg("created", &tid, slug, "pushed"));
        }
    }
    Ok(0)
}

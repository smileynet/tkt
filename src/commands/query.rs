//! `tkt query` — dump all tickets as JSON Lines.

use anyhow::Result;

use std::collections::{BTreeMap, HashSet};

use crate::commands::common::{is_json_output, tickets_dir};
use crate::core::{self, Status};

pub fn run(
    status_filter: Option<&str>,
    priority_filter: Option<&str>,
    parent_filter: Option<&str>,
    all_context: bool,
) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let ctx = crate::context::load(&dir);
    let done_ids: HashSet<&str> = corpus
        .iter()
        .filter(|ticket| ticket.status == Status::Done)
        .map(|ticket| ticket.id.as_str())
        .collect();

    let mut count: u32 = 0;
    let mut context_hidden = 0;
    for t in &corpus {
        if let Some(sf) = status_filter {
            if t.status.as_str() != sf {
                continue;
            }
        }
        if let Some(pf) = priority_filter {
            match t.priority {
                Some(p) if p.as_str() == pf => {}
                _ => continue,
            }
        }
        if let Some(parent) = parent_filter {
            if t.parent.as_deref() != Some(parent) {
                continue;
            }
        }
        if !all_context && !ctx.matches(&t.tags) {
            context_hidden += 1;
            continue;
        }

        count += 1;

        let blocked_by: Vec<String> = t
            .blocked_by
            .iter()
            .map(|d| format!("\"{}\"", core::json_string_escape(d)))
            .collect();
        let pending_blockers: Vec<&str> = t
            .blocked_by
            .iter()
            .map(String::as_str)
            .filter(|dependency| !done_ids.contains(dependency))
            .collect();

        let mut fields = vec![
            format!("\"id\":\"{}\"", core::json_string_escape(&t.id)),
            format!("\"title\":\"{}\"", core::json_string_escape(&t.title)),
            format!(
                "\"status\":\"{}\"",
                core::json_string_escape(t.status.as_str())
            ),
            format!("\"blocked_by\":[{}]", blocked_by.join(",")),
            format!(
                "\"pending_blockers\":{}",
                serde_json::to_string(&pending_blockers)?
            ),
            format!("\"tags\":{}", serde_json::to_string(&t.tags)?),
            format!(
                "\"validation_criteria\":{}",
                serde_json::to_string(&t.validation_criteria)?
            ),
            format!("\"requires\":{}", serde_json::to_string(&t.requires)?),
        ];

        if t.file.get("env").is_some() {
            fields.push(format!(
                "\"env\":\"{}\"",
                core::json_string_escape(t.env.as_str())
            ));
        }
        if let Some(priority) = t.priority {
            fields.push(format!(
                "\"priority\":\"{}\"",
                core::json_string_escape(priority.as_str())
            ));
        }
        if let Some(ref spec) = t.spec {
            fields.push(format!("\"spec\":\"{}\"", core::json_string_escape(spec)));
        }
        if let Some(ref parent) = t.parent {
            fields.push(format!(
                "\"parent\":\"{}\"",
                core::json_string_escape(parent)
            ));
        }

        fields.push(format!(
            "\"agents\":{}",
            serde_json::to_string(&t.agent_eligibility.signed_values())?
        ));
        fields.push(format!(
            "\"assignee\":{}",
            t.assignee
                .as_ref()
                .map(|owner| format!("\"{}\"", core::json_string_escape(owner.as_str())))
                .unwrap_or_else(|| "null".to_string())
        ));
        let mut raw_frontmatter: BTreeMap<&str, &str> = t
            .file
            .fm
            .iter()
            .filter(|(key, _)| {
                !key.is_empty()
                    && !matches!(
                        key.as_str(),
                        "id" | "title"
                            | "status"
                            | "blocked_by"
                            | "env"
                            | "priority"
                            | "spec"
                            | "parent"
                            | "assignee"
                            | "validation_criteria"
                            | "tags"
                            | "agents"
                            | "requires"
                    )
            })
            .map(|(key, value)| (key.as_str(), value.trim()))
            .collect();
        if t.priority.is_none() {
            if let Some(raw_priority) = t.file.get("priority") {
                raw_frontmatter.insert("priority", raw_priority);
            }
        }
        if !raw_frontmatter.is_empty() {
            fields.push(format!(
                "\"frontmatter\":{}",
                serde_json::to_string(&raw_frontmatter)?
            ));
        }
        println!("{{{}}}", fields.join(","));
    }
    if context_hidden > 0 && !all_context {
        let message = format!(
            "active tag context ({}) filtered out {} ticket(s); use --all-context to bypass tag context",
            ctx.serialize(),
            context_hidden
        );
        if is_json_output() {
            eprintln!(
                "{}",
                serde_json::json!({
                    "warning": {
                        "kind": "context_filter",
                        "message": message
                    }
                })
            );
        } else {
            eprintln!("warning: {message}");
        }
    }
    crate::RESULT_COUNT.store(count as i32, std::sync::atomic::Ordering::Relaxed);
    Ok(0)
}

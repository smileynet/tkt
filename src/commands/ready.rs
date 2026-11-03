//! `tkt ready` — show the frontier (unblocked tickets).

use anyhow::Result;

use crate::commands::common::{is_quiet, project_config, tickets_dir};
use crate::core::{self, AgentId, Status, Ticket};

pub fn run(json: bool, agent: Option<&AgentId>) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir)?;
    let corpus = core::load_corpus(&dir)?;
    let legacy_env = std::env::var("CREW_ENV")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            pcfg.ready_default_env
                .map(|value| value.as_str().to_string())
        })
        .unwrap_or_default();
    let mut capabilities = pcfg.machine_capabilities.clone();
    if !legacy_env.is_empty() && !capabilities.contains(&legacy_env) {
        capabilities.push(legacy_env);
    }
    let front = core::frontier_with_capabilities(&corpus, &capabilities);

    // Apply context filter (unless bypass is active)
    let ctx = crate::context::load(&dir);
    let scoped: Vec<&Ticket> = front.into_iter().filter(|t| ctx.matches(&t.tags)).collect();
    let hidden_restricted = scoped
        .iter()
        .filter(|ticket| ticket.agent_eligibility.check(agent).is_err())
        .count();
    let hidden_assigned = scoped
        .iter()
        .filter(|ticket| {
            ticket
                .assignee
                .as_ref()
                .is_some_and(|owner| Some(owner) != agent)
        })
        .count();
    let front: Vec<&Ticket> = scoped
        .into_iter()
        .filter(|ticket| ticket.agent_eligibility.check(agent).is_ok())
        .filter(|ticket| {
            ticket
                .assignee
                .as_ref()
                .is_none_or(|owner| Some(owner) == agent)
        })
        .collect();

    // Record frontier size for telemetry
    crate::RESULT_COUNT.store(front.len() as i32, std::sync::atomic::Ordering::Relaxed);

    let dbg = crate::telemetry::debug_mode();
    let open = corpus.iter().filter(|t| t.status == Status::Open).count();
    let wip_count = corpus
        .iter()
        .filter(|t| t.status == Status::InProgress)
        .count();
    let done = corpus.iter().filter(|t| t.status == Status::Done).count();
    crate::telemetry::debug_event(
        dbg,
        "",
        "",
        &format!(
            "corpus loaded: {} tickets ({} open, {} in_progress, {} done), frontier: {}",
            corpus.len(),
            open,
            wip_count,
            done,
            front.len()
        ),
    );

    if hidden_restricted > 0 && agent.is_none() {
        eprintln!("Hidden: {} agent-restricted ticket(s). Identify this invocation with --agent <id> or TKT_AGENT.", hidden_restricted);
    }
    if hidden_assigned > 0 {
        if agent.is_none() {
            eprintln!(
                "Hidden: {} ticket(s) have an assignee; identify this invocation to see tickets assigned to you.",
                hidden_assigned
            );
        } else {
            eprintln!(
                "Hidden: {} ticket(s) assigned to another agent.",
                hidden_assigned
            );
        }
    }
    if json {
        for t in &front {
            let blocked_by: Vec<String> = t
                .blocked_by
                .iter()
                .map(|d| format!("\"{}\"", core::json_string_escape(d)))
                .collect();
            let mut fields = vec![
                format!("\"id\":\"{}\"", core::json_string_escape(&t.id)),
                format!("\"title\":\"{}\"", core::json_string_escape(&t.title)),
                format!(
                    "\"status\":\"{}\"",
                    core::json_string_escape(t.status.as_str())
                ),
                format!("\"blocked_by\":[{}]", blocked_by.join(",")),
            ];
            if t.env != core::Env::Either {
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
            let agents = t
                .agent_eligibility
                .signed_values()
                .iter()
                .map(|agent| format!("\"{}\"", core::json_string_escape(agent)))
                .collect::<Vec<_>>()
                .join(",");
            fields.push(format!("\"agents\":[{}]", agents));
            fields.push(format!(
                "\"assignee\":{}",
                t.assignee
                    .as_ref()
                    .map(|owner| format!("\"{}\"", core::json_string_escape(owner.as_str())))
                    .unwrap_or_else(|| "null".to_string())
            ));
            println!("{{{}}}", fields.join(","));
        }
    } else if is_quiet() {
        for t in &front {
            println!("{}", t.id);
        }
    } else {
        if front.is_empty() {
            println!("No tickets ready.");
        } else {
            println!("Ready ({}):", front.len());
            for t in &front {
                let flag = match t.priority {
                    Some(core::Priority::Urgent) => "  [URGENT]",
                    Some(core::Priority::High) => "  [HIGH]",
                    Some(core::Priority::Low) => "  [low]",
                    _ => "",
                };
                let agents = t.agent_eligibility.signed_values();
                let policy = if agents.is_empty() {
                    String::new()
                } else {
                    format!("  [agents: {}]", agents.join(", "))
                };
                let assignee = t
                    .assignee
                    .as_ref()
                    .map(|owner| format!("  [assignee: {}]", owner.as_str()))
                    .unwrap_or_default();
                println!("  {}  {}{}{}{}", t.id, t.title, flag, policy, assignee);
            }
        }

        let wip: Vec<&Ticket> = corpus
            .iter()
            .filter(|t| t.status == Status::InProgress)
            .collect();
        if !wip.is_empty() {
            println!("\nIn progress ({}):", wip.len());
            for t in &wip {
                let assignee = t
                    .assignee
                    .as_ref()
                    .map(|owner| format!("  [assignee: {}]", owner.as_str()))
                    .unwrap_or_default();
                println!("  {}  {}{}", t.id, t.title, assignee);
            }
        }
    }
    Ok(0)
}

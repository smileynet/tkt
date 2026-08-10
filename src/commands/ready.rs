//! `tkt ready` — show the frontier (unblocked tickets).

use anyhow::Result;

use crate::core::{self, Status, Ticket};
use crate::commands::common::{is_quiet, project_config, tickets_dir};

pub fn run(json: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    let corpus = core::load_corpus(&dir)?;
    let front = core::frontier_with_default_env(&corpus, &pcfg.ready_default_env);

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
                println!("  {}  {}{}", t.id, t.title, flag);
            }
        }

        let wip: Vec<&Ticket> = corpus
            .iter()
            .filter(|t| t.status == Status::InProgress)
            .collect();
        if !wip.is_empty() {
            println!("\nIn progress ({}):", wip.len());
            for t in &wip {
                println!("  {}  {}", t.id, t.title);
            }
        }
    }
    Ok(0)
}

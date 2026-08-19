//! `tkt blocked` — show tickets with unsatisfied dependencies.

use anyhow::Result;

use crate::commands::common::{is_quiet, tickets_dir};
use crate::core::{self, Status, Ticket};

pub fn run() -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;

    let done: std::collections::HashSet<&str> = corpus
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();

    let mut blocked: Vec<&Ticket> = corpus
        .iter()
        .filter(|t| {
            t.status == Status::Open
                && !t.blocked_by.is_empty()
                && !t.blocked_by.iter().all(|dep| done.contains(dep.as_str()))
        })
        .collect();

    blocked.sort_by_key(|t| t.numeric_key());

    // Record count for telemetry
    crate::RESULT_COUNT.store(blocked.len() as i32, std::sync::atomic::Ordering::Relaxed);

    if blocked.is_empty() {
        if !is_quiet() {
            println!("No blocked tickets.");
        }
        return Ok(0);
    }

    if !is_quiet() {
        println!("Blocked ({}):", blocked.len());
    }
    for t in &blocked {
        if is_quiet() {
            println!("{}", t.id);
        } else {
            println!("  {}  {}", t.id, t.title);
            let undone_deps: Vec<String> = t
                .blocked_by
                .iter()
                .filter(|dep| !done.contains(dep.as_str()))
                .map(|dep| {
                    corpus
                        .iter()
                        .find(|c| c.id == *dep)
                        .map(|c| format!("{} {} ({})", dep, c.title, c.status.as_str()))
                        .unwrap_or_else(|| format!("{} (not found)", dep))
                })
                .collect();
            for dep in &undone_deps {
                println!("    blocked by: {}", dep);
            }
        }
    }
    Ok(0)
}

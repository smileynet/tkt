//! `tkt children` — computed progress without changing lifecycle status.

use anyhow::Result;
use serde_json::json;

use crate::commands::common::{is_json_output, is_quiet, tickets_dir};
use crate::core::{self, Status};

pub fn run(id: &str) -> Result<i32> {
    let corpus = core::load_corpus(&tickets_dir()?)?;
    let parent = corpus
        .iter()
        .find(|ticket| ticket.id == id)
        .ok_or_else(|| {
            crate::DomainError::new(
                crate::ErrorKind::NotFound,
                format!("no ticket with id {id}"),
            )
        })?;
    // An explicit parent view must not imply completion from a scoped subset.
    let mut children: Vec<_> = corpus
        .iter()
        .filter(|ticket| ticket.parent.as_deref() == Some(parent.id.as_str()))
        .collect();
    children.sort_by_key(|ticket| ticket.numeric_key());
    let done = children.iter().filter(|t| t.status == Status::Done).count();
    let cancelled = children
        .iter()
        .filter(|t| t.status == Status::Cancelled)
        .count();
    crate::RESULT_COUNT.store(children.len() as i32, std::sync::atomic::Ordering::Relaxed);
    if is_json_output() {
        println!(
            "{}",
            json!({
                "id": parent.id,
                "title": parent.title,
                "status": parent.status.as_str(),
                "progress": {"done": done, "total": children.len(), "cancelled": cancelled},
                "children": children.iter().map(|ticket| json!({
                    "id": ticket.id, "title": ticket.title, "status": ticket.status.as_str()
                })).collect::<Vec<_>>()
            })
        );
    } else if is_quiet() {
        for ticket in children {
            println!("{}", ticket.id);
        }
    } else {
        println!(
            "{} {} ({}): {}/{} done, {} cancelled",
            parent.id,
            parent.title,
            parent.status.as_str(),
            done,
            children.len(),
            cancelled
        );
        for ticket in children {
            println!(
                "  {} {} ({})",
                ticket.id,
                ticket.title,
                ticket.status.as_str()
            );
        }
    }
    Ok(0)
}

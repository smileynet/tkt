//! `tkt query` — dump all tickets as JSON Lines.

use anyhow::Result;

use crate::commands::common::tickets_dir;
use crate::core;

pub fn run(status_filter: Option<&str>, priority_filter: Option<&str>) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let ctx = crate::context::load(&dir);

    let mut count: u32 = 0;
    for t in &corpus {
        if !ctx.matches(&t.tags) {
            continue;
        }
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

        count += 1;

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
    crate::RESULT_COUNT.store(count as i32, std::sync::atomic::Ordering::Relaxed);
    Ok(0)
}

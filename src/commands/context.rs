//! `tkt context` — set, show, or clear the active tag context.

use anyhow::Result;

use crate::commands::common::{is_quiet, tickets_dir};
use crate::context;

/// Run the context command.
/// - No args + no clear: show current context
/// - Args (tags): set context
/// - clear: remove context
pub fn run(tags: &[String], clear: bool) -> Result<i32> {
    let dir = tickets_dir()?;

    if clear {
        context::save(&dir, &context::Context::default())?;
        if !is_quiet() {
            println!("context cleared");
        }
        return Ok(0);
    }

    if tags.is_empty() {
        // Show current context
        let ctx = context::load(&dir);
        if ctx.is_empty() {
            if !is_quiet() {
                println!("no context set");
            }
        } else {
            println!("{}", ctx.serialize());
        }
        return Ok(0);
    }

    // Set context from provided tags
    let raw = tags.join(" ");
    let ctx = context::parse_context(&raw);

    if ctx.is_empty() {
        context::save(&dir, &context::Context::default())?;
        if !is_quiet() {
            println!("context cleared");
        }
    } else {
        context::save(&dir, &ctx)?;
        if !is_quiet() {
            println!("context set: {}", ctx.serialize());
        }
    }

    Ok(0)
}

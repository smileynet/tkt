//! `tkt renumber` — move a ticket to a new ID atomically.

use anyhow::Result;

use crate::commands::common::{domain_bail, is_quiet, print_success};
use crate::core::validate;
use crate::mutation::MutationContext;
use crate::renumber::{apply_renumber, RenumberPlan};

pub fn run(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    if let Err(e) = validate::validate_id(new_id) {
        domain_bail!("new id: {}", e);
    }

    let ctx = MutationContext::open()?;

    // Plan
    let plan = RenumberPlan::single(&ctx.corpus, old_id, new_id, file_hint)
        .map_err(|e| crate::DomainError::new(crate::ErrorKind::Validation, e.to_string()))?;

    // Apply
    let result = apply_renumber(&ctx.tickets_dir, &plan)?;

    // Publish
    let path_refs: Vec<&str> = result.staged_paths.iter().map(|s| s.as_str()).collect();
    ctx.publish(
        &path_refs,
        &format!("chore(tickets): renumber {} -> {}", old_id, new_id),
    )?;

    // Report
    if !is_quiet() {
        let detail = if result.refs_updated > 0 {
            format!("{} refs updated", result.refs_updated)
        } else {
            String::new()
        };
        print_success("renumbered", old_id, &format!("→ {}", new_id), &detail);
    }
    Ok(0)
}

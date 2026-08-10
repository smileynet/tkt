//! `tkt rebase` — resolve ID collisions with upstream.

use anyhow::Result;

use crate::commands::common::{is_quiet, tickets_dir};
use crate::git;
use crate::renumber::{apply_renumber, RenumberPlan};

pub fn run(dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = dir.parent().unwrap().to_path_buf();

    // Step 1: fetch origin
    git::fetch(&repo)?;

    // Step 2: gather local + remote filenames
    let remote_names = git::remote_ticket_names(&repo);
    let local_names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Step 3: plan
    let plan = RenumberPlan::for_collisions(&local_names, &remote_names);

    if plan.is_empty() {
        if !is_quiet() {
            println!("No ID collisions with upstream.");
        }
        return Ok(0);
    }

    // Step 4: dry-run preview
    if dry_run {
        println!("Collisions detected ({}):", plan.entries.len());
        for entry in &plan.entries {
            println!("  {} → {} ({})", entry.old_id, entry.new_id, entry.slug);
        }
        println!("\nRun without --dry-run to apply.");
        return Ok(0);
    }

    // Step 5: apply
    let result = apply_renumber(&dir, &plan)?;

    // Step 6: publish (stage + commit)
    let add_paths: Vec<&str> = result.staged_paths.iter().map(|s| s.as_str()).collect();
    git::add(&repo, &add_paths)?;
    let msg = format!(
        "chore(tickets): rebase — renumber {} ticket(s) to resolve ID collision",
        plan.entries.len()
    );
    git::commit(&repo, &msg)?;

    // Step 7: report
    if !is_quiet() {
        println!("Renumbered {} ticket(s):", plan.entries.len());
        for entry in &plan.entries {
            println!("  {} → {} ({})", entry.old_id, entry.new_id, entry.slug);
        }
        if result.refs_updated > 0 {
            println!("  {} blocked_by reference(s) updated", result.refs_updated);
        }
    }
    Ok(0)
}

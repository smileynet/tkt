//! `tkt rebase` — resolve ID collisions with upstream.

use anyhow::Result;

use crate::commands::common::{is_quiet, is_quiet_json, print_quiet_json_success, tickets_dir};
use crate::git;
use crate::renumber::{apply_renumber, materialize_renumber, RenumberPlan};

pub fn run(dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = dir.parent().unwrap().to_path_buf();
    if !dry_run && git::current_branch(&repo)?.is_empty() {
        anyhow::bail!(
            "cannot resolve upstream collisions from detached HEAD; switch to a branch first"
        );
    }
    let Some(upstream) = git::resolve_upstream(&repo)? else {
        if is_quiet() {
            print_quiet_json_success("no remote configured; no collisions to resolve");
        } else {
            println!("No remote configured; no upstream collisions to resolve.");
        }
        return Ok(0);
    };

    // A dry-run reads the last fetched remote state but never performs network I/O.
    if !dry_run {
        git::fetch(&repo, &upstream)?;
    }

    // Step 2: gather local + remote filenames
    let remote_names = git::remote_ticket_names(&repo, &upstream)?;
    let local_names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Step 3: plan
    let plan = RenumberPlan::for_collisions(&local_names, &remote_names);

    if plan.is_empty() {
        if is_quiet() {
            print_quiet_json_success("no ID collisions with upstream");
        } else {
            println!("No ID collisions with upstream.");
        }
        return Ok(0);
    }

    // Parse and render every local ticket before previewing or mutating, so an
    // unreadable corpus never leaves collision recovery half-applied.
    let materialized = materialize_renumber(&dir, &plan, false)?;

    // Step 4: dry-run preview
    if dry_run {
        if is_quiet_json() {
            print_quiet_json_success(&format!(
                "would renumber {} ticket(s) to resolve collisions",
                plan.entries.len()
            ));
        } else {
            println!("Collisions detected ({}):", plan.entries.len());
            for entry in &plan.entries {
                println!("  {} → {} ({})", entry.old_id, entry.new_id, entry.slug);
            }
            print_body_citations(materialized.body_citations());
            println!("\nRun without --dry-run to apply.");
        }
        return Ok(0);
    }

    // Step 5: apply
    let result = apply_renumber(&materialized)?;

    // Step 6: publish (stage + commit)
    let add_paths: Vec<&str> = result.staged_paths.iter().map(|s| s.as_str()).collect();
    git::add(&repo, &add_paths)?;
    let msg = format!(
        "chore(tickets): rebase — renumber {} ticket(s) to resolve ID collision",
        plan.entries.len()
    );
    git::commit(&repo, &msg, &add_paths)?;

    // Step 7: report
    if is_quiet() {
        print_quiet_json_success(&format!(
            "renumbered {} ticket(s), updated {} blocked_by reference(s)",
            plan.entries.len(),
            result.refs_updated
        ));
    } else {
        println!("Renumbered {} ticket(s):", plan.entries.len());
        for entry in &plan.entries {
            println!("  {} → {} ({})", entry.old_id, entry.new_id, entry.slug);
        }
        if result.refs_updated > 0 {
            println!("  {} blocked_by reference(s) updated", result.refs_updated);
        }
        print_body_citations(&result.body_citations);
    }
    Ok(0)
}

fn print_body_citations(citations: &[(String, crate::renumber::BodyCitation, String)]) {
    if citations.is_empty() {
        return;
    }
    println!("Body citations that may need review (prose unchanged):");
    for (file, citation, new_id) in citations {
        println!(
            "  .tickets/{file}:{} `{}` may refer to renumbered ID {} → {}",
            citation.line, citation.text, citation.id, new_id
        );
    }
}

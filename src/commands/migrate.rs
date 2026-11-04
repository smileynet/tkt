//! `tkt migrate` — convert foreign ticket schemas to tkt format.

use anyhow::Result;

use crate::commands::common::{domain_bail, is_quiet, print_quiet_json_success, tickets_dir};
use crate::migrate::{self, DetectedFormat};

pub fn run(from: Option<&str>, detect: bool, confirm: bool) -> Result<i32> {
    let dir = tickets_dir()?;

    // --detect mode: report format and exit
    if detect {
        let detection = migrate::detect(&dir);
        if !is_quiet() {
            println!(
                "detected: {} (confidence: {:.0}%)",
                detection.format.as_str(),
                detection.confidence * 100.0
            );
            for signal in &detection.signals {
                println!("  {}", signal);
            }
        }
        return Ok(0);
    }

    // --from is required for migration
    let source = from.unwrap_or("");

    let source = if source.is_empty() {
        let detection = migrate::detect(&dir);
        if detection.format == DetectedFormat::Unknown {
            domain_bail!(
                "cannot auto-detect ticket format — use --from to specify (available: tk)"
            );
        }
        if !is_quiet() {
            println!(
                "auto-detected: {} (confidence: {:.0}%)",
                detection.format.as_str(),
                detection.confidence * 100.0
            );
        }
        detection.format.as_str().to_string()
    } else {
        source.to_string()
    };

    match source.as_str() {
        "tk" => run_tk_migration(&dir, confirm),
        other => {
            domain_bail!("unknown source format: {} (available: tk)", other);
        }
    }
}

fn run_tk_migration(dir: &std::path::Path, confirm: bool) -> Result<i32> {
    let dry_run = crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed);

    // Build plan
    let plan = migrate::plan_tk(dir);

    if plan.total == 0 {
        if is_quiet() {
            print_quiet_json_success("no tickets to migrate");
        } else {
            println!("No tickets to migrate.");
        }
        return Ok(0);
    }

    let prepared = migrate::prepare(dir, &plan)?;
    let orphaned = prepared.orphaned_deps();

    // Print plan
    if !is_quiet() {
        println!("Migration plan ({} tickets, tk → tkt):", plan.total);
        println!();
        println!("  Source               → ID     Title");
        println!("  ──────────────────── → ────── ─────");
        for entry in &plan.entries {
            println!(
                "  {:<20} → {:<6} {}",
                entry.old_slug, entry.new_id, entry.extracted_title
            );
        }
        println!();

        if !orphaned.is_empty() {
            println!("  ⚠ Unresolved deps (will be dropped):");
            for (slug, dep) in orphaned {
                println!("    {} references unknown dep: {}", slug, dep);
            }
            println!();
        }
    }

    if dry_run {
        let _ = prepared.apply()?;
        if is_quiet() {
            print_quiet_json_success(&format!("would migrate {} ticket(s)", plan.total));
        } else {
            println!("Review the plan, then run again with --confirm to apply.");
        }
        return Ok(0);
    }

    // Safety: do not apply without an explicit, separately-invoked confirmation.
    // A preview never leaves a marker behind, so it remains genuinely no-write.
    if !confirm {
        return Err(crate::DomainError::with_hint(
            crate::ErrorKind::GateFailed,
            "migration requires a reviewed preview and explicit --confirm".to_string(),
            "run: tkt migrate --from tk --dry-run (review the plan, then run: tkt migrate --from tk --confirm)"
                .to_string(),
        )
        .into());
    }

    // Apply
    let result = prepared.apply()?;

    if is_quiet() {
        print_quiet_json_success(&format!(
            "migrated {} ticket(s), renamed {}",
            result.files_written, result.files_renamed
        ));
    } else {
        println!(
            "✓ Migrated {} ticket(s) ({} renamed)",
            result.files_written, result.files_renamed
        );
        if !result.orphaned_deps.is_empty() {
            println!(
                "  ⚠ {} unresolved dep(s) dropped",
                result.orphaned_deps.len()
            );
        }
        println!("  Originals backed up to .tickets.bak/");
        println!("  Run `tkt validate` to verify.");
    }

    Ok(0)
}

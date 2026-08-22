//! `tkt migrate` — convert foreign ticket schemas to tkt format.

use anyhow::Result;

use crate::commands::common::{domain_bail, is_quiet, tickets_dir};
use crate::migrate::{self, DetectedFormat};

pub fn run(from: Option<&str>, detect: bool) -> Result<i32> {
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
        "tk" => run_tk_migration(&dir),
        other => {
            domain_bail!("unknown source format: {} (available: tk)", other);
        }
    }
}

fn run_tk_migration(dir: &std::path::Path) -> Result<i32> {
    let dry_run = crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed);

    // Build plan
    let plan = migrate::plan_tk(dir);

    if plan.total == 0 {
        if !is_quiet() {
            println!("No tickets to migrate.");
        }
        return Ok(0);
    }

    // Check for orphaned deps before applying
    let mut orphaned = Vec::new();
    for entry in &plan.entries {
        let content = std::fs::read_to_string(&entry.source_path).unwrap_or_default();
        let parsed_deps: Vec<String> = content
            .lines()
            .find(|l| l.trim().starts_with("deps:"))
            .map(|l| {
                l.trim()
                    .strip_prefix("deps:")
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|d| {
                        d.trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim()
                            .to_string()
                    })
                    .filter(|d| !d.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        for dep in &parsed_deps {
            if !plan.id_map.contains_key(dep.as_str()) {
                orphaned.push((entry.old_slug.clone(), dep.clone()));
            }
        }
    }

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
            for (slug, dep) in &orphaned {
                println!("    {} references unknown dep: {}", slug, dep);
            }
            println!();
        }
    }

    if dry_run {
        if !is_quiet() {
            println!("Run without --dry-run to apply.");
        }
        return Ok(0);
    }

    // Apply
    let result = migrate::apply(dir, &plan)?;

    if !is_quiet() {
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

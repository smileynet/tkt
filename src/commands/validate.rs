//! `tkt validate` — check for cycles, dangling deps, contract violations.

use anyhow::Result;

use crate::commands::common::{project_config, tickets_dir};
use crate::core::Ticket;
use crate::findings::{self, Finding};

pub fn run(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    let effective_strict = strict || pcfg.validate_strict;
    let mut all_findings: Vec<Finding> = Vec::new();

    let mut corpus: Vec<Ticket> = Vec::new();
    for entry in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "md") {
            match Ticket::parse(&entry.path()) {
                Ok(t) => corpus.push(t),
                Err(e) => all_findings.push(Finding {
                    file: entry.file_name().to_string_lossy().to_string(),
                    rule: "unparseable".to_string(),
                    message: e.to_string(),
                    severity: "error".to_string(),
                }),
            }
        }
    }

    all_findings.extend(findings::check_status(&corpus));
    all_findings.extend(findings::check_env(&corpus));
    all_findings.extend(findings::check_id_filename(&corpus));
    all_findings.extend(findings::check_duplicate_ids(&corpus));
    all_findings.extend(findings::check_dangling_deps(&corpus));
    all_findings.extend(findings::check_cycles(&corpus));
    all_findings.extend(findings::check_unchecked_acs(&corpus));

    let status = findings::status_from_findings(&all_findings, effective_strict);
    findings::print_findings(&all_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

/// Run validate with --fix mode: repair fixable issues, advise on the rest.
pub fn run_with_fix(strict: bool, brief: bool, dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;

    // Run the fix pass
    let result = crate::fix::run_fix(&dir, dry_run)?;

    // Report repairs
    if !result.repairs.is_empty() {
        let verb = if dry_run { "Would fix" } else { "Fixed" };
        println!("{} ({}):", verb, result.repairs.len());
        for r in &result.repairs {
            let tier_label = match r.tier {
                1 => "",
                2 => " [mapped]",
                _ => "",
            };
            println!("  {}: {}{}", r.file, r.description, tier_label);
        }
        println!();
    }

    // Report advisories
    if !result.advisories.is_empty() {
        println!("Needs manual review ({}):", result.advisories.len());
        for a in &result.advisories {
            println!("  {}: {}", a.file, a.message);
            println!("    → {}", a.suggestion);
        }
        println!();
    }

    if result.repairs.is_empty() && result.advisories.is_empty() {
        println!("Nothing to fix.");
    }

    // After fixing, run normal validate to show remaining state
    if !dry_run && !result.repairs.is_empty() {
        println!("--- post-fix validation ---");
        return run(strict, brief);
    }

    Ok(if result.advisories.is_empty() { 0 } else { 1 })
}

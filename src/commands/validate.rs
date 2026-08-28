//! `tkt validate` — check for cycles, dangling deps, contract violations.

use anyhow::Result;

use crate::commands::common::{project_config, tickets_dir};
use crate::core::Ticket;
use crate::findings::{self, Finding};

/// Collect all validation findings for the corpus in `dir`.
/// Shared by `run` (print + exit) and `run_with_fix` (before/after comparison)
/// so the two can never disagree on what a finding is.
fn collect_findings(dir: &std::path::Path) -> Result<Vec<Finding>> {
    let mut all_findings: Vec<Finding> = Vec::new();

    let mut corpus: Vec<Ticket> = Vec::new();
    for entry in std::fs::read_dir(dir)?.filter_map(|e| e.ok()) {
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
    all_findings.extend(crate::audit::check_resolution_quality(&corpus));

    Ok(all_findings)
}

/// Stable identity of a finding for before/after regression comparison.
fn finding_ids(findings: &[Finding]) -> std::collections::HashSet<(String, String)> {
    findings
        .iter()
        .map(|f| (f.file.clone(), f.rule.clone()))
        .collect()
}

pub fn run(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    let effective_strict = strict || pcfg.validate_strict;

    let all_findings = collect_findings(&dir)?;

    let status = findings::status_from_findings(&all_findings, effective_strict);
    crate::RESULT_COUNT.store(
        all_findings.len() as i32,
        std::sync::atomic::Ordering::Relaxed,
    );
    findings::print_findings(&all_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

/// Run validate with --fix mode: repair fixable issues, advise on the rest.
pub fn run_with_fix(strict: bool, brief: bool, dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;

    // Baseline finding identities BEFORE fixing (skip in dry-run — nothing is written).
    let before = if dry_run {
        std::collections::HashSet::new()
    } else {
        finding_ids(&collect_findings(&dir)?)
    };

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
        // Regression gate: a fix must never introduce a NEW finding. Compare
        // identities (file+rule), not counts, so a swap can't slip through.
        let after = finding_ids(&collect_findings(&dir)?);
        let new: Vec<&(String, String)> = after.difference(&before).collect();
        if !new.is_empty() {
            // Record the (worse) count for telemetry since we skip run() below.
            crate::RESULT_COUNT.store(after.len() as i32, std::sync::atomic::Ordering::Relaxed);
            let detail = new
                .iter()
                .map(|(f, r)| format!("{} [{}]", f, r))
                .collect::<Vec<_>>()
                .join(", ");
            let msg = format!(
                "validate --fix introduced {} new finding(s): {} — review and revert with: git checkout .tickets/",
                new.len(),
                detail
            );
            return Err(crate::DomainError::with_hint(
                crate::ErrorKind::GateFailed,
                msg,
                "git checkout .tickets/".to_string(),
            )
            .into());
        }
        println!("--- post-fix validation ---");
        return run(strict, brief);
    }

    Ok(if result.advisories.is_empty() { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(file: &str, rule: &str) -> Finding {
        Finding {
            file: file.into(),
            rule: rule.into(),
            message: String::new(),
            severity: "error".into(),
        }
    }

    #[test]
    fn regression_detected_when_new_finding_appears() {
        let before = finding_ids(&[f("01.md", "unparseable")]);
        // Fix mapped status but introduced a different rule on the same file.
        let after = finding_ids(&[f("01.md", "missing-resolution")]);
        let new: Vec<_> = after.difference(&before).collect();
        assert!(
            !new.is_empty(),
            "swap (unparseable -> missing-resolution) is a regression"
        );
    }

    #[test]
    fn no_regression_when_findings_only_reduced() {
        let before = finding_ids(&[
            f("02.md", "dangling-blocked-by"),
            f("03.md", "dangling-blocked-by"),
        ]);
        // Fix resolved one, left the other (subset of before).
        let after = finding_ids(&[f("03.md", "dangling-blocked-by")]);
        let new: Vec<_> = after.difference(&before).collect();
        assert!(
            new.is_empty(),
            "after is a subset of before -> no regression"
        );
    }

    #[test]
    fn no_regression_on_clean_fix() {
        let before = finding_ids(&[f("02.md", "dangling-blocked-by")]);
        let after = finding_ids(&[]);
        assert!(after.difference(&before).next().is_none());
    }

    #[test]
    fn identity_is_file_plus_rule_not_message() {
        // Same (file, rule) with different messages collapses to one identity.
        let mut a = f("01.md", "dangling-blocked-by");
        a.message = "ref 1".into();
        let mut b = f("01.md", "dangling-blocked-by");
        b.message = "ref 2".into();
        assert_eq!(finding_ids(&[a, b]).len(), 1);
    }
}

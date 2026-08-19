//! `tkt audit` — batch closure quality check.

use anyhow::Result;

use crate::audit;
use crate::commands::common::tickets_dir;
use crate::core;
use crate::findings;
use crate::git;

pub fn run(strict: bool, brief: bool, deep: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let repo = dir.parent().unwrap_or(&dir).to_path_buf();

    // Collect findings from all audit rules
    let mut all_findings = Vec::new();
    all_findings.extend(audit::check_resolution_quality(&corpus));
    all_findings.extend(audit::check_ac_completeness(&corpus));
    all_findings.extend(audit::check_stale_wip(&corpus, unix_now(), |path| {
        git_last_commit_ts(&repo, &dir, path)
    }));
    all_findings.extend(audit::check_frontier_health(&corpus));
    all_findings.extend(audit::check_validation_evidence(&corpus));

    // Deep analysis: evidence count, template detection (purely mechanical checks only)
    // Judgment calls (evidence quality, resolution substance) are in the companion skill.
    if deep {
        all_findings.extend(audit::check_evidence_count(&corpus));
        all_findings.extend(audit::check_template_only(&corpus));
    }

    let status = findings::status_from_findings(&all_findings, strict);
    findings::print_findings(&all_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

/// Current unix timestamp.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Git adapter: get last commit timestamp for a ticket file.
fn git_last_commit_ts(
    repo: &std::path::Path,
    dir: &std::path::Path,
    ticket_path: &std::path::Path,
) -> Option<u64> {
    let rel_path = ticket_path
        .strip_prefix(dir)
        .map(|p| format!(".tickets/{}", p.display()))
        .ok()?;
    let ts_str = git::git(repo, &["log", "-1", "--format=%ct", "--", &rel_path]).ok()?;
    ts_str.trim().parse::<u64>().ok()
}

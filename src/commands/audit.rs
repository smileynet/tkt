//! `tkt audit` — batch closure quality check.

use anyhow::Result;

use crate::audit;
use crate::commands::common::{project_config, tickets_dir};
use crate::core;
use crate::findings;
use crate::git;

pub fn run(strict: bool, brief: bool, deep: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let repo = dir.parent().unwrap_or(&dir).to_path_buf();

    // ADR 0003 grandfather cutoff: done tickets below this ID are exempt from the
    // resolution-quality rules only. Empty = no exemption; a non-numeric value is a
    // config error (consistent with the #190 config-parsing direction).
    let cfg = project_config(&dir);
    let resolution_baseline_id = parse_baseline_id(&cfg.audit_resolution_baseline_id)?;

    // Collect findings from all audit rules
    let mut all_findings = Vec::new();
    all_findings.extend(audit::check_resolution_quality(
        &corpus,
        resolution_baseline_id,
    ));
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

/// Parse the `[audit] resolution_baseline_id` config value into an optional cutoff.
/// Empty = no exemption (`None`). A non-numeric value is a structured config error
/// rather than a silent fallback (consistent with the #190 config-parsing direction).
fn parse_baseline_id(raw: &str) -> Result<Option<u64>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match trimmed.parse::<u64>() {
        Ok(id) => Ok(Some(id)),
        Err(_) => Err(crate::DomainError::with_hint(
            crate::ErrorKind::Validation,
            format!(
                "invalid [audit] resolution_baseline_id {trimmed:?} in .tickets/config.toml: expected a numeric ticket id"
            ),
            "set it to a ticket id like 36, or remove the key to disable the cutoff".to_string(),
        )
        .into()),
    }
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

#[cfg(test)]
mod tests {
    use super::parse_baseline_id;

    #[test]
    fn baseline_empty_is_none() {
        assert_eq!(parse_baseline_id("").unwrap(), None);
        assert_eq!(parse_baseline_id("   ").unwrap(), None);
    }

    #[test]
    fn baseline_numeric_parses() {
        assert_eq!(parse_baseline_id("36").unwrap(), Some(36));
        assert_eq!(parse_baseline_id(" 36 ").unwrap(), Some(36));
    }

    #[test]
    fn baseline_malformed_errors() {
        let err = parse_baseline_id("thirty-six").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("resolution_baseline_id"),
            "error should name the config key: {msg}"
        );
    }
}

//! `tkt audit` — batch closure quality check.

use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::core::{self, Status, Ticket};
use crate::findings::{self, Finding};
use crate::git;
use crate::commands::common::tickets_dir;

static RE_UNCHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());
static RE_CHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[x\]").unwrap());

pub fn run(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let mut audit_findings: Vec<Finding> = Vec::new();

    let frontier_ids: std::collections::HashSet<&str> = core::frontier(&corpus)
        .iter()
        .map(|t| t.id.as_str())
        .collect();

    for t in &corpus {
        let fname = t.path.file_name().unwrap().to_string_lossy().to_string();

        if t.status == Status::Done {
            let (unchecked, checked) = count_ac_boxes(&t.body);
            if unchecked > 0 && checked == 0 {
                audit_findings.push(Finding {
                    file: fname.clone(),
                    rule: "all-acs-unchecked-on-done".into(),
                    message: format!("{} unchecked box(es), none checked", unchecked),
                    severity: "warning".into(),
                });
            }

            if t.body.contains("## Resolution") {
                let has_content = t
                    .body
                    .split_once("## Resolution")
                    .map(|(_, after)| {
                        let text = after.lines().skip(1).collect::<Vec<_>>().join("\n");
                        let trimmed = text.trim();
                        !trimmed.is_empty() && trimmed != "TBD"
                    })
                    .unwrap_or(false);
                if !has_content {
                    audit_findings.push(Finding {
                        file: fname.clone(),
                        rule: "tbd-resolution".into(),
                        message: "resolution is empty or still TBD".into(),
                        severity: "warning".into(),
                    });
                }
            }

            if !t.body.contains("## Resolution") {
                audit_findings.push(Finding {
                    file: fname.clone(),
                    rule: "missing-resolution".into(),
                    message: "done ticket has no Resolution section".into(),
                    severity: "warning".into(),
                });
            }
        }

        if t.status == Status::InProgress {
            let rel_path = t
                .path
                .strip_prefix(&dir)
                .map(|p| format!(".tickets/{}", p.display()))
                .unwrap_or_default();
            if let Ok(ts_str) = git::git(
                dir.parent().unwrap_or(&dir),
                &["log", "-1", "--format=%ct", "--", &rel_path],
            ) {
                if let Ok(ts) = ts_str.trim().parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now > ts && (now - ts) > 7 * 24 * 60 * 60 {
                        let days = (now - ts) / (24 * 60 * 60);
                        audit_findings.push(Finding {
                            file: fname.clone(),
                            rule: "stale-wip".into(),
                            message: format!("in_progress for {} days (last commit)", days),
                            severity: "info".into(),
                        });
                    }
                }
            }
        }

        if t.status == Status::Open && t.is_high_priority() && frontier_ids.contains(t.id.as_str())
        {
            audit_findings.push(Finding {
                file: fname,
                rule: "high-priority-open".into(),
                message: "high-priority ticket still open".into(),
                severity: "info".into(),
            });
        }
    }

    let status = findings::status_from_findings(&audit_findings, strict);
    findings::print_findings(&audit_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

fn count_ac_boxes(body: &str) -> (usize, usize) {
    let section = match core::ac_section_range(body) {
        Some(range) => &body[range],
        None => return (0, 0),
    };
    let unchecked = RE_UNCHECKED_AC.find_iter(section).count();
    let checked = RE_CHECKED_AC.find_iter(section).count();
    (unchecked, checked)
}

//! `tkt sync-plan` — drift-check ticket status vs a plan table.

use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::commands::common::{domain_bail, is_dry_run, tickets_dir};
use crate::core::{self, Status, Ticket};
use crate::findings::{self, Finding};
use crate::git;

static RE_PLAN_ROW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\|\s*(\d+)\s*\|[^|]*\|([^|]*)\|\s*$").unwrap());

pub fn run(
    check: bool,
    fix: bool,
    strict: bool,
    brief: bool,
    plan_path: Option<&str>,
) -> Result<i32> {
    let dry_run = is_dry_run();
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let plan = match plan_path {
        Some(p) => PathBuf::from(p),
        None => repo.join("docs").join("plan.md"),
    };
    if !plan.is_file() {
        domain_bail!("no plan file at {}", plan.display());
    }

    let corpus = core::load_corpus(&dir)?;
    let corpus_map: std::collections::HashMap<&str, &Ticket> =
        corpus.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut plan_text = std::fs::read_to_string(&plan)?;

    let mut findings: Vec<Finding> = Vec::new();
    let mut fixed_count = 0;

    for caps in RE_PLAN_ROW.captures_iter(&plan_text.clone()) {
        let tid = caps[1].trim();
        let status_cell = &caps[2];
        let plan_done = status_cell.contains("✅");

        match corpus_map.get(tid) {
            Some(t) => {
                let ticket_done = t.status == Status::Done;
                if plan_done != ticket_done {
                    if fix && !dry_run {
                        let new_status = if ticket_done { " ✅ done " } else { " open " };
                        let row_re = Regex::new(&format!(
                            r"(?m)^(\|\s*{}\s*\|[^|]*\|)[^|]*(\|\s*)$",
                            regex::escape(tid)
                        ))
                        .unwrap();
                        plan_text = row_re
                            .replace(&plan_text, format!("${{1}}{}${{2}}", new_status))
                            .to_string();
                        fixed_count += 1;
                    } else {
                        // Derivable/cosmetic drift: the status cell disagrees with the
                        // ticket, but `--fix` can regenerate it. Advisory (warning), not
                        // blocking — escalated to a failure only under --check/--strict.
                        findings.push(Finding {
                            file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                            rule: "plan-status-drift".into(),
                            message: format!(
                                "plan says {}, ticket is {} (run: tkt sync-plan --fix)",
                                if plan_done { "done" } else { "not done" },
                                t.status.as_str()
                            ),
                            severity: "warning".into(),
                        });
                    }
                }
            }
            None => {
                // Non-derivable conflict: the plan references an id no ticket has.
                // `--fix` cannot resolve this — it is a genuine error regardless of mode.
                findings.push(Finding {
                    file: plan.file_name().unwrap().to_string_lossy().to_string(),
                    rule: "plan-orphan-row".into(),
                    message: format!("plan row references id {} with no matching ticket", tid),
                    severity: "error".into(),
                });
            }
        }
    }

    let plan_ids: std::collections::HashSet<String> = RE_PLAN_ROW
        .captures_iter(&plan_text)
        .map(|c| c[1].trim().to_string())
        .collect();
    for t in &corpus {
        if t.status != Status::Done && !plan_ids.contains(&*t.id) {
            findings.push(Finding {
                file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                rule: "missing-plan-row".into(),
                message: format!("{} ticket has no plan row", t.status.as_str()),
                severity: "warning".into(),
            });
        }
    }

    if fix && !dry_run && fixed_count > 0 {
        std::fs::write(&plan, &plan_text)?;
    }

    let errors: Vec<&Finding> = findings.iter().filter(|f| f.severity == "error").collect();
    let warnings: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.severity == "warning")
        .collect();
    // Advisory by default: derivable drift (warnings) does NOT fail the run.
    // Escalate to failure when a real conflict exists (error), or when the caller
    // opts into a gate via --check (CI) or --strict.
    let gate = check || strict;
    let status = if !errors.is_empty() || (gate && !warnings.is_empty()) {
        "fail"
    } else {
        "pass"
    };

    if fix {
        let would = if dry_run { "would fix" } else { "fixed" };
        if !findings.is_empty() {
            findings::print_findings(&findings, brief, status);
        } else if brief {
            println!("pass ({} {}, 0 remaining)", would, fixed_count);
        } else {
            println!(
                "{{\"status\":\"pass\",\"findings\":[],\"{}\":{}}}",
                if dry_run { "would_fix" } else { "fixed" },
                fixed_count
            );
        }
    } else {
        findings::print_findings(&findings, brief, status);
    }
    Ok(if status == "fail" { 1 } else { 0 })
}

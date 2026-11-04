//! `tkt sync-plan` — drift-check ticket status vs a plan table.

use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet_json, print_quiet_json_success, tickets_dir,
};
use crate::core::{self, Status, Ticket};
use crate::findings::{self, Finding};
use crate::git;

static RE_PLAN_ROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\|\s*([A-Za-z0-9][A-Za-z0-9_-]*)\s*\|[^|]*\|([^|]*)\|\s*$").unwrap()
});

fn plan_rows(text: &str) -> impl Iterator<Item = regex::Captures<'_>> {
    RE_PLAN_ROW
        .captures_iter(text)
        .filter(|caps| !is_header(caps))
}

fn is_header(caps: &regex::Captures<'_>) -> bool {
    caps[1].trim().eq_ignore_ascii_case("id") && caps[2].trim().eq_ignore_ascii_case("status")
}

fn parse_plan_status(cell: &str) -> Option<Status> {
    let cell = cell.trim();
    if cell == "✅ done" {
        Some(Status::Done)
    } else if cell.starts_with("cancelled (") && cell.ends_with(')') {
        Some(Status::Cancelled)
    } else {
        Status::parse(cell).ok()
    }
}

pub(super) fn plan_status_value(ticket: &Ticket) -> String {
    match ticket.status {
        Status::Done => "✅ done".to_string(),
        Status::Cancelled => format!(
            "cancelled ({})",
            core::cancellation_record(&ticket.body)
                .and_then(|record| record.reason)
                .as_deref()
                .unwrap_or("reason missing")
        ),
        status => status.as_str().to_string(),
    }
}

pub(crate) fn update_status(repo: &std::path::Path, id: &str, status: Status) -> Result<bool> {
    let value = if status == Status::Done {
        "✅ done".to_string()
    } else {
        status.as_str().to_string()
    };
    update_status_cell(repo, id, &value)
}

pub(crate) fn update_cancellation(repo: &std::path::Path, id: &str, reason: &str) -> Result<bool> {
    update_status_cell(repo, id, &format!("cancelled ({reason})"))
}

fn update_status_cell(repo: &std::path::Path, id: &str, value: &str) -> Result<bool> {
    let path = repo.join("docs/plan.md");
    if !path.is_file() {
        return Ok(false);
    }
    if !git::git(
        repo,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--",
            "docs/plan.md",
        ],
    )?
    .is_empty()
    {
        domain_bail!(Conflict, "docs/plan.md has uncommitted changes; commit or stash them before changing ticket status");
    }
    let text = std::fs::read_to_string(&path)?;
    let updated = rewrite_status_cell(&text, id, value);
    if updated == text {
        return Ok(false);
    }
    crate::effects::FilePlan(vec![crate::effects::FileEffect::ReplaceSnapshot {
        path,
        before: text,
        text: updated,
    }])
    .execute(crate::effects::Mode::current())?;
    Ok(true)
}

pub(crate) fn rewrite_status_cell(text: &str, id: &str, value: &str) -> String {
    RE_PLAN_ROW
        .replace_all(text, |caps: &regex::Captures| {
            if is_header(caps) || caps[1].trim() != id {
                return caps[0].to_string();
            }
            let cell = caps.get(2).unwrap();
            let row = caps.get(0).unwrap();
            let start = cell.start() - row.start();
            let end = cell.end() - row.start();
            format!("{} {} {}", &caps[0][..start], value, &caps[0][end..])
        })
        .into_owned()
}

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
    let original_plan = plan_text.clone();

    let mut findings: Vec<Finding> = Vec::new();
    let mut fixed_count = 0;

    for caps in plan_rows(&plan_text.clone()) {
        let tid = caps[1].trim();
        let status_cell = &caps[2];
        let plan_status = parse_plan_status(status_cell);

        match corpus_map.get(tid) {
            Some(t) => {
                let cancellation_value = format!(
                    "cancelled ({})",
                    core::cancellation_record(&t.body)
                        .and_then(|record| record.reason)
                        .as_deref()
                        .unwrap_or("reason missing")
                );
                let status_disagrees = plan_status != Some(t.status)
                    || (t.status == Status::Cancelled && status_cell.trim() != cancellation_value);
                if status_disagrees {
                    if fix {
                        let new_status = match t.status {
                            Status::Done => " ✅ done ".to_string(),
                            Status::Cancelled => format!(" {} ", cancellation_value),
                            _ => format!(" {} ", t.status.as_str()),
                        };
                        let row_re = Regex::new(&format!(
                            r"(?m)^(\|\s*{}\s*\|[^|]*\|)[^|]*(\|\s*)$",
                            regex::escape(tid)
                        ))
                        .unwrap();
                        plan_text = row_re
                            .replace(&plan_text, format!("${{1}}{}${{2}}", new_status))
                            .to_string();
                        fixed_count += 1;
                    }
                    if !fix || dry_run {
                        // Derivable/cosmetic drift: the status cell disagrees with the
                        // ticket, but `--fix` can regenerate it. Advisory (warning), not
                        // blocking — escalated to a failure only under --check/--strict.
                        findings.push(Finding {
                            file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                            rule: if plan_status.is_none() {
                                "plan-status-invalid"
                            } else {
                                "plan-status-drift"
                            }
                            .into(),
                            message: format!(
                                "plan says {}, ticket is {} (run: tkt sync-plan --fix)",
                                status_cell.trim(),
                                if t.status == Status::Cancelled {
                                    cancellation_value.as_str()
                                } else {
                                    t.status.as_str()
                                }
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

    let plan_ids: std::collections::HashSet<String> = plan_rows(&plan_text)
        .map(|c| c[1].trim().to_string())
        .collect();
    for t in &corpus {
        if t.status != Status::Done && !plan_ids.contains(&*t.id) {
            if fix {
                if !plan_text.contains("## Unplanned tickets\n") {
                    plan_text.push_str(
                        "\n## Unplanned tickets\n\n| ID | Work | Status |\n|---:|---|---|\n",
                    );
                }
                plan_text.push_str(&format!(
                    "| {} | {} | {} |\n",
                    t.id,
                    t.title.replace('|', "&#124;").replace(['\n', '\r'], " "),
                    plan_status_value(t)
                ));
                fixed_count += 1;
                if !dry_run {
                    continue;
                }
            }
            findings.push(Finding {
                file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                rule: "missing-plan-row".into(),
                message: format!("{} ticket has no plan row", plan_status_value(t)),
                severity: "warning".into(),
            });
        }
    }

    if fix && fixed_count > 0 {
        // Route through atomic_write (temp-file + rename) so an interrupted
        // write never leaves a torn plan file — same guarantee ticket writes
        // get (AGENTS.md constraint). Was std::fs::write (defect, #183).
        crate::effects::FilePlan(vec![crate::effects::FileEffect::ReplaceSnapshot {
            path: plan.clone(),
            before: original_plan,
            text: plan_text.clone(),
        }])
        .execute(crate::effects::Mode::current())?;
    }
    if fix && dry_run && fixed_count == 0 {
        crate::effects::FilePlan(vec![]).execute(crate::effects::Mode::Preview)?;
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

    if fix && status == "pass" && is_quiet_json() {
        print_quiet_json_success(&format!(
            "{} plan row(s)",
            if dry_run {
                format!("would reconcile {}", warnings.len())
            } else {
                format!("fixed {}", fixed_count)
            }
        ));
        return Ok(0);
    }

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

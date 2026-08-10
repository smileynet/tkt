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

//! `tkt close` — mark a ticket as done, append resolution.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, print_success, slug_from_filename,
};
use crate::core::{self, AcSelection, Status};
use crate::mutation::MutationContext;

pub fn run(
    id: &str,
    note: Option<&str>,
    ac_indices: &[u32],
    check_all: bool,
    force: bool,
    evidence: &[String],
) -> Result<i32> {
    let ctx = MutationContext::open()?;
    let t = ctx.find_ticket(id)?;

    if force && !ctx.config.close_allow_force {
        return Err(crate::DomainError::with_hint(
            crate::ErrorKind::GateFailed,
            "--force is disabled by project config (close.allow_force = false)".to_string(),
            "remove --force, or set close.allow_force = true in .tickets/config.toml".to_string(),
        )
        .into());
    }

    if let Some(remote_status) = ctx.remote_status(t) {
        if remote_status == "done" {
            domain_bail!(AlreadyDone, "{} is already done (updated on remote)", id);
        }
    }
    if t.status == Status::Done {
        domain_bail!(AlreadyDone, "{} is already done", t.id);
    }

    // --- Evidence / validation_criteria pairing ---
    let criteria = &t.validation_criteria;
    let evidence_map = if !evidence.is_empty() && !criteria.is_empty() {
        Some(parse_evidence(evidence, criteria.len())?)
    } else {
        None
    };

    // --- Aggregated close gates ---
    // Collect every unmet requirement up front and report them together, so a
    // caller missing several inputs fixes them in ONE retry instead of being
    // drip-fed one rejection per round-trip. Each entry pairs a human reason
    // with the exact flag(s) that satisfy it.
    if !force {
        let ac_stats = t.file.ac_stats();
        let mut unmet: Vec<String> = Vec::new();
        let mut remedies: Vec<&str> = Vec::new();

        if ctx.config.close_require_resolution && note.is_none() {
            unmet.push("no resolution (--resolution / --note)".to_string());
            remedies.push("--resolution \"...\"");
        }
        if ctx.config.close_require_validation_criteria && criteria.is_empty() {
            unmet.push(
                "ticket has no validation_criteria (add via tkt edit --validation)".to_string(),
            );
            remedies.push("tkt edit <id> --validation \"...\"");
        }
        if ctx.config.close_require_validation_evidence == "true"
            && !criteria.is_empty()
            && evidence.is_empty()
        {
            unmet.push(format!(
                "{} validation criteria present but no --evidence provided",
                criteria.len()
            ));
            remedies.push("--evidence \"...\"");
        }
        if ctx.config.close_require_checked_acs
            && ac_stats.total > 0
            && ac_stats.unchecked == ac_stats.total
            && ac_indices.is_empty()
            && !check_all
        {
            unmet.push(format!(
                "all {} acceptance criteria unchecked",
                ac_stats.total
            ));
            remedies.push("--check-all (or --ac N,N)");
        }

        if !unmet.is_empty() {
            let hint = format!(
                "supply: {} — or use --force to close anyway",
                remedies.join(", ")
            );
            return Err(crate::DomainError::with_hint(
                crate::ErrorKind::GateFailed,
                format!(
                    "close blocked by {} unmet gate(s): {}",
                    unmet.len(),
                    unmet.join("; ")
                ),
                hint,
            )
            .into());
        }
    }

    // Non-blocking advisory: evidence recommended but config is "warn", not "true".
    if ctx.config.close_require_validation_evidence == "warn"
        && !criteria.is_empty()
        && evidence.is_empty()
        && !force
    {
        eprintln!(
            "  {} {} validation criteria present but no --evidence provided",
            crate::color::sym_warn(),
            criteria.len()
        );
    }

    // Partial evidence gate: evidence provided but doesn't cover all criteria
    if !criteria.is_empty() && !evidence.is_empty() && evidence.len() < criteria.len() && !force {
        let gap = criteria.len() - evidence.len();
        match ctx.config.close_require_validation_evidence.as_str() {
            "true" => {
                return Err(crate::DomainError::with_hint(
                    crate::ErrorKind::GateFailed,
                    format!(
                        "{} evidence items provided for {} criteria ({} missing) — provide evidence for all criteria or use --force",
                        evidence.len(),
                        criteria.len(),
                        gap
                    ),
                    "provide --evidence for every criterion (positional or N=text), or --force to close anyway".to_string(),
                )
                .into());
            }
            "warn" => {
                eprintln!(
                    "  {} {} evidence items for {} criteria ({} unevidenced)",
                    crate::color::sym_warn(),
                    evidence.len(),
                    criteria.len(),
                    gap
                );
            }
            _ => {}
        }
    }

    // Dry-run: show what would happen
    if is_dry_run() {
        println!(
            "Would close {} {} (→ done)",
            t.id,
            slug_from_filename(&t.path)
        );
        if let Some(n) = note {
            println!("  Resolution: {}", n);
        }
        if let Some(ref emap) = evidence_map {
            println!("  Verification: {} criteria with evidence", emap.len());
        }
        // Compute what would be unblocked
        let mut corpus_clone = ctx.corpus.clone();
        if let Some(ticket) = corpus_clone.iter_mut().find(|x| x.id == t.id) {
            ticket.status = Status::Done;
        }
        let unblocked: Vec<_> = core::frontier(&corpus_clone)
            .into_iter()
            .filter(|x| !core::frontier(&ctx.corpus).iter().any(|f| f.id == x.id))
            .collect();
        if !unblocked.is_empty() {
            let items: Vec<String> = unblocked
                .iter()
                .map(|x| format!("{} {}", x.id, x.title))
                .collect();
            println!("  Would unblock: {}", items.join(", "));
        }
        println!("  Would commit and push");
        return Ok(0);
    }

    let mut file = t.file.clone();

    file.set_status(Status::Done);

    let resolution = note.unwrap_or("TBD");
    let spike_branch = crate::git::current_branch(&ctx.repo)
        .ok()
        .filter(|b| b.starts_with("spike/"));

    // Build resolution text with evidence if available
    let full_resolution = if let Some(ref emap) = evidence_map {
        let mut parts = vec![resolution.to_string()];
        parts.push(String::new());
        parts.push("### Verification".to_string());
        for (i, criterion) in criteria.iter().enumerate() {
            let ev = emap.get(i).map(|s| s.as_str()).unwrap_or("(no evidence)");
            parts.push(format!("{}. ✓ {} — \"{}\"", i + 1, criterion, ev));
        }
        parts.join("\n")
    } else {
        resolution.to_string()
    };

    file.append_resolution(&chrono_date(), &full_resolution, spike_branch.as_deref());

    let after_stats = if check_all {
        file.check_acs(AcSelection::All)
    } else if !ac_indices.is_empty() {
        file.check_acs(AcSelection::Indices(ac_indices))
    } else {
        file.ac_stats()
    };

    file.write()?;

    let rel_path = ctx.rel_path(&file.path);
    ctx.publish(&[&rel_path], &format!("chore(tickets): close {}", id))?;

    if !is_quiet() {
        let verb = if note.is_some() {
            "Resolution written"
        } else {
            "Resolution stub appended"
        };
        print_success("closed", &t.id, &slug_from_filename(&file.path), verb);
        if after_stats.total > 0 {
            println!(
                "  acceptance criteria: {}/{} checked{}",
                after_stats.checked,
                after_stats.total,
                if after_stats.unchecked > 0 {
                    format!(
                        " {} {} unchecked",
                        crate::color::sym_warn(),
                        after_stats.unchecked
                    )
                } else {
                    format!(" {}", crate::color::sym_ok())
                }
            );
        }

        let pre_frontier: std::collections::HashSet<String> = core::frontier(&ctx.corpus)
            .iter()
            .map(|t| t.id.clone())
            .collect();
        match core::load_corpus(&ctx.tickets_dir) {
            Ok(new_corpus) => {
                let post_frontier: Vec<&core::Ticket> = core::frontier(&new_corpus)
                    .into_iter()
                    .filter(|t| !pre_frontier.contains(&t.id))
                    .collect();
                if !post_frontier.is_empty() {
                    let items: Vec<String> = post_frontier
                        .iter()
                        .map(|t| format!("{} {}", t.id, t.title))
                        .collect();
                    println!(
                        "  {} unblocked: {}",
                        crate::color::sym_arrow(),
                        items.join(", ")
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "  {} could not compute unblocked tickets: {}",
                    crate::color::sym_warn(),
                    e
                );
            }
        }
    }

    Ok(0)
}

/// Pure-Rust ISO 8601 date (YYYY-MM-DD) without external dependencies.
fn chrono_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let days = secs.div_euclid(86400) as i32;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Parse evidence strings into a vector indexed by criterion position.
/// Supports positional (fills in order) and named (`N=text`, 1-based index).
fn parse_evidence(evidence: &[String], criteria_count: usize) -> Result<Vec<String>> {
    let mut result: Vec<Option<String>> = vec![None; criteria_count];
    let mut positional_idx = 0;

    for item in evidence {
        // Check for named format: starts with digit(s) followed by '='
        if let Some(eq_pos) = item.find('=') {
            let prefix = &item[..eq_pos];
            if let Ok(n) = prefix.parse::<usize>() {
                if n == 0 || n > criteria_count {
                    domain_bail!(
                        GateFailed,
                        "--evidence {}={}: criterion {} does not exist (ticket has {})",
                        n,
                        &item[eq_pos + 1..],
                        n,
                        criteria_count
                    );
                }
                if result[n - 1].is_some() {
                    domain_bail!(
                        GateFailed,
                        "--evidence: criterion {} has duplicate evidence (index {} used twice)",
                        n,
                        n
                    );
                }
                result[n - 1] = Some(item[eq_pos + 1..].to_string());
                continue;
            }
        }
        // Positional: assign to next unfilled slot
        while positional_idx < criteria_count && result[positional_idx].is_some() {
            positional_idx += 1;
        }
        if positional_idx >= criteria_count {
            domain_bail!(
                GateFailed,
                "--evidence: more evidence items than validation criteria ({})",
                criteria_count
            );
        }
        result[positional_idx] = Some(item.clone());
        positional_idx += 1;
    }

    // Verify all slots are filled — unfilled slots mean missing evidence
    let unfilled: Vec<usize> = result
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_none())
        .map(|(i, _)| i + 1)
        .collect();
    if !unfilled.is_empty() {
        domain_bail!(
            GateFailed,
            "--evidence: criteria {} have no evidence (provide positional or named evidence for each)",
            unfilled
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(result
        .into_iter()
        .map(|opt| opt.unwrap_or_default())
        .collect())
}

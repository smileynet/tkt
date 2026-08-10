//! `tkt close` — mark a ticket as done, append resolution.

use anyhow::Result;

use crate::commands::common::{domain_bail, is_quiet, slug_from_filename, success_msg};
use crate::core::{self, AcSelection, Status};
use crate::mutation::MutationContext;

pub fn run(
    id: &str,
    note: Option<&str>,
    ac_indices: &[u32],
    check_all: bool,
    force: bool,
) -> Result<i32> {
    let ctx = MutationContext::open()?;
    let t = ctx.find_ticket(id)?;

    if ctx.config.close_require_resolution && note.is_none() && !force {
        domain_bail!("project config requires --resolution (or --note) to close a ticket");
    }

    if let Some(remote_status) = ctx.remote_status(t) {
        if remote_status == "done" {
            domain_bail!("{} is already done (updated on remote)", id);
        }
    }
    if t.status == Status::Done {
        domain_bail!("{} is already done", t.id);
    }

    let mut file = t.file.clone();
    let before_stats = file.ac_stats();

    if ctx.config.close_require_checked_acs
        && before_stats.total > 0
        && before_stats.unchecked == before_stats.total
        && ac_indices.is_empty()
        && !check_all
        && !force
    {
        domain_bail!(
            "all {} acceptance criteria are unchecked — check at least one with --ac, use --check-all, or use --force to close anyway",
            before_stats.total
        );
    }

    file.set_status(Status::Done);

    let resolution = note.unwrap_or("TBD");
    let spike_branch = crate::git::current_branch(&ctx.repo)
        .ok()
        .filter(|b| b.starts_with("spike/"));
    file.append_resolution(&chrono_date(), resolution, spike_branch.as_deref());

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
        println!(
            "{}",
            success_msg("closed", &t.id, &slug_from_filename(&file.path), verb)
        );
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

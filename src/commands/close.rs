//! `tkt close` — mark a ticket as done, append resolution.

use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::commands::common::{domain_bail, is_quiet, slug_from_filename, success_msg};
use crate::core::{self, Status};
use crate::mutation::MutationContext;

static RE_UNCHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());
static RE_CHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[x\]").unwrap());

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

    let (unchecked_before, checked_before) = count_ac_boxes(&t.body);
    let total_acs = unchecked_before + checked_before;

    if ctx.config.close_require_checked_acs
        && total_acs > 0
        && unchecked_before == total_acs
        && ac_indices.is_empty()
        && !check_all
        && !force
    {
        domain_bail!(
            "all {} acceptance criteria are unchecked — check at least one with --ac, use --check-all, or use --force to close anyway",
            total_acs
        );
    }

    let mut file = t.file.clone();
    file.set_field("status", "done");

    if !file.body.contains("## Resolution") {
        let date = chrono_date();
        let resolution = note.unwrap_or("TBD");

        let branch_note = crate::git::current_branch(&ctx.repo)
            .ok()
            .filter(|b| b.starts_with("spike/"))
            .map(|b| format!("\n\nSpike branch: {}", b))
            .unwrap_or_default();

        file.body = format!(
            "{}\n\n## Resolution ({})\n\n{}{}\n",
            file.body.trim_end(),
            date,
            resolution,
            branch_note
        );
    }

    if check_all {
        if let Some(range) = core::ac_section_range(&file.body) {
            let section = file.body[range.clone()].replace("- [ ]", "- [x]");
            file.body.replace_range(range, &section);
        }
    } else if !ac_indices.is_empty() {
        file.body = flip_ac_boxes(&file.body, ac_indices);
    }

    file.write()?;

    let (unchecked_after, _) = count_ac_boxes(&file.body);
    let checked_after = total_acs.saturating_sub(unchecked_after);

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
        if total_acs > 0 {
            println!(
                "  acceptance criteria: {}/{} checked{}",
                checked_after,
                total_acs,
                if unchecked_after > 0 {
                    format!(
                        " {} {} unchecked",
                        crate::color::sym_warn(),
                        unchecked_after
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

fn count_ac_boxes(body: &str) -> (usize, usize) {
    let section = match core::ac_section_range(body) {
        Some(range) => &body[range],
        None => return (0, 0),
    };
    let unchecked = RE_UNCHECKED_AC.find_iter(section).count();
    let checked = RE_CHECKED_AC.find_iter(section).count();
    (unchecked, checked)
}

fn flip_ac_boxes(body: &str, indices: &[u32]) -> String {
    let mut result = body.to_string();
    let range = match core::ac_section_range(body) {
        Some(r) => r,
        None => return result,
    };
    let section = &body[range.clone()];
    let matches: Vec<_> = RE_UNCHECKED_AC.find_iter(section).collect();

    for &idx in indices.iter().rev() {
        let i = (idx as usize).saturating_sub(1);
        if i < matches.len() {
            let m = &matches[i];
            let abs_start = range.start + m.start();
            let abs_end = range.start + m.end();
            result.replace_range(abs_start..abs_end, "- [x]");
        }
    }
    result
}

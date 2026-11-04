//! Transfer one unchecked body criterion; publish both links in one commit.

use anyhow::Result;

use super::common::{domain_bail, is_dry_run, is_quiet, print_success};
use crate::core::{ac_transfer, Status};
use crate::mutation::MutationContext;

pub fn run(id: &str, index: usize, to: &str) -> Result<i32> {
    if id == to || index == 0 {
        domain_bail!("source and destination must differ; --ac is 1-based");
    }
    let ctx = MutationContext::open()?;
    let source = ctx.find_ticket(id)?;
    let destination = ctx.find_ticket(to)?;
    for ticket in [source, destination] {
        if let Some(status) = ctx
            .remote_status(ticket)
            .filter(|status| status != ticket.status.as_str())
        {
            domain_bail!(
                Conflict,
                "ticket {} has changed remotely ({})",
                ticket.id,
                status
            );
        }
    }
    let mut src = source.file.clone();
    let mut dst = destination.file.clone();
    let list = ac_transfer::criteria(&src);
    let Some(criterion) = list.get(index - 1) else {
        domain_bail!("no acceptance criterion {} on ticket {}", index, id)
    };
    if src.body[criterion.range.end..]
        .trim_start_matches('\r')
        .strip_prefix('\n')
        .and_then(|tail| tail.lines().next())
        .is_some_and(|line| {
            !line.trim().is_empty() && (line.starts_with("  ") || line.starts_with('\t'))
        })
    {
        domain_bail!(
            "multiline acceptance criteria must be split into single-line criteria before transfer"
        );
    }
    let suffix = format!(" <!-- tkt-from:{id}:{index} -->");
    let moved_suffix = format!(" <!-- tkt-moved:{to} -->");
    let recovering = criterion.state == ">";
    let text = if recovering {
        criterion.text.strip_suffix(&moved_suffix).ok_or_else(|| {
            crate::DomainError::new(
                crate::ErrorKind::Conflict,
                "criterion was moved to a different destination".into(),
            )
        })?
    } else {
        if criterion.state != " "
            || criterion.text.trim().is_empty()
            || criterion.text.contains("<!-- tkt-")
        {
            domain_bail!("only an unchecked, previously untransferred criterion can be moved");
        }
        if matches!(source.status, Status::Done | Status::Cancelled) {
            domain_bail!("cannot transfer from a terminal ticket");
        }
        criterion.text
    };
    let expected = format!("{text}{suffix}");
    let receiving = ac_transfer::criteria(&dst);
    let matching: Vec<_> = receiving
        .iter()
        .filter(|ac| ac.text.ends_with(&suffix))
        .collect();
    if matching.len() > 1
        || matching
            .first()
            .is_some_and(|ac| ac.text != expected || ac.state == ">")
    {
        domain_bail!(
            Conflict,
            "receiving criterion changed; inspect both tickets before retrying"
        );
    }
    if recovering && matching.is_empty() {
        domain_bail!(Conflict, "moved source has no matching receiving criterion");
    }
    if !recovering && matches!(destination.status, Status::Done | Status::Cancelled) {
        domain_bail!("destination must be nonterminal");
    }
    if matching.is_empty() {
        let line = format!("- [ ] {expected}\n");
        if let Some(range) = ac_transfer::section_range(&dst.body) {
            dst.body.insert_str(range.end, &format!("\n{line}\n"));
        } else {
            dst.body = format!(
                "{}\n\n## Acceptance criteria\n\n{line}",
                dst.body.trim_end()
            );
        }
    }
    let replacement = format!("- [>] {text}{moved_suffix}");
    let range = criterion.range.clone();
    src.body.replace_range(range, &replacement);
    let changed = dst.serialize() != destination.file.serialize()
        || src.serialize() != source.file.serialize();
    // Destination first: an interrupted source write leaves the original unchecked
    // and a deterministic back-reference lets retry finish without appending twice.
    if dst.serialize() != destination.file.serialize() {
        dst.write()?;
    }
    if src.serialize() != source.file.serialize() {
        src.write()?;
    }
    let paths = [ctx.rel_path(&src.path), ctx.rel_path(&dst.path)];
    let refs: Vec<_> = paths.iter().map(String::as_str).collect();
    let dirty = crate::git::git(
        &ctx.repo,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--",
            &paths[0],
            &paths[1],
        ],
    )?;
    if !dirty.trim().is_empty() || (is_dry_run() && changed) {
        ctx.publish(
            &refs,
            &format!("chore(tickets): move AC {id}:{index} to {to}"),
            None,
            None,
        )?;
    }
    if is_dry_run() {
        print_success(
            "would move AC",
            id,
            "",
            &format!("{index} to {to}; validation criteria/evidence stay unchanged"),
        );
        return Ok(0);
    }
    if !is_quiet() || super::common::is_json_output() {
        print_success(
            "moved AC",
            id,
            "",
            &format!("{index} to {to}; validation criteria/evidence unchanged"),
        );
    }
    Ok(0)
}

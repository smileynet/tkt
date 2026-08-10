//! `tkt edit` — surgical field corrections.

use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::commands::common::{
    check_remote_status, commit_and_publish, domain_bail, is_quiet, preflight_mutation,
    slug_from_filename, success_msg,
};
use crate::core::{self, Status};

static RE_UNCHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());

#[allow(clippy::too_many_arguments)]
pub fn run(
    id: &str,
    title: Option<&str>,
    blocked_by: Option<&str>,
    env: Option<&str>,
    spec: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    ac_indices: &[u32],
) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status == "done" {
            domain_bail!("ticket {} was closed on remote", id);
        }
    }

    let mut file = t.file.clone();
    let mut changed: Vec<&str> = Vec::new();

    if let Some(title_val) = title {
        if title_val.is_empty() {
            domain_bail!("title is required and cannot be cleared");
        }
        if let Err(e) = core::validate::validate_free_text(title_val, "title", 200) {
            domain_bail!("{}", e);
        }
        file.set_field(
            "title",
            &format!("\"{}\"", core::yaml_scalar_escape(title_val)),
        );
        changed.push("title");
    }
    if let Some(deps_str) = blocked_by {
        let deps: Vec<&str> = deps_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for dep in &deps {
            if let Err(e) = core::validate::validate_id(dep) {
                domain_bail!("--blocked-by: {}", e);
            }
        }
        if let Err(e) = core::validate::validate_no_self_dep(id, &deps) {
            domain_bail!("{}", e);
        }
        let formatted = deps
            .iter()
            .map(|d| format!("\"{}\"", core::yaml_scalar_escape(d)))
            .collect::<Vec<_>>()
            .join(", ");
        file.set_field("blocked_by", &format!("[{}]", formatted));
        changed.push("blocked_by");
    }
    if let Some(env_val) = env {
        if env_val.is_empty() {
            file.remove_field("env");
        } else {
            if !core::ENV_VALUES.contains(&env_val) {
                domain_bail!(
                    "env must be one of {} (or '' to clear)",
                    core::ENV_VALUES.join("/")
                );
            }
            file.set_field("env", env_val);
        }
        changed.push("env");
    }
    if let Some(spec_val) = spec {
        if spec_val.is_empty() {
            file.remove_field("spec");
        } else {
            if let Err(e) = core::validate::validate_free_text(spec_val, "spec", 100) {
                domain_bail!("{}", e);
            }
            file.set_field(
                "spec",
                &format!("\"{}\"", core::yaml_scalar_escape(spec_val)),
            );
        }
        changed.push("spec");
    }
    if let Some(prio_val) = priority {
        if prio_val.is_empty() {
            file.remove_field("priority");
        } else {
            if let Err(e) = core::validate::validate_priority(prio_val) {
                domain_bail!("{} (or '' to clear)", e);
            }
            file.set_field("priority", prio_val);
        }
        changed.push("priority");
    }
    if let Some(status_val) = status {
        if status_val.is_empty() {
            domain_bail!(
                "status cannot be cleared — use a valid value (backlog/open/in_progress/done)"
            );
        }
        if Status::parse(status_val).is_err() {
            domain_bail!(
                "status must be one of {} (got {:?})",
                core::STATUS_VALUES.join("/"),
                status_val
            );
        }
        file.set_field("status", status_val);
        changed.push("status");
    }
    if !ac_indices.is_empty() {
        file.body = flip_ac_boxes(&file.body, ac_indices);
        changed.push("ac");
    }

    if changed.is_empty() {
        domain_bail!("nothing to edit — pass at least one field option");
    }

    file.write()?;
    let rel_path = file
        .path
        .strip_prefix(&repo)
        .unwrap_or(&file.path)
        .to_string_lossy()
        .replace('\\', "/");
    commit_and_publish(
        &repo,
        remote,
        &[&rel_path],
        &format!("chore(tickets): edit {} ({})", id, changed.join(", ")),
    )?;
    if !is_quiet() {
        println!(
            "{}",
            success_msg(
                "edited",
                id,
                &slug_from_filename(&file.path),
                &changed.join(", ")
            )
        );
    }
    Ok(0)
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

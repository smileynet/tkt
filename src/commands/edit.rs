//! `tkt edit` — surgical field corrections.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_quiet, print_success, slug_from_filename,
};
use crate::core::{self, AcSelection, Env, Priority, Status};
use crate::mutation::MutationContext;

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
    validation_criteria: Option<&[String]>,
) -> Result<i32> {
    let ctx = MutationContext::open()?;
    let t = ctx.find_ticket(id)?;

    if let Some(remote_status) = ctx.remote_status(t) {
        if remote_status == "done" {
            domain_bail!(AlreadyDone, "ticket {} was closed on remote", id);
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
        file.set_blocked_by(&deps);
        changed.push("blocked_by");
    }
    if let Some(env_val) = env {
        if env_val.is_empty() {
            file.set_env(None);
        } else {
            let parsed = Env::parse(env_val).map_err(|_| {
                crate::DomainError::new(
                    crate::ErrorKind::Validation,
                    format!(
                        "env must be one of {} (or '' to clear)",
                        core::ENV_VALUES.join("/")
                    ),
                )
            })?;
            file.set_env(Some(parsed));
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
            file.set_priority(None);
        } else {
            let parsed = Priority::parse(prio_val).ok_or_else(|| {
                crate::DomainError::new(
                    crate::ErrorKind::Validation,
                    "priority must be one of urgent/high/medium/low (or '' to clear)".to_string(),
                )
            })?;
            file.set_priority(Some(parsed));
        }
        changed.push("priority");
    }
    if let Some(status_val) = status {
        if status_val.is_empty() {
            domain_bail!(
                "status cannot be cleared — use a valid value (backlog/open/in_progress/done)"
            );
        }
        let parsed = Status::parse(status_val).map_err(|_| {
            crate::DomainError::new(
                crate::ErrorKind::Validation,
                format!(
                    "status must be one of {} (got {:?})",
                    core::STATUS_VALUES.join("/"),
                    status_val
                ),
            )
        })?;
        file.set_status(parsed);
        changed.push("status");
    }
    if !ac_indices.is_empty() {
        file.check_acs(AcSelection::Indices(ac_indices));
        changed.push("ac");
    }
    if let Some(vc) = validation_criteria {
        file.set_validation_criteria(vc);
        changed.push("validation_criteria");
    }

    if changed.is_empty() {
        domain_bail!("nothing to edit — pass at least one field option");
    }

    if is_dry_run() {
        println!(
            "Would edit {} {} ({})",
            id,
            slug_from_filename(&file.path),
            changed.join(", ")
        );
        return Ok(0);
    }

    file.write()?;
    let rel_path = ctx.rel_path(&file.path);
    ctx.publish(
        &[&rel_path],
        &format!("chore(tickets): edit {} ({})", id, changed.join(", ")),
    )?;
    if !is_quiet() {
        print_success(
            "edited",
            id,
            &slug_from_filename(&file.path),
            &changed.join(", "),
        );
    }
    Ok(0)
}

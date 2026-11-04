//! `tkt edit` — surgical field corrections.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, is_dry_run, is_json_output, is_quiet, is_quiet_json, print_quiet_json_success,
    print_success, slug_from_filename,
};
use crate::core::{self, AcSelection, Env, Priority, Status};
use crate::mutation::MutationContext;

#[allow(clippy::too_many_arguments)]
pub fn run(
    id: &str,
    title: Option<&str>,
    tags: Option<&[String]>,
    add_tags: &[String],
    remove_tags: &[String],
    blocked_by: Option<&str>,
    parent: Option<&str>,
    env: Option<&str>,
    spec: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
    ac_indices: &[u32],
    validation_criteria: Option<&[String]>,
    agents: Option<&[String]>,
    exclude_agents: Option<&[String]>,
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
    if let Some(tags) = edit_tags(&t.tags, tags, add_tags, remove_tags) {
        if tags.is_empty() {
            file.remove_field("tags");
        } else {
            let values = tags
                .iter()
                .map(|tag| format!("\"{}\"", core::yaml_scalar_escape(tag)))
                .collect::<Vec<_>>()
                .join(", ");
            file.set_field("tags", &format!("[{}]", values));
        }
        changed.push("tags");
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
    if let Some(parent) = parent {
        if parent.is_empty() {
            file.remove_parent();
        } else {
            validate_parent_assignment(parent, id, &ctx.corpus)?;
            file.set_parent(parent);
        }
        changed.push("parent");
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
        if t.status == Status::Cancelled {
            domain_bail!(
                Conflict,
                "{} is cancelled; use tkt reopen {} to return it to open",
                t.id,
                t.id
            );
        }
        if status_val == "done" || status_val == "cancelled" {
            let (message, hint) = if status_val == "done" {
                (
                    "status done can only be set by close",
                    "use tkt close <id> so resolution, acceptance, and evidence gates run",
                )
            } else {
                (
                    "status cancelled can only be set by cancel",
                    "use tkt cancel <id> to record a reason and handle dependent edges",
                )
            };
            return Err(crate::DomainError::with_hint(
                crate::ErrorKind::Validation,
                message.to_string(),
                hint.to_string(),
            )
            .into());
        }
        if status_val.is_empty() {
            domain_bail!("status cannot be cleared — use a valid value (backlog/open/in_progress)");
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
    if let Some(values) = agents {
        if values == [String::new()] {
            file.set_agent_eligibility(&core::AgentEligibility::Any);
        } else {
            let policy = core::AgentEligibility::allow(values).map_err(|error| {
                crate::DomainError::new(crate::ErrorKind::Validation, error.to_string())
            })?;
            file.set_agent_eligibility(&policy);
        }
        changed.push("agents");
    }
    if let Some(values) = exclude_agents {
        if values == [String::new()] {
            file.set_agent_eligibility(&core::AgentEligibility::Any);
        } else {
            let policy = core::AgentEligibility::exclude(values).map_err(|error| {
                crate::DomainError::new(crate::ErrorKind::Validation, error.to_string())
            })?;
            file.set_agent_eligibility(&policy);
        }
        changed.push("agents");
    }

    if changed.is_empty() {
        domain_bail!("nothing to edit — pass at least one field option");
    }

    let plan_changed = if changed.contains(&"status") {
        super::sync_plan::update_status(&ctx.repo, id, Status::parse(file.get("status").unwrap())?)?
    } else {
        false
    };
    file.write()?;
    let rel_path = ctx.rel_path(&file.path);
    let mut paths = vec![rel_path.as_str()];
    if plan_changed {
        paths.push("docs/plan.md");
    }
    ctx.publish(
        &paths,
        &format!("chore(tickets): edit {} ({})", id, changed.join(", ")),
        None,
        None,
    )?;

    if is_dry_run() {
        if is_quiet_json() {
            print_quiet_json_success(&format!("would edit {} ({})", id, changed.join(", ")));
        } else {
            println!(
                "Would edit {} {} ({})",
                id,
                slug_from_filename(&file.path),
                changed.join(", ")
            );
        }
        return Ok(0);
    }
    if !is_quiet() || is_json_output() {
        print_success(
            "edited",
            id,
            &slug_from_filename(&file.path),
            &changed.join(", "),
        );
    }
    Ok(0)
}

fn edit_tags(
    current: &[String],
    replacement: Option<&[String]>,
    add: &[String],
    remove: &[String],
) -> Option<Vec<String>> {
    if replacement.is_none() && add.is_empty() && remove.is_empty() {
        return None;
    }
    let mut tags: Vec<String> = replacement
        .unwrap_or(current)
        .iter()
        .filter(|tag| !tag.is_empty())
        .cloned()
        .collect();
    for tag in add.iter().filter(|tag| !tag.is_empty()) {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags.retain(|tag| {
        !remove
            .iter()
            .any(|removed| !removed.is_empty() && removed == tag)
    });
    Some(tags)
}

fn validate_parent_assignment(parent: &str, own_id: &str, corpus: &[core::Ticket]) -> Result<()> {
    if let Err(error) = core::validate::validate_id(parent) {
        domain_bail!("--parent: {}", error);
    }
    if let Err(error) = core::validate::validate_parent_ref(own_id, parent) {
        domain_bail!("{}", error);
    }
    if !corpus.iter().any(|ticket| ticket.id == parent) {
        domain_bail!(NotFound, "parent ticket {} not found", parent);
    }

    let existing = crate::findings::check_parent_cycles(corpus);
    let mut prospective = corpus.to_vec();
    let ticket = prospective
        .iter_mut()
        .find(|ticket| ticket.id == own_id)
        .expect("the edited ticket is present in the mutation context");
    ticket.parent = Some(parent.to_string());
    if let Some(cycle) = crate::findings::check_parent_cycles(&prospective)
        .into_iter()
        .find(|finding| {
            finding.rule == "parent-cycle"
                && !existing
                    .iter()
                    .any(|old| old.rule == finding.rule && old.message == finding.message)
        })
    {
        domain_bail!(Validation, "{}", cycle.message);
    }
    Ok(())
}

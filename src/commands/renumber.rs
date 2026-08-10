//! `tkt renumber` — move a ticket to a new ID atomically.

use anyhow::Result;

use crate::commands::common::{
    domain_bail, has_remote, is_quiet, project_config, success_msg, tickets_dir,
};
use crate::core::{self, validate, Ticket};
use crate::git;

pub fn run(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    if let Err(e) = validate::validate_id(new_id) {
        domain_bail!("new id: {}", e);
    }

    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    let repo = git::repo_root(&dir)?;
    let corpus = core::load_corpus(&dir)?;

    let holders: Vec<&Ticket> = corpus.iter().filter(|t| t.id == old_id).collect();
    if holders.is_empty() {
        domain_bail!("no ticket with id {:?}", old_id);
    }
    if holders.len() > 1 && file_hint.is_none() {
        let names: Vec<_> = holders
            .iter()
            .map(|t| t.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        domain_bail!(
            "id {:?} is held by {} files ({}) — pass --file",
            old_id,
            holders.len(),
            names.join(", ")
        );
    }

    let src = if holders.len() == 1 {
        holders[0]
    } else {
        holders
            .iter()
            .find(|t| t.path.file_name().unwrap().to_string_lossy() == file_hint.unwrap())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--file {:?} does not hold id {:?}",
                    file_hint.unwrap(),
                    old_id
                )
            })?
    };

    if corpus.iter().any(|t| t.id == new_id) {
        domain_bail!("id {:?} already exists locally", new_id);
    }

    let old_path = src.path.clone();
    let slug = old_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .split_once('-')
        .map(|x| x.1)
        .unwrap_or("unknown.md")
        .to_string();
    let new_path = dir.join(format!("{}-{}", new_id, slug));

    let mut file = src.file.clone();
    let old_raw = file.get("id").unwrap_or("");
    let new_val = if old_raw.starts_with('"') {
        format!("\"{}\"", new_id)
    } else {
        new_id.to_string()
    };
    file.set_field("id", &new_val);
    file.path = new_path.clone();
    file.write()?;
    std::fs::remove_file(&old_path)?;

    let mut refs_updated = 0;
    if holders.len() == 1 {
        for other in &corpus {
            if other.path == old_path {
                continue;
            }
            if other.blocked_by.contains(&old_id.to_string()) {
                let mut other_file = other.file.clone();
                let new_deps: Vec<String> = other
                    .blocked_by
                    .iter()
                    .map(|d| {
                        if d == old_id {
                            new_id.to_string()
                        } else {
                            d.clone()
                        }
                    })
                    .collect();
                let formatted = new_deps
                    .iter()
                    .map(|d| format!("\"{}\"", core::yaml_scalar_escape(d)))
                    .collect::<Vec<_>>()
                    .join(", ");
                other_file.set_field("blocked_by", &format!("[{}]", formatted));
                other_file.write()?;
                refs_updated += 1;
            }
        }
    }

    let old_rel = old_path
        .strip_prefix(&repo)
        .unwrap_or(&old_path)
        .to_string_lossy()
        .replace('\\', "/");
    let new_rel = new_path
        .strip_prefix(&repo)
        .unwrap_or(&new_path)
        .to_string_lossy()
        .replace('\\', "/");
    git::git(&repo, &["add", &old_rel, &new_rel])?;
    for other in &corpus {
        if other.path == old_path {
            continue;
        }
        if other.blocked_by.contains(&old_id.to_string()) {
            let rel = other
                .path
                .strip_prefix(&repo)
                .unwrap_or(&other.path)
                .to_string_lossy()
                .replace('\\', "/");
            git::add(&repo, &[&rel])?;
        }
    }
    git::commit(
        &repo,
        &format!("chore(tickets): renumber {} -> {}", old_id, new_id),
    )?;
    if has_remote(&repo) && pcfg.push_enabled {
        git::push_with_retry(&repo)?;
    }

    let detail = if refs_updated > 0 {
        format!("{} refs updated", refs_updated)
    } else {
        String::new()
    };
    if !is_quiet() {
        println!(
            "{}",
            success_msg("renumbered", old_id, &format!("→ {}", new_id), &detail)
        );
    }
    Ok(0)
}

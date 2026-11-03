//! `tkt config` — manage user/project configuration.

use anyhow::Result;

use crate::commands::common::{
    is_dry_run, is_quiet, is_quiet_json, print_quiet_json_success, project_config, tickets_dir,
};

pub fn run(
    set: Option<&str>,
    get: Option<&str>,
    unset: Option<&str>,
    project: bool,
    list: bool,
    show: bool,
) -> Result<i32> {
    if project && set.is_none() && unset.is_none() {
        return Err(crate::DomainError::new(
            crate::ErrorKind::Validation,
            "--project is valid only with --set or --unset".to_string(),
        )
        .into());
    }

    if let Some(pair) = set {
        let (key, value) = pair.split_once('=').ok_or_else(|| {
            crate::DomainError::new(
                crate::ErrorKind::Validation,
                format!("expected key=value format, got {:?}", pair),
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if is_dry_run() {
            if is_quiet_json() {
                print_quiet_json_success(&format!("would set {}", key));
            } else {
                println!(
                    "Would set {} = {:?} ({})",
                    key,
                    value,
                    if project { "project" } else { "user" }
                );
            }
            return Ok(0);
        }
        if project {
            let dir = tickets_dir()?;
            warn_if_detached_project_write(&dir);
            crate::config::set_project_config(&dir, key, value)?;
        } else {
            crate::config::Config::set(key, value)?;
        }
        if is_quiet() {
            print_quiet_json_success(&format!("set {}", key));
        } else {
            println!("{} {} = {:?}", crate::color::sym_ok(), key, value);
        }
        return Ok(0);
    }

    if let Some(key) = get {
        let cfg = crate::config::Config::load();
        println!("{}", cfg.get(key));
        return Ok(0);
    }

    if let Some(key) = unset {
        if is_dry_run() {
            if is_quiet_json() {
                print_quiet_json_success(&format!("would unset {}", key));
            } else {
                println!(
                    "Would unset {:?} ({})",
                    key,
                    if project { "project" } else { "user" }
                );
            }
            return Ok(0);
        }
        let existed = if project {
            let dir = tickets_dir()?;
            warn_if_detached_project_write(&dir);
            crate::config::unset_project_config(&dir, key)?
        } else {
            crate::config::Config::unset(key)?
        };
        if is_quiet() {
            print_quiet_json_success(&format!(
                "{} {}",
                if existed { "unset" } else { "unchanged" },
                key
            ));
        } else {
            if existed {
                println!(
                    "{} unset {:?} (reverted to default)",
                    crate::color::sym_ok(),
                    key
                );
            } else {
                println!("(no value was set for {:?})", key);
            }
        }
        return Ok(0);
    }

    if show {
        let dir = tickets_dir()?;
        let pcfg = project_config(&dir)?;
        let config_path = dir.join("config.toml");
        let has_file = config_path.is_file();

        println!("# Project config: .tickets/config.toml");
        if has_file {
            println!("# Source: {}", config_path.display());
        } else {
            println!("# (no config file — all defaults)");
        }
        println!();
        for entry in pcfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
        return Ok(0);
    }

    if list {
        let cfg = crate::config::Config::load();
        for entry in cfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
        return Ok(0);
    }

    let cfg = crate::config::Config::load();
    let path = crate::config::config_file_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    println!("# User config ({})", path);
    for entry in cfg.list() {
        println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
    }
    if let Ok(dir) = tickets_dir() {
        let pcfg = project_config(&dir)?;
        println!();
        println!("# Project config (.tickets/config.toml)");
        for entry in pcfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
    }
    Ok(0)
}

fn warn_if_detached_project_write(tickets_dir: &std::path::Path) {
    let repo = tickets_dir.parent().unwrap_or(tickets_dir);
    if crate::git::current_branch(repo).is_ok_and(|branch| branch.is_empty()) {
        eprintln!(
            "warning: writing .tickets/config.toml from detached HEAD; the change stays in this worktree and is not committed automatically"
        );
    }
}

//! `tkt config` — manage user/project configuration.

use anyhow::Result;

use crate::commands::common::{is_quiet, project_config, tickets_dir};

pub fn run(
    set: Option<&str>,
    get: Option<&str>,
    unset: Option<&str>,
    list: bool,
    show: bool,
) -> Result<i32> {
    if let Some(pair) = set {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected key=value format, got {:?}", pair))?;
        let key = key.trim();
        let value = value.trim();
        crate::config::Config::set(key, value)?;
        if !is_quiet() {
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
        let existed = crate::config::Config::unset(key)?;
        if !is_quiet() {
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
        let pcfg = project_config(&dir);
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
    println!("# User config (~/.config/tkt/config.toml)");
    for entry in cfg.list() {
        println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
    }
    if let Ok(dir) = tickets_dir() {
        let pcfg = project_config(&dir);
        println!();
        println!("# Project config (.tickets/config.toml)");
        for entry in pcfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
    }
    Ok(0)
}

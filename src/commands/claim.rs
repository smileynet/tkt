//! `tkt claim` — mark an open ticket as in_progress.

use anyhow::Result;

use crate::core::{self, Status};
use crate::commands::common::{
    check_remote_status, commit_and_publish, domain_bail, is_quiet, preflight_mutation,
    slug_from_filename, success_msg,
};

pub fn run(id: &str) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status != "open" {
            domain_bail!("{} is {}, not open (updated on remote)", id, remote_status);
        }
    }
    if t.status != Status::Open {
        domain_bail!("{} is {}, not open", t.id, t.status.as_str());
    }

    let mut file = t.file.clone();
    file.set_field("status", "in_progress");
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
        &format!("chore(tickets): claim {}", id),
    )?;

    if !is_quiet() {
        println!(
            "{}",
            success_msg(
                "claimed",
                &t.id,
                &slug_from_filename(&file.path),
                &format!("{} in_progress", crate::color::sym_arrow())
            )
        );
    }
    Ok(0)
}

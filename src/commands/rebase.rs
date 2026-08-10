//! `tkt rebase` — resolve ID collisions with upstream.

use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::commands::common::{is_quiet, tickets_dir};
use crate::core;
use crate::git;

static RE_TICKET_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)-(.+)\.md$").unwrap());

pub fn run(dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = dir.parent().unwrap().to_path_buf();

    // Step 1: fetch origin
    git::fetch(&repo)?;

    // Step 2: get remote IDs
    let remote_names = git::remote_ticket_names(&repo);
    let remote_ids: std::collections::HashSet<String> = remote_names
        .iter()
        .filter_map(|n| RE_TICKET_FILENAME.captures(n).map(|c| c[1].to_string()))
        .collect();

    // Step 3: get local ticket files and identify collisions
    // A collision = local file has an ID that also exists on remote, but the file is NOT on remote
    // (different slug means it's a different ticket claiming the same ID)
    let local_names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let remote_name_set: std::collections::HashSet<&str> =
        remote_names.iter().map(|s| s.as_str()).collect();

    let mut collisions: Vec<(String, String, PathBuf)> = Vec::new(); // (old_id, slug, path)
    for name in &local_names {
        if let Some(caps) = RE_TICKET_FILENAME.captures(name) {
            let id = caps[1].to_string();
            let slug = caps[2].to_string();
            // Collision: same ID exists on remote but with a DIFFERENT filename
            if remote_ids.contains(&id) && !remote_name_set.contains(name.as_str()) {
                collisions.push((id, slug, dir.join(name)));
            }
        }
    }

    if collisions.is_empty() {
        if !is_quiet() {
            println!("No ID collisions with upstream.");
        }
        return Ok(0);
    }

    // Step 4: compute the renumber plan — assign next available IDs
    // Combine remote + local IDs to find the true max
    let all_names: Vec<String> = local_names
        .iter()
        .chain(remote_names.iter())
        .cloned()
        .collect();
    let width = core::id_width(&all_names);

    // Sort collisions by old ID for deterministic ordering
    collisions.sort_by(|a, b| a.0.cmp(&b.0));

    // Build renumber map: old_id → new_id
    let base_id = core::max_id(&all_names) + 1;
    let mut renumber_map: Vec<(String, String)> = Vec::new();
    for (i, (old_id, _slug, _path)) in collisions.iter().enumerate() {
        let new_id = format!("{:0>width$}", base_id + i as u64, width = width);
        renumber_map.push((old_id.clone(), new_id));
    }

    // Step 5: dry-run report
    if dry_run {
        println!("Collisions detected ({}):", renumber_map.len());
        for ((old_id, slug, _), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
            println!("  {} → {} ({})", old_id, new_id, slug);
        }
        println!("\nRun without --dry-run to apply.");
        return Ok(0);
    }

    // Step 6: perform the renumber
    // 6a: rename files and update frontmatter id
    let mut renamed_paths: Vec<String> = Vec::new();
    for ((old_id, slug, old_path), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
        let new_filename = format!("{}-{}.md", new_id, slug);
        let new_path = dir.join(&new_filename);

        // Rename file
        std::fs::rename(old_path, &new_path)?;

        // Update frontmatter id field
        let mut file = core::TicketFile::parse(&new_path)?;
        file.set_field("id", &format!("\"{}\"", new_id));
        file.write()?;

        renamed_paths.push(format!(".tickets/{}-{}.md", old_id, slug));
        renamed_paths.push(format!(".tickets/{}", new_filename));
    }

    // 6b: update blocked_by references across the ENTIRE corpus
    let id_map: std::collections::HashMap<&str, &str> = renumber_map
        .iter()
        .map(|(old, new)| (old.as_str(), new.as_str()))
        .collect();

    let updated_corpus: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let mut refs_updated = 0;
    let mut modified_paths: Vec<String> = Vec::new();
    for name in &updated_corpus {
        let path = dir.join(name);
        let mut file = core::TicketFile::parse(&path)?;
        if let Some(deps_raw) = file.get("blocked_by") {
            let mut changed = false;
            let mut new_deps: Vec<String> = Vec::new();
            // Parse the blocked_by array
            for dep in deps_raw
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
            {
                if let Some(&new_id) = id_map.get(dep.as_str()) {
                    new_deps.push(new_id.to_string());
                    changed = true;
                } else {
                    new_deps.push(dep);
                }
            }
            if changed {
                let formatted = new_deps
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", ");
                file.set_field("blocked_by", &format!("[{}]", formatted));
                file.write()?;
                modified_paths.push(format!(".tickets/{}", name));
                refs_updated += 1;
            }
        }
    }

    // Step 7: commit atomically — only stage files we changed
    let mut all_paths = renamed_paths.clone();
    all_paths.extend(modified_paths);
    let add_paths: Vec<&str> = all_paths.iter().map(|s| s.as_str()).collect();
    git::add(&repo, &add_paths)?;
    let msg = format!(
        "chore(tickets): rebase — renumber {} ticket(s) to resolve ID collision",
        renumber_map.len()
    );
    git::commit(&repo, &msg)?;

    // Report
    if !is_quiet() {
        println!("Renumbered {} ticket(s):", renumber_map.len());
        for ((old_id, slug, _), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
            println!("  {} → {} ({})", old_id, new_id, slug);
        }
        if refs_updated > 0 {
            println!("  {} blocked_by reference(s) updated", refs_updated);
        }
    }
    Ok(0)
}

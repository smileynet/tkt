//! Pure domain logic for ID remapping (renumber/rebase planning + application).
//!
//! Planning is pure (no I/O). Application touches the filesystem but is
//! decoupled from git — callers handle staging and publishing.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

use crate::core::{self, Ticket};

static RE_TICKET_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)-(.+)\.md$").unwrap());

// --- Plan types ---

/// A single ID movement: old_id → new_id for a given slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenumberEntry {
    pub old_id: String,
    pub new_id: String,
    pub slug: String,
}

/// A plan for one or more ID movements. Pure data — no side effects.
#[derive(Debug, Clone)]
pub struct RenumberPlan {
    pub entries: Vec<RenumberEntry>,
}

/// A fully preflighted renumber operation. Every file to rename or rewrite is
/// identified and rendered before `apply_renumber` starts mutating the filesystem.
#[derive(Debug, Clone)]
pub struct MaterializedRenumberPlan {
    writes: Vec<PlannedWrite>,
    staged_paths: Vec<String>,
    refs_updated: usize,
}

#[derive(Debug, Clone)]
struct PlannedWrite {
    source: PathBuf,
    target: PathBuf,
    contents: String,
}

impl RenumberPlan {
    /// Plan renumbers to resolve ID collisions between local and remote filenames.
    ///
    /// A collision = a local file has an ID that also exists on remote, but the
    /// filename is different (i.e., a different ticket claims the same ID).
    /// New IDs are assigned sequentially starting from max(all IDs) + 1.
    pub fn for_collisions(local_names: &[String], remote_names: &[String]) -> Self {
        let remote_ids: HashSet<String> = remote_names
            .iter()
            .filter_map(|n| RE_TICKET_FILENAME.captures(n).map(|c| c[1].to_string()))
            .collect();

        let remote_name_set: HashSet<&str> = remote_names.iter().map(|s| s.as_str()).collect();

        let mut collisions: Vec<(String, String)> = Vec::new(); // (id, slug)
        for name in local_names {
            if let Some(caps) = RE_TICKET_FILENAME.captures(name) {
                let id = caps[1].to_string();
                let slug = caps[2].to_string();
                if remote_ids.contains(&id) && !remote_name_set.contains(name.as_str()) {
                    collisions.push((id, slug));
                }
            }
        }

        if collisions.is_empty() {
            return RenumberPlan {
                entries: Vec::new(),
            };
        }

        // Deterministic ordering
        collisions.sort();

        // Compute next available IDs
        let all_names: Vec<String> = local_names
            .iter()
            .chain(remote_names.iter())
            .cloned()
            .collect();
        let width = core::id_width(&all_names);
        let base_id = core::max_id(&all_names) + 1;

        let entries = collisions
            .into_iter()
            .enumerate()
            .map(|(i, (old_id, slug))| RenumberEntry {
                old_id,
                new_id: format!("{:0>width$}", base_id + i as u64, width = width),
                slug,
            })
            .collect();

        RenumberPlan { entries }
    }

    /// Plan a single-ticket renumber. Validates that old_id exists, new_id is free,
    /// and resolves file_hint ambiguity. Pure — only reads the corpus.
    pub fn single(
        corpus: &[Ticket],
        old_id: &str,
        new_id: &str,
        file_hint: Option<&str>,
    ) -> Result<Self> {
        let holders: Vec<&Ticket> = corpus.iter().filter(|t| t.id == old_id).collect();
        if holders.is_empty() {
            bail!("no ticket with id {:?}", old_id);
        }
        if holders.len() > 1 && file_hint.is_none() {
            let names: Vec<_> = holders
                .iter()
                .map(|t| t.path.file_name().unwrap().to_string_lossy().to_string())
                .collect();
            bail!(
                "id {:?} is held by {} files ({}) — pass --file",
                old_id,
                holders.len(),
                names.join(", ")
            );
        }
        if corpus.iter().any(|t| t.id == new_id) {
            bail!("id {:?} already exists locally", new_id);
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

        let slug = src
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .split_once('-')
            .map(|x| x.1.trim_end_matches(".md"))
            .unwrap_or("unknown")
            .to_string();

        Ok(RenumberPlan {
            entries: vec![RenumberEntry {
                old_id: old_id.to_string(),
                new_id: new_id.to_string(),
                slug,
            }],
        })
    }

    /// Whether this plan has any entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// --- Application ---

/// Result of applying a renumber plan to the filesystem.
pub struct AppliedRenumber {
    /// Relative paths (from repo root) that were modified — for git staging.
    pub staged_paths: Vec<String>,
    /// Number of blocked_by references rewritten across other tickets.
    pub refs_updated: usize,
}

/// Parse and render every affected ticket before a renumber begins.
///
/// `enforce_birth_window` is used by the manual `renumber` command: an ID that is
/// cited by another ticket cannot be moved. Collision recovery (`rebase`) supplies
/// `false`, because its purpose is specifically to repair a newly-created local ID
/// collision and rewrite its inbound dependency references.
pub fn materialize_renumber(
    dir: &Path,
    plan: &RenumberPlan,
    enforce_birth_window: bool,
) -> Result<MaterializedRenumberPlan> {
    let id_map: HashMap<&str, &str> = plan
        .entries
        .iter()
        .map(|e| (e.old_id.as_str(), e.new_id.as_str()))
        .collect();
    let source_names: HashSet<String> = plan
        .entries
        .iter()
        .map(|entry| format!("{}-{}.md", entry.old_id, entry.slug))
        .collect();

    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();

    // Parse the complete potentially affected corpus up front. Unlike load_corpus,
    // this intentionally does not skip a bad file: applying a partial plan would
    // violate the birth-window and files-are-the-database contracts.
    let tickets: Vec<(String, Ticket)> = names
        .iter()
        .map(|name| {
            let path = dir.join(name);
            Ticket::parse(&path).map(|ticket| (name.clone(), ticket))
        })
        .collect::<Result<_>>()?;

    for source_name in &source_names {
        if !names.contains(source_name) {
            bail!("renumber source {:?} does not exist", source_name);
        }
    }

    if enforce_birth_window {
        for (name, ticket) in &tickets {
            if source_names.contains(name) {
                continue;
            }
            for entry in &plan.entries {
                if ticket.blocked_by.iter().any(|id| id == &entry.old_id)
                    || body_cites_id(&ticket.body, &entry.old_id)
                {
                    bail!(
                        "cannot renumber {}: {} cites this id outside its birth window",
                        entry.old_id,
                        name
                    );
                }
            }
        }
    }

    let mut writes = Vec::new();
    let mut staged_paths = Vec::new();
    let mut refs_updated = 0;

    for (name, ticket) in tickets {
        let source = dir.join(&name);
        let mut target = source.clone();
        let mut file = ticket.file;
        let mut changed = false;

        if let Some(entry) = plan
            .entries
            .iter()
            .find(|entry| name == format!("{}-{}.md", entry.old_id, entry.slug))
        {
            target = dir.join(format!("{}-{}.md", entry.new_id, entry.slug));
            if target.exists() {
                bail!(
                    "renumber target {:?} already exists",
                    target.file_name().unwrap()
                );
            }
            let old_raw = file.get("id").unwrap_or("");
            let new_value = if old_raw.starts_with('"') {
                format!("\"{}\"", entry.new_id)
            } else {
                entry.new_id.clone()
            };
            file.set_field("id", &new_value);
            changed = true;
            staged_paths.push(format!(".tickets/{}", name));
            staged_paths.push(format!(
                ".tickets/{}",
                target.file_name().unwrap().to_string_lossy()
            ));
        }

        let rewritten_deps: Vec<String> = ticket
            .blocked_by
            .iter()
            .map(|id| {
                id_map
                    .get(id.as_str())
                    .copied()
                    .unwrap_or(id.as_str())
                    .to_string()
            })
            .collect();
        if rewritten_deps != ticket.blocked_by {
            file.set_blocked_by(&rewritten_deps);
            changed = true;
            refs_updated += 1;
        }

        if changed {
            if source == target {
                staged_paths.push(format!(".tickets/{}", name));
            }
            writes.push(PlannedWrite {
                source,
                target,
                contents: file.serialize(),
            });
        }
    }

    staged_paths.sort();
    staged_paths.dedup();

    Ok(MaterializedRenumberPlan {
        writes,
        staged_paths,
        refs_updated,
    })
}

/// Apply a fully materialized renumber plan. This function performs no corpus
/// discovery or parsing after mutation begins.
pub fn apply_renumber(plan: &MaterializedRenumberPlan) -> Result<AppliedRenumber> {
    // Move every source first. Targets were verified absent during preflight.
    for write in &plan.writes {
        if write.source != write.target {
            std::fs::rename(&write.source, &write.target)?;
        }
    }

    for write in &plan.writes {
        core::atomic_write(&write.target, &write.contents)?;
    }

    Ok(AppliedRenumber {
        staged_paths: plan.staged_paths.clone(),
        refs_updated: plan.refs_updated,
    })
}

fn body_cites_id(body: &str, id: &str) -> bool {
    body.match_indices(id).any(|(start, _)| {
        let before = body[..start].chars().last();
        let after = body[start + id.len()..].chars().next();
        !before.is_some_and(is_id_char) && !after.is_some_and(is_id_char)
    })
}

fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_')
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn for_collisions_no_collisions() {
        let local = vec!["01-auth.md".to_string(), "02-api.md".to_string()];
        let remote = vec!["01-auth.md".to_string(), "02-api.md".to_string()];
        let plan = RenumberPlan::for_collisions(&local, &remote);
        assert!(plan.is_empty());
    }

    #[test]
    fn for_collisions_same_files_no_collision() {
        let local = vec!["01-auth.md".to_string()];
        let remote = vec!["01-auth.md".to_string()];
        let plan = RenumberPlan::for_collisions(&local, &remote);
        assert!(plan.is_empty());
    }

    #[test]
    fn for_collisions_detects_id_clash() {
        let local = vec!["01-my-feature.md".to_string()];
        let remote = vec!["01-their-feature.md".to_string()];
        let plan = RenumberPlan::for_collisions(&local, &remote);
        assert_eq!(plan.entries.len(), 1);
        assert_eq!(plan.entries[0].old_id, "01");
        assert_eq!(plan.entries[0].new_id, "02");
        assert_eq!(plan.entries[0].slug, "my-feature");
    }

    #[test]
    fn for_collisions_multiple_clashes() {
        let local = vec![
            "01-local-a.md".to_string(),
            "02-local-b.md".to_string(),
            "03-shared.md".to_string(),
        ];
        let remote = vec![
            "01-remote-a.md".to_string(),
            "02-remote-b.md".to_string(),
            "03-shared.md".to_string(), // same file — not a collision
        ];
        let plan = RenumberPlan::for_collisions(&local, &remote);
        assert_eq!(plan.entries.len(), 2);
        // New IDs start from max(01,02,03)+1 = 04
        assert_eq!(plan.entries[0].old_id, "01");
        assert_eq!(plan.entries[0].new_id, "04");
        assert_eq!(plan.entries[1].old_id, "02");
        assert_eq!(plan.entries[1].new_id, "05");
    }

    #[test]
    fn for_collisions_preserves_id_width() {
        let local = vec!["001-local.md".to_string()];
        let remote = vec!["001-remote.md".to_string()];
        let plan = RenumberPlan::for_collisions(&local, &remote);
        assert_eq!(plan.entries[0].new_id, "002");
    }

    #[test]
    fn single_validates_old_id_exists() {
        let corpus = make_corpus(&[("01", "auth")]);
        let result = RenumberPlan::single(&corpus, "99", "02", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no ticket"));
    }

    #[test]
    fn single_validates_new_id_free() {
        let corpus = make_corpus(&[("01", "auth"), ("02", "api")]);
        let result = RenumberPlan::single(&corpus, "01", "02", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn single_produces_entry() {
        let corpus = make_corpus(&[("01", "auth"), ("02", "api")]);
        let plan = RenumberPlan::single(&corpus, "01", "05", None).unwrap();
        assert_eq!(plan.entries.len(), 1);
        assert_eq!(plan.entries[0].old_id, "01");
        assert_eq!(plan.entries[0].new_id, "05");
        assert_eq!(plan.entries[0].slug, "auth");
    }

    #[test]
    fn apply_renumber_renames_and_rewrites() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-auth.md"),
            "---\nid: \"01\"\ntitle: \"Auth\"\nstatus: open\nblocked_by: []\n---\n\n# Auth\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("02-api.md"),
            "---\nid: \"02\"\ntitle: \"API\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# API\n",
        )
        .unwrap();

        let plan = RenumberPlan {
            entries: vec![RenumberEntry {
                old_id: "01".to_string(),
                new_id: "05".to_string(),
                slug: "auth".to_string(),
            }],
        };

        let materialized = materialize_renumber(dir.path(), &plan, false).unwrap();
        let result = apply_renumber(&materialized).unwrap();
        assert_eq!(result.refs_updated, 1);
        assert!(!dir.path().join("01-auth.md").exists());
        assert!(dir.path().join("05-auth.md").exists());

        // Check id was rewritten
        let content = std::fs::read_to_string(dir.path().join("05-auth.md")).unwrap();
        assert!(content.contains("id: \"05\""));

        // Check blocked_by was rewritten
        let api_content = std::fs::read_to_string(dir.path().join("02-api.md")).unwrap();
        assert!(api_content.contains("\"05\""));
        assert!(!api_content.contains("\"01\""));
    }

    #[test]
    fn materialize_refuses_block_style_citation_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("01-auth.md");
        let citer = dir.path().join("02-api.md");
        std::fs::write(
            &source,
            "---\nid: \"01\"\ntitle: \"Auth\"\nstatus: open\nblocked_by: []\n---\n\n# Auth\n",
        )
        .unwrap();
        std::fs::write(
            &citer,
            "---\nid: \"02\"\ntitle: \"API\"\nstatus: open\nblocked_by:\n  - \"01\"\n---\n\n# API\n",
        )
        .unwrap();
        let before_source = std::fs::read_to_string(&source).unwrap();
        let before_citer = std::fs::read_to_string(&citer).unwrap();
        assert_eq!(Ticket::parse(&citer).unwrap().blocked_by, vec!["01"]);
        let plan = RenumberPlan {
            entries: vec![RenumberEntry {
                old_id: "01".to_string(),
                new_id: "05".to_string(),
                slug: "auth".to_string(),
            }],
        };

        let err = materialize_renumber(dir.path(), &plan, true).unwrap_err();
        assert!(err.to_string().contains("outside its birth window"));
        assert_eq!(std::fs::read_to_string(&source).unwrap(), before_source);
        assert_eq!(std::fs::read_to_string(&citer).unwrap(), before_citer);
        assert!(source.exists());
        assert!(!dir.path().join("05-auth.md").exists());
    }

    #[test]
    fn materialize_refuses_body_citation_and_malformed_corpus_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("01-auth.md");
        let prose = dir.path().join("02-api.md");
        std::fs::write(
            &source,
            "---\nid: \"01\"\ntitle: \"Auth\"\nstatus: open\nblocked_by: []\n---\n\n# Auth\n",
        )
        .unwrap();
        std::fs::write(
            &prose,
            "---\nid: \"02\"\ntitle: \"API\"\nstatus: open\nblocked_by: []\n---\n\n# API\n\nSee #01 first.\n",
        )
        .unwrap();
        let plan = RenumberPlan {
            entries: vec![RenumberEntry {
                old_id: "01".to_string(),
                new_id: "05".to_string(),
                slug: "auth".to_string(),
            }],
        };
        let before = std::fs::read_to_string(&source).unwrap();

        assert!(materialize_renumber(dir.path(), &plan, true).is_err());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), before);

        std::fs::remove_file(&prose).unwrap();
        let malformed = dir.path().join("02-broken.md");
        std::fs::write(&malformed, "not frontmatter\n").unwrap();
        assert!(materialize_renumber(dir.path(), &plan, true).is_err());
        assert_eq!(std::fs::read_to_string(&source).unwrap(), before);
        assert!(malformed.exists());
        assert!(!dir.path().join("05-auth.md").exists());
    }

    #[test]
    fn apply_uses_only_preflighted_writes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-auth.md"),
            "---\nid: \"01\"\ntitle: \"Auth\"\nstatus: open\nblocked_by: []\n---\n\n# Auth\n",
        )
        .unwrap();
        let plan = RenumberPlan {
            entries: vec![RenumberEntry {
                old_id: "01".to_string(),
                new_id: "05".to_string(),
                slug: "auth".to_string(),
            }],
        };
        let materialized = materialize_renumber(dir.path(), &plan, true).unwrap();

        // This file appears after preflight. Apply must not discover or parse it.
        let late_bad_file = dir.path().join("02-late-bad.md");
        std::fs::write(&late_bad_file, "not frontmatter\n").unwrap();
        let result = apply_renumber(&materialized).unwrap();

        assert_eq!(result.refs_updated, 0);
        assert!(!dir.path().join("01-auth.md").exists());
        assert!(dir.path().join("05-auth.md").exists());
        assert_eq!(
            std::fs::read_to_string(&late_bad_file).unwrap(),
            "not frontmatter\n"
        );
    }

    #[test]
    fn apply_renumber_multiple_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-a.md"),
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: []\n---\n\n# A\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("02-b.md"),
            "---\nid: \"02\"\ntitle: \"B\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# B\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("03-c.md"),
            "---\nid: \"03\"\ntitle: \"C\"\nstatus: open\nblocked_by: [\"01\", \"02\"]\n---\n\n# C\n",
        )
        .unwrap();

        let plan = RenumberPlan {
            entries: vec![
                RenumberEntry {
                    old_id: "01".to_string(),
                    new_id: "10".to_string(),
                    slug: "a".to_string(),
                },
                RenumberEntry {
                    old_id: "02".to_string(),
                    new_id: "11".to_string(),
                    slug: "b".to_string(),
                },
            ],
        };

        let materialized = materialize_renumber(dir.path(), &plan, false).unwrap();
        let result = apply_renumber(&materialized).unwrap();
        assert_eq!(result.refs_updated, 2); // 02-b refs 01, 03-c refs both

        // 03-c should have both refs updated
        let c_content = std::fs::read_to_string(dir.path().join("03-c.md")).unwrap();
        assert!(c_content.contains("\"10\""));
        assert!(c_content.contains("\"11\""));
        assert!(!c_content.contains("\"01\""));
        assert!(!c_content.contains("\"02\""));
    }

    // --- Test helpers ---

    fn make_corpus(tickets: &[(&str, &str)]) -> Vec<Ticket> {
        tickets
            .iter()
            .map(|(id, slug)| {
                let content = format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: open\nblocked_by: []\n---\n\n# {}\n",
                    id, slug, slug
                );
                let path = PathBuf::from(format!("{}-{}.md", id, slug));
                Ticket::parse_str(&content, &path).unwrap()
            })
            .collect()
    }
}

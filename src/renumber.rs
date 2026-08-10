//! Pure domain logic for ID remapping (renumber/rebase planning + application).
//!
//! Planning is pure (no I/O). Application touches the filesystem but is
//! decoupled from git — callers handle staging and publishing.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

use crate::core::{self, Ticket, TicketFile};

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

/// Apply a renumber plan to the filesystem. Renames files, rewrites `id` fields,
/// and updates all `blocked_by` references across the corpus.
///
/// Does NOT handle git staging/commit/push — caller is responsible.
pub fn apply_renumber(dir: &Path, plan: &RenumberPlan) -> Result<AppliedRenumber> {
    let mut staged_paths: Vec<String> = Vec::new();
    let mut refs_updated = 0;

    // Build the ID mapping for reference rewrites
    let id_map: HashMap<&str, &str> = plan
        .entries
        .iter()
        .map(|e| (e.old_id.as_str(), e.new_id.as_str()))
        .collect();

    // Phase 1: rename files and update id fields
    for entry in &plan.entries {
        let old_filename = format!("{}-{}.md", entry.old_id, entry.slug);
        let new_filename = format!("{}-{}.md", entry.new_id, entry.slug);
        let old_path = dir.join(&old_filename);
        let new_path = dir.join(&new_filename);

        // On Windows, fs::rename can't overwrite — delete first if exists
        if new_path.exists() {
            std::fs::remove_file(&new_path)?;
        }
        std::fs::rename(&old_path, &new_path)?;

        // Update the id field in frontmatter
        let mut file = TicketFile::parse(&new_path)?;
        let old_raw = file.get("id").unwrap_or("");
        let new_val = if old_raw.starts_with('"') {
            format!("\"{}\"", entry.new_id)
        } else {
            entry.new_id.clone()
        };
        file.set_field("id", &new_val);
        file.write()?;

        staged_paths.push(format!(".tickets/{}", old_filename));
        staged_paths.push(format!(".tickets/{}", new_filename));
    }

    // Phase 2: update blocked_by references across the entire corpus
    let corpus_names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    for name in &corpus_names {
        let path = dir.join(name);
        let mut file = TicketFile::parse(&path)?;
        if let Some(deps_raw) = file.get("blocked_by") {
            let deps: Vec<String> = deps_raw
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let mut changed = false;
            let new_deps: Vec<String> = deps
                .into_iter()
                .map(|d| {
                    if let Some(&new_id) = id_map.get(d.as_str()) {
                        changed = true;
                        new_id.to_string()
                    } else {
                        d
                    }
                })
                .collect();

            if changed {
                file.set_blocked_by(&new_deps);
                file.write()?;
                staged_paths.push(format!(".tickets/{}", name));
                refs_updated += 1;
            }
        }
    }

    Ok(AppliedRenumber {
        staged_paths,
        refs_updated,
    })
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

        let result = apply_renumber(dir.path(), &plan).unwrap();
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

        let result = apply_renumber(dir.path(), &plan).unwrap();
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

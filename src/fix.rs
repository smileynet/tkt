//! `tkt validate --fix` — guided auto-repair with safety guardrails.
//!
//! Tier 1 (mechanical, auto-apply): quoting, removing invalid optional fields.
//! Tier 2 (mapped, apply with warning): status mapping (closed→done).
//! Tier 3 (advisory, print guidance): ambiguous status, foreign schema, non-ticket files.

use std::path::Path;

use crate::core::{self, TicketFile, ENV_VALUES, STATUS_VALUES};

/// A single repair action.
#[derive(Debug)]
pub struct Repair {
    pub file: String,
    pub description: String,
    pub tier: u8,
}

/// An advisory (tier 3) — not auto-fixable.
#[derive(Debug)]
pub struct Advisory {
    pub file: String,
    pub message: String,
    pub suggestion: String,
}

/// Result of running fix on a directory.
pub struct FixResult {
    pub repairs: Vec<Repair>,
    pub advisories: Vec<Advisory>,
}

/// Known status mappings (tier 2).
fn map_status(s: &str) -> Option<&'static str> {
    match s {
        "closed" => Some("done"),
        "cancelled" | "canceled" => Some("done"),
        _ => None,
    }
}

/// Run the fix pass on a .tickets/ directory.
pub fn run_fix(dir: &Path, dry_run: bool) -> anyhow::Result<FixResult> {
    let mut repairs = Vec::new();
    let mut advisories = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    // Build the corpus index once for blocked_by ref resolution (#162).
    let all_names: Vec<String> = entries
        .iter()
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    let index = core::corpus_index(&all_names);

    for entry in &entries {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();

        // Try to parse as a TicketFile (raw frontmatter) — if this fails, it's tier 3
        let file = match TicketFile::parse_str(&std::fs::read_to_string(&path)?, &path) {
            Ok(f) => f,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("missing required field: id")
                    || msg.contains("missing required field: title")
                {
                    advisories.push(Advisory {
                        file: fname,
                        message: "not a tkt ticket (missing required fields)".into(),
                        suggestion: "move to docs/ or remove from .tickets/".into(),
                    });
                } else if msg.contains("missing required field") {
                    advisories.push(Advisory {
                        file: fname,
                        message: msg,
                        suggestion: "add the missing field manually".into(),
                    });
                } else {
                    advisories.push(Advisory {
                        file: fname,
                        message: format!("parse error: {}", msg),
                        suggestion: "inspect file manually".into(),
                    });
                }
                continue;
            }
        };

        let mut modified_file = file.clone();
        let mut file_repairs: Vec<Repair> = Vec::new();

        // --- Tier 1: Quote unquoted id ---
        if let Some(raw_id) = modified_file.get("id") {
            let trimmed = raw_id.trim().to_string();
            if !trimmed.starts_with('"') {
                let quoted = format!("\"{}\"", trimmed);
                modified_file.set_field("id", &quoted);
                file_repairs.push(Repair {
                    file: fname.clone(),
                    description: format!("quoted id {} → {}", trimmed, quoted),
                    tier: 1,
                });
            }
        }

        // --- Tier 1: Normalize blocked_by elements (quote + resolve refs #162) ---
        if let Some(raw_deps) = modified_file.get("blocked_by") {
            let trimmed = raw_deps.trim().to_string();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let inner = &trimmed[1..trimmed.len() - 1];
                if !inner.trim().is_empty() {
                    let elements: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                    let mut new_elements: Vec<String> = Vec::new();
                    let mut resolved_any = false;
                    let mut quoted_any = false;
                    for e in &elements {
                        let was_unquoted = !e.starts_with('"');
                        let clean = e.trim_matches('"');
                        // Resolve ref against the corpus (pad width / strip slug) when unique.
                        let canonical = match core::resolve_blocked_by_ref(clean, &index) {
                            Some(r) => {
                                resolved_any = true;
                                r
                            }
                            None => clean.to_string(),
                        };
                        if was_unquoted {
                            quoted_any = true;
                        }
                        new_elements.push(format!("\"{}\"", canonical));
                    }
                    if resolved_any || quoted_any {
                        let new_val = format!("[{}]", new_elements.join(", "));
                        modified_file.set_field("blocked_by", &new_val);
                        let desc = if resolved_any {
                            "normalized blocked_by refs (pad/slug)".to_string()
                        } else {
                            "quoted blocked_by elements".to_string()
                        };
                        file_repairs.push(Repair {
                            file: fname.clone(),
                            description: desc,
                            tier: 1,
                        });
                    }
                }
            }
        }

        // --- Tier 1: Remove invalid env ---
        if let Some(raw_env) = modified_file.get("env") {
            let val = raw_env.trim().trim_matches('"').to_string();
            if !ENV_VALUES.contains(&val.as_str()) {
                modified_file.remove_field("env");
                file_repairs.push(Repair {
                    file: fname.clone(),
                    description: format!("removed invalid env: {}", val),
                    tier: 1,
                });
            }
        }

        // --- Tier 1: Remove invalid priority ---
        if let Some(raw_prio) = modified_file.get("priority") {
            let val = raw_prio.trim().trim_matches('"').to_string();
            if core::Priority::parse(&val).is_none() {
                modified_file.remove_field("priority");
                file_repairs.push(Repair {
                    file: fname.clone(),
                    description: format!("removed invalid priority: {}", val),
                    tier: 1,
                });
            }
        }

        // --- Tier 2: Map known status values ---
        if let Some(raw_status) = modified_file.get("status") {
            let val = raw_status.trim().trim_matches('"').to_string();
            if !STATUS_VALUES.contains(&val.as_str()) {
                if let Some(mapped) = map_status(&val) {
                    modified_file.set_field("status", mapped);
                    file_repairs.push(Repair {
                        file: fname.clone(),
                        description: format!("status: {} → {} (mapped)", val, mapped),
                        tier: 2,
                    });
                } else {
                    // Tier 3: unknown status, advisory only
                    advisories.push(Advisory {
                        file: fname.clone(),
                        message: format!("unknown status: {:?}", val),
                        suggestion:
                            "did you mean backlog or open? Run: tkt edit <id> --status backlog"
                                .to_string(),
                    });
                }
            }
        }

        // --- Tier 3: done ticket with no resolution (likely hand-flipped) ---
        // A ticket closed via `tkt close` always has a `## Resolution` section.
        // Its absence on a done ticket signals a hand-edit that skipped the close
        // gates. Advise only — never fabricate a resolution (body is user-owned).
        let effective_status = modified_file
            .get("status")
            .map(|s| s.trim().trim_matches('"').to_string())
            .unwrap_or_default();
        if effective_status == "done" && !modified_file.body.contains("## Resolution") {
            advisories.push(Advisory {
                file: fname.clone(),
                message: "done ticket has no resolution recorded".into(),
                suggestion:
                    "record how it was resolved: tkt close <id> --force --resolution \"...\""
                        .into(),
            });
        }

        // Write if there are repairs and not dry-run
        if !file_repairs.is_empty() && !dry_run {
            modified_file.write()?;
        }
        repairs.extend(file_repairs);
    }

    Ok(FixResult {
        repairs,
        advisories,
    })
}

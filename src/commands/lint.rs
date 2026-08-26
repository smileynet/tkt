//! `tkt lint` — style normalization for cleaner diffs.

use std::path::Path;

use anyhow::Result;

use crate::core::TicketFile;

/// Canonical field order for frontmatter.
const CANONICAL_ORDER: &[&str] = &[
    "id",
    "title",
    "status",
    "blocked_by",
    "priority",
    "env",
    "spec",
    "validation_criteria",
];

pub fn run(check: bool, ids: &[String]) -> Result<i32> {
    let tickets_dir = crate::commands::common::tickets_dir()?;
    let files = collect_files(&tickets_dir, ids)?;

    if files.is_empty() {
        if !ids.is_empty() {
            eprintln!("tkt: no matching tickets found");
            return Ok(1);
        }
        eprintln!("tkt: no tickets in .tickets/");
        return Ok(0);
    }

    let mut changed = 0u32;
    let mut errors = 0u32;

    for path in &files {
        let tf = match TicketFile::parse(path) {
            Ok(tf) => tf,
            Err(e) => {
                eprintln!("  skip {}: {}", path.display(), e);
                errors += 1;
                continue;
            }
        };

        let original = std::fs::read_to_string(path)?;
        let canonical = render_canonical(&tf);

        if original != canonical {
            changed += 1;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if check {
                eprintln!("  would change: {}", name);
            } else {
                std::fs::write(path, &canonical)?;
                eprintln!("  ✓ {}", name);
            }
        }
    }

    if check {
        if changed > 0 {
            eprintln!("\ntkt lint: {} file(s) would change", changed);
            Ok(1)
        } else {
            eprintln!("tkt lint: all files canonical");
            Ok(0)
        }
    } else {
        if changed > 0 {
            eprintln!("\ntkt lint: {} file(s) normalized", changed);
        } else {
            eprintln!("tkt lint: all files already canonical");
        }
        if errors > 0 {
            eprintln!("tkt lint: {} file(s) skipped (parse errors)", errors);
        }
        Ok(0)
    }
}

/// Render a TicketFile in canonical format.
fn render_canonical(tf: &TicketFile) -> String {
    let mut ordered: Vec<(String, String)> = Vec::new();

    // First: known fields in canonical order
    for &field in CANONICAL_ORDER {
        if let Some(entry) = tf.fm.iter().find(|(k, _)| k == field) {
            ordered.push((entry.0.clone(), normalize_value(field, &entry.1)));
        }
    }

    // Then: unknown fields in original relative order
    for (k, v) in &tf.fm {
        if k.is_empty() {
            continue; // skip blank lines in frontmatter
        }
        if !CANONICAL_ORDER.contains(&k.as_str()) {
            ordered.push((k.clone(), normalize_value(k, v)));
        }
    }

    // Build output
    let mut parts = vec!["---".to_string()];
    for (k, v) in &ordered {
        if v.contains('\n') {
            // Multi-line value: write key: then continuation lines as-is
            parts.push(format!("{}:{}", k, v));
        } else {
            parts.push(format!("{}: {}", k, v.trim_start()));
        }
    }
    parts.push("---".to_string());

    let header = parts.join("\n");

    // Ensure exactly one blank line between closing fence and body
    let body = tf.body.trim_start_matches('\n');
    if body.is_empty() {
        format!("{}\n", header)
    } else {
        format!("{}\n\n{}", header, body)
    }
}

/// Normalize a field value based on its key.
fn normalize_value(key: &str, raw: &str) -> String {
    // Multi-line values: preserve as-is (they contain continuation lines)
    if raw.contains('\n') {
        return raw.to_string();
    }

    let trimmed = raw.trim();
    match key {
        "id" => {
            let unquoted = trimmed.trim_matches('"').trim_matches('\'');
            format!("\"{}\"", unquoted)
        }
        "blocked_by" => normalize_blocked_by(trimmed),
        _ => trimmed.to_string(),
    }
}

/// Normalize blocked_by array: ensure ["01", "04"] format.
/// Handles inline arrays, bare scalars (01, 04), and empty values.
fn normalize_blocked_by(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "[]".to_string();
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        if inner.trim().is_empty() {
            return "[]".to_string();
        }
        let items: Vec<String> = inner
            .split(',')
            .map(|s| {
                let clean = s.trim().trim_matches('"').trim_matches('\'');
                format!("\"{}\"", clean)
            })
            .collect();
        return format!("[{}]", items.join(", "));
    }
    // Bare scalar (e.g., "01, 04" or "01") — parse as comma-separated IDs
    let items: Vec<String> = trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .filter(|s| !s.is_empty())
        .map(|s| format!("\"{}\"", s))
        .collect();
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", items.join(", "))
    }
}

/// Collect ticket file paths to lint.
fn collect_files(tickets_dir: &Path, ids: &[String]) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(tickets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false)
            && path.file_name().map(|n| n != "config.toml").unwrap_or(true)
        {
            if ids.is_empty() {
                files.push(path);
            } else {
                // Check if this file matches any of the requested IDs
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                for id in ids {
                    if name.starts_with(&format!("{}-", id)) {
                        files.push(path.clone());
                        break;
                    }
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

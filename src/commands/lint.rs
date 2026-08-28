//! `tkt lint` — style normalization for cleaner diffs.

use std::path::Path;

use anyhow::Result;

use crate::core::{self, TicketFile};

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

    // Build the corpus index from the WHOLE directory (not the id-filtered subset)
    // so blocked_by refs to un-linted tickets still resolve correctly.
    let all_names: Vec<String> = std::fs::read_dir(&tickets_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    let index = core::corpus_index(&all_names);

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
        let canonical = render_canonical(&tf, &index);

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
fn render_canonical(tf: &TicketFile, index: &core::CorpusIndex) -> String {
    let mut ordered: Vec<(String, String)> = Vec::new();

    // First: known fields in canonical order
    for &field in CANONICAL_ORDER {
        if let Some(entry) = tf.fm.iter().find(|(k, _)| k == field) {
            ordered.push((entry.0.clone(), normalize_value(field, &entry.1, index)));
        }
    }

    // Then: unknown fields in original relative order
    for (k, v) in &tf.fm {
        if k.is_empty() {
            continue; // skip blank lines in frontmatter
        }
        if !CANONICAL_ORDER.contains(&k.as_str()) {
            ordered.push((k.clone(), normalize_value(k, v, index)));
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
fn normalize_value(key: &str, raw: &str, index: &core::CorpusIndex) -> String {
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
        "blocked_by" => normalize_blocked_by(trimmed, index),
        _ => trimmed.to_string(),
    }
}

/// Normalize a single blocked_by element: resolve to canonical id when a unique
/// match exists (pad width, strip numeric-prefixed slug), else keep as-is.
fn normalize_element(raw: &str, index: &core::CorpusIndex) -> String {
    let clean = raw.trim().trim_matches('"').trim_matches('\'');
    match core::resolve_blocked_by_ref(clean, index) {
        Some(resolved) => format!("\"{}\"", resolved),
        None => format!("\"{}\"", clean),
    }
}

/// Normalize blocked_by array: ensure ["01", "04"] format, resolving each ref
/// against the corpus (padding + slug-strip) when a unique match exists.
fn normalize_blocked_by(raw: &str, index: &core::CorpusIndex) -> String {
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
            .map(|s| normalize_element(s, index))
            .collect();
        return format!("[{}]", items.join(", "));
    }
    // Bare scalar (e.g., "01, 04" or "01") — parse as comma-separated IDs
    let items: Vec<String> = trimmed
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| normalize_element(s, index))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(names: &[&str]) -> core::CorpusIndex {
        core::corpus_index(&names.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    // Corpus with canonical ids 01 and 04 — resolution is a no-op for these.
    fn canon_idx() -> core::CorpusIndex {
        idx(&["01-a.md", "04-b.md"])
    }

    #[test]
    fn normalize_inline_array_preserved() {
        assert_eq!(
            normalize_blocked_by("[\"01\", \"04\"]", &canon_idx()),
            "[\"01\", \"04\"]"
        );
    }

    #[test]
    fn normalize_empty_array() {
        assert_eq!(normalize_blocked_by("[]", &canon_idx()), "[]");
        assert_eq!(normalize_blocked_by("", &canon_idx()), "[]");
    }

    #[test]
    fn normalize_bare_scalar_preserves_deps() {
        // Regression (#131): bare scalar must not be destroyed as []
        assert_eq!(
            normalize_blocked_by("01, 04", &canon_idx()),
            "[\"01\", \"04\"]"
        );
    }

    #[test]
    fn normalize_single_bare_scalar() {
        assert_eq!(normalize_blocked_by("01", &canon_idx()), "[\"01\"]");
    }

    #[test]
    fn normalize_unquoted_inline_array() {
        assert_eq!(
            normalize_blocked_by("[01, 04]", &canon_idx()),
            "[\"01\", \"04\"]"
        );
    }

    #[test]
    fn normalize_blocked_by_padding() {
        // #162: underpadded ref "5" pads to "05" when that ticket exists.
        let i = idx(&["01-a.md", "05-b.md"]);
        assert_eq!(normalize_blocked_by("[\"5\"]", &i), "[\"05\"]");
        assert_eq!(normalize_blocked_by("5", &i), "[\"05\"]");
    }

    #[test]
    fn normalize_blocked_by_slug_strip() {
        // #162: numeric-prefixed slug ref resolves to bare id.
        let i = idx(&["004-spike-foo.md", "007-bar.md"]);
        assert_eq!(normalize_blocked_by("[\"004-spike-foo\"]", &i), "[\"004\"]");
    }

    #[test]
    fn normalize_blocked_by_leaves_dangling() {
        // #162: unresolvable ref is left untouched (validate still flags it).
        let i = idx(&["01-a.md", "05-b.md"]);
        assert_eq!(normalize_blocked_by("[\"99\"]", &i), "[\"99\"]");
    }
}

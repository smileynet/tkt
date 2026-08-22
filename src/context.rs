//! Context: session-scoped tag filter that affects reads and writes.
//!
//! Storage: `.tickets/.context` (git-ignored, repo-local).
//! Override: `TKT_CONTEXT` env var.
//! Format: space-separated tags with +/- prefix, e.g. `+backend +api -frontend`

use std::path::{Path, PathBuf};

/// Active context: tags to include and exclude.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Tags that must be present (AND logic)
    pub include: Vec<String>,
    /// Tags that must NOT be present
    pub exclude: Vec<String>,
}

impl Context {
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// Check if a ticket's tags match this context.
    /// - If context is empty, everything matches.
    /// - Include tags: ticket must have ALL of them (or be untagged).
    /// - Exclude tags: ticket must NOT have any of them.
    /// - Untagged tickets match any positive context (don't hide untagged work).
    pub fn matches(&self, ticket_tags: &[String]) -> bool {
        if self.is_empty() {
            return true;
        }

        // Exclude check: if ticket has any excluded tag, reject
        for excl in &self.exclude {
            if ticket_tags.iter().any(|t| t == excl) {
                return false;
            }
        }

        // Include check: if ticket is untagged, allow (don't hide unscoped work)
        if ticket_tags.is_empty() {
            return true;
        }

        // Ticket must have ALL included tags
        self.include
            .iter()
            .all(|inc| ticket_tags.iter().any(|t| t == inc))
    }

    /// Serialize to storage format: `+tag1 +tag2 -tag3`
    pub fn serialize(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for t in &self.include {
            parts.push(format!("+{}", t));
        }
        for t in &self.exclude {
            parts.push(format!("-{}", t));
        }
        parts.join(" ")
    }
}

/// Parse a context string: `+backend +api -frontend`
pub fn parse_context(raw: &str) -> Context {
    let mut include = Vec::new();
    let mut exclude = Vec::new();

    for token in raw.split_whitespace() {
        if let Some(tag) = token.strip_prefix('+') {
            if !tag.is_empty() {
                include.push(tag.to_string());
            }
        } else if let Some(tag) = token.strip_prefix('-') {
            if !tag.is_empty() {
                exclude.push(tag.to_string());
            }
        } else if !token.is_empty() {
            // Bare word treated as include
            include.push(token.to_string());
        }
    }

    Context { include, exclude }
}

/// Load the active context for a tickets directory.
/// Priority: TKT_CONTEXT env > .tickets/.context file
pub fn load(tickets_dir: &Path) -> Context {
    // 1. Env var override
    if let Ok(val) = std::env::var("TKT_CONTEXT") {
        if !val.is_empty() {
            return parse_context(&val);
        }
    }

    // 2. File-based context
    let path = context_file_path(tickets_dir);
    if let Ok(content) = std::fs::read_to_string(&path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return parse_context(trimmed);
        }
    }

    Context::default()
}

/// Save context to .tickets/.context
pub fn save(tickets_dir: &Path, ctx: &Context) -> std::io::Result<()> {
    let path = context_file_path(tickets_dir);
    if ctx.is_empty() {
        // Clear: remove the file
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    } else {
        std::fs::write(&path, format!("{}\n", ctx.serialize()))?;
    }
    Ok(())
}

/// Path to the context state file.
fn context_file_path(tickets_dir: &Path) -> PathBuf {
    tickets_dir.join(".context")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_context_basic() {
        let ctx = parse_context("+backend +api -frontend");
        assert_eq!(ctx.include, vec!["backend", "api"]);
        assert_eq!(ctx.exclude, vec!["frontend"]);
    }

    #[test]
    fn parse_context_bare_words() {
        let ctx = parse_context("backend api");
        assert_eq!(ctx.include, vec!["backend", "api"]);
        assert!(ctx.exclude.is_empty());
    }

    #[test]
    fn parse_context_empty() {
        let ctx = parse_context("");
        assert!(ctx.is_empty());
    }

    #[test]
    fn matches_empty_context() {
        let ctx = Context::default();
        assert!(ctx.matches(&[]));
        assert!(ctx.matches(&["anything".into()]));
    }

    #[test]
    fn matches_include() {
        let ctx = parse_context("+backend");
        assert!(ctx.matches(&["backend".into(), "api".into()]));
        assert!(!ctx.matches(&["frontend".into()]));
        // Untagged tickets match (don't hide unscoped work)
        assert!(ctx.matches(&[]));
    }

    #[test]
    fn matches_exclude() {
        let ctx = parse_context("-frontend");
        assert!(ctx.matches(&["backend".into()]));
        assert!(!ctx.matches(&["frontend".into()]));
        assert!(ctx.matches(&[]));
    }

    #[test]
    fn matches_combined() {
        let ctx = parse_context("+backend -experimental");
        assert!(ctx.matches(&["backend".into()]));
        assert!(!ctx.matches(&["backend".into(), "experimental".into()]));
        assert!(!ctx.matches(&["frontend".into()]));
        assert!(ctx.matches(&[]));
    }

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = parse_context("+backend +api -legacy");
        save(dir.path(), &ctx).unwrap();

        let loaded = load(dir.path());
        assert_eq!(loaded.include, vec!["backend", "api"]);
        assert_eq!(loaded.exclude, vec!["legacy"]);
    }

    #[test]
    fn clear_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = parse_context("+backend");
        save(dir.path(), &ctx).unwrap();
        assert!(dir.path().join(".context").exists());

        save(dir.path(), &Context::default()).unwrap();
        assert!(!dir.path().join(".context").exists());
    }
}

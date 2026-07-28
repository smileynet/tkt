use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// A parsed ticket from .tickets/{id}-{slug}.md
#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: String,
    pub blocked_by: Vec<String>,
    pub priority: Option<String>,
    pub env: Option<String>,
    pub spec: Option<String>,
    pub path: PathBuf,
    pub body: String,
}

/// Frontmatter fields (serde-driven)
#[derive(Debug, Deserialize)]
struct Frontmatter {
    id: String,
    title: String,
    status: String,
    #[serde(default)]
    blocked_by: Vec<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    env: Option<String>,
    #[serde(default)]
    spec: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TicketError {
    #[error("no frontmatter delimiters in {path}")]
    NoFrontmatter { path: PathBuf },
    #[error("invalid frontmatter in {path}: {reason}")]
    InvalidFrontmatter { path: PathBuf, reason: String },
}

impl Ticket {
    /// Parse a ticket from a .md file with YAML frontmatter.
    pub fn parse(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;

        let (frontmatter_str, body) = split_frontmatter(&content)
            .ok_or_else(|| TicketError::NoFrontmatter { path: path.to_owned() })?;

        let fm: Frontmatter = serde_yaml::from_str(frontmatter_str)
            .map_err(|e| TicketError::InvalidFrontmatter {
                path: path.to_owned(),
                reason: e.to_string(),
            })?;

        Ok(Ticket {
            id: fm.id,
            title: fm.title,
            status: fm.status,
            blocked_by: fm.blocked_by,
            priority: fm.priority,
            env: fm.env,
            spec: fm.spec,
            path: path.to_owned(),
            body: body.to_owned(),
        })
    }

    pub fn is_done(&self) -> bool {
        self.status == "done"
    }

    pub fn is_open(&self) -> bool {
        self.status == "open"
    }

    pub fn is_high_priority(&self) -> bool {
        self.priority.as_deref() == Some("high")
    }
}

/// Split content into (frontmatter, body) at the --- delimiters.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;
    let end = content.find("\n---\n")
        .or_else(|| content.find("\n---\r\n"))
        .or_else(|| content.find("\r\n---\r\n"))?;
    let frontmatter = &content[..end];
    let body = &content[end + 4..]; // skip \n---\n
    Some((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_basic_ticket() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "---\nid: \"01\"\ntitle: \"Test ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n- [ ] AC1").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.id, "01");
        assert_eq!(t.title, "Test ticket");
        assert_eq!(t.status, "open");
        assert!(t.blocked_by.is_empty());
        assert!(t.is_open());
    }

    #[test]
    fn parse_ticket_with_deps() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "---\nid: \"05\"\ntitle: \"Depends on others\"\nstatus: open\nblocked_by: [\"01\", \"03\"]\npriority: high\nenv: corp\nspec: my-spec\n---\n\n# Body").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.blocked_by, vec!["01", "03"]);
        assert!(t.is_high_priority());
        assert_eq!(t.env.as_deref(), Some("corp"));
        assert_eq!(t.spec.as_deref(), Some("my-spec"));
    }
}

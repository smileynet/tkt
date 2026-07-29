use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

// --- Constants ---

pub const STATUS_VALUES: &[&str] = &["open", "in_progress", "done"];
pub const ENV_VALUES: &[&str] = &["corp", "personal", "either"];

// --- Compiled regex patterns ---

static RE_FM_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([A-Za-z_][A-Za-z0-9_-]*):(.*)$").unwrap()
});
static RE_BRACKET_LIST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\[(.*)\]"#).unwrap()
});
static RE_NUMERIC_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+)(.*)$").unwrap()
});
static RE_FILENAME_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+)-").unwrap()
});

// --- Ticket ---

/// A parsed ticket preserving raw frontmatter for surgical edits.
#[derive(Debug, Clone)]
pub struct Ticket {
    pub path: PathBuf,
    /// Ordered frontmatter entries: (key, raw_value) — key="" for blank lines.
    pub fm: Vec<(String, String)>,
    /// Everything after the closing --- fence.
    pub body: String,
}

impl Ticket {
    /// Parse a ticket from a .md file with YAML frontmatter.
    pub fn parse(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        Self::parse_str(&content, path)
    }

    /// Parse from a string (for testing).
    pub fn parse_str(content: &str, path: &Path) -> Result<Self> {
        let lines: Vec<&str> = content.split('\n').collect();

        // Opening fence
        if lines.is_empty() || !is_fence(lines[0]) {
            bail!("{}: no opening frontmatter fence on line 1", path.display());
        }

        // Parse frontmatter lines
        let key_re = &*RE_FM_KEY;
        let mut fm: Vec<(String, String)> = Vec::new();
        let mut close_idx = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if is_fence(line) {
                close_idx = Some(i);
                break;
            }
            if let Some(caps) = key_re.captures(line) {
                fm.push((caps[1].to_string(), caps[2].to_string()));
            } else if (line.starts_with(' ') || line.starts_with('\t')) && !fm.is_empty() {
                // Continuation line
                let last = fm.last_mut().unwrap();
                last.1.push('\n');
                last.1.push_str(line);
            } else if line.trim().is_empty() {
                fm.push((String::new(), (*line).to_string()));
            } else {
                bail!("{}: unparseable frontmatter line {}: {:?}", path.display(), i + 1, line);
            }
        }

        let close_idx = close_idx
            .ok_or_else(|| anyhow::anyhow!("{}: no closing frontmatter fence", path.display()))?;

        // Check required fields
        for req in &["id", "title", "status", "blocked_by"] {
            if !fm.iter().any(|(k, _)| k == req) {
                bail!("{}: missing required field: {}", path.display(), req);
            }
        }

        let body = lines[close_idx + 1..].join("\n");

        Ok(Ticket {
            path: path.to_owned(),
            fm,
            body,
        })
    }

    /// Get a field's trimmed value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fm.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.trim())
    }

    pub fn id(&self) -> &str {
        self.get("id")
            .unwrap_or("")
            .trim_matches('"')
            .trim_matches('\'')
    }

    pub fn title(&self) -> &str {
        self.get("title")
            .unwrap_or("")
            .trim_matches('"')
            .trim_matches('\'')
    }

    pub fn status(&self) -> &str {
        self.get("status").unwrap_or("")
    }

    pub fn blocked_by(&self) -> Vec<String> {
        let raw = self.get("blocked_by").unwrap_or("");
        let re = &*RE_BRACKET_LIST;
        match re.captures(raw) {
            Some(caps) => caps[1]
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn env(&self) -> &str {
        self.get("env")
            .map(|v| v.trim_matches('"'))
            .unwrap_or("either")
    }

    pub fn priority(&self) -> Option<&str> {
        self.get("priority").map(|v| v.trim_matches('"'))
    }

    pub fn is_high_priority(&self) -> bool {
        self.priority() == Some("high")
    }

    pub fn numeric_key(&self) -> (u64, String) {
        let id = self.id();
        let re = &*RE_NUMERIC_PREFIX;
        match re.captures(id) {
            Some(caps) => (caps[1].parse().unwrap_or(u64::MAX), caps[2].to_string()),
            None => (u64::MAX, id.to_string()),
        }
    }

    // --- Mutation ---

    /// Set a field value (raw text). Replaces if exists, appends if not.
    pub fn set_field(&mut self, key: &str, value: &str) {
        for (k, v) in self.fm.iter_mut() {
            if k == key {
                *v = format!(" {}", value);
                return;
            }
        }
        self.fm.push((key.to_string(), format!(" {}", value)));
    }

    /// Remove a field entirely (for clearing optional fields).
    pub fn remove_field(&mut self, key: &str) {
        self.fm.retain(|(k, _)| k != key);
    }

    // --- Serialization ---

    /// Serialize preserving raw frontmatter (surgical writes).
    pub fn serialize(&self) -> String {
        let mut parts = vec!["---".to_string()];
        for (k, v) in &self.fm {
            if k.is_empty() {
                parts.push(v.clone());
            } else {
                parts.push(format!("{}:{}", k, v));
            }
        }
        parts.push("---".to_string());
        let header = parts.join("\n");
        format!("{}\n{}", header, self.body)
    }

    /// Write the ticket back to disk.
    pub fn write(&self) -> Result<()> {
        std::fs::write(&self.path, self.serialize())
            .with_context(|| format!("writing {}", self.path.display()))
    }
}

fn is_fence(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---"
}

// --- Corpus ---

/// Load all tickets from a .tickets/ directory.
pub fn load_corpus(dir: &Path) -> Result<Vec<Ticket>> {
    let mut tickets = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let ticket = Ticket::parse(&entry.path())?;
        tickets.push(ticket);
    }
    Ok(tickets)
}

/// Compute the frontier: open tickets with all deps done, env-filtered, priority-sorted.
pub fn frontier(corpus: &[Ticket]) -> Vec<&Ticket> {
    let crew_env = std::env::var("CREW_ENV").unwrap_or_default();
    let done: std::collections::HashSet<&str> = corpus.iter()
        .filter(|t| t.status() == "done")
        .map(|t| t.id())
        .collect();

    let mut pool: Vec<&Ticket> = corpus.iter()
        .filter(|t| {
            if t.status() != "open" {
                return false;
            }
            if !t.blocked_by().iter().all(|dep| done.contains(dep.as_str())) {
                return false;
            }
            if !crew_env.is_empty() && t.env() != "either" && t.env() != crew_env {
                return false;
            }
            true
        })
        .collect();

    pool.sort_by_key(|t| (!t.is_high_priority(), t.numeric_key()));
    pool
}

/// Find the maximum numeric id in a list of filenames.
pub fn max_id(names: &[String]) -> u64 {
    names.iter()
        .filter_map(|n| RE_FILENAME_ID.captures(n).map(|c| c[1].parse::<u64>().unwrap_or(0)))
        .max()
        .unwrap_or(0)
}

/// Determine the id zero-padding width from existing filenames.
pub fn id_width(names: &[String]) -> usize {
    names.iter()
        .filter_map(|n| RE_FILENAME_ID.captures(n).map(|c| c[1].len()))
        .max()
        .unwrap_or(2)
}

/// Find a ticket by id in the corpus.
pub fn find_ticket<'a>(corpus: &'a [Ticket], id: &str) -> Result<&'a Ticket> {
    corpus.iter()
        .find(|t| t.id() == id)
        .ok_or_else(|| anyhow::anyhow!("no ticket with id {}", id))
}

/// Escape a string for use inside YAML double-quoted scalars.
/// Handles: backslash, double-quote, newline, carriage return, tab, null, and other control chars.
pub fn yaml_scalar_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                // YAML unicode escape: \xNN for ASCII control chars
                out.push_str(&format!("\\x{:02X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Escape a string for use inside a JSON string value (between quotes).
/// Handles: backslash, double-quote, newline, carriage return, tab, and control chars.
pub fn json_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Generate the text for a new ticket file.
pub fn new_ticket_text(id: &str, title: &str, blocked_by: &[String], env: Option<&str>, spec: Option<&str>, priority: Option<&str>) -> String {
    let mut fm_lines = vec![
        format!("id: \"{}\"", yaml_scalar_escape(id)),
        format!("title: \"{}\"", yaml_scalar_escape(title)),
        "status: open".to_string(),
    ];
    let deps = blocked_by.iter()
        .map(|d| format!("\"{}\"", yaml_scalar_escape(d)))
        .collect::<Vec<_>>()
        .join(", ");
    fm_lines.push(format!("blocked_by: [{}]", deps));
    if let Some(e) = env {
        fm_lines.push(format!("env: {}", e));
    }
    if let Some(s) = spec {
        fm_lines.push(format!("spec: \"{}\"", yaml_scalar_escape(s)));
    }
    if let Some(p) = priority {
        fm_lines.push(format!("priority: {}", p));
    }
    let body = format!("\n# {}\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n", title);
    format!("---\n{}\n---\n{}", fm_lines.join("\n"), body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_basic_ticket() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "---\nid: \"01\"\ntitle: \"Test ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n- [ ] AC1\n").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.id(), "01");
        assert_eq!(t.title(), "Test ticket");
        assert_eq!(t.status(), "open");
        assert!(t.blocked_by().is_empty());
    }

    #[test]
    fn parse_ticket_with_deps() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "---\nid: \"05\"\ntitle: \"Depends on others\"\nstatus: open\nblocked_by: [\"01\", \"03\"]\npriority: high\nenv: corp\nspec: \"my-spec\"\n---\n\n# Body\n").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.blocked_by(), vec!["01", "03"]);
        assert!(t.is_high_priority());
        assert_eq!(t.env(), "corp");
    }

    #[test]
    fn frontier_filters_correctly() {
        let content_done = "---\nid: \"01\"\ntitle: \"Done\"\nstatus: done\nblocked_by: []\n---\n";
        let content_open = "---\nid: \"02\"\ntitle: \"Open\"\nstatus: open\nblocked_by: [\"01\"]\n---\n";
        let content_blocked = "---\nid: \"03\"\ntitle: \"Blocked\"\nstatus: open\nblocked_by: [\"99\"]\n---\n";

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("01-done.md"), content_done).unwrap();
        std::fs::write(dir.path().join("02-open.md"), content_open).unwrap();
        std::fs::write(dir.path().join("03-blocked.md"), content_blocked).unwrap();

        let corpus = load_corpus(dir.path()).unwrap();
        let front = frontier(&corpus);
        assert_eq!(front.len(), 1);
        assert_eq!(front[0].id(), "02");
    }

    #[test]
    fn surgical_edit_preserves_unknown_fields() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\ncustom_field: hello\n---\n\n# Body\n";
        let mut t = Ticket::parse_str(content, Path::new("test.md")).unwrap();
        t.set_field("status", "done");
        let out = t.serialize();
        assert!(out.contains("status: done"));
        assert!(out.contains("custom_field: hello"));
    }

    #[test]
    fn yaml_escape_handles_special_chars() {
        assert_eq!(yaml_scalar_escape("hello"), "hello");
        assert_eq!(yaml_scalar_escape(""), "");
        assert_eq!(yaml_scalar_escape(r#"Fix "ready""#), r#"Fix \"ready\""#);
        assert_eq!(yaml_scalar_escape("back\\slash"), "back\\\\slash");
        assert_eq!(yaml_scalar_escape("line\nbreak"), "line\\nbreak");
        assert_eq!(yaml_scalar_escape("tab\there"), "tab\\there");
        assert_eq!(yaml_scalar_escape("cr\rhere"), "cr\\rhere");
        assert_eq!(yaml_scalar_escape("null\0byte"), "null\\0byte");
        assert_eq!(yaml_scalar_escape("unicode: café"), "unicode: café");
        // Combined adversarial
        assert_eq!(yaml_scalar_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn json_escape_handles_special_chars() {
        assert_eq!(json_string_escape("hello"), "hello");
        assert_eq!(json_string_escape(""), "");
        assert_eq!(json_string_escape(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(json_string_escape("back\\slash"), "back\\\\slash");
        assert_eq!(json_string_escape("line\nbreak"), "line\\nbreak");
        assert_eq!(json_string_escape("tab\there"), "tab\\there");
        assert_eq!(json_string_escape("cr\rhere"), "cr\\rhere");
        assert_eq!(json_string_escape("unicode: café"), "unicode: café");
        // Control char gets \u escape
        let with_ctrl = format!("ctrl{}here", '\x01');
        assert_eq!(json_string_escape(&with_ctrl), "ctrl\\u0001here");
        // Combined adversarial
        assert_eq!(json_string_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn new_ticket_text_escapes_title() {
        let text = new_ticket_text("01", "Fix \"ready\" command", &[], None, None, None);
        assert!(text.contains(r#"title: "Fix \"ready\" command""#));
        // Should be valid frontmatter (parseable)
        let t = Ticket::parse_str(&text, Path::new("test.md")).unwrap();
        // The title accessor trims quotes, so escaped quotes appear as-is in raw value
        assert!(t.get("title").unwrap().contains("ready"));
    }
}

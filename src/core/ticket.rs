use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

// --- Constants ---

pub const STATUS_VALUES: &[&str] = &["open", "in_progress", "done"];
pub const ENV_VALUES: &[&str] = &["corp", "personal", "either"];

// --- Compiled regex patterns ---

static RE_FM_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z_][A-Za-z0-9_-]*):(.*)$").unwrap());
static RE_NUMERIC_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)(.*)$").unwrap());
static RE_FILENAME_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-").unwrap());

// --- Enums ---

/// Ticket lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    InProgress,
    Done,
}

impl Status {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "open" => Ok(Status::Open),
            "in_progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            other => bail!(
                "invalid status {:?} (expected open/in_progress/done)",
                other
            ),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in_progress",
            Status::Done => "done",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Environment filter for tickets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Corp,
    Personal,
    Either,
}

impl Env {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "corp" => Ok(Env::Corp),
            "personal" => Ok(Env::Personal),
            "either" => Ok(Env::Either),
            other => bail!("invalid env {:?} (expected corp/personal/either)", other),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Env::Corp => "corp",
            Env::Personal => "personal",
            Env::Either => "either",
        }
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ticket priority (currently only "high" is valid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
}

impl Priority {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "high" => Ok(Priority::High),
            other => bail!("invalid priority {:?} (expected high)", other),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::High => "high",
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// --- TicketFile ---

/// Raw frontmatter editor. Preserves field ordering and unknown fields for surgical edits.
#[derive(Debug, Clone)]
pub struct TicketFile {
    pub path: PathBuf,
    /// Ordered frontmatter entries: (key, raw_value) — key="" for blank lines.
    pub fm: Vec<(String, String)>,
    /// Everything after the closing --- fence.
    pub body: String,
}

impl TicketFile {
    /// Parse a ticket file from disk.
    pub fn parse(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse_str(&content, path)
    }

    /// Parse from a string (for testing).
    pub fn parse_str(content: &str, path: &Path) -> Result<Self> {
        let lines: Vec<&str> = content.split('\n').collect();

        if lines.is_empty() || !is_fence(lines[0]) {
            bail!("{}: no opening frontmatter fence on line 1", path.display());
        }

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
                let last = fm.last_mut().unwrap();
                last.1.push('\n');
                last.1.push_str(line);
            } else if line.trim().is_empty() {
                fm.push((String::new(), (*line).to_string()));
            } else {
                bail!(
                    "{}: unparseable frontmatter line {}: {:?}",
                    path.display(),
                    i + 1,
                    line
                );
            }
        }

        let close_idx = close_idx
            .ok_or_else(|| anyhow::anyhow!("{}: no closing frontmatter fence", path.display()))?;

        for req in &["id", "title", "status", "blocked_by"] {
            if !fm.iter().any(|(k, _)| k == req) {
                bail!("{}: missing required field: {}", path.display(), req);
            }
        }

        let body = lines[close_idx + 1..].join("\n");

        Ok(TicketFile {
            path: path.to_owned(),
            fm,
            body,
        })
    }

    /// Get a field's trimmed value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fm
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.trim())
    }

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

    /// Write the ticket file back to disk.
    pub fn write(&self) -> Result<()> {
        std::fs::write(&self.path, self.serialize())
            .with_context(|| format!("writing {}", self.path.display()))
    }
}

// --- Ticket ---

/// A parsed, validated ticket with typed fields. Constructed from a TicketFile.
/// Field access is zero-cost (&str borrows on owned Strings).
#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub blocked_by: Vec<String>,
    pub env: Env,
    pub priority: Option<Priority>,
    pub spec: Option<String>,
    pub path: PathBuf,
    pub body: String,
    /// The underlying raw file for mutations.
    pub file: TicketFile,
}

impl Ticket {
    /// Parse and validate a ticket from disk.
    pub fn parse(path: &Path) -> Result<Self> {
        let file = TicketFile::parse(path)?;
        Self::from_file(file)
    }

    /// Parse and validate from a string (for testing).
    #[allow(dead_code)]
    pub fn parse_str(content: &str, path: &Path) -> Result<Self> {
        let file = TicketFile::parse_str(content, path)?;
        Self::from_file(file)
    }

    /// Construct a validated Ticket from a TicketFile.
    fn from_file(file: TicketFile) -> Result<Self> {
        let path_display = file.path.display().to_string();

        let raw_id = file.get("id").unwrap_or("");
        let id = yaml_scalar_unescape(raw_id.trim_matches('"').trim_matches('\''));

        let raw_title = file.get("title").unwrap_or("");
        let title = yaml_scalar_unescape(raw_title.trim_matches('"').trim_matches('\''));

        let raw_status = file.get("status").unwrap_or("");
        let status =
            Status::parse(raw_status).with_context(|| format!("{}: bad status", path_display))?;

        let blocked_by = parse_blocked_by(file.get("blocked_by").unwrap_or(""));

        let raw_env = file
            .get("env")
            .map(|v| v.trim_matches('"'))
            .unwrap_or("either");
        let env = Env::parse(raw_env).with_context(|| format!("{}: bad env", path_display))?;

        let priority = match file.get("priority").map(|v| v.trim_matches('"')) {
            Some(p) if !p.is_empty() => Some(
                Priority::parse(p).with_context(|| format!("{}: bad priority", path_display))?,
            ),
            _ => None,
        };

        let spec = file
            .get("spec")
            .map(|v| yaml_scalar_unescape(v.trim_matches('"').trim_matches('\'')));

        Ok(Ticket {
            id,
            title,
            status,
            blocked_by,
            env,
            priority,
            spec,
            path: file.path.clone(),
            body: file.body.clone(),
            file,
        })
    }

    /// Numeric sort key: (numeric prefix, remainder).
    pub fn numeric_key(&self) -> (u64, String) {
        let re = &*RE_NUMERIC_PREFIX;
        match re.captures(&self.id) {
            Some(caps) => (caps[1].parse().unwrap_or(u64::MAX), caps[2].to_string()),
            None => (u64::MAX, self.id.clone()),
        }
    }

    /// Whether this ticket has high priority.
    pub fn is_high_priority(&self) -> bool {
        self.priority == Some(Priority::High)
    }
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
    let done: std::collections::HashSet<&str> = corpus
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();

    let mut pool: Vec<&Ticket> = corpus
        .iter()
        .filter(|t| {
            if t.status != Status::Open {
                return false;
            }
            if !t.blocked_by.iter().all(|dep| done.contains(dep.as_str())) {
                return false;
            }
            if !crew_env.is_empty() && t.env != Env::Either && t.env.as_str() != crew_env {
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
    names
        .iter()
        .filter_map(|n| {
            RE_FILENAME_ID
                .captures(n)
                .map(|c| c[1].parse::<u64>().unwrap_or(0))
        })
        .max()
        .unwrap_or(0)
}

/// Determine the id zero-padding width from existing filenames.
pub fn id_width(names: &[String]) -> usize {
    names
        .iter()
        .filter_map(|n| RE_FILENAME_ID.captures(n).map(|c| c[1].len()))
        .max()
        .unwrap_or(2)
}

/// Find a ticket by id in the corpus.
pub fn find_ticket<'a>(corpus: &'a [Ticket], id: &str) -> Result<&'a Ticket> {
    corpus
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("no ticket with id {}", id))
}

// --- Helpers ---

fn is_fence(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---"
}

/// Parse a blocked_by field value into a Vec of IDs.
fn parse_blocked_by(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// --- YAML/JSON escaping ---

/// Escape a string for use inside YAML double-quoted scalars.
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
                out.push_str(&format!("\\x{:02X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Decode YAML double-quoted scalar escapes when reading values.
pub fn yaml_scalar_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('0') => out.push('\0'),
                Some('x') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        out.push(byte as char);
                    } else {
                        out.push_str("\\x");
                        out.push_str(&hex);
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Escape a string for use inside a JSON string value (between quotes).
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
pub fn new_ticket_text(
    id: &str,
    title: &str,
    blocked_by: &[String],
    env: Option<&str>,
    spec: Option<&str>,
    priority: Option<&str>,
) -> String {
    let mut fm_lines = vec![
        format!("id: \"{}\"", yaml_scalar_escape(id)),
        format!("title: \"{}\"", yaml_scalar_escape(title)),
        "status: open".to_string(),
    ];
    let deps = blocked_by
        .iter()
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
    let body = format!(
        "\n# {}\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n",
        title
    );
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
        assert_eq!(t.id, "01");
        assert_eq!(t.title, "Test ticket");
        assert_eq!(t.status, Status::Open);
        assert!(t.blocked_by.is_empty());
    }

    #[test]
    fn parse_ticket_with_deps() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "---\nid: \"05\"\ntitle: \"Depends on others\"\nstatus: open\nblocked_by: [\"01\", \"03\"]\npriority: high\nenv: corp\nspec: \"my-spec\"\n---\n\n# Body\n").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.blocked_by, vec!["01", "03"]);
        assert!(t.is_high_priority());
        assert_eq!(t.env, Env::Corp);
        assert_eq!(t.spec.as_deref(), Some("my-spec"));
    }

    #[test]
    fn frontier_filters_correctly() {
        let content_done = "---\nid: \"01\"\ntitle: \"Done\"\nstatus: done\nblocked_by: []\n---\n";
        let content_open =
            "---\nid: \"02\"\ntitle: \"Open\"\nstatus: open\nblocked_by: [\"01\"]\n---\n";
        let content_blocked =
            "---\nid: \"03\"\ntitle: \"Blocked\"\nstatus: open\nblocked_by: [\"99\"]\n---\n";

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("01-done.md"), content_done).unwrap();
        std::fs::write(dir.path().join("02-open.md"), content_open).unwrap();
        std::fs::write(dir.path().join("03-blocked.md"), content_blocked).unwrap();

        let corpus = load_corpus(dir.path()).unwrap();
        let front = frontier(&corpus);
        assert_eq!(front.len(), 1);
        assert_eq!(front[0].id, "02");
    }

    #[test]
    fn surgical_edit_preserves_unknown_fields() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\ncustom_field: hello\n---\n\n# Body\n";
        let mut t = Ticket::parse_str(content, Path::new("test.md")).unwrap();
        t.file.set_field("status", "done");
        let out = t.file.serialize();
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
        let with_ctrl = format!("ctrl{}here", '\x01');
        assert_eq!(json_string_escape(&with_ctrl), "ctrl\\u0001here");
        assert_eq!(json_string_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn new_ticket_text_escapes_title() {
        let text = new_ticket_text("01", "Fix \"ready\" command", &[], None, None, None);
        assert!(text.contains(r#"title: "Fix \"ready\" command""#));
        let t = Ticket::parse_str(&text, Path::new("test.md")).unwrap();
        assert!(t.title.contains("ready"));
    }

    #[test]
    fn status_enum_parse() {
        assert_eq!(Status::parse("open").unwrap(), Status::Open);
        assert_eq!(Status::parse("in_progress").unwrap(), Status::InProgress);
        assert_eq!(Status::parse("done").unwrap(), Status::Done);
        assert!(Status::parse("invalid").is_err());
    }

    #[test]
    fn env_enum_parse() {
        assert_eq!(Env::parse("corp").unwrap(), Env::Corp);
        assert_eq!(Env::parse("personal").unwrap(), Env::Personal);
        assert_eq!(Env::parse("either").unwrap(), Env::Either);
        assert!(Env::parse("invalid").is_err());
    }

    #[test]
    fn invalid_status_rejected_at_parse() {
        let content =
            "---\nid: \"01\"\ntitle: \"Bad\"\nstatus: invalid\nblocked_by: []\n---\n\n# Bad\n";
        let result = Ticket::parse_str(content, Path::new("test.md"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("status") || msg.contains("invalid"));
    }

    #[test]
    fn invalid_env_rejected_at_parse() {
        let content =
            "---\nid: \"01\"\ntitle: \"Bad\"\nstatus: open\nblocked_by: []\nenv: bogus\n---\n\n# Bad\n";
        let result = Ticket::parse_str(content, Path::new("test.md"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("env") || msg.contains("bogus"));
    }
}

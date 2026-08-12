//! `tkt init` — scaffold .tickets/ and deploy agent integration files.

use anyhow::Result;

use crate::color::{sym_ok, sym_warn};
use crate::commands::common::is_quiet;

// --- Marker constants ---

const MARKER_BEGIN: &str = "<!-- tkt:begin -->";
const MARKER_END: &str = "<!-- tkt:end -->";

// --- Canonical snippet (AGENTS.md / stdout) ---

const SNIPPET_AGENTS: &str = r#"## Tickets

This project uses [tkt](https://github.com/smileynet/tkt) for work tracking. Tickets live in `.tickets/`.

```
tkt ready                                         # what to work on next
tkt claim <id>                                    # mark as in_progress (shared repos)
tkt close <id> --check-all --resolution "..."     # mark done
tkt validate --brief                              # check for issues
```

### Workflow

Single-agent: `tkt ready` → `tkt close <id> --check-all --resolution "..."`
Shared-repo: `tkt ready` → `tkt claim <id>` → work → `tkt close <id> --check-all --resolution "..."`

If a claim push is rejected, someone else got there first — pick the next frontier ticket.
"#;

// --- Per-agent variants ---

const SNIPPET_CLAUDE: &str = r#"## Tickets

This project uses tkt for work tracking. Run `tkt ready` to see what's available.

Commands: ready, claim <id>, close <id> --check-all --resolution "...", validate --brief

Workflow: tkt ready → close <id> --check-all --resolution "done: what was shipped"
For shared repos: tkt ready → claim <id> → work → close <id> --check-all --resolution "..."
"#;

const SNIPPET_CURSOR: &str = r#"---
description: tkt ticket management workflow
alwaysApply: true
---

# tkt Workflow

This project uses tkt for work tracking (.tickets/ directory).

## Commands
- `tkt ready` — see unblocked tickets (frontier)
- `tkt claim <id>` — mark in_progress (shared repos only)
- `tkt close <id> --check-all --resolution "..."` — mark done
- `tkt validate --brief` — check for issues

## Workflow
1. `tkt ready` → pick the first listed ticket
2. Read the ticket file completely
3. Do the work described
4. Verify acceptance criteria
5. `tkt close <id> --check-all --resolution "what was done"`
"#;

const SNIPPET_KIRO: &str = r#"# tkt Integration

When .tickets/ exists, work the frontier.

## Commands
tkt ready              # frontier (open + deps done + env match)
tkt claim <id>         # status → in_progress, pushed
tkt close <id> --check-all --resolution "..."  # mark done
tkt validate --brief   # check for issues

## Workflow
Single-agent: tkt ready → close <id> --check-all --resolution "..."
Shared-repo: tkt ready → claim <id> → work → close <id>

## Frontier Rule
Pick the first ticket `tkt ready` lists — it already applies priority sorting
(urgent > high > medium > low) then lowest-number-first.
"#;

const SNIPPET_WINDSURF: &str = r#"---
trigger: always_on
---

# tkt Workflow

This project uses tkt for work tracking. Tickets in .tickets/.

Commands: ready, claim <id>, close <id> --check-all --resolution "...", validate --brief
Workflow: tkt ready → close <id> --check-all --resolution "done: description"
"#;

const DEFAULT_CONFIG: &str = r#"[push]
enabled = true

[close]
require_validation_criteria = false
require_validation_evidence = "warn"
"#;

// --- Entry point ---

pub fn run(
    write: Option<Option<String>>,
    target: Option<&str>,
    all: bool,
    agent_only: bool,
) -> Result<i32> {
    let repo_root = crate::git::repo_root_cwd()?;

    // Step 1: Project bootstrapping (unless --agent-only)
    if !agent_only {
        bootstrap_project(&repo_root)?;
    }

    // Step 2: Agent instructions
    if all {
        write_all_targets(&repo_root)?;
    } else if let Some(t) = target {
        write_target(&repo_root, t)?;
    } else if let Some(filename) = write {
        let file = filename.unwrap_or_else(|| "AGENTS.md".to_string());
        let path = repo_root.join(&file);
        write_with_markers(&path, SNIPPET_AGENTS)?;
        if !is_quiet() {
            println!("  {} updated {} (tkt section)", sym_ok(), file);
        }
    } else {
        // Default: print to stdout
        print!("{}", SNIPPET_AGENTS);
    }

    Ok(0)
}

// --- Project bootstrapping ---

fn bootstrap_project(repo_root: &std::path::Path) -> Result<()> {
    let tickets_dir = repo_root.join(".tickets");

    if tickets_dir.exists() {
        if !is_quiet() {
            println!("  {} .tickets/ already exists", sym_ok());
        }
    } else {
        std::fs::create_dir_all(&tickets_dir)?;
        if !is_quiet() {
            println!("  {} created .tickets/", sym_ok());
        }
    }

    let config_path = tickets_dir.join("config.toml");
    if config_path.exists() {
        if !is_quiet() {
            println!("  {} .tickets/config.toml already exists", sym_ok());
        }
    } else {
        std::fs::write(&config_path, DEFAULT_CONFIG)?;
        if !is_quiet() {
            println!("  {} created .tickets/config.toml", sym_ok());
        }
    }

    Ok(())
}

// --- Target writing ---

fn write_all_targets(repo_root: &std::path::Path) -> Result<()> {
    // AGENTS.md (markers)
    let agents_path = repo_root.join("AGENTS.md");
    write_with_markers(&agents_path, SNIPPET_AGENTS)?;
    if !is_quiet() {
        println!("  {} updated AGENTS.md (tkt section)", sym_ok());
    }

    // CLAUDE.md (markers)
    let claude_path = repo_root.join("CLAUDE.md");
    write_with_markers(&claude_path, SNIPPET_CLAUDE)?;
    if !is_quiet() {
        println!("  {} updated CLAUDE.md (tkt section)", sym_ok());
    }

    // .cursor/rules/tkt.mdc (owned file)
    write_owned_file(repo_root, ".cursor/rules/tkt.mdc", SNIPPET_CURSOR)?;

    // .kiro/steering/tkt.md (owned file)
    write_owned_file(repo_root, ".kiro/steering/tkt.md", SNIPPET_KIRO)?;

    // .github/copilot-instructions.md (markers, only if .github/ exists)
    let github_dir = repo_root.join(".github");
    if github_dir.exists() {
        let copilot_path = github_dir.join("copilot-instructions.md");
        write_with_markers(&copilot_path, SNIPPET_AGENTS)?;
        if !is_quiet() {
            println!(
                "  {} updated .github/copilot-instructions.md (tkt section)",
                sym_ok()
            );
        }
    } else if !is_quiet() {
        println!(
            "  {} skipped .github/copilot-instructions.md (.github/ doesn't exist)",
            sym_warn()
        );
    }

    // .windsurf/rules/tkt.md (owned file)
    write_owned_file(repo_root, ".windsurf/rules/tkt.md", SNIPPET_WINDSURF)?;

    Ok(())
}

fn write_target(repo_root: &std::path::Path, target: &str) -> Result<()> {
    match target {
        "agents" | "codex" => {
            let path = repo_root.join("AGENTS.md");
            write_with_markers(&path, SNIPPET_AGENTS)?;
            if !is_quiet() {
                println!("  {} updated AGENTS.md (tkt section)", sym_ok());
            }
        }
        "claude" => {
            let path = repo_root.join("CLAUDE.md");
            write_with_markers(&path, SNIPPET_CLAUDE)?;
            if !is_quiet() {
                println!("  {} updated CLAUDE.md (tkt section)", sym_ok());
            }
        }
        "cursor" => {
            write_owned_file(repo_root, ".cursor/rules/tkt.mdc", SNIPPET_CURSOR)?;
        }
        "kiro" => {
            write_owned_file(repo_root, ".kiro/steering/tkt.md", SNIPPET_KIRO)?;
        }
        "copilot" => {
            let github_dir = repo_root.join(".github");
            if github_dir.exists() {
                let path = github_dir.join("copilot-instructions.md");
                write_with_markers(&path, SNIPPET_AGENTS)?;
                if !is_quiet() {
                    println!(
                        "  {} updated .github/copilot-instructions.md (tkt section)",
                        sym_ok()
                    );
                }
            } else if !is_quiet() {
                println!(
                    "  {} skipped .github/copilot-instructions.md (.github/ doesn't exist)",
                    sym_warn()
                );
            }
        }
        "windsurf" => {
            write_owned_file(repo_root, ".windsurf/rules/tkt.md", SNIPPET_WINDSURF)?;
        }
        _ => {
            anyhow::bail!("unknown target: {}", target);
        }
    }
    Ok(())
}

// --- File writing helpers ---

/// Write content between markers in a file. Creates file if missing, appends if no markers found.
fn write_with_markers(path: &std::path::Path, content: &str) -> Result<()> {
    let wrapped = format!("{}\n{}{}\n", MARKER_BEGIN, content, MARKER_END);

    if !path.exists() {
        // Create parent dirs if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &wrapped)?;
        return Ok(());
    }

    let existing = std::fs::read_to_string(path)?;

    if let (Some(begin), Some(end)) = (existing.find(MARKER_BEGIN), existing.find(MARKER_END)) {
        // Replace between markers (inclusive)
        let end_of_marker = end + MARKER_END.len();
        // Skip trailing newline after end marker if present
        let end_pos = if existing[end_of_marker..].starts_with('\n') {
            end_of_marker + 1
        } else {
            end_of_marker
        };
        let mut result = String::with_capacity(existing.len());
        result.push_str(&existing[..begin]);
        result.push_str(&wrapped);
        result.push_str(&existing[end_pos..]);
        std::fs::write(path, result)?;
    } else {
        // Append with blank line separator
        let mut result = existing;
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
        result.push_str(&wrapped);
        std::fs::write(path, result)?;
    }

    Ok(())
}

/// Write a file that tkt fully owns (no markers needed). Creates parent dirs.
fn write_owned_file(repo_root: &std::path::Path, rel_path: &str, content: &str) -> Result<()> {
    let path = repo_root.join(rel_path);
    let verb = if path.exists() { "updated" } else { "created" };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    if !is_quiet() {
        println!("  {} {} {}", sym_ok(), verb, rel_path);
    }
    Ok(())
}

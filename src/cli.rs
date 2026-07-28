use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use regex::Regex;

use crate::core::{self, Ticket};
use crate::git;

#[derive(Parser)]
#[command(name = "tkt", about = "Git-native ticket CLI (.tickets/ contract)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Frontier: open tickets with all deps done, env-filtered, priority-aware
    Ready {
        #[arg(long)]
        json: bool,
    },
    /// Allocate a new ticket id (fetch, scan, create, commit, push)
    New {
        slug: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        spec: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long, value_delimiter = ',')]
        blocked_by: Option<Vec<String>>,
    },
    /// Allocate N sequential ids in one commit/push
    Batch {
        #[arg(required = true)]
        items: Vec<String>,
        #[arg(long)]
        spec: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long, value_delimiter = ',')]
        blocked_by: Option<Vec<String>>,
    },
    /// Mark open ticket in_progress (pushed = visible WIP)
    Claim { id: String },
    /// Mark done, append dated Resolution stub, warn unchecked ACs
    Close {
        id: String,
        #[arg(long)]
        note: Option<String>,
        #[arg(long, value_delimiter = ',')]
        ac: Option<Vec<u32>>,
    },
    /// Surgical field corrections
    Edit {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        env: Option<String>,
        #[arg(long)]
        spec: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long, value_delimiter = ',')]
        ac: Option<Vec<u32>>,
    },
    /// Move a ticket to a new id atomically
    Renumber {
        old_id: String,
        new_id: String,
        #[arg(long)]
        file: Option<String>,
    },
    /// Drift-check ticket status vs a plan table
    SyncPlan {
        #[arg(long, group = "mode")]
        check: bool,
        #[arg(long, group = "mode")]
        fix: bool,
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        brief: bool,
        plan: Option<String>,
    },
    /// Contract + decay findings (JSON, exit 0/1)
    Validate {
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        brief: bool,
    },
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Ready { json } => cmd_ready(json),
        Commands::New { slug, title, spec, env, priority, blocked_by } => {
            cmd_new(&slug, title.as_deref(), spec.as_deref(), env.as_deref(), priority.as_deref(), &blocked_by.unwrap_or_default())
        }
        Commands::Batch { .. } => { eprintln!("tkt: batch not yet implemented"); Err(anyhow::anyhow!("unimplemented")) }
        Commands::Claim { id } => cmd_claim(&id),
        Commands::Close { id, note, ac } => cmd_close(&id, note.as_deref(), &ac.unwrap_or_default()),
        Commands::Edit { .. } => { eprintln!("tkt: edit not yet implemented"); Err(anyhow::anyhow!("unimplemented")) }
        Commands::Renumber { .. } => { eprintln!("tkt: renumber not yet implemented"); Err(anyhow::anyhow!("unimplemented")) }
        Commands::SyncPlan { .. } => { eprintln!("tkt: sync-plan not yet implemented"); Err(anyhow::anyhow!("unimplemented")) }
        Commands::Validate { .. } => { eprintln!("tkt: validate not yet implemented"); Err(anyhow::anyhow!("unimplemented")) }
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("tkt: {}", e);
            1
        }
    }
}

// --- Helpers ---

fn tickets_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let root = git::repo_root(&cwd)?;
    let dir = root.join(".tickets");
    if !dir.is_dir() {
        bail!("no .tickets/ directory in {}", root.display());
    }
    Ok(dir)
}

fn has_remote(repo: &Path) -> bool {
    git::git(repo, &["remote"]).map(|s| !s.is_empty()).unwrap_or(false)
}

fn ticket_filenames(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

// --- Commands ---

fn cmd_ready(json: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let front = core::frontier(&corpus);

    if json {
        for t in &front {
            println!("{{\"id\":\"{}\",\"title\":\"{}\",\"status\":\"{}\"}}", t.id(), t.title(), t.status());
        }
    } else {
        for t in &front {
            let flag = if t.is_high_priority() { "  [HIGH]" } else { "" };
            println!("{}  {}{}", t.id(), t.title(), flag);
        }
        let wip: Vec<&Ticket> = corpus.iter()
            .filter(|t| t.status() == "in_progress")
            .collect();
        if !wip.is_empty() {
            let ids: Vec<&str> = wip.iter().map(|t| t.id()).collect();
            println!("\nin progress (claimed elsewhere): {}", ids.join(", "));
        }
    }
    Ok(0)
}

fn cmd_new(slug: &str, title: Option<&str>, spec: Option<&str>, env: Option<&str>, priority: Option<&str>, blocked_by: &[String]) -> Result<i32> {
    // Validate slug
    let slug_re = Regex::new(r"^[a-z0-9][a-z0-9-]*$").unwrap();
    if !slug_re.is_match(slug) {
        bail!("invalid slug {:?} — allowed: lowercase letters, digits, dashes", slug);
    }

    let title_owned = slug.replace('-', " ");
    let title = title.unwrap_or(&title_owned);
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);

    if remote {
        git::fetch(&repo)?;
    }

    let names = ticket_filenames(&dir);
    let next_id = core::max_id(&names) + 1;
    let width = core::id_width(&names);
    let tid = format!("{:0>width$}", next_id, width = width);

    let filename = format!("{}-{}.md", tid, slug);
    let path = dir.join(&filename);
    let content = core::new_ticket_text(&tid, title, blocked_by, env, spec, priority);
    std::fs::write(&path, &content)?;

    // Commit
    let rel_path = format!(".tickets/{}", filename);
    git::add(&repo, &[&rel_path])?;
    git::commit(&repo, &format!("chore(tickets): new {} {}", tid, slug))?;

    if !remote {
        println!("created {} (no remote — id claim is local only, status: open)", filename);
        return Ok(0);
    }

    // Push (single attempt for now — race handling is v2)
    match git::push(&repo) {
        Ok(()) => {
            println!("allocated {} (pushed — id claimed, status: open)", filename);
            Ok(0)
        }
        Err(e) => {
            bail!("push failed (race?): {}. Pull and retry manually.", e);
        }
    }
}

fn cmd_claim(id: &str) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let t = core::find_ticket(&corpus, id)?;

    if t.status() != "open" {
        bail!("{} is {}, not open", t.id(), t.status());
    }

    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);
    if remote {
        git::fetch(&repo)?;
    }

    let mut t = t.clone();
    t.set_field("status", "in_progress");
    t.write()?;

    let rel_path = t.path.strip_prefix(&repo)
        .unwrap_or(&t.path)
        .to_string_lossy()
        .replace('\\', "/");
    git::add(&repo, &[&rel_path])?;
    git::commit(&repo, &format!("chore(tickets): claim {}", id))?;

    if remote {
        git::push(&repo)?;
    }

    println!("claimed {} (in_progress pushed)", t.path.file_name().unwrap().to_string_lossy());
    Ok(0)
}

fn cmd_close(id: &str, note: Option<&str>, ac_indices: &[u32]) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let t = core::find_ticket(&corpus, id)?;

    if t.status() == "done" {
        bail!("{} is already done", t.id());
    }

    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);
    if remote {
        git::fetch(&repo)?;
    }

    let mut t = t.clone();
    t.set_field("status", "done");

    // Append Resolution section if not present
    if !t.body.contains("## Resolution") {
        let date = chrono_date();
        let resolution = note.unwrap_or("TBD");
        t.body = format!("{}\n\n## Resolution ({})\n\n{}\n",
            t.body.trim_end(), date, resolution);
    }

    // Flip AC boxes if specified
    if !ac_indices.is_empty() {
        t.body = flip_ac_boxes(&t.body, ac_indices);
    }

    t.write()?;

    // Warn about unchecked ACs
    let unchecked_re = Regex::new(r"- \[ \]").unwrap();
    let unchecked = unchecked_re.find_iter(&t.body).count();
    if unchecked > 0 {
        eprintln!("warning: {} unchecked acceptance box(es) — fill in before trusting history", unchecked);
    }

    let rel_path = t.path.strip_prefix(&repo)
        .unwrap_or(&t.path)
        .to_string_lossy()
        .replace('\\', "/");
    git::add(&repo, &[&rel_path])?;
    git::commit(&repo, &format!("chore(tickets): close {}", id))?;

    if remote {
        git::push(&repo)?;
    }

    let verb = if note.is_some() { "written" } else { "stub appended" };
    println!("closed {} (dated Resolution {})", t.path.file_name().unwrap().to_string_lossy(), verb);
    Ok(0)
}

// --- Utilities ---

fn chrono_date() -> String {
    // Simple date without pulling in chrono crate
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            // Fallback for Windows
            let output = std::process::Command::new("cmd")
                .args(["/C", "echo %date:~6,4%-%date:~3,2%-%date:~0,2%"])
                .output();
            match output {
                Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                Err(_) => "UNDATED".to_string(),
            }
        }
    }
}

fn flip_ac_boxes(body: &str, indices: &[u32]) -> String {
    let re = Regex::new(r"- \[ \]").unwrap();
    let mut result = body.to_string();
    let matches: Vec<_> = re.find_iter(body).collect();

    // Work backwards to preserve indices
    for &idx in indices.iter().rev() {
        let i = (idx as usize).saturating_sub(1); // 1-based to 0-based
        if i < matches.len() {
            let m = &matches[i];
            result.replace_range(m.start()..m.end(), "- [x]");
        }
    }
    result
}

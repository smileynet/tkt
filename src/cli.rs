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
        Commands::Batch { items, spec, env, priority, blocked_by } => {
            cmd_batch(&items, spec.as_deref(), env.as_deref(), priority.as_deref(), &blocked_by.unwrap_or_default())
        }
        Commands::Claim { id } => cmd_claim(&id),
        Commands::Close { id, note, ac } => cmd_close(&id, note.as_deref(), &ac.unwrap_or_default()),
        Commands::Edit { id, title, blocked_by, env, spec, priority, ac } => {
            cmd_edit(&id, title.as_deref(), blocked_by.as_deref(), env.as_deref(), spec.as_deref(), priority.as_deref(), &ac.unwrap_or_default())
        }
        Commands::Renumber { old_id, new_id, file } => cmd_renumber(&old_id, &new_id, file.as_deref()),
        Commands::SyncPlan { check, fix, strict, brief, plan } => {
            cmd_sync_plan(check, fix, strict, brief, plan.as_deref())
        }
        Commands::Validate { strict, brief } => cmd_validate(strict, brief),
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
            println!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"status\":\"{}\"}}",
                core::json_string_escape(t.id()),
                core::json_string_escape(t.title()),
                core::json_string_escape(t.status()),
            );
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

    // Push (with retry on rejection — race detection)
    match git::push(&repo)? {
        git::PushResult::Success => {
            println!("allocated {} (pushed — id claimed, status: open)", filename);
            Ok(0)
        }
        git::PushResult::Failed(stderr) => {
            bail!("push failed: {}", stderr);
        }
        git::PushResult::Rejected => {
            // Lost race: undo commit, pull, re-scan for next id
            git::undo_commit_keep_file(&repo)?;
            std::fs::remove_file(&path)?;
            git::pull_rebase(&repo)?;

            // Re-scan and retry with new id
            let names = ticket_filenames(&dir);
            let next_id = core::max_id(&names) + 1;
            let width = core::id_width(&names);
            let tid2 = format!("{:0>width$}", next_id, width = width);
            let filename2 = format!("{}-{}.md", tid2, slug);
            let path2 = dir.join(&filename2);
            let content2 = core::new_ticket_text(&tid2, title, blocked_by, env, spec, priority);
            std::fs::write(&path2, &content2)?;
            let rel_path2 = format!(".tickets/{}", filename2);
            git::add(&repo, &[&rel_path2])?;
            git::commit(&repo, &format!("chore(tickets): new {} {}", tid2, slug))?;

            match git::push(&repo)? {
                git::PushResult::Success => {
                    let note = format!(" (renumbered {}→{})", tid, tid2);
                    println!("allocated {}{} (pushed — id claimed, status: open)", filename2, note);
                    Ok(0)
                }
                git::PushResult::Failed(stderr) => {
                    bail!("push failed on retry: {}", stderr);
                }
                git::PushResult::Rejected => {
                    bail!("allocation failed after 2 attempts (push repeatedly rejected)");
                }
            }
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
        git::push_with_retry(&repo)?;
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
        git::push_with_retry(&repo)?;
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

// --- cmd_edit ---

fn cmd_edit(id: &str, title: Option<&str>, blocked_by: Option<&str>, env: Option<&str>, spec: Option<&str>, priority: Option<&str>, ac_indices: &[u32]) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let t = core::find_ticket(&corpus, id)?;
    let mut t = t.clone();
    let mut changed: Vec<&str> = Vec::new();

    if let Some(title_val) = title {
        if title_val.is_empty() {
            bail!("title is required and cannot be cleared");
        }
        t.set_field("title", &format!("\"{}\"", core::yaml_scalar_escape(title_val)));
        changed.push("title");
    }
    if let Some(deps_str) = blocked_by {
        let deps: Vec<&str> = deps_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let formatted = deps.iter().map(|d| format!("\"{}\"", core::yaml_scalar_escape(d))).collect::<Vec<_>>().join(", ");
        t.set_field("blocked_by", &format!("[{}]", formatted));
        changed.push("blocked_by");
    }
    if let Some(env_val) = env {
        if env_val.is_empty() {
            t.remove_field("env");
        } else {
            if !core::ENV_VALUES.contains(&env_val) {
                bail!("env must be one of {} (or '' to clear)", core::ENV_VALUES.join("/"));
            }
            t.set_field("env", env_val);
        }
        changed.push("env");
    }
    if let Some(spec_val) = spec {
        if spec_val.is_empty() {
            t.remove_field("spec");
        } else {
            t.set_field("spec", &format!("\"{}\"", core::yaml_scalar_escape(spec_val)));
        }
        changed.push("spec");
    }
    if let Some(prio_val) = priority {
        if prio_val.is_empty() {
            t.remove_field("priority");
        } else {
            if prio_val != "high" {
                bail!("priority must be 'high' (or '' to clear)");
            }
            t.set_field("priority", prio_val);
        }
        changed.push("priority");
    }
    if !ac_indices.is_empty() {
        t.body = flip_ac_boxes(&t.body, ac_indices);
        changed.push("ac");
    }

    if changed.is_empty() {
        bail!("nothing to edit — pass at least one field option");
    }

    t.write()?;
    let repo = git::repo_root(&dir)?;
    let rel_path = t.path.strip_prefix(&repo).unwrap_or(&t.path).to_string_lossy().replace('\\', "/");
    git::add(&repo, &[&rel_path])?;
    git::commit(&repo, &format!("chore(tickets): edit {} ({})", id, changed.join(", ")))?;
    if has_remote(&repo) {
        git::push_with_retry(&repo)?;
    } else {
        eprintln!("committed locally, no remote configured");
    }
    println!("edited {}: {}", t.path.file_name().unwrap().to_string_lossy(), changed.join(", "));
    Ok(0)
}

// --- cmd_validate ---

fn cmd_validate(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let mut findings: Vec<Finding> = Vec::new();

    let mut corpus: Vec<Ticket> = Vec::new();
    for entry in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "md") {
            match Ticket::parse(&entry.path()) {
                Ok(t) => corpus.push(t),
                Err(e) => findings.push(Finding {
                    file: entry.file_name().to_string_lossy().to_string(),
                    rule: "unparseable".to_string(),
                    message: e.to_string(),
                    severity: "error".to_string(),
                }),
            }
        }
    }

    let mut ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for t in &corpus {
        let name = t.path.file_name().unwrap().to_string_lossy().to_string();
        if !core::STATUS_VALUES.contains(&t.status()) {
            findings.push(Finding { file: name.clone(), rule: "bad-status".into(),
                message: format!("status {:?} not in {}", t.status(), core::STATUS_VALUES.join("/")), severity: "error".into() });
        }
        let env = t.env();
        if env != "either" && !core::ENV_VALUES.contains(&env) {
            findings.push(Finding { file: name.clone(), rule: "bad-env".into(),
                message: format!("env {:?} not in {}", env, core::ENV_VALUES.join("/")), severity: "error".into() });
        }
        if !name.starts_with(&format!("{}-", t.id())) {
            findings.push(Finding { file: name.clone(), rule: "id-filename-mismatch".into(),
                message: format!("id {:?} vs filename", t.id()), severity: "error".into() });
        }
        if let Some(existing) = ids.get(t.id()) {
            findings.push(Finding { file: name.clone(), rule: "duplicate-id".into(),
                message: format!("id {:?} also in {}", t.id(), existing), severity: "error".into() });
        }
        ids.entry(t.id().to_string()).or_insert(name.clone());
    }

    // Dangling blocked_by
    let known: std::collections::HashSet<&str> = corpus.iter().map(|t| t.id()).collect();
    for t in &corpus {
        for dep in t.blocked_by() {
            if !known.contains(dep.as_str()) {
                findings.push(Finding {
                    file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                    rule: "dangling-blocked-by".into(),
                    message: format!("ref {:?} has no ticket", dep), severity: "error".into(),
                });
            }
        }
    }

    // Unchecked ACs on done tickets
    let unchecked_re = Regex::new(r"- \[ \]").unwrap();
    for t in &corpus {
        if t.status() == "done" {
            let count = unchecked_re.find_iter(&t.body).count();
            if count > 0 {
                findings.push(Finding {
                    file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                    rule: "unchecked-acs-on-done".into(),
                    message: format!("{} unchecked box(es)", count), severity: "warning".into(),
                });
            }
        }
    }

    let errors: Vec<&Finding> = findings.iter().filter(|f| f.severity == "error").collect();
    let warnings: Vec<&Finding> = findings.iter().filter(|f| f.severity == "warning").collect();
    let status = if !errors.is_empty() || (strict && !warnings.is_empty()) { "fail" } else { "pass" };

    if brief {
        for f in &findings {
            println!("{}: {} [{}] {}", f.severity, f.file, f.rule, f.message);
        }
        println!("{} ({} finding(s))", status, findings.len());
    } else {
        println!("{{\"status\":\"{}\",\"findings\":[{}]}}",
            status,
            findings.iter().map(|f| format!(
                "{{\"file\":\"{}\",\"rule\":\"{}\",\"message\":\"{}\",\"severity\":\"{}\"}}",
                f.file, f.rule, f.message.replace('"', "\\\""), f.severity
            )).collect::<Vec<_>>().join(",")
        );
    }
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_sync_plan ---

fn cmd_sync_plan(check: bool, _fix: bool, strict: bool, brief: bool, plan_path: Option<&str>) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let plan = match plan_path {
        Some(p) => PathBuf::from(p),
        None => repo.join("docs").join("plan.md"),
    };
    if !plan.is_file() {
        bail!("no plan file at {}", plan.display());
    }

    let corpus = core::load_corpus(&dir)?;
    let corpus_map: std::collections::HashMap<&str, &Ticket> = corpus.iter().map(|t| (t.id(), t)).collect();
    let mut plan_text = std::fs::read_to_string(&plan)?;
    let plan_row_re = Regex::new(r"(?m)^\|\s*(\d+)\s*\|[^|]*\|([^|]*)\|\s*$").unwrap();

    let mut findings: Vec<Finding> = Vec::new();
    let mut fixed_count = 0;

    for caps in plan_row_re.captures_iter(&plan_text.clone()) {
        let tid = caps[1].trim();
        let status_cell = &caps[2];
        let plan_done = status_cell.contains("✅");

        if let Some(t) = corpus_map.get(tid) {
            let ticket_done = t.status() == "done";
            if plan_done != ticket_done {
                if _fix {
                    let new_status = if ticket_done { " ✅ done " } else { " open " };
                    let row_re = Regex::new(&format!(r"(?m)^(\|\s*{}\s*\|[^|]*\|)[^|]*(\|\s*)$", regex::escape(tid))).unwrap();
                    plan_text = row_re.replace(&plan_text, format!("${{1}}{}${{2}}", new_status)).to_string();
                    fixed_count += 1;
                } else {
                    findings.push(Finding {
                        file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                        rule: "plan-status-drift".into(),
                        message: format!("plan says {}, ticket is {}",
                            if plan_done { "done" } else { "not done" }, t.status()),
                        severity: "error".into(),
                    });
                }
            }
        }
    }

    // Missing plan rows
    let plan_ids: std::collections::HashSet<String> = plan_row_re.captures_iter(&plan_text)
        .map(|c| c[1].trim().to_string())
        .collect();
    for t in &corpus {
        if t.status() != "done" && !plan_ids.contains(t.id()) {
            findings.push(Finding {
                file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                rule: "missing-plan-row".into(),
                message: format!("{} ticket has no plan row", t.status()),
                severity: "warning".into(),
            });
        }
    }

    if _fix && fixed_count > 0 {
        std::fs::write(&plan, &plan_text)?;
    }

    let errors: Vec<&Finding> = findings.iter().filter(|f| f.severity == "error").collect();
    let warnings: Vec<&Finding> = findings.iter().filter(|f| f.severity == "warning").collect();
    let status = if !errors.is_empty() || (strict && !warnings.is_empty()) { "fail" } else { "pass" };

    if _fix {
        if !findings.is_empty() {
            print_findings(&findings, brief, status);
        } else if brief {
            println!("pass (fixed {}, 0 remaining)", fixed_count);
        } else {
            println!("{{\"status\":\"pass\",\"findings\":[],\"fixed\":{}}}", fixed_count);
        }
    } else {
        print_findings(&findings, brief, status);
    }
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_batch ---

fn cmd_batch(items: &[String], spec: Option<&str>, env: Option<&str>, priority: Option<&str>, blocked_by: &[String]) -> Result<i32> {
    // Parse items: "slug" or "slug:title"
    let mut parsed: Vec<(&str, String)> = Vec::new();
    for raw in items {
        let (slug, title) = match raw.split_once(':') {
            Some((s, t)) => (s, t.trim().to_string()),
            None => (raw.as_str(), raw.replace('-', " ")),
        };
        let slug_re = Regex::new(r"^[a-z0-9][a-z0-9-]*$").unwrap();
        if !slug_re.is_match(slug) {
            bail!("invalid slug {:?}", slug);
        }
        parsed.push((slug, title));
    }

    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);
    if remote {
        git::fetch(&repo)?;
    }

    let names = ticket_filenames(&dir);
    let base = core::max_id(&names) + 1;
    let width = core::id_width(&names);

    let mut files: Vec<String> = Vec::new();
    for (i, (slug, title)) in parsed.iter().enumerate() {
        let tid = format!("{:0>width$}", base + i as u64, width = width);
        let filename = format!("{}-{}.md", tid, slug);
        let path = dir.join(&filename);
        let content = core::new_ticket_text(&tid, title, blocked_by, env, spec, priority);
        std::fs::write(&path, &content)?;
        files.push(format!(".tickets/{}", filename));
    }

    // Stage all files
    for f in &files {
        git::add(&repo, &[f.as_str()])?;
    }
    let tids: Vec<String> = (0..parsed.len())
        .map(|i| format!("{:0>width$}", base + i as u64, width = width))
        .collect();
    git::commit(&repo, &format!("chore(tickets): batch {} ({})", tids.join(","), parsed.iter().map(|(s,_)| *s).collect::<Vec<_>>().join(", ")))?;

    if remote {
        git::push_with_retry(&repo)?;
    }

    for (i, (slug, _)) in parsed.iter().enumerate() {
        let tid = format!("{:0>width$}", base + i as u64, width = width);
        println!("allocated {}-{}.md (pushed — id claimed, status: open)", tid, slug);
    }
    Ok(0)
}

// --- cmd_renumber ---

fn cmd_renumber(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    let id_re = Regex::new(r"^\d+$").unwrap();
    if !id_re.is_match(new_id) {
        bail!("new id must be digits, got {:?}", new_id);
    }

    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let corpus = core::load_corpus(&dir)?;

    let holders: Vec<&Ticket> = corpus.iter().filter(|t| t.id() == old_id).collect();
    if holders.is_empty() {
        bail!("no ticket with id {:?}", old_id);
    }
    if holders.len() > 1 && file_hint.is_none() {
        let names: Vec<_> = holders.iter().map(|t| t.path.file_name().unwrap().to_string_lossy().to_string()).collect();
        bail!("id {:?} is held by {} files ({}) — pass --file", old_id, holders.len(), names.join(", "));
    }

    let src = if holders.len() == 1 {
        holders[0]
    } else {
        holders.iter().find(|t| t.path.file_name().unwrap().to_string_lossy() == file_hint.unwrap())
            .ok_or_else(|| anyhow::anyhow!("--file {:?} does not hold id {:?}", file_hint.unwrap(), old_id))?
    };

    if corpus.iter().any(|t| t.id() == new_id) {
        bail!("id {:?} already exists locally", new_id);
    }

    // Rename file
    let old_path = src.path.clone();
    let slug = old_path.file_name().unwrap().to_string_lossy()
        .splitn(2, '-').nth(1).unwrap_or("unknown.md").to_string();
    let new_path = dir.join(format!("{}-{}", new_id, slug));

    let mut t = src.clone();
    // Preserve quoting style for id field
    let old_raw = t.get("id").unwrap_or("");
    let new_val = if old_raw.starts_with('"') { format!("\"{}\"", new_id) } else { new_id.to_string() };
    t.set_field("id", &new_val);
    t.path = new_path.clone();
    t.write()?;
    std::fs::remove_file(&old_path)?;

    // Update inbound refs
    let mut refs_updated = 0;
    if holders.len() == 1 {
        for other in &corpus {
            if other.path == old_path { continue; }
            if other.blocked_by().contains(&old_id.to_string()) {
                let mut other = other.clone();
                let raw = other.get("blocked_by").unwrap_or("").to_string();
                let updated = raw.replace(old_id, new_id);
                other.set_field("blocked_by", &updated);
                other.write()?;
                refs_updated += 1;
            }
        }
    }

    // Commit (stage old removal + new file + any updated refs)
    let old_rel = old_path.strip_prefix(&repo).unwrap_or(&old_path).to_string_lossy().replace('\\', "/");
    let new_rel = new_path.strip_prefix(&repo).unwrap_or(&new_path).to_string_lossy().replace('\\', "/");
    git::git(&repo, &["add", &old_rel, &new_rel])?;
    // Stage any modified ref files
    for other in &corpus {
        if other.path == old_path { continue; }
        if other.blocked_by().contains(&old_id.to_string()) {
            let rel = other.path.strip_prefix(&repo).unwrap_or(&other.path).to_string_lossy().replace('\\', "/");
            git::add(&repo, &[&rel])?;
        }
    }
    git::commit(&repo, &format!("chore(tickets): renumber {} -> {}", old_id, new_id))?;
    if has_remote(&repo) {
        git::push_with_retry(&repo)?;
    } else {
        eprintln!("committed locally, no remote configured");
    }

    println!("renumbered {} -> {} ({} inbound ref(s) updated)", old_id, new_path.file_name().unwrap().to_string_lossy(), refs_updated);
    Ok(0)
}

// --- Helpers ---

#[derive(Debug)]
struct Finding {
    file: String,
    rule: String,
    message: String,
    severity: String,
}

fn print_findings(findings: &[Finding], brief: bool, status: &str) {
    if brief {
        for f in findings {
            println!("{}: {} [{}] {}", f.severity, f.file, f.rule, f.message);
        }
        println!("{} ({} finding(s))", status, findings.len());
    } else {
        println!("{{\"status\":\"{}\",\"findings\":[{}]}}",
            status,
            findings.iter().map(|f| format!(
                "{{\"file\":\"{}\",\"rule\":\"{}\",\"message\":\"{}\",\"severity\":\"{}\"}}",
                core::json_string_escape(&f.file),
                core::json_string_escape(&f.rule),
                core::json_string_escape(&f.message),
                core::json_string_escape(&f.severity),
            )).collect::<Vec<_>>().join(",")
        );
    }
}

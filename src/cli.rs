use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Result;
use clap::{Parser, Subcommand};
use regex::Regex;

use crate::core::{self, Status, Ticket};
use crate::findings::{self, Finding};
use crate::git;
use crate::transaction::{GitTransaction, PublishResult};

// --- Compiled regex patterns ---

static RE_UNCHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());
static RE_PLAN_ROW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\|\s*(\d+)\s*\|[^|]*\|([^|]*)\|\s*$").unwrap());

/// Domain-level failure: expected conditions like "ticket not found", "status conflict",
/// "validation drift". These exit with code 1.
/// Operational failures (I/O, git crash, parse errors) use anyhow directly and exit with code 2.
#[derive(Debug)]
struct DomainError(String);

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DomainError {}

/// Bail with a domain error (exit code 1).
macro_rules! domain_bail {
    ($($arg:tt)*) => {
        return Err(DomainError(format!($($arg)*)).into())
    };
}

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
    /// Dump all tickets as JSON Lines (one object per line)
    Query,
}

pub fn run() -> i32 {
    let start = std::time::Instant::now();
    let cli = Cli::parse();
    let cmd_name = command_name(&cli.command);

    // Debug mode setup
    let dbg = crate::telemetry::debug_mode();
    let session = crate::telemetry::generate_session_id();
    let project = std::env::current_dir()
        .ok()
        .and_then(|cwd| crate::git::repo_root(&cwd).ok())
        .map(|root| crate::telemetry::project_slug(&root))
        .unwrap_or_else(|| "unknown".to_string());
    crate::telemetry::debug_event(
        dbg,
        &session,
        &project,
        &format!("session={} project={} cmd={}", session, project, cmd_name),
    );

    let result = match cli.command {
        Commands::Ready { json } => cmd_ready(json),
        Commands::New {
            slug,
            title,
            spec,
            env,
            priority,
            blocked_by,
        } => cmd_new(
            &slug,
            title.as_deref(),
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            &blocked_by.unwrap_or_default(),
        ),
        Commands::Batch {
            items,
            spec,
            env,
            priority,
            blocked_by,
        } => cmd_batch(
            &items,
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            &blocked_by.unwrap_or_default(),
        ),
        Commands::Claim { id } => cmd_claim(&id),
        Commands::Close { id, note, ac } => {
            cmd_close(&id, note.as_deref(), &ac.unwrap_or_default())
        }
        Commands::Edit {
            id,
            title,
            blocked_by,
            env,
            spec,
            priority,
            ac,
        } => cmd_edit(
            &id,
            title.as_deref(),
            blocked_by.as_deref(),
            env.as_deref(),
            spec.as_deref(),
            priority.as_deref(),
            &ac.unwrap_or_default(),
        ),
        Commands::Renumber {
            old_id,
            new_id,
            file,
        } => cmd_renumber(&old_id, &new_id, file.as_deref()),
        Commands::SyncPlan {
            check,
            fix,
            strict,
            brief,
            plan,
        } => cmd_sync_plan(check, fix, strict, brief, plan.as_deref()),
        Commands::Validate { strict, brief } => cmd_validate(strict, brief),
        Commands::Query => cmd_query(),
    };
    let exit_code = match result {
        Ok(code) => code,
        Err(e) => {
            if e.downcast_ref::<DomainError>().is_some() {
                eprintln!("tkt: {}", e);
                1
            } else {
                eprintln!("tkt: crash: {}", e);
                2
            }
        }
    };

    // Record telemetry event (silently — never affects CLI behavior)
    record_telemetry(&cmd_name, exit_code, start.elapsed().as_millis() as u64);

    crate::telemetry::debug_event(
        dbg,
        &session,
        &project,
        &format!(
            "exit={} duration={:.1}s",
            exit_code,
            start.elapsed().as_secs_f64()
        ),
    );

    exit_code
}

// --- Telemetry helpers ---

fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::Ready { .. } => "ready",
        Commands::New { .. } => "new",
        Commands::Batch { .. } => "batch",
        Commands::Claim { .. } => "claim",
        Commands::Close { .. } => "close",
        Commands::Edit { .. } => "edit",
        Commands::Renumber { .. } => "renumber",
        Commands::SyncPlan { .. } => "sync-plan",
        Commands::Validate { .. } => "validate",
        Commands::Query => "query",
    }
    .to_string()
}

fn record_telemetry(cmd: &str, exit_code: i32, duration_ms: u64) {
    use crate::telemetry;

    let (consent, _) = telemetry::check_consent();
    if consent != telemetry::Consent::Enabled {
        return;
    }

    // Derive project slug from current repo (best effort)
    let project = std::env::current_dir()
        .ok()
        .and_then(|cwd| crate::git::repo_root(&cwd).ok())
        .map(|root| telemetry::project_slug(&root))
        .unwrap_or_else(|| "unknown".to_string());

    // Use a process-stable session ID (generated once per invocation via LazyLock)
    static SESSION: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(telemetry::generate_session_id);

    let event = telemetry::Event {
        ts: telemetry::iso_timestamp(),
        session: SESSION.clone(),
        project,
        cmd: cmd.to_string(),
        exit_code,
        duration_ms,
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: telemetry::os_string().to_string(),
        arch: telemetry::arch_string().to_string(),
    };

    telemetry::record_event(&event);
}

// --- Helpers ---

fn tickets_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let root = git::repo_root(&cwd)?;
    let dir = root.join(".tickets");
    if !dir.is_dir() {
        domain_bail!("no .tickets/ directory in {}", root.display());
    }
    Ok(dir)
}

fn has_remote(repo: &Path) -> bool {
    git::has_remote(repo).unwrap_or(false)
}

/// Preflight for mutation commands: resolves context, fetches, loads corpus, finds ticket.
/// Returns (repo, remote, ticket) ready for mutation.
fn preflight_mutation() -> Result<(PathBuf, bool, Vec<Ticket>)> {
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);

    let dbg = crate::telemetry::debug_mode();
    if remote {
        let fetch_start = std::time::Instant::now();
        git::fetch(&repo)?;
        crate::telemetry::debug_event(
            dbg,
            "",
            "",
            &format!(
                "git fetch origin ({:.1}s)",
                fetch_start.elapsed().as_secs_f64()
            ),
        );
    }
    let corpus = core::load_corpus(&dir)?;
    let open = corpus.iter().filter(|t| t.status == Status::Open).count();
    let wip = corpus
        .iter()
        .filter(|t| t.status == Status::InProgress)
        .count();
    let done = corpus.iter().filter(|t| t.status == Status::Done).count();
    crate::telemetry::debug_event(
        dbg,
        "",
        "",
        &format!(
            "corpus loaded: {} tickets ({} open, {} in_progress, {} done)",
            corpus.len(),
            open,
            wip,
            done
        ),
    );
    Ok((repo, remote, corpus))
}

/// Check the remote state of a ticket. Returns the remote ticket's status if available.
fn check_remote_status(repo: &Path, remote: bool, ticket: &Ticket) -> Option<String> {
    if !remote {
        return None;
    }
    let remote_path = format!(
        ".tickets/{}",
        ticket.path.file_name().unwrap().to_string_lossy()
    );
    if let Ok(content) = git::git(repo, &["show", &format!("origin/main:{}", remote_path)]) {
        if let Ok(remote_file) = core::TicketFile::parse_str(&content, &ticket.path) {
            return remote_file.get("status").map(|s| s.to_string());
        }
    }
    None
}

/// Commit and push a mutation. Handles local-only messaging.
fn commit_and_publish(repo: &Path, remote: bool, paths: &[&str], message: &str) -> Result<()> {
    let dbg = crate::telemetry::debug_mode();
    git::add(repo, paths)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git add {:?}", paths));
    git::commit(repo, message)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git commit {:?}", message));
    if remote {
        let push_start = std::time::Instant::now();
        git::push_with_retry(repo)?;
        crate::telemetry::debug_event(
            dbg,
            "",
            "",
            &format!("git push ({:.1}s)", push_start.elapsed().as_secs_f64()),
        );
    }
    Ok(())
}

// --- Commands ---

fn cmd_ready(json: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let front = core::frontier(&corpus);

    let dbg = crate::telemetry::debug_mode();
    let open = corpus.iter().filter(|t| t.status == Status::Open).count();
    let wip = corpus
        .iter()
        .filter(|t| t.status == Status::InProgress)
        .count();
    let done = corpus.iter().filter(|t| t.status == Status::Done).count();
    crate::telemetry::debug_event(
        dbg,
        "",
        "",
        &format!(
            "corpus loaded: {} tickets ({} open, {} in_progress, {} done), frontier: {}",
            corpus.len(),
            open,
            wip,
            done,
            front.len()
        ),
    );

    if json {
        for t in &front {
            let blocked_by: Vec<String> = t
                .blocked_by
                .iter()
                .map(|d| format!("\"{}\"", core::json_string_escape(d)))
                .collect();
            let mut fields = vec![
                format!("\"id\":\"{}\"", core::json_string_escape(&t.id)),
                format!("\"title\":\"{}\"", core::json_string_escape(&t.title)),
                format!(
                    "\"status\":\"{}\"",
                    core::json_string_escape(t.status.as_str())
                ),
                format!("\"blocked_by\":[{}]", blocked_by.join(",")),
            ];
            if t.env != core::Env::Either {
                fields.push(format!(
                    "\"env\":\"{}\"",
                    core::json_string_escape(t.env.as_str())
                ));
            }
            if let Some(priority) = t.priority {
                fields.push(format!(
                    "\"priority\":\"{}\"",
                    core::json_string_escape(priority.as_str())
                ));
            }
            if let Some(ref spec) = t.spec {
                fields.push(format!("\"spec\":\"{}\"", core::json_string_escape(spec)));
            }
            println!("{{{}}}", fields.join(","));
        }
    } else {
        for t in &front {
            let flag = if t.is_high_priority() { "  [HIGH]" } else { "" };
            println!("{}  {}{}", t.id, t.title, flag);
        }
        let wip: Vec<&Ticket> = corpus
            .iter()
            .filter(|t| t.status == Status::InProgress)
            .collect();
        if !wip.is_empty() {
            let ids: Vec<&str> = wip.iter().map(|t| t.id.as_str()).collect();
            println!("\nin progress (claimed elsewhere): {}", ids.join(", "));
        }
    }
    Ok(0)
}

fn cmd_new(
    slug: &str,
    title: Option<&str>,
    spec: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    blocked_by: &[String],
) -> Result<i32> {
    // Validate inputs
    if let Err(e) = core::validate::validate_slug(slug) {
        domain_bail!("{}", e);
    }

    let title_owned = slug.replace('-', " ");
    let title = title.unwrap_or(&title_owned);
    if let Err(e) = core::validate::validate_free_text(title, "title", 200) {
        domain_bail!("{}", e);
    }
    if let Some(s) = spec {
        if let Err(e) = core::validate::validate_free_text(s, "spec", 100) {
            domain_bail!("{}", e);
        }
    }
    if let Some(e) = env {
        if let Err(err) = core::validate::validate_env(e) {
            domain_bail!("{}", err);
        }
    }
    if let Some(p) = priority {
        if let Err(err) = core::validate::validate_priority(p) {
            domain_bail!("{}", err);
        }
    }
    for dep in blocked_by {
        if let Err(e) = core::validate::validate_id(dep) {
            domain_bail!("--blocked-by: {}", e);
        }
    }
    let dir = tickets_dir()?;
    let txn = GitTransaction::new(&dir)?;

    // Scan and allocate
    let names = txn.scan_names();
    let (tid, _width) = GitTransaction::next_id(&names);

    // Check for self-dependency with the allocated ID
    let dep_strs: Vec<&str> = blocked_by.iter().map(|s| s.as_str()).collect();
    if let Err(e) = core::validate::validate_no_self_dep(&tid, &dep_strs) {
        domain_bail!("{}", e);
    }

    let filename = format!("{}-{}.md", tid, slug);
    let path = dir.join(&filename);
    let content = core::new_ticket_text(&tid, title, blocked_by, env, spec, priority);
    std::fs::write(&path, &content)?;

    let rel_path = format!(".tickets/{}", filename);
    git::add(&txn.repo, &[&rel_path])?;
    git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid, slug))?;

    match txn.try_push()? {
        PublishResult::Done(outcome) => {
            let msg = match outcome {
                crate::transaction::PublishOutcome::LocalOnly => {
                    format!(
                        "created {} (no remote — id claim is local only, status: open)",
                        filename
                    )
                }
                _ => format!("allocated {} (pushed — id claimed, status: open)", filename),
            };
            println!("{}", msg);
            Ok(0)
        }
        PublishResult::NeedsRetry => {
            // Rescan after rebase and retry with new id
            let names = txn.scan_names();
            let (tid2, _width) = GitTransaction::next_id(&names);
            let filename2 = format!("{}-{}.md", tid2, slug);
            let path2 = dir.join(&filename2);
            let content2 = core::new_ticket_text(&tid2, title, blocked_by, env, spec, priority);
            std::fs::write(&path2, &content2)?;
            let rel_path2 = format!(".tickets/{}", filename2);
            git::add(&txn.repo, &[&rel_path2])?;
            git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid2, slug))?;

            txn.push_retry()?;
            println!(
                "allocated {} (pushed — id claimed, status: open, renumbered {}→{})",
                filename2, tid, tid2
            );
            Ok(0)
        }
    }
}

fn cmd_claim(id: &str) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    // Check remote state
    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status != "open" {
            domain_bail!("{} is {}, not open (updated on remote)", id, remote_status);
        }
    }
    if t.status != Status::Open {
        domain_bail!("{} is {}, not open", t.id, t.status.as_str());
    }

    let mut file = t.file.clone();
    file.set_field("status", "in_progress");
    file.write()?;

    let rel_path = file
        .path
        .strip_prefix(&repo)
        .unwrap_or(&file.path)
        .to_string_lossy()
        .replace('\\', "/");
    commit_and_publish(
        &repo,
        remote,
        &[&rel_path],
        &format!("chore(tickets): claim {}", id),
    )?;

    println!(
        "claimed {} (in_progress pushed)",
        file.path.file_name().unwrap().to_string_lossy()
    );
    Ok(0)
}

fn cmd_close(id: &str, note: Option<&str>, ac_indices: &[u32]) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    // Check remote state
    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status == "done" {
            domain_bail!("{} is already done (updated on remote)", id);
        }
    }
    if t.status == Status::Done {
        domain_bail!("{} is already done", t.id);
    }

    let mut file = t.file.clone();
    file.set_field("status", "done");

    // Append Resolution section if not present
    if !file.body.contains("## Resolution") {
        let date = chrono_date();
        let resolution = note.unwrap_or("TBD");
        file.body = format!(
            "{}\n\n## Resolution ({})\n\n{}\n",
            file.body.trim_end(),
            date,
            resolution
        );
    }

    // Flip AC boxes if specified
    if !ac_indices.is_empty() {
        file.body = flip_ac_boxes(&file.body, ac_indices);
    }

    file.write()?;

    // Warn about unchecked ACs
    let unchecked = RE_UNCHECKED_AC.find_iter(&file.body).count();
    if unchecked > 0 {
        eprintln!(
            "warning: {} unchecked acceptance box(es) — fill in before trusting history",
            unchecked
        );
    }

    let rel_path = file
        .path
        .strip_prefix(&repo)
        .unwrap_or(&file.path)
        .to_string_lossy()
        .replace('\\', "/");
    commit_and_publish(
        &repo,
        remote,
        &[&rel_path],
        &format!("chore(tickets): close {}", id),
    )?;

    let verb = if note.is_some() {
        "written"
    } else {
        "stub appended"
    };
    println!(
        "closed {} (dated Resolution {})",
        file.path.file_name().unwrap().to_string_lossy(),
        verb
    );
    Ok(0)
}

// --- Utilities ---

fn chrono_date() -> String {
    // Pure-Rust ISO 8601 date (YYYY-MM-DD) without external dependencies.
    // Uses SystemTime → days since Unix epoch → civil date conversion.
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Civil date from Unix timestamp (days since 1970-01-01)
    let days = secs.div_euclid(86400) as i32;

    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn flip_ac_boxes(body: &str, indices: &[u32]) -> String {
    let mut result = body.to_string();
    let matches: Vec<_> = RE_UNCHECKED_AC.find_iter(body).collect();

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

fn cmd_edit(
    id: &str,
    title: Option<&str>,
    blocked_by: Option<&str>,
    env: Option<&str>,
    spec: Option<&str>,
    priority: Option<&str>,
    ac_indices: &[u32],
) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    // Check remote state (don't block local-only tickets)
    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status == "done" {
            domain_bail!("ticket {} was closed on remote", id);
        }
    }

    let mut file = t.file.clone();
    let mut changed: Vec<&str> = Vec::new();

    if let Some(title_val) = title {
        if title_val.is_empty() {
            domain_bail!("title is required and cannot be cleared");
        }
        if let Err(e) = core::validate::validate_free_text(title_val, "title", 200) {
            domain_bail!("{}", e);
        }
        file.set_field(
            "title",
            &format!("\"{}\"", core::yaml_scalar_escape(title_val)),
        );
        changed.push("title");
    }
    if let Some(deps_str) = blocked_by {
        let deps: Vec<&str> = deps_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        for dep in &deps {
            if let Err(e) = core::validate::validate_id(dep) {
                domain_bail!("--blocked-by: {}", e);
            }
        }
        if let Err(e) = core::validate::validate_no_self_dep(id, &deps) {
            domain_bail!("{}", e);
        }
        let formatted = deps
            .iter()
            .map(|d| format!("\"{}\"", core::yaml_scalar_escape(d)))
            .collect::<Vec<_>>()
            .join(", ");
        file.set_field("blocked_by", &format!("[{}]", formatted));
        changed.push("blocked_by");
    }
    if let Some(env_val) = env {
        if env_val.is_empty() {
            file.remove_field("env");
        } else {
            if !core::ENV_VALUES.contains(&env_val) {
                domain_bail!(
                    "env must be one of {} (or '' to clear)",
                    core::ENV_VALUES.join("/")
                );
            }
            file.set_field("env", env_val);
        }
        changed.push("env");
    }
    if let Some(spec_val) = spec {
        if spec_val.is_empty() {
            file.remove_field("spec");
        } else {
            if let Err(e) = core::validate::validate_free_text(spec_val, "spec", 100) {
                domain_bail!("{}", e);
            }
            file.set_field(
                "spec",
                &format!("\"{}\"", core::yaml_scalar_escape(spec_val)),
            );
        }
        changed.push("spec");
    }
    if let Some(prio_val) = priority {
        if prio_val.is_empty() {
            file.remove_field("priority");
        } else {
            if prio_val != "high" {
                domain_bail!("priority must be 'high' (or '' to clear)");
            }
            file.set_field("priority", prio_val);
        }
        changed.push("priority");
    }
    if !ac_indices.is_empty() {
        file.body = flip_ac_boxes(&file.body, ac_indices);
        changed.push("ac");
    }

    if changed.is_empty() {
        domain_bail!("nothing to edit — pass at least one field option");
    }

    file.write()?;
    let rel_path = file
        .path
        .strip_prefix(&repo)
        .unwrap_or(&file.path)
        .to_string_lossy()
        .replace('\\', "/");
    commit_and_publish(
        &repo,
        remote,
        &[&rel_path],
        &format!("chore(tickets): edit {} ({})", id, changed.join(", ")),
    )?;
    println!(
        "edited {}: {}",
        file.path.file_name().unwrap().to_string_lossy(),
        changed.join(", ")
    );
    Ok(0)
}

// --- cmd_validate ---

fn cmd_validate(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let mut all_findings: Vec<Finding> = Vec::new();

    // Load corpus, collecting parse errors
    let mut corpus: Vec<Ticket> = Vec::new();
    for entry in std::fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "md") {
            match Ticket::parse(&entry.path()) {
                Ok(t) => corpus.push(t),
                Err(e) => all_findings.push(Finding {
                    file: entry.file_name().to_string_lossy().to_string(),
                    rule: "unparseable".to_string(),
                    message: e.to_string(),
                    severity: "error".to_string(),
                }),
            }
        }
    }

    // Run all validation rules
    all_findings.extend(findings::check_status(&corpus));
    all_findings.extend(findings::check_env(&corpus));
    all_findings.extend(findings::check_id_filename(&corpus));
    all_findings.extend(findings::check_duplicate_ids(&corpus));
    all_findings.extend(findings::check_dangling_deps(&corpus));
    all_findings.extend(findings::check_cycles(&corpus));
    all_findings.extend(findings::check_unchecked_acs(&corpus));

    let status = findings::status_from_findings(&all_findings, strict);
    findings::print_findings(&all_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_sync_plan ---

fn cmd_sync_plan(
    _check: bool,
    _fix: bool,
    strict: bool,
    brief: bool,
    plan_path: Option<&str>,
) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let plan = match plan_path {
        Some(p) => PathBuf::from(p),
        None => repo.join("docs").join("plan.md"),
    };
    if !plan.is_file() {
        domain_bail!("no plan file at {}", plan.display());
    }

    let corpus = core::load_corpus(&dir)?;
    let corpus_map: std::collections::HashMap<&str, &Ticket> =
        corpus.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut plan_text = std::fs::read_to_string(&plan)?;

    let mut findings: Vec<Finding> = Vec::new();
    let mut fixed_count = 0;

    for caps in RE_PLAN_ROW.captures_iter(&plan_text.clone()) {
        let tid = caps[1].trim();
        let status_cell = &caps[2];
        let plan_done = status_cell.contains("✅");

        if let Some(t) = corpus_map.get(tid) {
            let ticket_done = t.status == Status::Done;
            if plan_done != ticket_done {
                if _fix {
                    let new_status = if ticket_done { " ✅ done " } else { " open " };
                    let row_re = Regex::new(&format!(
                        r"(?m)^(\|\s*{}\s*\|[^|]*\|)[^|]*(\|\s*)$",
                        regex::escape(tid)
                    ))
                    .unwrap();
                    plan_text = row_re
                        .replace(&plan_text, format!("${{1}}{}${{2}}", new_status))
                        .to_string();
                    fixed_count += 1;
                } else {
                    findings.push(Finding {
                        file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                        rule: "plan-status-drift".into(),
                        message: format!(
                            "plan says {}, ticket is {}",
                            if plan_done { "done" } else { "not done" },
                            t.status.as_str()
                        ),
                        severity: "error".into(),
                    });
                }
            }
        }
    }

    // Missing plan rows
    let plan_ids: std::collections::HashSet<String> = RE_PLAN_ROW
        .captures_iter(&plan_text)
        .map(|c| c[1].trim().to_string())
        .collect();
    for t in &corpus {
        if t.status != Status::Done && !plan_ids.contains(&*t.id) {
            findings.push(Finding {
                file: t.path.file_name().unwrap().to_string_lossy().to_string(),
                rule: "missing-plan-row".into(),
                message: format!("{} ticket has no plan row", t.status.as_str()),
                severity: "warning".into(),
            });
        }
    }

    if _fix && fixed_count > 0 {
        std::fs::write(&plan, &plan_text)?;
    }

    let errors: Vec<&Finding> = findings.iter().filter(|f| f.severity == "error").collect();
    let warnings: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.severity == "warning")
        .collect();
    let status = if !errors.is_empty() || (strict && !warnings.is_empty()) {
        "fail"
    } else {
        "pass"
    };

    if _fix {
        if !findings.is_empty() {
            findings::print_findings(&findings, brief, status);
        } else if brief {
            println!("pass (fixed {}, 0 remaining)", fixed_count);
        } else {
            println!(
                "{{\"status\":\"pass\",\"findings\":[],\"fixed\":{}}}",
                fixed_count
            );
        }
    } else {
        findings::print_findings(&findings, brief, status);
    }
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_query ---

fn cmd_query() -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;

    for t in &corpus {
        let blocked_by: Vec<String> = t
            .blocked_by
            .iter()
            .map(|d| format!("\"{}\"", core::json_string_escape(d)))
            .collect();

        let mut fields = vec![
            format!("\"id\":\"{}\"", core::json_string_escape(&t.id)),
            format!("\"title\":\"{}\"", core::json_string_escape(&t.title)),
            format!(
                "\"status\":\"{}\"",
                core::json_string_escape(t.status.as_str())
            ),
            format!("\"blocked_by\":[{}]", blocked_by.join(",")),
        ];

        // Optional fields — include only when present
        if t.env != core::Env::Either {
            fields.push(format!(
                "\"env\":\"{}\"",
                core::json_string_escape(t.env.as_str())
            ));
        }
        if let Some(priority) = t.priority {
            fields.push(format!(
                "\"priority\":\"{}\"",
                core::json_string_escape(priority.as_str())
            ));
        }
        if let Some(ref spec) = t.spec {
            fields.push(format!("\"spec\":\"{}\"", core::json_string_escape(spec)));
        }

        println!("{{{}}}", fields.join(","));
    }
    Ok(0)
}

// --- cmd_batch ---

fn cmd_batch(
    items: &[String],
    spec: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    blocked_by: &[String],
) -> Result<i32> {
    // Validate shared options
    if let Some(s) = spec {
        if let Err(e) = core::validate::validate_free_text(s, "spec", 100) {
            domain_bail!("{}", e);
        }
    }
    if let Some(e) = env {
        if let Err(err) = core::validate::validate_env(e) {
            domain_bail!("{}", err);
        }
    }
    if let Some(p) = priority {
        if let Err(err) = core::validate::validate_priority(p) {
            domain_bail!("{}", err);
        }
    }
    for dep in blocked_by {
        if let Err(e) = core::validate::validate_id(dep) {
            domain_bail!("--blocked-by: {}", e);
        }
    }

    // Parse items: "slug" or "slug:title"
    let mut parsed: Vec<(&str, String)> = Vec::new();
    for raw in items {
        let (slug, title) = match raw.split_once(':') {
            Some((s, t)) => (s, t.trim().to_string()),
            None => (raw.as_str(), raw.replace('-', " ")),
        };
        if let Err(e) = core::validate::validate_slug(slug) {
            domain_bail!("{}", e);
        }
        if let Err(e) = core::validate::validate_free_text(&title, "title", 200) {
            domain_bail!("{}", e);
        }
        parsed.push((slug, title));
    }

    // Check for duplicate slugs
    let slugs: Vec<&str> = parsed.iter().map(|(s, _)| *s).collect();
    if let Err(e) = core::validate::validate_no_duplicate_slugs(&slugs) {
        domain_bail!("{}", e);
    }

    let dir = tickets_dir()?;
    let txn = GitTransaction::new(&dir)?;
    let names = txn.scan_names();

    let allocate_and_commit =
        |names: &[String], parsed: &[(&str, String)]| -> Result<(u64, usize)> {
            let base = core::max_id(names) + 1;
            let width = core::id_width(names);
            let mut files: Vec<String> = Vec::new();
            for (i, (slug, title)) in parsed.iter().enumerate() {
                let tid = format!("{:0>width$}", base + i as u64, width = width);
                let filename = format!("{}-{}.md", tid, slug);
                let path = txn.dir.join(&filename);
                let content = core::new_ticket_text(&tid, title, blocked_by, env, spec, priority);
                std::fs::write(&path, &content)?;
                files.push(format!(".tickets/{}", filename));
            }
            for f in &files {
                git::add(&txn.repo, &[f.as_str()])?;
            }
            let tids: Vec<String> = (0..parsed.len())
                .map(|i| format!("{:0>width$}", base + i as u64, width = width))
                .collect();
            git::commit(
                &txn.repo,
                &format!(
                    "chore(tickets): batch {} ({})",
                    tids.join(","),
                    parsed
                        .iter()
                        .map(|(s, _)| *s)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )?;
            Ok((base, width))
        };

    let (mut base, mut width) = allocate_and_commit(&names, &parsed)?;

    match txn.try_push()? {
        PublishResult::Done(_) => {}
        PublishResult::NeedsRetry => {
            let names = txn.scan_names();
            let result = allocate_and_commit(&names, &parsed)?;
            base = result.0;
            width = result.1;
            txn.push_retry()?;
        }
    }

    for (i, (slug, _)) in parsed.iter().enumerate() {
        let tid = format!("{:0>width$}", base + i as u64, width = width);
        println!(
            "allocated {}-{}.md (pushed — id claimed, status: open)",
            tid, slug
        );
    }
    Ok(0)
}

// --- cmd_renumber ---

fn cmd_renumber(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    if let Err(e) = core::validate::validate_id(new_id) {
        domain_bail!("new id: {}", e);
    }

    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let corpus = core::load_corpus(&dir)?;

    let holders: Vec<&Ticket> = corpus.iter().filter(|t| t.id == old_id).collect();
    if holders.is_empty() {
        domain_bail!("no ticket with id {:?}", old_id);
    }
    if holders.len() > 1 && file_hint.is_none() {
        let names: Vec<_> = holders
            .iter()
            .map(|t| t.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        domain_bail!(
            "id {:?} is held by {} files ({}) — pass --file",
            old_id,
            holders.len(),
            names.join(", ")
        );
    }

    let src = if holders.len() == 1 {
        holders[0]
    } else {
        holders
            .iter()
            .find(|t| t.path.file_name().unwrap().to_string_lossy() == file_hint.unwrap())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--file {:?} does not hold id {:?}",
                    file_hint.unwrap(),
                    old_id
                )
            })?
    };

    if corpus.iter().any(|t| t.id == new_id) {
        domain_bail!("id {:?} already exists locally", new_id);
    }

    // Rename file
    let old_path = src.path.clone();
    let slug = old_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .split_once('-')
        .map(|x| x.1)
        .unwrap_or("unknown.md")
        .to_string();
    let new_path = dir.join(format!("{}-{}", new_id, slug));

    let mut file = src.file.clone();
    // Preserve quoting style for id field
    let old_raw = file.get("id").unwrap_or("");
    let new_val = if old_raw.starts_with('"') {
        format!("\"{}\"", new_id)
    } else {
        new_id.to_string()
    };
    file.set_field("id", &new_val);
    file.path = new_path.clone();
    file.write()?;
    std::fs::remove_file(&old_path)?;

    // Update inbound refs
    let mut refs_updated = 0;
    if holders.len() == 1 {
        for other in &corpus {
            if other.path == old_path {
                continue;
            }
            if other.blocked_by.contains(&old_id.to_string()) {
                let mut other_file = other.file.clone();
                let new_deps: Vec<String> = other
                    .blocked_by
                    .iter()
                    .map(|d| {
                        if d == old_id {
                            new_id.to_string()
                        } else {
                            d.clone()
                        }
                    })
                    .collect();
                let formatted = new_deps
                    .iter()
                    .map(|d| format!("\"{}\"", core::yaml_scalar_escape(d)))
                    .collect::<Vec<_>>()
                    .join(", ");
                other_file.set_field("blocked_by", &format!("[{}]", formatted));
                other_file.write()?;
                refs_updated += 1;
            }
        }
    }

    // Commit (stage old removal + new file + any updated refs)
    let old_rel = old_path
        .strip_prefix(&repo)
        .unwrap_or(&old_path)
        .to_string_lossy()
        .replace('\\', "/");
    let new_rel = new_path
        .strip_prefix(&repo)
        .unwrap_or(&new_path)
        .to_string_lossy()
        .replace('\\', "/");
    git::git(&repo, &["add", &old_rel, &new_rel])?;
    // Stage any modified ref files
    for other in &corpus {
        if other.path == old_path {
            continue;
        }
        if other.blocked_by.contains(&old_id.to_string()) {
            let rel = other
                .path
                .strip_prefix(&repo)
                .unwrap_or(&other.path)
                .to_string_lossy()
                .replace('\\', "/");
            git::add(&repo, &[&rel])?;
        }
    }
    git::commit(
        &repo,
        &format!("chore(tickets): renumber {} -> {}", old_id, new_id),
    )?;
    if has_remote(&repo) {
        git::push_with_retry(&repo)?;
    } else {
        eprintln!("committed locally, no remote configured");
    }

    println!(
        "renumbered {} -> {} ({} inbound ref(s) updated)",
        old_id,
        new_path.file_name().unwrap().to_string_lossy(),
        refs_updated
    );
    Ok(0)
}

// --- Helpers ---

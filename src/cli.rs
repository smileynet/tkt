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
static RE_CHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[x\]").unwrap());
static RE_PLAN_ROW: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\|\s*(\d+)\s*\|[^|]*\|([^|]*)\|\s*$").unwrap());

/// Regex for extracting ID and slug from a ticket filename: "01-my-slug.md"
static RE_TICKET_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)-(.+)\.md$").unwrap());

/// Global quiet flag — set once at startup, read by command functions.
static QUIET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
#[command(
    name = "tkt",
    about = "Git-native ticket CLI (.tickets/ contract)",
    version
)]
struct Cli {
    /// Suppress confirmations, emit only essential data
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Color output: always, never, or auto (default: auto)
    #[arg(long, global = true)]
    color: Option<String>,

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
        #[arg(long)]
        status: Option<String>,
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
        #[arg(long)]
        status: Option<String>,
        #[arg(long, value_delimiter = ',')]
        blocked_by: Option<Vec<String>>,
    },
    /// Mark open ticket in_progress (pushed = visible WIP)
    Claim { id: String },
    /// Mark done, append dated Resolution stub, warn unchecked ACs
    /// Mark done, append dated Resolution, warn unchecked ACs
    Close {
        id: String,
        /// Resolution text (what was done)
        #[arg(long)]
        note: Option<String>,
        /// Resolution text (alias for --note, clearer naming)
        #[arg(long, conflicts_with = "note")]
        resolution: Option<String>,
        /// Check specific AC boxes (1-based indices)
        #[arg(long, value_delimiter = ',')]
        ac: Option<Vec<u32>>,
        /// Check all AC boxes at once
        #[arg(long)]
        check_all: bool,
        /// Force close even if all ACs are unchecked
        #[arg(long)]
        force: bool,
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
        #[arg(long)]
        status: Option<String>,
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
    Query {
        /// Filter by status (open, in_progress, done, backlog)
        #[arg(long)]
        status: Option<String>,
        /// Filter by priority (urgent, high, medium, low)
        #[arg(long)]
        priority: Option<String>,
    },
    /// Show blocked tickets with their blockers
    Blocked,
    /// Machine-readable feature manifest for agent/automation discovery
    Capabilities,
    /// Resolve ID collisions with upstream (origin always wins)
    Rebase {
        /// Show what would be renumbered without changing anything
        #[arg(long)]
        dry_run: bool,
    },
    /// Batch closure quality check (unchecked ACs, TBD resolutions, stale WIP)
    Audit {
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        brief: bool,
    },
    /// Manage user-level configuration (~/.config/tkt/config.toml)
    Config {
        /// Set a config value: tkt config --set key=value
        #[arg(long)]
        set: Option<String>,
        /// Get a config value: tkt config --get key
        #[arg(long)]
        get: Option<String>,
        /// Remove a config value (revert to default): tkt config --unset key
        #[arg(long)]
        unset: Option<String>,
        /// List all config values with sources
        #[arg(long)]
        list: bool,
        /// Show effective project config (.tickets/config.toml) with sources
        #[arg(long)]
        show: bool,
    },
    /// Manage telemetry consent and inspect collected data
    Telemetry {
        /// Enable telemetry (opt in to local recording)
        #[arg(long)]
        enable: bool,
        /// Disable telemetry (opt out)
        #[arg(long)]
        disable: bool,
        /// Show current telemetry status and storage summary
        #[arg(long)]
        status: bool,
        /// Print recent telemetry events
        #[arg(long)]
        show: bool,
        /// Delete all local telemetry data
        #[arg(long)]
        clear: bool,
    },
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

    // Store quiet flag for access by command functions
    QUIET.store(cli.quiet, std::sync::atomic::Ordering::Relaxed);

    // Initialize color mode from --color flag and environment
    crate::color::init(cli.color.as_deref());

    let result = match cli.command {
        Commands::Ready { json } => crate::commands::ready::run(json),
        Commands::New {
            slug,
            title,
            spec,
            env,
            priority,
            status,
            blocked_by,
        } => crate::commands::new::run(
            &slug,
            title.as_deref(),
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &blocked_by.unwrap_or_default(),
        ),
        Commands::Batch {
            items,
            spec,
            env,
            priority,
            status,
            blocked_by,
        } => crate::commands::batch::run(
            &items,
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &blocked_by.unwrap_or_default(),
        ),
        Commands::Claim { id } => crate::commands::claim::run(&id),
        Commands::Close {
            id,
            note,
            resolution,
            ac,
            check_all,
            force,
        } => {
            let text = resolution.or(note);
            crate::commands::close::run(
                &id,
                text.as_deref(),
                &ac.unwrap_or_default(),
                check_all,
                force,
            )
        }
        Commands::Edit {
            id,
            title,
            blocked_by,
            env,
            spec,
            priority,
            status,
            ac,
        } => crate::commands::edit::run(
            &id,
            title.as_deref(),
            blocked_by.as_deref(),
            env.as_deref(),
            spec.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &ac.unwrap_or_default(),
        ),
        Commands::Renumber {
            old_id,
            new_id,
            file,
        } => crate::commands::renumber::run(&old_id, &new_id, file.as_deref()),
        Commands::SyncPlan {
            check,
            fix,
            strict,
            brief,
            plan,
        } => crate::commands::sync_plan::run(check, fix, strict, brief, plan.as_deref()),
        Commands::Validate { strict, brief } => crate::commands::validate::run(strict, brief),
        Commands::Query { status, priority } => crate::commands::query::run(status.as_deref(), priority.as_deref()),
        Commands::Blocked => crate::commands::blocked::run(),
        Commands::Capabilities => crate::commands::capabilities::run(),
        Commands::Rebase { dry_run } => crate::commands::rebase::run(dry_run),
        Commands::Audit { strict, brief } => crate::commands::audit::run(strict, brief),
        Commands::Config {
            set,
            get,
            unset,
            list,
            show,
        } => crate::commands::config::run(set.as_deref(), get.as_deref(), unset.as_deref(), list, show),
        Commands::Telemetry {
            enable,
            disable,
            status,
            show,
            clear,
        } => crate::commands::telemetry::run(enable, disable, status, show, clear),
    };
    let exit_code = match result {
        Ok(code) => code,
        Err(e) => {
            if e.downcast_ref::<DomainError>().is_some() {
                eprintln!("tkt: {} {}", crate::color::sym_err(), e);
                1
            } else {
                eprintln!("tkt: {} crash: {}", crate::color::sym_err(), e);
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
        Commands::Query { .. } => "query",
        Commands::Blocked => "blocked",
        Commands::Capabilities => "capabilities",
        Commands::Rebase { .. } => "rebase",
        Commands::Audit { .. } => "audit",
        Commands::Config { .. } => "config",
        Commands::Telemetry { .. } => "telemetry",
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

/// Load project-level config from .tickets/config.toml.
/// Warns on unknown keys. Returns defaults if file is missing.
fn project_config(tickets_dir: &Path) -> crate::config::ProjectConfig {
    let cfg = crate::config::ProjectConfig::load(tickets_dir);
    for key in &cfg.unknown_keys {
        eprintln!(
            "warning: unknown config key {:?} in .tickets/config.toml",
            key
        );
    }
    cfg
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
/// Respects project config push.enabled — if false, skips push even when remote exists.
fn commit_and_publish(repo: &Path, remote: bool, paths: &[&str], message: &str) -> Result<()> {
    let dbg = crate::telemetry::debug_mode();
    git::add(repo, paths)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git add {:?}", paths));
    git::commit(repo, message)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git commit {:?}", message));

    // Check project config push.enabled
    let should_push = if remote {
        let dir = repo.join(".tickets");
        if dir.is_dir() {
            let pcfg = crate::config::ProjectConfig::load(&dir);
            pcfg.push_enabled
        } else {
            true
        }
    } else {
        false
    };

    if should_push {
        let push_start = std::time::Instant::now();
        git::push_with_retry(repo)?;
        crate::telemetry::debug_event(
            dbg,
            "",
            "",
            &format!("git push ({:.1}s)", push_start.elapsed().as_secs_f64()),
        );
    } else if remote {
        crate::telemetry::debug_event(dbg, "", "", "push skipped (push.enabled=false)");
    }
    Ok(())
}

// --- Commands ---

fn cmd_ready(json: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    let corpus = core::load_corpus(&dir)?;
    let front = core::frontier_with_default_env(&corpus, &pcfg.ready_default_env);

    let dbg = crate::telemetry::debug_mode();
    let open = corpus.iter().filter(|t| t.status == Status::Open).count();
    let wip_count = corpus
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
            wip_count,
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
    } else if is_quiet() {
        // Quiet mode: one ID per line, no headers
        for t in &front {
            println!("{}", t.id);
        }
    } else {
        // Human mode with hierarchy
        if front.is_empty() {
            println!("No tickets ready.");
        } else {
            println!("Ready ({}):", front.len());
            for t in &front {
                let flag = match t.priority {
                    Some(core::Priority::Urgent) => "  [URGENT]",
                    Some(core::Priority::High) => "  [HIGH]",
                    Some(core::Priority::Low) => "  [low]",
                    _ => "",
                };
                println!("  {}  {}{}", t.id, t.title, flag);
            }
        }

        let wip: Vec<&Ticket> = corpus
            .iter()
            .filter(|t| t.status == Status::InProgress)
            .collect();
        if !wip.is_empty() {
            println!("\nIn progress ({}):", wip.len());
            for t in &wip {
                println!("  {}  {}", t.id, t.title);
            }
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
    status: Option<&str>,
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
    if let Some(s) = status {
        if let Err(err) = core::validate::validate_status(s) {
            domain_bail!("{}", err);
        }
    }
    for dep in blocked_by {
        if let Err(e) = core::validate::validate_id(dep) {
            domain_bail!("--blocked-by: {}", e);
        }
    }
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);

    // Apply project config default_priority if user didn't specify one
    let priority = if priority.is_none() && !pcfg.new_default_priority.is_empty() {
        Some(pcfg.new_default_priority.as_str())
    } else {
        priority
    };

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
    let content = core::new_ticket_text(&tid, title, blocked_by, env, spec, priority, status);
    std::fs::write(&path, &content)?;

    let rel_path = format!(".tickets/{}", filename);
    git::add(&txn.repo, &[&rel_path])?;
    git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid, slug))?;

    match txn.try_push()? {
        PublishResult::Done(outcome) => {
            if is_quiet() {
                println!("{}", tid);
            } else {
                let detail = match outcome {
                    crate::transaction::PublishOutcome::LocalOnly => "local only",
                    _ => "pushed",
                };
                println!("{}", success_msg("created", &tid, slug, detail));
            }
            Ok(0)
        }
        PublishResult::NeedsRetry => {
            // Rescan after rebase and retry with new id
            let names = txn.scan_names();
            let (tid2, _width) = GitTransaction::next_id(&names);
            let filename2 = format!("{}-{}.md", tid2, slug);
            let path2 = dir.join(&filename2);
            let content2 =
                core::new_ticket_text(&tid2, title, blocked_by, env, spec, priority, status);
            std::fs::write(&path2, &content2)?;
            let rel_path2 = format!(".tickets/{}", filename2);
            git::add(&txn.repo, &[&rel_path2])?;
            git::commit(&txn.repo, &format!("chore(tickets): new {} {}", tid2, slug))?;

            txn.push_retry()?;
            if is_quiet() {
                println!("{}", tid2);
            } else {
                println!(
                    "{}",
                    success_msg(
                        "created",
                        &tid2,
                        slug,
                        &format!("pushed, renumbered {}→{}", tid, tid2)
                    )
                );
            }
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

    if !is_quiet() {
        println!(
            "{}",
            success_msg(
                "claimed",
                &t.id,
                &slug_from_filename(&file.path),
                &format!("{} in_progress", crate::color::sym_arrow())
            )
        );
    }
    Ok(0)
}

fn cmd_close(
    id: &str,
    note: Option<&str>,
    ac_indices: &[u32],
    check_all: bool,
    force: bool,
) -> Result<i32> {
    let (repo, remote, corpus) = preflight_mutation()?;
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);

    let t = match core::find_ticket(&corpus, id) {
        Ok(t) => t,
        Err(_) => domain_bail!("no ticket with id {:?}", id),
    };

    // Project config: require_resolution
    if pcfg.close_require_resolution && note.is_none() && !force {
        domain_bail!("project config requires --resolution (or --note) to close a ticket");
    }

    // Check remote state
    if let Some(remote_status) = check_remote_status(&repo, remote, t) {
        if remote_status == "done" {
            domain_bail!("{} is already done (updated on remote)", id);
        }
    }
    if t.status == Status::Done {
        domain_bail!("{} is already done", t.id);
    }

    // Count ACs BEFORE mutation to decide if we should block
    let (unchecked_before, checked_before) = count_ac_boxes(&t.body);
    let total_acs = unchecked_before + checked_before;

    // Error if ALL ACs are unchecked (unless --force, --ac, or --check-all will handle it)
    // Respects project config: close.require_checked_acs = false disables this guard
    if pcfg.close_require_checked_acs
        && total_acs > 0
        && unchecked_before == total_acs
        && ac_indices.is_empty()
        && !check_all
        && !force
    {
        domain_bail!(
            "all {} acceptance criteria are unchecked — check at least one with --ac, use --check-all, or use --force to close anyway",
            total_acs
        );
    }

    let mut file = t.file.clone();
    file.set_field("status", "done");

    // Append Resolution section if not present
    if !file.body.contains("## Resolution") {
        let date = chrono_date();
        let resolution = note.unwrap_or("TBD");

        // If on a spike/ branch, note it in the resolution
        let branch_note = git::current_branch(&repo)
            .ok()
            .filter(|b| b.starts_with("spike/"))
            .map(|b| format!("\n\nSpike branch: {}", b))
            .unwrap_or_default();

        file.body = format!(
            "{}\n\n## Resolution ({})\n\n{}{}\n",
            file.body.trim_end(),
            date,
            resolution,
            branch_note
        );
    }

    // Flip AC boxes if specified
    if check_all {
        // Check all unchecked boxes in the AC section only
        if let Some(range) = core::ac_section_range(&file.body) {
            let section = file.body[range.clone()].replace("- [ ]", "- [x]");
            file.body.replace_range(range, &section);
        }
    } else if !ac_indices.is_empty() {
        file.body = flip_ac_boxes(&file.body, ac_indices);
    }

    file.write()?;

    // Count unchecked ACs after mutation (for reporting)
    let (unchecked_after, _) = count_ac_boxes(&file.body);
    let checked_after = total_acs.saturating_sub(unchecked_after);

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

    // Prominent output with AC status
    if !is_quiet() {
        let verb = if note.is_some() {
            "Resolution written"
        } else {
            "Resolution stub appended"
        };
        println!(
            "{}",
            success_msg("closed", &t.id, &slug_from_filename(&file.path), verb)
        );
        if total_acs > 0 {
            println!(
                "  acceptance criteria: {}/{} checked{}",
                checked_after,
                total_acs,
                if unchecked_after > 0 {
                    format!(
                        " {} {} unchecked",
                        crate::color::sym_warn(),
                        unchecked_after
                    )
                } else {
                    format!(" {}", crate::color::sym_ok())
                }
            );
        }

        // Show newly unblocked tickets
        let pre_frontier: std::collections::HashSet<String> = core::frontier(&corpus)
            .iter()
            .map(|t| t.id.clone())
            .collect();
        let dir = tickets_dir()?;
        match core::load_corpus(&dir) {
            Ok(new_corpus) => {
                let post_frontier: Vec<&core::Ticket> = core::frontier(&new_corpus)
                    .into_iter()
                    .filter(|t| !pre_frontier.contains(&t.id))
                    .collect();
                if !post_frontier.is_empty() {
                    let items: Vec<String> = post_frontier
                        .iter()
                        .map(|t| format!("{} {}", t.id, t.title))
                        .collect();
                    println!(
                        "  {} unblocked: {}",
                        crate::color::sym_arrow(),
                        items.join(", ")
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "  {} could not compute unblocked tickets: {}",
                    crate::color::sym_warn(),
                    e
                );
            }
        }
    }

    Ok(0)
}

// --- Output formatting ---

/// Check if quiet mode is active.
fn is_quiet() -> bool {
    QUIET.load(std::sync::atomic::Ordering::Relaxed)
}

/// Format a success message in the action-result pattern.
fn success_msg(verb: &str, id: &str, slug: &str, detail: &str) -> String {
    let sym = crate::color::sym_ok();
    if detail.is_empty() {
        format!("{} {} {} {}", sym, verb, id, slug)
    } else {
        format!("{} {} {} {} ({})", sym, verb, id, slug, detail)
    }
}

/// Extract slug from a ticket filename: "01-auth-system.md" → "auth-system"
fn slug_from_filename(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    stem.split_once('-')
        .map(|(_, s)| s.to_string())
        .unwrap_or(stem)
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

/// Count unchecked and checked AC boxes within the acceptance criteria section only.
fn count_ac_boxes(body: &str) -> (usize, usize) {
    let section = match core::ac_section_range(body) {
        Some(range) => &body[range],
        None => return (0, 0),
    };
    let unchecked = RE_UNCHECKED_AC.find_iter(section).count();
    let checked = RE_CHECKED_AC.find_iter(section).count();
    (unchecked, checked)
}

fn flip_ac_boxes(body: &str, indices: &[u32]) -> String {
    let mut result = body.to_string();
    let range = match core::ac_section_range(body) {
        Some(r) => r,
        None => return result,
    };
    let section = &body[range.clone()];
    let matches: Vec<_> = RE_UNCHECKED_AC.find_iter(section).collect();

    // Work backwards to preserve indices (offsets are relative to section start)
    for &idx in indices.iter().rev() {
        let i = (idx as usize).saturating_sub(1); // 1-based to 0-based
        if i < matches.len() {
            let m = &matches[i];
            let abs_start = range.start + m.start();
            let abs_end = range.start + m.end();
            result.replace_range(abs_start..abs_end, "- [x]");
        }
    }
    result
}

// --- cmd_edit ---

#[allow(clippy::too_many_arguments)]
fn cmd_edit(
    id: &str,
    title: Option<&str>,
    blocked_by: Option<&str>,
    env: Option<&str>,
    spec: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
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
            if let Err(e) = core::validate::validate_priority(prio_val) {
                domain_bail!("{} (or '' to clear)", e);
            }
            file.set_field("priority", prio_val);
        }
        changed.push("priority");
    }
    if let Some(status_val) = status {
        if status_val.is_empty() {
            domain_bail!(
                "status cannot be cleared — use a valid value (backlog/open/in_progress/done)"
            );
        }
        if core::Status::parse(status_val).is_err() {
            domain_bail!(
                "status must be one of {} (got {:?})",
                core::STATUS_VALUES.join("/"),
                status_val
            );
        }
        file.set_field("status", status_val);
        changed.push("status");
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
    if !is_quiet() {
        println!(
            "{}",
            success_msg(
                "edited",
                id,
                &slug_from_filename(&file.path),
                &changed.join(", ")
            )
        );
    }
    Ok(0)
}

// --- cmd_validate ---

fn cmd_validate(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
    // CLI --strict overrides; if not passed, use project config default
    let effective_strict = strict || pcfg.validate_strict;
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

    let status = findings::status_from_findings(&all_findings, effective_strict);
    findings::print_findings(&all_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_audit ---

// --- cmd_rebase ---

pub fn cmd_rebase_impl(dry_run: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = dir.parent().unwrap().to_path_buf();

    // Step 1: fetch origin
    git::fetch(&repo)?;

    // Step 2: get remote IDs
    let remote_names = git::remote_ticket_names(&repo);
    let remote_ids: std::collections::HashSet<String> = remote_names
        .iter()
        .filter_map(|n| RE_TICKET_FILENAME.captures(n).map(|c| c[1].to_string()))
        .collect();

    // Step 3: get local ticket files and identify collisions
    // A collision = local file has an ID that also exists on remote, but the file is NOT on remote
    // (different slug means it's a different ticket claiming the same ID)
    let local_names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let remote_name_set: std::collections::HashSet<&str> =
        remote_names.iter().map(|s| s.as_str()).collect();

    let mut collisions: Vec<(String, String, PathBuf)> = Vec::new(); // (old_id, slug, path)
    for name in &local_names {
        if let Some(caps) = RE_TICKET_FILENAME.captures(name) {
            let id = caps[1].to_string();
            let slug = caps[2].to_string();
            // Collision: same ID exists on remote but with a DIFFERENT filename
            if remote_ids.contains(&id) && !remote_name_set.contains(name.as_str()) {
                collisions.push((id, slug, dir.join(name)));
            }
        }
    }

    if collisions.is_empty() {
        if !is_quiet() {
            println!("No ID collisions with upstream.");
        }
        return Ok(0);
    }

    // Step 4: compute the renumber plan — assign next available IDs
    // Combine remote + local IDs to find the true max
    let all_names: Vec<String> = local_names
        .iter()
        .chain(remote_names.iter())
        .cloned()
        .collect();
    let width = core::id_width(&all_names);

    // Sort collisions by old ID for deterministic ordering
    collisions.sort_by(|a, b| a.0.cmp(&b.0));

    // Build renumber map: old_id → new_id
    let base_id = core::max_id(&all_names) + 1;
    let mut renumber_map: Vec<(String, String)> = Vec::new();
    for (i, (old_id, _slug, _path)) in collisions.iter().enumerate() {
        let new_id = format!("{:0>width$}", base_id + i as u64, width = width);
        renumber_map.push((old_id.clone(), new_id));
    }

    // Step 5: dry-run report
    if dry_run {
        println!("Collisions detected ({}):", renumber_map.len());
        for ((old_id, slug, _), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
            println!("  {} → {} ({})", old_id, new_id, slug);
        }
        println!("\nRun without --dry-run to apply.");
        return Ok(0);
    }

    // Step 6: perform the renumber
    // 6a: rename files and update frontmatter id
    let mut renamed_paths: Vec<String> = Vec::new();
    for ((old_id, slug, old_path), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
        let new_filename = format!("{}-{}.md", new_id, slug);
        let new_path = dir.join(&new_filename);

        // Rename file
        std::fs::rename(old_path, &new_path)?;

        // Update frontmatter id field
        let mut file = core::TicketFile::parse(&new_path)?;
        file.set_field("id", &format!("\"{}\"", new_id));
        file.write()?;

        renamed_paths.push(format!(".tickets/{}-{}.md", old_id, slug));
        renamed_paths.push(format!(".tickets/{}", new_filename));
    }

    // 6b: update blocked_by references across the ENTIRE corpus
    let id_map: std::collections::HashMap<&str, &str> = renumber_map
        .iter()
        .map(|(old, new)| (old.as_str(), new.as_str()))
        .collect();

    let updated_corpus: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let mut refs_updated = 0;
    let mut modified_paths: Vec<String> = Vec::new();
    for name in &updated_corpus {
        let path = dir.join(name);
        let mut file = core::TicketFile::parse(&path)?;
        if let Some(deps_raw) = file.get("blocked_by") {
            let mut changed = false;
            let mut new_deps: Vec<String> = Vec::new();
            // Parse the blocked_by array
            for dep in deps_raw
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
            {
                if let Some(&new_id) = id_map.get(dep.as_str()) {
                    new_deps.push(new_id.to_string());
                    changed = true;
                } else {
                    new_deps.push(dep);
                }
            }
            if changed {
                let formatted = new_deps
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", ");
                file.set_field("blocked_by", &format!("[{}]", formatted));
                file.write()?;
                modified_paths.push(format!(".tickets/{}", name));
                refs_updated += 1;
            }
        }
    }

    // Step 7: commit atomically — only stage files we changed
    let mut all_paths = renamed_paths.clone();
    all_paths.extend(modified_paths);
    let add_paths: Vec<&str> = all_paths.iter().map(|s| s.as_str()).collect();
    git::add(&repo, &add_paths)?;
    let msg = format!(
        "chore(tickets): rebase — renumber {} ticket(s) to resolve ID collision",
        renumber_map.len()
    );
    git::commit(&repo, &msg)?;

    // Report
    if !is_quiet() {
        println!("Renumbered {} ticket(s):", renumber_map.len());
        for ((old_id, slug, _), (_, new_id)) in collisions.iter().zip(renumber_map.iter()) {
            println!("  {} → {} ({})", old_id, new_id, slug);
        }
        if refs_updated > 0 {
            println!("  {} blocked_by reference(s) updated", refs_updated);
        }
    }
    Ok(0)
}

fn cmd_audit(strict: bool, brief: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let mut audit_findings: Vec<Finding> = Vec::new();

    // Compute frontier for high-priority check (only report if actually ready to work)
    let frontier_ids: std::collections::HashSet<&str> = core::frontier(&corpus)
        .iter()
        .map(|t| t.id.as_str())
        .collect();

    for t in &corpus {
        let fname = t.path.file_name().unwrap().to_string_lossy().to_string();

        if t.status == Status::Done {
            // Check: all ACs unchecked on a done ticket
            let (unchecked, checked) = count_ac_boxes(&t.body);
            if unchecked > 0 && checked == 0 {
                audit_findings.push(Finding {
                    file: fname.clone(),
                    rule: "all-acs-unchecked-on-done".into(),
                    message: format!("{} unchecked box(es), none checked", unchecked),
                    severity: "warning".into(),
                });
            }

            // Check: TBD resolution stub or empty resolution
            if t.body.contains("## Resolution") {
                // Extract text after the Resolution heading
                let has_content = t
                    .body
                    .split_once("## Resolution")
                    .map(|(_, after)| {
                        // Skip the heading line (may have a date suffix)
                        let text = after.lines().skip(1).collect::<Vec<_>>().join("\n");
                        let trimmed = text.trim();
                        !trimmed.is_empty() && trimmed != "TBD"
                    })
                    .unwrap_or(false);
                if !has_content {
                    audit_findings.push(Finding {
                        file: fname.clone(),
                        rule: "tbd-resolution".into(),
                        message: "resolution is empty or still TBD".into(),
                        severity: "warning".into(),
                    });
                }
            }

            // Check: no Resolution section at all
            if !t.body.contains("## Resolution") {
                audit_findings.push(Finding {
                    file: fname.clone(),
                    rule: "missing-resolution".into(),
                    message: "done ticket has no Resolution section".into(),
                    severity: "warning".into(),
                });
            }
        }

        // Check: stale WIP (in_progress with old last-commit date)
        if t.status == Status::InProgress {
            let rel_path = t
                .path
                .strip_prefix(&dir)
                .map(|p| format!(".tickets/{}", p.display()))
                .unwrap_or_default();
            if let Ok(ts_str) = git::git(
                dir.parent().unwrap_or(&dir),
                &["log", "-1", "--format=%ct", "--", &rel_path],
            ) {
                if let Ok(ts) = ts_str.trim().parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now > ts && (now - ts) > 7 * 24 * 60 * 60 {
                        let days = (now - ts) / (24 * 60 * 60);
                        audit_findings.push(Finding {
                            file: fname.clone(),
                            rule: "stale-wip".into(),
                            message: format!("in_progress for {} days (last commit)", days),
                            severity: "info".into(),
                        });
                    }
                }
            }
        }

        // Check: high-priority still open (only if on the frontier — blocked tickets don't count)
        if t.status == Status::Open && t.is_high_priority() && frontier_ids.contains(t.id.as_str())
        {
            audit_findings.push(Finding {
                file: fname,
                rule: "high-priority-open".into(),
                message: "high-priority ticket still open".into(),
                severity: "info".into(),
            });
        }
    }

    let status = findings::status_from_findings(&audit_findings, strict);
    findings::print_findings(&audit_findings, brief, status);
    Ok(if status == "fail" { 1 } else { 0 })
}

// --- cmd_sync_plan ---

pub fn cmd_sync_plan_impl(
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

fn cmd_query(status_filter: Option<&str>, priority_filter: Option<&str>) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;

    for t in &corpus {
        // Apply filters
        if let Some(sf) = status_filter {
            if t.status.as_str() != sf {
                continue;
            }
        }
        if let Some(pf) = priority_filter {
            match t.priority {
                Some(p) if p.as_str() == pf => {}
                _ => continue,
            }
        }

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

// --- cmd_blocked ---

fn cmd_blocked() -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;

    let done: std::collections::HashSet<&str> = corpus
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();

    // Find open tickets that have at least one undone dependency
    let mut blocked: Vec<&Ticket> = corpus
        .iter()
        .filter(|t| {
            t.status == Status::Open
                && !t.blocked_by.is_empty()
                && !t.blocked_by.iter().all(|dep| done.contains(dep.as_str()))
        })
        .collect();

    blocked.sort_by_key(|t| t.numeric_key());

    if blocked.is_empty() {
        if !is_quiet() {
            println!("No blocked tickets.");
        }
        return Ok(0);
    }

    if !is_quiet() {
        println!("Blocked ({}):", blocked.len());
    }
    for t in &blocked {
        if is_quiet() {
            println!("{}", t.id);
        } else {
            println!("  {}  {}", t.id, t.title);
            // Show which dependencies are not done
            let undone_deps: Vec<String> = t
                .blocked_by
                .iter()
                .filter(|dep| !done.contains(dep.as_str()))
                .map(|dep| {
                    // Look up the blocker's title and status
                    corpus
                        .iter()
                        .find(|c| c.id == *dep)
                        .map(|c| format!("{} {} ({})", dep, c.title, c.status.as_str()))
                        .unwrap_or_else(|| format!("{} (not found)", dep))
                })
                .collect();
            for dep in &undone_deps {
                println!("    blocked by: {}", dep);
            }
        }
    }
    Ok(0)
}

// --- cmd_capabilities ---

fn cmd_capabilities() -> Result<i32> {
    let version = env!("CARGO_PKG_VERSION");
    let json = serde_json::json!({
        "version": version,
        "commands": {
            "ready": {
                "description": "Show frontier (unblocked tickets)",
                "flags": ["--json"],
                "reads": true,
                "mutates": false
            },
            "new": {
                "description": "Create and claim a new ticket",
                "flags": ["--title", "--blocked-by", "--priority", "--env", "--spec", "--status"],
                "reads": false,
                "mutates": true
            },
            "claim": {
                "description": "Mark ticket in_progress (pushed WIP)",
                "flags": [],
                "reads": false,
                "mutates": true
            },
            "close": {
                "description": "Mark ticket done with resolution",
                "flags": ["--resolution", "--note", "--ac", "--check-all", "--force"],
                "reads": false,
                "mutates": true
            },
            "edit": {
                "description": "Surgical field corrections",
                "flags": ["--title", "--blocked-by", "--priority", "--env", "--spec", "--status", "--ac"],
                "reads": false,
                "mutates": true
            },
            "query": {
                "description": "Dump all tickets as JSON Lines",
                "flags": [],
                "reads": true,
                "mutates": false
            },
            "validate": {
                "description": "Check for cycles, dangling deps, contract issues",
                "flags": ["--strict", "--brief"],
                "reads": true,
                "mutates": false
            },
            "config": {
                "description": "Manage user/project configuration",
                "flags": ["--set", "--get", "--unset", "--list", "--show"],
                "reads": true,
                "mutates": true
            },
            "capabilities": {
                "description": "Machine-readable feature manifest",
                "flags": [],
                "reads": true,
                "mutates": false
            }
        },
        "workflows": {
            "single_agent": "ready → close <id> --check-all --resolution '...'",
            "shared_repo": "ready → claim <id> → [work] → close <id> --check-all --resolution '...'",
            "scripting": "ready --json | jq '.id' | xargs tkt claim"
        },
        "config": {
            "user": "~/.config/tkt/config.toml",
            "project": ".tickets/config.toml"
        }
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(0)
}

// --- cmd_config ---

fn cmd_config(
    set: Option<&str>,
    get: Option<&str>,
    unset: Option<&str>,
    list: bool,
    show: bool,
) -> Result<i32> {
    if let Some(pair) = set {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected key=value format, got {:?}", pair))?;
        let key = key.trim();
        let value = value.trim();
        crate::config::Config::set(key, value)?;
        if !is_quiet() {
            println!("{} {} = {:?}", crate::color::sym_ok(), key, value);
        }
        return Ok(0);
    }

    if let Some(key) = get {
        let cfg = crate::config::Config::load();
        println!("{}", cfg.get(key));
        return Ok(0);
    }

    if let Some(key) = unset {
        let existed = crate::config::Config::unset(key)?;
        if !is_quiet() {
            if existed {
                println!(
                    "{} unset {:?} (reverted to default)",
                    crate::color::sym_ok(),
                    key
                );
            } else {
                println!("(no value was set for {:?})", key);
            }
        }
        return Ok(0);
    }

    if show {
        // Dump effective project config
        let dir = tickets_dir()?;
        let pcfg = project_config(&dir);
        let config_path = dir.join("config.toml");
        let has_file = config_path.is_file();

        println!("# Project config: .tickets/config.toml");
        if has_file {
            println!("# Source: {}", config_path.display());
        } else {
            println!("# (no config file — all defaults)");
        }
        println!();
        for entry in pcfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
        return Ok(0);
    }

    if list {
        let cfg = crate::config::Config::load();
        for entry in cfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
        return Ok(0);
    }

    // No flag provided — show both user and project config
    let cfg = crate::config::Config::load();
    println!("# User config (~/.config/tkt/config.toml)");
    for entry in cfg.list() {
        println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
    }
    if let Ok(dir) = tickets_dir() {
        let pcfg = project_config(&dir);
        println!();
        println!("# Project config (.tickets/config.toml)");
        for entry in pcfg.list() {
            println!("{} = {:?} ({})", entry.key, entry.value, entry.source);
        }
    }
    Ok(0)
}

// --- cmd_telemetry ---

fn cmd_telemetry(
    enable: bool,
    disable: bool,
    status: bool,
    show: bool,
    clear: bool,
) -> Result<i32> {
    use crate::telemetry;

    if enable {
        telemetry::write_consent(true)
            .map_err(|e| anyhow::anyhow!("failed to write consent: {}", e))?;
        println!("telemetry enabled — events will be recorded locally");
        return Ok(0);
    }

    if disable {
        telemetry::write_consent(false)
            .map_err(|e| anyhow::anyhow!("failed to write consent: {}", e))?;
        println!("telemetry disabled");
        return Ok(0);
    }

    if clear {
        if let Some(dir) = telemetry::telemetry_dir() {
            if dir.is_dir() {
                std::fs::remove_dir_all(&dir)
                    .map_err(|e| anyhow::anyhow!("failed to clear telemetry: {}", e))?;
                println!("telemetry data cleared");
            } else {
                println!("no telemetry data to clear");
            }
        } else {
            println!("no telemetry data directory");
        }
        return Ok(0);
    }

    if show {
        return cmd_telemetry_show();
    }

    // --status or no flags: show status
    let _ = status;
    cmd_telemetry_status()
}

fn cmd_telemetry_status() -> Result<i32> {
    use crate::telemetry;

    // Run cleanup before reporting (enforces retention)
    telemetry::cleanup_telemetry_dir();

    let (consent, reason) = telemetry::check_consent();
    let state_str = match consent {
        telemetry::Consent::Enabled => "enabled",
        telemetry::Consent::Disabled => "disabled",
    };
    let reason_str = match reason {
        telemetry::ConsentReason::DoNotTrack => "DO_NOT_TRACK=1",
        telemetry::ConsentReason::EnvVar => "TKT_TELEMETRY env var",
        telemetry::ConsentReason::CiDetected => "CI environment detected",
        telemetry::ConsentReason::ConfigFile => "consent.toml",
        telemetry::ConsentReason::Default => "default — never opted in",
    };
    println!("telemetry: {} ({})", state_str, reason_str);

    // Storage summary
    if let Some(dir) = telemetry::telemetry_dir() {
        if dir.is_dir() {
            let mut total_bytes: u64 = 0;
            let mut total_events: usize = 0;
            let mut projects: Vec<(String, usize, u64)> = Vec::new();

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "jsonl") {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let lines = std::fs::read_to_string(&path)
                            .map(|c| c.lines().filter(|l| !l.is_empty()).count())
                            .unwrap_or(0);
                        let name = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        total_bytes += size;
                        total_events += lines;
                        projects.push((name, lines, size));
                    }
                }
            }
            projects.sort_by_key(|p| std::cmp::Reverse(p.1));

            println!(
                "storage: {} ({} events across {} projects)",
                format_bytes(total_bytes),
                total_events,
                projects.len()
            );
            for (name, events, bytes) in &projects {
                println!("  {}: {} events ({})", name, events, format_bytes(*bytes));
            }
        } else {
            println!("storage: 0 bytes (no data)");
        }
    } else {
        println!("storage: unavailable (no data directory)");
    }

    // Consent file location
    if let Some(path) = telemetry::consent_file_path() {
        let exists = if path.is_file() { "found" } else { "not found" };
        println!("consent file: {} ({})", path.display(), exists);
    }

    // Env overrides
    let dnt = std::env::var("DO_NOT_TRACK").unwrap_or_else(|_| "unset".to_string());
    let tkt_tel = std::env::var("TKT_TELEMETRY").unwrap_or_else(|_| "unset".to_string());
    let ci = std::env::var("CI").unwrap_or_else(|_| "unset".to_string());
    println!(
        "env overrides: DO_NOT_TRACK={}, TKT_TELEMETRY={}, CI={}",
        dnt, tkt_tel, ci
    );

    Ok(0)
}

fn cmd_telemetry_show() -> Result<i32> {
    use crate::telemetry;

    let dir = match telemetry::telemetry_dir() {
        Some(d) if d.is_dir() => d,
        _ => {
            println!("no telemetry data");
            return Ok(0);
        }
    };

    let mut all_lines: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if !line.is_empty() {
                            all_lines.push(line.to_string());
                        }
                    }
                }
            }
        }
    }

    if all_lines.is_empty() {
        println!("no telemetry events recorded");
        return Ok(0);
    }

    // Sort by timestamp (ISO 8601 — lexicographic sort works)
    all_lines.sort_by(|a, b| extract_ts(a.as_str()).cmp(extract_ts(b.as_str())));

    // Show last 20 events
    let start = all_lines.len().saturating_sub(20);
    println!(
        "recent events ({} total, showing last {}):",
        all_lines.len(),
        all_lines.len() - start
    );
    println!();
    for line in &all_lines[start..] {
        // Parse and display human-readable
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            println!(
                "  {} {} cmd={} exit={} {}ms",
                val["ts"].as_str().unwrap_or("?"),
                val["project"].as_str().unwrap_or("?"),
                val["cmd"].as_str().unwrap_or("?"),
                val["exit_code"].as_i64().unwrap_or(-1),
                val["duration_ms"].as_u64().unwrap_or(0),
            );
        } else {
            println!("  {}", line);
        }
    }
    Ok(0)
}

/// Extract "ts" value from a JSONL line for sorting (best-effort).
fn extract_ts(line: &str) -> &str {
    // Quick extraction: find "ts":"<value>"
    if let Some(start) = line.find("\"ts\":\"") {
        let rest = &line[start + 6..];
        if let Some(end) = rest.find('"') {
            return &rest[..end];
        }
    }
    ""
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// --- cmd_batch ---

fn cmd_batch(
    items: &[String],
    spec: Option<&str>,
    env: Option<&str>,
    priority: Option<&str>,
    status: Option<&str>,
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
    if let Some(s) = status {
        if let Err(err) = core::validate::validate_status(s) {
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
                let content =
                    core::new_ticket_text(&tid, title, blocked_by, env, spec, priority, status);
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
        if is_quiet() {
            println!("{}", tid);
        } else {
            println!("{}", success_msg("created", &tid, slug, "pushed"));
        }
    }
    Ok(0)
}

// --- cmd_renumber ---

pub fn cmd_renumber_impl(old_id: &str, new_id: &str, file_hint: Option<&str>) -> Result<i32> {
    if let Err(e) = core::validate::validate_id(new_id) {
        domain_bail!("new id: {}", e);
    }

    let dir = tickets_dir()?;
    let pcfg = project_config(&dir);
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
    if has_remote(&repo) && pcfg.push_enabled {
        git::push_with_retry(&repo)?;
    }

    let detail = if refs_updated > 0 {
        format!("{} refs updated", refs_updated)
    } else {
        String::new()
    };
    if !is_quiet() {
        println!(
            "{}",
            success_msg("renumbered", old_id, &format!("→ {}", new_id), &detail)
        );
    }
    Ok(0)
}

// --- Helpers ---

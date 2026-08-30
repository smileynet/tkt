use clap::{Parser, Subcommand};

const fn version_string() -> &'static str {
    const HASH: &str = env!("TKT_GIT_HASH");
    if HASH.is_empty() {
        env!("CARGO_PKG_VERSION")
    } else {
        concat!(env!("CARGO_PKG_VERSION"), " (", env!("TKT_GIT_HASH"), ")")
    }
}

#[derive(Parser)]
#[command(
    name = "tkt",
    about = "Git-native ticket CLI (.tickets/ contract)",
    version = version_string(),
)]
struct Cli {
    /// Suppress confirmations, emit only essential data
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Color output: always, never, or auto (default: auto)
    #[arg(long, global = true)]
    color: Option<String>,

    /// Show what would happen without making changes
    #[arg(long, global = true)]
    dry_run: bool,

    /// Output format: json or text (default: text)
    #[arg(short = 'o', long = "output", global = true)]
    output_format: Option<String>,

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
    /// Initialize .tickets/ and deploy agent integration files
    Init {
        /// Write agent snippet into a file (default: AGENTS.md) using markers
        #[arg(long)]
        write: Option<Option<String>>,
        /// Generate for a specific agent tool
        #[arg(long, value_parser = ["agents", "claude", "cursor", "kiro", "copilot", "opencode", "windsurf"])]
        target: Option<String>,
        /// Generate for all known agent tools
        #[arg(long)]
        all: bool,
        /// Skip directory/config creation, only output agent snippet
        #[arg(long)]
        agent_only: bool,
    },
    /// Health check: verify setup, find issues, scan multiple projects
    Doctor {
        /// Path to scan for .tickets/ dirs (omit for current project only)
        path: Option<String>,
        /// Apply safe fixes to fixable issues
        #[arg(long)]
        fix: bool,
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
    /// Normalize ticket frontmatter style (quoting, field order, whitespace)
    Lint {
        /// Report deviations without fixing (exit 1 if any file would change)
        #[arg(long)]
        check: bool,
        /// Specific ticket IDs to lint (omit for all)
        ids: Vec<String>,
    },
    /// Convert foreign ticket schemas to tkt format
    Migrate {
        /// Source format to convert from (available: tk)
        #[arg(long)]
        from: Option<String>,
        /// Detect and report the current ticket format
        #[arg(long)]
        detect: bool,
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
        /// Validation criteria (repeatable)
        #[arg(long = "validation", visible_alias = "vc", num_args = 1, action = clap::ArgAction::Append)]
        validation_criteria: Vec<String>,
        /// Tags for categorization (repeatable, also auto-applied from active context)
        #[arg(long = "tags", value_delimiter = ',')]
        tags: Vec<String>,
        /// Machine capabilities required (e.g., gpu,linux,corp)
        #[arg(long = "requires", value_delimiter = ',')]
        requires: Vec<String>,
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
        /// Validation criteria (repeatable)
        #[arg(long = "validation", visible_alias = "vc", num_args = 1, action = clap::ArgAction::Append)]
        validation_criteria: Vec<String>,
        /// Tags for categorization (repeatable, also auto-applied from active context)
        #[arg(long = "tags", value_delimiter = ',')]
        tags: Vec<String>,
        /// Machine capabilities required (e.g., gpu,linux,corp)
        #[arg(long = "requires", value_delimiter = ',')]
        requires: Vec<String>,
    },
    /// Set, show, or clear the active tag context for scoped work
    Context {
        /// Tags to set (+tag to include, -tag to exclude, bare word = include)
        tags: Vec<String>,
        /// Clear the active context
        #[arg(long)]
        clear: bool,
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
        /// Evidence for validation criteria (repeatable, positional or N=text for named)
        #[arg(long, num_args = 1, action = clap::ArgAction::Append)]
        evidence: Vec<String>,
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
        /// Validation criteria (repeatable, replaces existing list)
        #[arg(long = "validation", visible_alias = "vc", num_args = 1, action = clap::ArgAction::Append)]
        validation_criteria: Vec<String>,
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
        /// Auto-repair fixable issues (quoting, invalid optional fields, status mapping)
        #[arg(long)]
        fix: bool,
        /// Show what --fix would change without writing (requires --fix)
        #[arg(long)]
        dry_run: bool,
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
        /// Run deep content analysis (evidence quality, template detection)
        #[arg(long)]
        deep: bool,
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
        /// Show all events (default: last 20)
        #[arg(long)]
        all: bool,
        /// Delete all local telemetry data
        #[arg(long)]
        clear: bool,
    },
}

pub fn run() -> i32 {
    let start = std::time::Instant::now();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Record parse failures in telemetry (agents using wrong syntax)
            // Skip help/version requests — those are intentional, not errors
            use clap::error::ErrorKind;
            let is_error = !matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion);
            if is_error {
                record_parse_error(&start);
            }
            e.exit();
        }
    };
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
    crate::QUIET.store(cli.quiet, std::sync::atomic::Ordering::Relaxed);

    // Store dry-run flag
    crate::DRY_RUN.store(cli.dry_run, std::sync::atomic::Ordering::Relaxed);

    // Store JSON output flag
    let json_output = cli
        .output_format
        .as_deref()
        .map(|f| f.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    crate::JSON_OUTPUT.store(json_output, std::sync::atomic::Ordering::Relaxed);

    // Initialize color mode from --color flag and environment
    crate::color::init(cli.color.as_deref());

    // Extract notable flags before dispatch (command is moved into match)
    let flags = notable_flags(&cli.command);

    let result = match cli.command {
        Commands::Ready { json } => crate::commands::ready::run(json || json_output),
        Commands::Init {
            write,
            target,
            all,
            agent_only,
        } => crate::commands::init::run(write, target.as_deref(), all, agent_only),
        Commands::Doctor { path, fix, strict } => {
            crate::commands::doctor::run(path.as_deref(), fix, strict)
        }
        Commands::Lint { check, ids } => crate::commands::lint::run(check, &ids),
        Commands::Migrate { from, detect } => {
            crate::commands::migrate::run(from.as_deref(), detect)
        }
        Commands::New {
            slug,
            title,
            spec,
            env,
            priority,
            status,
            blocked_by,
            validation_criteria,
            tags,
            requires,
        } => crate::commands::new::run(
            &slug,
            title.as_deref(),
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &blocked_by.unwrap_or_default(),
            &validation_criteria,
            &tags,
            &requires,
        ),
        Commands::Batch {
            items,
            spec,
            env,
            priority,
            status,
            blocked_by,
            validation_criteria,
            tags,
            requires,
        } => crate::commands::batch::run(
            &items,
            spec.as_deref(),
            env.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &blocked_by.unwrap_or_default(),
            &validation_criteria,
            &tags,
            &requires,
        ),
        Commands::Context { tags, clear } => crate::commands::context::run(&tags, clear),
        Commands::Claim { id } => crate::commands::claim::run(&id),
        Commands::Close {
            id,
            note,
            resolution,
            ac,
            check_all,
            force,
            evidence,
        } => {
            let text = resolution.or(note);
            crate::commands::close::run(
                &id,
                text.as_deref(),
                &ac.unwrap_or_default(),
                check_all,
                force,
                &evidence,
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
            validation_criteria,
        } => crate::commands::edit::run(
            &id,
            title.as_deref(),
            blocked_by.as_deref(),
            env.as_deref(),
            spec.as_deref(),
            priority.as_deref(),
            status.as_deref(),
            &ac.unwrap_or_default(),
            if validation_criteria.is_empty() {
                None
            } else {
                Some(&validation_criteria)
            },
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
        Commands::Validate {
            strict,
            brief,
            fix,
            dry_run,
        } => {
            if fix || dry_run {
                crate::commands::validate::run_with_fix(strict, brief, dry_run)
            } else {
                crate::commands::validate::run(strict, brief)
            }
        }
        Commands::Query { status, priority } => {
            crate::commands::query::run(status.as_deref(), priority.as_deref())
        }
        Commands::Blocked => crate::commands::blocked::run(),
        Commands::Capabilities => crate::commands::capabilities::run(),
        Commands::Rebase { dry_run } => crate::commands::rebase::run(dry_run),
        Commands::Audit {
            strict,
            brief,
            deep,
        } => crate::commands::audit::run(strict, brief, deep),
        Commands::Config {
            set,
            get,
            unset,
            list,
            show,
        } => crate::commands::config::run(
            set.as_deref(),
            get.as_deref(),
            unset.as_deref(),
            list,
            show,
        ),
        Commands::Telemetry {
            enable,
            disable,
            status,
            show,
            all,
            clear,
        } => crate::commands::telemetry::run(enable, disable, status, show, all, clear),
    };
    let (exit_code, error_kind) = match result {
        Ok(code) => (code, None),
        Err(e) => {
            if let Some(de) = e.downcast_ref::<crate::DomainError>() {
                let code = de.kind.exit_code();
                if crate::JSON_OUTPUT.load(std::sync::atomic::Ordering::Relaxed) {
                    emit_json_error(de);
                }
                eprintln!("tkt: {} {}", crate::color::sym_err(), de.message);
                (code, Some(de.kind.as_str()))
            } else {
                if crate::JSON_OUTPUT.load(std::sync::atomic::Ordering::Relaxed) {
                    let envelope = format!(
                        "{{\"ok\":false,\"error\":{{\"kind\":\"io\",\"message\":{}}},\"exit_code\":2}}",
                        json_escape(&e.to_string())
                    );
                    eprintln!("{}", envelope);
                }
                eprintln!("tkt: {} crash: {}", crate::color::sym_err(), e);
                (2, Some("io"))
            }
        }
    };

    // Record telemetry event (silently — never affects CLI behavior)
    record_telemetry(
        &cmd_name,
        exit_code,
        start.elapsed().as_millis() as u64,
        error_kind,
        flags,
    );

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
        Commands::Init { .. } => "init",
        Commands::Doctor { .. } => "doctor",
        Commands::Lint { .. } => "lint",
        Commands::Migrate { .. } => "migrate",
        Commands::New { .. } => "new",
        Commands::Batch { .. } => "batch",
        Commands::Claim { .. } => "claim",
        Commands::Close { .. } => "close",
        Commands::Context { .. } => "context",
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

/// Extract names of notable flags that were explicitly provided.
/// Never includes flag values — just names. Sorted alphabetically.
fn notable_flags(cmd: &Commands) -> Vec<&'static str> {
    let mut flags = Vec::new();

    match cmd {
        Commands::New {
            spec,
            env,
            priority,
            status,
            blocked_by,
            validation_criteria,
            tags,
            requires,
            ..
        } => {
            if blocked_by.is_some() {
                flags.push("blocked-by");
            }
            if env.is_some() {
                flags.push("env");
            }
            if priority.is_some() {
                flags.push("priority");
            }
            if !requires.is_empty() {
                flags.push("requires");
            }
            if spec.is_some() {
                flags.push("spec");
            }
            if status.is_some() {
                flags.push("status");
            }
            if !tags.is_empty() {
                flags.push("tags");
            }
            if !validation_criteria.is_empty() {
                flags.push("validation");
            }
        }
        Commands::Batch {
            spec,
            env,
            priority,
            status,
            blocked_by,
            validation_criteria,
            tags,
            requires,
            ..
        } => {
            if blocked_by.is_some() {
                flags.push("blocked-by");
            }
            if env.is_some() {
                flags.push("env");
            }
            if priority.is_some() {
                flags.push("priority");
            }
            if !requires.is_empty() {
                flags.push("requires");
            }
            if spec.is_some() {
                flags.push("spec");
            }
            if status.is_some() {
                flags.push("status");
            }
            if !tags.is_empty() {
                flags.push("tags");
            }
            if !validation_criteria.is_empty() {
                flags.push("validation");
            }
        }
        Commands::Close {
            note,
            resolution,
            ac,
            check_all,
            force,
            evidence,
            ..
        } => {
            if ac.is_some() {
                flags.push("ac");
            }
            if *check_all {
                flags.push("check-all");
            }
            if !evidence.is_empty() {
                flags.push("evidence");
            }
            if *force {
                flags.push("force");
            }
            if note.is_some() {
                flags.push("note");
            }
            if resolution.is_some() {
                flags.push("resolution");
            }
        }
        Commands::Edit {
            title,
            blocked_by,
            env,
            spec,
            priority,
            status,
            ac,
            validation_criteria,
            ..
        } => {
            if ac.is_some() {
                flags.push("ac");
            }
            if blocked_by.is_some() {
                flags.push("blocked-by");
            }
            if env.is_some() {
                flags.push("env");
            }
            if priority.is_some() {
                flags.push("priority");
            }
            if spec.is_some() {
                flags.push("spec");
            }
            if status.is_some() {
                flags.push("status");
            }
            if title.is_some() {
                flags.push("title");
            }
            if !validation_criteria.is_empty() {
                flags.push("validation");
            }
        }
        Commands::Validate {
            strict, brief, fix, ..
        } => {
            if *brief {
                flags.push("brief");
            }
            if *fix {
                flags.push("fix");
            }
            if *strict {
                flags.push("strict");
            }
        }
        Commands::Audit {
            strict,
            brief,
            deep,
        } => {
            if *brief {
                flags.push("brief");
            }
            if *deep {
                flags.push("deep");
            }
            if *strict {
                flags.push("strict");
            }
        }
        Commands::Query { status, priority } => {
            if priority.is_some() {
                flags.push("priority");
            }
            if status.is_some() {
                flags.push("status");
            }
        }
        Commands::SyncPlan {
            check,
            fix,
            strict,
            brief,
            ..
        } => {
            if *brief {
                flags.push("brief");
            }
            if *check {
                flags.push("check");
            }
            if *fix {
                flags.push("fix");
            }
            if *strict {
                flags.push("strict");
            }
        }
        Commands::Ready { json } => {
            if *json {
                flags.push("json");
            }
        }
        // Commands with no notable optional flags
        Commands::Claim { .. }
        | Commands::Blocked
        | Commands::Capabilities
        | Commands::Context { .. }
        | Commands::Migrate { .. }
        | Commands::Renumber { .. }
        | Commands::Rebase { .. }
        | Commands::Config { .. }
        | Commands::Init { .. }
        | Commands::Doctor { .. }
        | Commands::Lint { .. } => {}
        Commands::Telemetry { all, .. } => {
            if *all {
                flags.push("all");
            }
        }
    }

    // Global --dry-run flag (applies to all commands)
    if crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed) {
        flags.push("dry-run");
    }

    // flags are already inserted alphabetically due to match arm ordering
    flags
}

/// Record a telemetry event for parse/syntax errors (clap rejection path).
/// Called when clap rejects input before dispatch. We can't know the exact command
/// or flags — just that something failed to parse.
fn record_parse_error(start: &std::time::Instant) {
    use crate::telemetry;

    let (consent, _) = telemetry::check_consent();
    if consent != telemetry::Consent::Enabled {
        return;
    }

    let project = std::env::current_dir()
        .ok()
        .and_then(|cwd| crate::git::repo_root(&cwd).ok())
        .map(|root| telemetry::project_slug(&root))
        .unwrap_or_else(|| "unknown".to_string());

    static SESSION: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(telemetry::generate_session_id);

    let event = telemetry::Event {
        ts: telemetry::iso_timestamp(),
        session: SESSION.clone(),
        project,
        cmd: "?".to_string(),
        exit_code: 2,
        duration_ms: start.elapsed().as_millis() as u64,
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: telemetry::os_string().to_string(),
        arch: telemetry::arch_string().to_string(),
        error_kind: Some("parse"),
        flags: vec![],
        result_count: None,
    };

    telemetry::record_event(&event);
}

fn record_telemetry(
    cmd: &str,
    exit_code: i32,
    duration_ms: u64,
    error_kind: Option<&'static str>,
    flags: Vec<&'static str>,
) {
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
        error_kind,
        flags,
        result_count: {
            let rc = crate::RESULT_COUNT.load(std::sync::atomic::Ordering::Relaxed);
            if rc >= 0 {
                Some(rc as u32)
            } else {
                None
            }
        },
    };

    telemetry::record_event(&event);
}

/// Emit a structured JSON error envelope to stderr (last line).
fn emit_json_error(de: &crate::DomainError) {
    let hint_part = match &de.hint {
        Some(h) => format!(",\"hint\":{}", json_escape(h)),
        None => String::new(),
    };
    let envelope = format!(
        "{{\"ok\":false,\"error\":{{\"kind\":\"{}\",\"message\":{}{}}},\"exit_code\":{}}}",
        de.kind.as_str(),
        json_escape(&de.message),
        hint_part,
        de.kind.exit_code()
    );
    eprintln!("{}", envelope);
}

/// Escape a string as a JSON string literal (with quotes).
pub(crate) fn json_escape(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
}

/// Emit a structured JSON success envelope to stdout.
pub(crate) fn emit_json_success(result: &str) {
    println!("{{\"ok\":true,\"result\":{}}}", json_escape(result));
}

/// Emit a JSON success envelope carrying advisory hints (machine-legible).
/// Hints are advisory only — presence never changes the exit code.
pub(crate) fn emit_json_success_with_hints(result: &str, hints: &[crate::nudge::Hint]) {
    if hints.is_empty() {
        emit_json_success(result);
        return;
    }
    let arr: Vec<String> = hints.iter().map(|h| h.to_json()).collect();
    println!(
        "{{\"ok\":true,\"result\":{},\"hints\":[{}]}}",
        json_escape(result),
        arr.join(",")
    );
}

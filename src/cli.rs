use clap::{Parser, Subcommand};

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
        Commands::Ready { .. } => todo!("ready"),
        Commands::New { .. } => todo!("new"),
        Commands::Batch { .. } => todo!("batch"),
        Commands::Claim { .. } => todo!("claim"),
        Commands::Close { .. } => todo!("close"),
        Commands::Edit { .. } => todo!("edit"),
        Commands::Renumber { .. } => todo!("renumber"),
        Commands::SyncPlan { .. } => todo!("sync-plan"),
        Commands::Validate { .. } => todo!("validate"),
    };
    #[allow(unreachable_code)]
    result
}

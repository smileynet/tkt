use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::ProjectConfig;

#[derive(Clone, Copy)]
pub enum Event {
    New,
    Claim,
    Close,
}

impl Event {
    pub fn index(self) -> usize {
        match self {
            Self::New => 0,
            Self::Claim => 1,
            Self::Close => 2,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Claim => "claim",
            Self::Close => "close",
        }
    }
}

pub struct PreparedEvent {
    parent: Option<String>,
}

impl PreparedEvent {
    pub fn prepare(config: &ProjectConfig, repo: &Path, event: Event) -> Option<Self> {
        if config.lifecycle_hooks[event.index()].is_empty() {
            return None;
        }
        Some(Self {
            parent: crate::git::git(repo, &["rev-parse", "--verify", "HEAD"]).ok(),
        })
    }

    pub fn capture(&self, repo: &Path) -> Option<String> {
        let commit = crate::git::git(repo, &["rev-parse", "HEAD"]).ok()?;
        let parent = crate::git::git(repo, &["show", "-s", "--format=%P", &commit]).ok()?;
        (parent == self.parent.as_deref().unwrap_or("")).then_some(commit)
    }
}

pub fn dispatch(config: &ProjectConfig, repo: &Path, event: Event, id: &str, commit: &str) {
    let _ = crate::effects::Phase::Lifecycle(Notification {
        config,
        repo,
        event,
        id,
        commit,
    })
    .execute(crate::effects::Mode::current());
}

pub(crate) struct Notification<'a> {
    config: &'a ProjectConfig,
    repo: &'a Path,
    event: Event,
    id: &'a str,
    commit: &'a str,
}

impl Notification<'_> {
    pub(crate) fn summary(&self) -> crate::effects::EffectSummary {
        notification_summary(self.repo, self.event)
    }

    pub(crate) fn apply(self) {
        let Self {
            config,
            repo,
            event,
            id,
            commit,
        } = self;
        match crate::git::git(repo, &["rev-parse", "HEAD"]) {
            Ok(head) if head == commit => (),
            Ok(_) => {
                eprintln!("advisory hook for {id} skipped: HEAD changed during publication; inspect the committed event before manual replay");
                return;
            }
            Err(_) => return,
        }
        let argv = &config.lifecycle_hooks[event.index()];
        if argv.is_empty() || std::env::var_os("TKT_HOOK_ACTIVE").is_some() {
            return;
        }
        let trusted = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args([
                "config",
                "--local",
                "--no-includes",
                "--get-all",
                "tkt.lifecycleHooks",
            ])
            .output()
            .is_ok_and(|out| out.status.success() && out.stdout == b"true\n");
        if !trusted {
            eprintln!("advisory hook {} for {id} skipped: repository has no local hook trust; inspect its configured commands before opting in", event.name());
            return;
        }
        let program = Path::new(&argv[0]);
        let executable = if program.is_relative() && program.components().count() > 1 {
            repo.join(program)
        } else {
            program.to_path_buf()
        };
        let mut child = match Command::new(executable)
            .args(&argv[1..])
            .current_dir(repo)
            .env("TKT_HOOK_ACTIVE", "1")
            .env("TKT_HOOK_EVENT", event.name())
            .env("TKT_HOOK_ID", id)
            .env("TKT_HOOK_COMMIT", commit)
            .env("TKT_HOOK_REPO", repo)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                eprintln!("advisory hook {} for {id} could not start: {error}; ticket commit {commit} is retained", event.name());
                return;
            }
        };
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        eprintln!("advisory hook {} for {id} failed ({status}); ticket commit {commit} is retained", event.name());
                    }
                    return;
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                outcome => {
                    let reason = match outcome {
                        Err(error) => error.to_string(),
                        _ => "two-second deadline exceeded".into(),
                    };
                    let _ = child.kill();
                    let _ = child.wait();
                    eprintln!(
                        "advisory hook {} for {id}: {reason}; ticket commit {commit} is retained",
                        event.name()
                    );
                    return;
                }
            }
        }
    }
}

pub(crate) fn notification_summary(repo: &Path, event: Event) -> crate::effects::EffectSummary {
    crate::effects::EffectSummary {
        operation: match event {
            Event::New => "notify-new-if-trusted",
            Event::Claim => "notify-claim-if-trusted",
            Event::Close => "notify-close-if-trusted",
        },
        paths: vec![repo.into()],
        unresolved: true,
    }
}

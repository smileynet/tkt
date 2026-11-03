use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    remote: PathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    let repo = temp.path().join("work");
    git(
        temp.path(),
        &[
            "init",
            "--bare",
            "-q",
            "-b",
            "main",
            remote.to_str().unwrap(),
        ],
    );
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            repo.to_str().unwrap(),
        ],
    );
    git(&repo, &["config", "user.name", "Fixture"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::create_dir_all(repo.join(".tickets")).unwrap();
    std::fs::write(
        repo.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/plan.md"), "| 01 | Seed | open |\n").unwrap();
    git(&repo, &["add", ".tickets/01-seed.md", "docs/plan.md"]);
    git(&repo, &["commit", "-qm", "seed"]);
    git(&repo, &["push", "-qu", "origin", "HEAD:main"]);
    Fixture {
        _temp: temp,
        repo,
        remote,
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", &dir.to_string_lossy()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn cli(fixture: &Fixture, args: &[&str], marker: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_HOOK_MARKER", marker)
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn install_hook(repo: &Path, name: &str, script: &str) {
    let hooks = repo.join(".git/hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let path = hooks.join(name);
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn commit_paths(repo: &Path) -> Vec<String> {
    git(repo, &["show", "--pretty=", "--name-only", "HEAD"])
        .lines()
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn ticket_commits_skip_precommit_hook_and_stage_only_ticket_and_plan() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("pre-commit-ran");
    install_hook(
        &fixture.repo,
        "pre-commit",
        "#!/bin/sh\nsleep 3\necho ran > \"$TKT_HOOK_MARKER\"\nexit 41\n",
    );
    std::fs::write(fixture.repo.join("staged-note.md"), "keep staged\n").unwrap();
    git(&fixture.repo, &["add", "staged-note.md"]);
    std::fs::write(fixture.repo.join("untracked-note.md"), "keep untracked\n").unwrap();

    let created = cli(&fixture, &["new", "feature"], &marker);
    assert!(created.status.success(), "{}", output_text(&created));
    assert!(!marker.exists());
    assert_eq!(commit_paths(&fixture.repo), [".tickets/02-feature.md"]);
    assert_eq!(
        git(&fixture.repo, &["diff", "--cached", "--name-only"]),
        "staged-note.md"
    );

    let claimed = cli(&fixture, &["--no-push", "claim", "01"], &marker);
    assert!(claimed.status.success(), "{}", output_text(&claimed));
    assert!(!marker.exists());
    assert!(output_text(&claimed).contains("not pushed"));
    assert_eq!(
        commit_paths(&fixture.repo),
        [".tickets/01-seed.md", "docs/plan.md"]
    );
    assert!(std::fs::read_to_string(fixture.repo.join("docs/plan.md"))
        .unwrap()
        .contains("in_progress"));
    assert_eq!(
        git(&fixture.repo, &["diff", "--cached", "--name-only"]),
        "staged-note.md"
    );
    assert!(fixture.repo.join("untracked-note.md").exists());
}

#[test]
fn commit_hooks_run_only_when_project_policy_opts_in() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("pre-commit-ran");
    std::fs::write(
        fixture.repo.join(".tickets/config.toml"),
        "[commit]\nhooks = \"run\"\n",
    )
    .unwrap();
    install_hook(
        &fixture.repo,
        "pre-commit",
        "#!/bin/sh\necho ran > \"$TKT_HOOK_MARKER\"\nexit 41\n",
    );

    let created = cli(&fixture, &["new", "hooked"], &marker);
    assert_eq!(created.status.code(), Some(2), "{}", output_text(&created));
    assert!(marker.exists());
    assert_eq!(
        git(&fixture.repo, &["rev-parse", "HEAD"]),
        git(&fixture.repo, &["rev-parse", "origin/main"])
    );
}

#[test]
fn invalid_commit_hook_policy_rejects_mutation_before_commit() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("pre-commit-ran");
    std::fs::write(
        fixture.repo.join(".tickets/config.toml"),
        "[commit]\nhooks = \"sometimes\"\n",
    )
    .unwrap();
    let output = cli(&fixture, &["new", "invalid-policy"], &marker);
    assert_eq!(output.status.code(), Some(1), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(text.contains("commit.hooks"), "{text}");
    assert!(text.contains("sometimes"), "{text}");
    assert!(!fixture.repo.join(".tickets/02-invalid-policy.md").exists());
    assert_eq!(
        git(&fixture.repo, &["rev-parse", "HEAD"]),
        git(&fixture.repo, &["rev-parse", "origin/main"])
    );
}

#[test]
fn failed_prepush_reports_durable_commit_and_retry() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("pre-push-ran");
    install_hook(
        &fixture.repo,
        "pre-push",
        "#!/bin/sh\necho pre-push-marker > \"$TKT_HOOK_MARKER\"\necho consumer-pre-push-failed >&2\nexit 42\n",
    );

    let created = cli(&fixture, &["new", "blocked-push", "-o", "json"], &marker);
    assert_eq!(created.status.code(), Some(2), "{}", output_text(&created));
    let text = output_text(&created);
    assert!(
        text.contains("created locally but was not pushed"),
        "{text}"
    );
    assert!(text.contains("consumer-pre-push-failed"), "{text}");
    assert!(text.contains("git push"), "{text}");
    assert!(marker.exists());
    let commit = git(&fixture.repo, &["rev-parse", "--short", "HEAD"]);
    assert!(text.contains(&commit), "{text}");
    assert!(fixture.repo.join(".tickets/02-blocked-push.md").exists());
    assert_eq!(
        git(&fixture.repo, &["rev-parse", "HEAD^"]),
        git(&fixture.repo, &["rev-parse", "origin/main"])
    );
}

#[test]
fn failed_mutation_push_reports_commit_id_hook_output_and_retry() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("pre-push-ran");
    install_hook(
        &fixture.repo,
        "pre-push",
        "#!/bin/sh\necho pre-push-marker > \"$TKT_HOOK_MARKER\"\necho mutation-hook-failed >&2\nexit 42\n",
    );

    let claimed = cli(&fixture, &["claim", "01"], &marker);
    assert_eq!(claimed.status.code(), Some(2), "{}", output_text(&claimed));
    let text = output_text(&claimed);
    assert!(
        text.contains("created locally but was not pushed"),
        "{text}"
    );
    assert!(text.contains("mutation-hook-failed"), "{text}");
    assert!(text.contains("git push"), "{text}");
    assert!(marker.exists());
    let commit = git(&fixture.repo, &["rev-parse", "--short", "HEAD"]);
    assert!(text.contains(&commit), "{text}");
    assert!(
        git(&fixture.repo, &["show", "HEAD:.tickets/01-seed.md"]).contains("status: in_progress")
    );
    assert_eq!(
        git(&fixture.repo, &["rev-parse", "HEAD^"]),
        git(&fixture.repo, &["rev-parse", "origin/main"])
    );
}

#[test]
fn deferred_local_ticket_can_be_renumbered_before_a_later_safe_push() {
    let fixture = fixture();
    let marker = fixture._temp.path().join("unused-hook-marker");
    let deferred = cli(&fixture, &["--no-push", "new", "deferred"], &marker);
    assert!(deferred.status.success(), "{}", output_text(&deferred));
    assert!(fixture.repo.join(".tickets/02-deferred.md").exists());

    let competitor = fixture._temp.path().join("competitor");
    git(
        fixture._temp.path(),
        &[
            "clone",
            "-q",
            fixture.remote.to_str().unwrap(),
            competitor.to_str().unwrap(),
        ],
    );
    git(&competitor, &["config", "user.name", "Competitor"]);
    git(
        &competitor,
        &["config", "user.email", "competitor@example.invalid"],
    );
    std::fs::write(
        competitor.join(".tickets/02-remote.md"),
        "---\nid: \"02\"\ntitle: \"Remote\"\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    git(&competitor, &["add", ".tickets/02-remote.md"]);
    git(&competitor, &["commit", "-qm", "remote ticket"]);
    git(&competitor, &["push", "-q", "origin", "HEAD:main"]);

    let rebased = cli(&fixture, &["rebase"], &marker);
    assert!(rebased.status.success(), "{}", output_text(&rebased));
    assert!(fixture.repo.join(".tickets/03-deferred.md").exists());
    assert!(!fixture.repo.join(".tickets/02-deferred.md").exists());

    let pushed = cli(&fixture, &["new", "after-rebase"], &marker);
    assert!(pushed.status.success(), "{}", output_text(&pushed));
    let files = git(
        &fixture.repo,
        &["ls-tree", "-r", "--name-only", "origin/main"],
    );
    assert!(files.contains(".tickets/02-remote.md"), "{files}");
    assert!(files.contains(".tickets/03-deferred.md"), "{files}");
    assert!(files.contains(".tickets/04-after-rebase.md"), "{files}");
}

#[test]
fn no_push_skips_fetch_as_well_as_push() {
    let fixture = fixture();
    let missing_remote = fixture._temp.path().join("unavailable.git");
    git(
        &fixture.repo,
        &[
            "remote",
            "set-url",
            "origin",
            missing_remote.to_str().unwrap(),
        ],
    );
    let marker = fixture._temp.path().join("unused-hook-marker");
    let created = cli(&fixture, &["--no-push", "new", "offline"], &marker);
    assert!(created.status.success(), "{}", output_text(&created));
    assert!(output_text(&created).contains("local only"));
    assert!(fixture.repo.join(".tickets/02-offline.md").exists());
    assert_eq!(
        git(&fixture.repo, &["rev-parse", "HEAD^"]),
        git(&fixture.repo, &["rev-parse", "origin/main"])
    );
}

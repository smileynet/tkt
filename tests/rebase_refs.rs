//! Evidence fixture for collision rebase body-reference behavior.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn tkt_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("tkt");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", &dir.to_string_lossy()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .expect("git failed");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git_blob_hash(dir: &Path, bytes: &[u8]) -> String {
    let mut child = Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "hash-object", "--stdin"])
        .env("DO_NOT_TRACK", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("git hash-object failed to start");
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn body(bytes: &[u8]) -> &[u8] {
    let mut separators = bytes
        .windows(4)
        .enumerate()
        .filter(|(_, window)| *window == b"---\n");
    separators.next().unwrap();
    let end = separators.next().unwrap().0 + 4;
    &bytes[end..]
}

#[test]
fn collision_rebase_preserves_local_body_citations() {
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let a = tmp.path().join("clone_a");
    let b = tmp.path().join("clone_b");
    Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(&remote)
        .env("DO_NOT_TRACK", "1")
        .status()
        .unwrap();
    for clone in [&a, &b] {
        Command::new("git")
            .args(["clone", "-q"])
            .arg(&remote)
            .arg(clone)
            .env("DO_NOT_TRACK", "1")
            .status()
            .unwrap();
        git(clone, &["config", "user.email", "test@test"]);
        git(clone, &["config", "user.name", "test"]);
        git(clone, &["config", "core.autocrlf", "false"]);
    }

    std::fs::create_dir_all(a.join(".tickets")).unwrap();
    std::fs::write(a.join(".tickets/01-seed.md"), "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\nInline code `#991` and `ticket 992` are examples; https://example.invalid/issues#993, example.invalid/path#998, and owner/repo#994 are external. External link [#999](https://github.com/owner/repo/issues/999) is not local prose.\n\n```md\n#995 ticket 996\n```\n###997 is a Markdown heading, not a citation.\n").unwrap();
    git(&a, &["add", "-A"]);
    git(&a, &["commit", "-qm", "seed"]);
    git(&a, &["push", "-q", "origin", "HEAD:main"]);

    std::fs::write(a.join(".tickets/02-alpha.md"), "---\nid: \"02\"\ntitle: \"Alpha\"\nstatus: open\nblocked_by: []\n---\n\n# Alpha\nUpstream prose also cites #02 and ticket 02.\n").unwrap();
    git(&a, &["add", "-A"]);
    git(&a, &["commit", "-qm", "alpha"]);
    git(&a, &["push", "-q"]);

    git(&b, &["pull", "-q"]);
    std::fs::write(b.join(".tickets/02-beta.md"), "---\nid: \"02\"\ntitle: \"Beta\"\nstatus: open\nblocked_by: []\n---\n\n# Beta\nLocal prose cites old ID #02 and ticket 02 after collision; #999 and ticket 998 are unrelated.\n").unwrap();
    std::fs::write(b.join(".tickets/03-gamma.md"), "---\nid: \"03\"\ntitle: \"Gamma\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# Gamma\nDependency prose cites #02 and ticket 02.\n").unwrap();
    git(&b, &["add", "-A"]);
    git(&b, &["commit", "-qm", "beta and gamma"]);

    let before_beta = std::fs::read(b.join(".tickets/02-beta.md")).unwrap();
    let before_gamma = std::fs::read(b.join(".tickets/03-gamma.md")).unwrap();
    let remote_alpha_before = git(&remote, &["show", "main:.tickets/02-alpha.md"]);
    let remote_alpha_blob_before = git(&remote, &["rev-parse", "main:.tickets/02-alpha.md"]);
    let query = Command::new(tkt_bin())
        .args(["query", "-o", "json"])
        .current_dir(&b)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .output()
        .unwrap();
    assert!(
        query.status.success(),
        "query failed: {}",
        String::from_utf8_lossy(&query.stderr)
    );
    assert!(String::from_utf8_lossy(&query.stdout).contains("Beta"));
    assert!(String::from_utf8_lossy(&query.stdout).contains("Alpha"));

    let dry_run = Command::new(tkt_bin())
        .args(["--dry-run", "rebase"])
        .current_dir(&b)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .output()
        .unwrap();
    assert!(dry_run.status.success());
    let dry_run_stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert!(dry_run_stdout.contains("Body citations that may need review"));
    assert!(dry_run_stdout.contains(".tickets/04-beta.md"));
    assert_eq!(
        before_beta,
        std::fs::read(b.join(".tickets/02-beta.md")).unwrap()
    );
    assert_eq!(
        before_gamma,
        std::fs::read(b.join(".tickets/03-gamma.md")).unwrap()
    );

    let output = Command::new(tkt_bin())
        .arg("rebase")
        .current_dir(&b)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "rebase failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rebase_stdout = String::from_utf8_lossy(&output.stdout);
    assert!(rebase_stdout.contains("Body citations that may need review"));
    assert!(rebase_stdout.contains(".tickets/02-alpha.md"));
    assert!(rebase_stdout.contains(".tickets/04-beta.md"));
    assert!(rebase_stdout.contains(".tickets/03-gamma.md"));
    assert!(rebase_stdout.contains("#02` may refer to renumbered ID 02 → 04"));
    let beta = std::fs::read_dir(b.join(".tickets"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.ends_with("-beta.md"))
        .unwrap();
    let after_beta = std::fs::read(b.join(".tickets").join(beta)).unwrap();
    let after_gamma = std::fs::read(b.join(".tickets/03-gamma.md")).unwrap();
    let remote_alpha_after = git(&remote, &["show", "main:.tickets/02-alpha.md"]);
    let remote_alpha_blob_after = git(&remote, &["rev-parse", "main:.tickets/02-alpha.md"]);
    let validation = Command::new(tkt_bin())
        .args(["validate", "-o", "json"])
        .current_dir(&b)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .output()
        .unwrap();

    println!("rebase stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!(
        "local beta full blob before: {}",
        git_blob_hash(&b, &before_beta)
    );
    println!(
        "local beta full blob after: {}",
        git_blob_hash(&b, &after_beta)
    );
    println!(
        "local beta body blob before: {}",
        git_blob_hash(&b, body(&before_beta))
    );
    println!(
        "local beta body blob after: {}",
        git_blob_hash(&b, body(&after_beta))
    );
    println!(
        "local beta body length before/after: {}/{}",
        body(&before_beta).len(),
        body(&after_beta).len()
    );
    println!(
        "local gamma full blob before: {}",
        git_blob_hash(&b, &before_gamma)
    );
    println!(
        "local gamma full blob after: {}",
        git_blob_hash(&b, &after_gamma)
    );
    println!(
        "local gamma body blob before: {}",
        git_blob_hash(&b, body(&before_gamma))
    );
    println!(
        "local gamma body blob after: {}",
        git_blob_hash(&b, body(&after_gamma))
    );
    println!(
        "local gamma body length before/after: {}/{}",
        body(&before_gamma).len(),
        body(&after_gamma).len()
    );
    println!(
        "local beta before: {}",
        String::from_utf8_lossy(&before_beta)
    );
    println!("local beta after: {}", String::from_utf8_lossy(&after_beta));
    println!(
        "local gamma before: {}",
        String::from_utf8_lossy(&before_gamma)
    );
    println!(
        "local gamma after: {}",
        String::from_utf8_lossy(&after_gamma)
    );
    println!("upstream alpha before: {remote_alpha_before}");
    println!("upstream alpha after: {remote_alpha_after}");
    println!(
        "upstream alpha blob before/after: {}/{}",
        remote_alpha_blob_before, remote_alpha_blob_after
    );
    println!(
        "validate exit/stdout/stderr: {}/{}/{}",
        validation.status.code().unwrap_or(1),
        String::from_utf8_lossy(&validation.stdout),
        String::from_utf8_lossy(&validation.stderr)
    );
    assert_eq!(body(&before_beta), body(&after_beta));
    assert_eq!(body(&before_gamma), body(&after_gamma));
    let validation_output = String::from_utf8_lossy(&validation.stdout);
    assert!(validation.status.success());
    assert!(validation_output.contains("dangling-body-reference"));
    assert!(validation_output.contains("#999"));
    assert!(validation_output.contains("ticket 998"));
    assert_eq!(
        validation_output.matches("dangling-body-reference").count(),
        2
    );
    assert_eq!(validation_output.matches("#999").count(), 1);
    for ignored in [
        "#991",
        "ticket 992",
        "#993",
        "#994",
        "#995",
        "ticket 996",
        "#997",
        "#998",
    ] {
        assert!(
            !validation_output.contains(ignored),
            "unexpected warning for {ignored}: {validation_output}"
        );
    }
    assert_eq!(remote_alpha_before, remote_alpha_after);
    assert_eq!(remote_alpha_blob_before, remote_alpha_blob_after);
}

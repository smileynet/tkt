//! Private integration executable: inject remote and branch races at exact Git boundaries.
use std::ffi::OsStr;
use std::process::{Command, ExitCode, Stdio};

fn advance(real: &OsStr, root: &OsStr, text: &str) {
    std::fs::write(std::path::Path::new(root).join("README.md"), text).unwrap();
    for tail in [
        vec!["add", "README.md"],
        vec![
            "commit",
            "-qm",
            "concurrent mutation",
            "--only",
            "--",
            "README.md",
        ],
    ] {
        assert!(Command::new(real)
            .args([
                OsStr::new("-C"),
                root,
                OsStr::new("-c"),
                OsStr::new("core.hooksPath=")
            ])
            .args(tail)
            .env("DO_NOT_TRACK", "1")
            .status()
            .unwrap()
            .success());
    }
}
fn main() -> ExitCode {
    let real = std::env::var_os("TKT_TEST_REAL_GIT").expect("private Git path");
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let root = &args[args.iter().position(|arg| arg == "-C").unwrap() + 1];
    if args.iter().any(|arg| arg == "push") {
        let race = std::env::var_os("TKT_TEST_GIT_RACE").unwrap();
        if let Ok(text) = std::fs::read_to_string(&race) {
            std::fs::remove_file(race).unwrap();
            let rows: Vec<_> = text.lines().collect();
            assert!(Command::new(&real)
                .arg(format!("--git-dir={}", rows[0]))
                .args(["update-ref", "refs/heads/main", rows[1], rows[2]])
                .env("DO_NOT_TRACK", "1")
                .status()
                .unwrap()
                .success());
            if rows[3] == "advanced" {
                advance(&real, root, "Other local mutation\n");
            }
        }
    }
    if args.iter().any(|arg| arg == "tkt import allocation retry")
        && std::env::var_os("TKT_TEST_CAS_MISMATCH")
            .is_some_and(|path| std::path::Path::new(&path).exists())
    {
        advance(&real, root, "Concurrent mutator data\n");
    }
    let status = Command::new(real)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    ExitCode::from(status.code().unwrap_or(1) as u8)
}

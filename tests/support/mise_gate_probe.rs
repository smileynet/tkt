use std::io::Write;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    assert_eq!(
        std::env::args().skip(1).collect::<Vec<_>>(),
        ["run", "check"]
    );
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var_os("TKT_GATE_FIXTURE_LOG").unwrap())
        .unwrap();
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(head.status.success());
    writeln!(log, "{}", String::from_utf8(head.stdout).unwrap().trim()).unwrap();
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_CONFIG_COUNT",
        "GIT_PREFIX",
    ] {
        assert!(
            std::env::var_os(key).is_none(),
            "inherited local Git variable {key}"
        );
    }
    match std::env::var("TKT_GATE_FIXTURE_MODE").unwrap().as_str() {
        "fail" => ExitCode::from(1),
        "sleep" => {
            std::thread::sleep(std::time::Duration::from_secs(10));
            ExitCode::SUCCESS
        }
        "head" => {
            let status = Command::new("git")
                .args([
                    "-c",
                    "core.hooksPath=",
                    "commit",
                    "--allow-empty",
                    "-qm",
                    "concurrent head",
                ])
                .env("DO_NOT_TRACK", "1")
                .status()
                .unwrap();
            assert!(status.success());
            ExitCode::SUCCESS
        }
        "dirty" => {
            std::fs::write("README.md", "Changed during gate\n").unwrap();
            ExitCode::SUCCESS
        }
        _ => {
            println!("Fixture full-check command completed");
            ExitCode::SUCCESS
        }
    }
}

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture {
    temp: tempfile::TempDir,
    root: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory,
    Link(PathBuf),
    File(Vec<u8>),
}

#[test]
fn remote_collision_preview_uses_cached_state_without_mutating_either_repository() {
    let f = Fixture::new();
    let remote = f.temp.path().join("remote.git");
    f.git(&["init", "--bare", "-q", remote.to_str().unwrap()]);
    f.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
    f.git(&["push", "-qu", "origin", "main"]);
    let other = f.temp.path().join("other");
    f.git(&[
        "clone",
        "-q",
        "--branch",
        "main",
        remote.to_str().unwrap(),
        other.to_str().unwrap(),
    ]);
    for (key, value) in [
        ("user.name", "Remote fixture"),
        ("user.email", "fixture@example.invalid"),
        ("gc.auto", "0"),
        ("maintenance.auto", "false"),
    ] {
        f.git(&["-C", other.to_str().unwrap(), "config", key, value]);
    }
    f.git(&[
        "-C",
        other.to_str().unwrap(),
        "mv",
        ".tickets/01-fixture.md",
        ".tickets/01-upstream.md",
    ]);
    f.git(&[
        "-C",
        other.to_str().unwrap(),
        "commit",
        "-qm",
        "upstream collision",
    ]);
    f.git(&["-C", other.to_str().unwrap(), "push", "-q"]);
    f.git(&["fetch", "-q", "origin"]);
    for args in [vec!["--dry-run", "rebase"], vec!["rebase", "--dry-run"]] {
        let before = f.snapshot();
        let out = f.cli(&args);
        assert!(out.status.success(), "{out:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("Collisions detected"),
            "{out:?}"
        );
        assert_eq!(f.snapshot(), before);
    }
}

#[test]
fn import_preview_and_pending_recovery_preserve_all_local_state() {
    let f = Fixture::new();
    let bin = f.temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let node = f
        .command("node")
        .args(["--print", "process.execPath"])
        .output()
        .unwrap();
    assert!(
        node.status.success(),
        "Node fixture infrastructure: {node:?}"
    );
    let node = PathBuf::from(String::from_utf8(node.stdout).unwrap().trim());
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&node, bin.join("gh.exe")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&node, bin.join("gh")).unwrap();
    let preload = f.temp.path().join("provider.cjs");
    std::fs::write(&preload, r#"
const args=process.argv.slice(1);
if(require('node:path').basename(args[0])!=='api'||args[args.indexOf('--method')+1]!=='GET')process.exit(71);
const endpoint=args.find(a=>a.startsWith('repos/'));
process.stdout.write(JSON.stringify(endpoint.endsWith('/issues')?
[{id:101,number:1,state:'open',title:'Imported owner specification',body:'Preserve source prose',html_url:'https://github.com/owner/project/issues/1',repository_url:'https://api.github.com/repos/owner/project',labels:[]}]:
{id:77,full_name:'owner/project',html_url:'https://github.com/owner/project'}));process.exit(0);
"#).unwrap();
    let mut path = vec![bin];
    path.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    for pending in [false, true] {
        if pending {
            std::fs::write(
                f.root.join(".git/tkt-github-import.json"),
                "owner recovery bytes",
            )
            .unwrap();
        }
        let before = f.snapshot();
        let out = f
            .command(env!("CARGO_BIN_EXE_tkt"))
            .args([
                "--dry-run",
                "import",
                "--github",
                "--repo",
                "owner/project",
                "--vc",
                "Verify imported work",
            ])
            .env("PATH", std::env::join_paths(&path).unwrap())
            .env(
                "NODE_OPTIONS",
                format!(
                    "--require=\"{}\"",
                    preload.to_string_lossy().replace('\\', "/")
                ),
            )
            .env("GH_TOKEN", "fixture-invalid-token")
            .env("GH_CONFIG_DIR", f.temp.path().join("private-gh"))
            .env_remove("GITHUB_TOKEN")
            .env_remove("GH_ENTERPRISE_TOKEN")
            .env_remove("GITHUB_ENTERPRISE_TOKEN")
            .output()
            .unwrap();
        assert_eq!(out.status.success(), !pending, "{out:?}");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            text.contains(if pending {
                "pending import journal"
            } else {
                "would import"
            }),
            "{text}"
        );
        assert_eq!(f.snapshot(), before);
    }
}

impl Fixture {
    fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(&self.root)
            .env("DO_NOT_TRACK", "1")
            .env("TKT_NO_USER_CONFIG", "1")
            .env("TKT_UPDATE_CHECK", "0")
            .env("TKT_DEBUG", "log")
            .env("APPDATA", self.temp.path().join("config"))
            .env("LOCALAPPDATA", self.temp.path().join("data"))
            .env("XDG_CONFIG_HOME", self.temp.path().join("config"))
            .env("XDG_DATA_HOME", self.temp.path().join("data"))
            .env("XDG_STATE_HOME", self.temp.path().join("state"))
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_COUNT", "0")
            .env("GIT_OPTIONAL_LOCKS", "0");
        command.env("TKT_PREVIEW_MARKER", self.temp.path().join("hook-ran"));
        for key in [
            "TKT_AGENT",
            "TKT_CONTEXT",
            "TKT_HOOK_ACTIVE",
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_CONFIG_PARAMETERS",
            "NODE_OPTIONS",
        ] {
            command.env_remove(key);
        }
        command
    }

    fn git(&self, args: &[&str]) -> String {
        let out = self.command("git").args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap().trim().into()
    }

    fn cli(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_tkt"))
            .args(args)
            .output()
            .unwrap()
    }

    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("tkt-mutation-preview-")
            .tempdir()
            .unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        let f = Self { temp, root };
        f.git(&["init", "-q", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Preview fixture"),
            ("user.email", "fixture@example.invalid"),
            ("core.autocrlf", "false"),
            ("gc.auto", "0"),
            ("maintenance.auto", "false"),
        ] {
            f.git(&["config", key, value]);
        }
        std::fs::create_dir(f.root.join(".tickets")).unwrap();
        let hook = serde_json::to_string(&vec![
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            "--exact".into(),
            "preview_hook_probe".into(),
            "--ignored".into(),
        ])
        .unwrap();
        std::fs::write(f.root.join(".tickets/config.toml"), format!("[push]\nenabled=false\n[close]\nallow_force=true\n[hooks]\non_new={hook}\non_claim={hook}\non_close={hook}\n")).unwrap();
        f.git(&["config", "--local", "tkt.lifecycleHooks", "true"]);
        for (id, status, check) in [
            ("01", "open", " "),
            ("02", "in_progress", " "),
            ("03", "done", "x"),
            ("04", "open", " "),
        ] {
            std::fs::write(f.root.join(format!(".tickets/{id}-fixture.md")), format!("---\nid: \"{id}\"\ntitle: \"Fixture {id}\"\nstatus: {status}\nblocked_by: []\nvalidation_criteria:\n  - \"Observable fixture behavior is verified\"\n---\n\nThe fixture preserves owner specification while testing explicit command effects.\n\n## Acceptance criteria\n\n- [{check}] Observable fixture behavior is verified\n{}", if status == "done" { "\n## Resolution\n\nVerified the fixture behavior against its private repository and retained evidence.\n" } else { "" })).unwrap();
        }
        std::fs::create_dir(f.root.join("docs")).unwrap();
        std::fs::write(
            f.root.join("docs/plan.md"),
            "| ID | Work | Status |\n|---|---|---|\n| 01 | Fixture | done |\n",
        )
        .unwrap();
        std::fs::write(f.root.join(".tickets/.context"), "old\n").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-qm", "seed"]);
        f
    }

    fn snapshot(&self) -> BTreeMap<PathBuf, Entry> {
        fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Entry>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let relative = entry.path().strip_prefix(root).unwrap().to_path_buf();
                if entry.file_type().unwrap().is_dir() {
                    out.insert(relative, Entry::Directory);
                    visit(root, &entry.path(), out);
                } else if entry.file_type().unwrap().is_symlink() {
                    out.insert(
                        relative,
                        Entry::Link(std::fs::read_link(entry.path()).unwrap()),
                    );
                } else {
                    out.insert(relative, Entry::File(std::fs::read(entry.path()).unwrap()));
                }
            }
        }
        let mut out = BTreeMap::new();
        visit(self.temp.path(), self.temp.path(), &mut out);
        out
    }
}

#[test]
fn local_mutation_previews_preserve_files_git_and_process_markers() {
    let cases: &[(&str, &[&str])] = &[
        (
            "new",
            &[
                "new",
                "preview",
                "--title",
                "Preview",
                "--vc",
                "Verify preview",
            ],
        ),
        (
            "batch",
            &[
                "batch",
                "first:First",
                "second:Second",
                "--vc",
                "Verify preview",
            ],
        ),
        ("claim", &["claim", "01"]),
        (
            "close",
            &[
                "close",
                "02",
                "--force",
                "--resolution",
                "Verified private fixture acceptance and preserved evidence for this preview.",
                "--check-all",
                "--evidence",
                "Private fixture assertion",
            ],
        ),
        (
            "cancel",
            &[
                "cancel",
                "01",
                "--reason",
                "wontfix",
                "--note",
                "Fixture cancellation preview",
            ],
        ),
        ("reopen", &["reopen", "01"]),
        ("edit", &["edit", "01", "--title", "Changed"]),
        ("note", &["note", "01", "--append", "New owner note"]),
        ("renumber", &["renumber", "01", "05"]),
        ("move-ac", &["move-ac", "01", "--ac", "1", "--to", "04"]),
        ("plan-fix", &["sync-plan", "--fix"]),
        (
            "plan-generate",
            &["sync-plan", "--generate", "projection.md"],
        ),
        ("validate-fix", &["validate", "--fix"]),
        ("lint", &["lint"]),
        ("context-set", &["context", "new"]),
        ("context-clear", &["context", "--clear"]),
        ("config-set", &["config", "--set", "debug=true"]),
        ("config-unset", &["config", "--unset", "debug"]),
        (
            "project-set",
            &["config", "--project", "--set", "push.enabled=true"],
        ),
        (
            "project-unset",
            &["config", "--project", "--unset", "push.enabled"],
        ),
        ("init", &["init", "--all"]),
        ("native-install", &["init", "--write-nudge", "both"]),
        (
            "native-check",
            &["init", "--write-nudge", "both", "--nudge-action", "check"],
        ),
        (
            "native-remove",
            &["init", "--write-nudge", "both", "--nudge-action", "remove"],
        ),
        ("hooks-install", &["hooks", "install"]),
        ("hooks-remove", &["hooks", "remove"]),
        ("migrate", &["migrate", "--from", "tk", "--confirm"]),
        ("undo", &["undo"]),
        ("doctor", &["doctor", "--fix"]),
    ];
    let mut covered: std::collections::BTreeSet<_> =
        cases.iter().map(|(_, args)| args[0]).collect();
    // Import/rebase have separate remote/provider fixtures below. Telemetry's
    // private-path unit matrix never invokes ambient Windows Known Folders.
    covered.extend(["import", "rebase", "telemetry"]);
    let inventory_fixture = Fixture::new();
    let inventory = inventory_fixture.cli(&["capabilities", "-o", "json"]);
    assert!(inventory.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&inventory.stdout).unwrap();
    let declared: std::collections::BTreeSet<_> = manifest["commands"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, command)| command["effects"] != "read_only")
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(covered, declared, "new mutators require a preview fixture");
    let mut failures = Vec::new();
    for (name, args) in cases {
        let f = Fixture::new();
        match *name {
            "native-remove" | "native-check" => {
                assert!(f.cli(&["init", "--write-nudge", "both"]).status.success())
            }
            "hooks-remove" => assert!(f.cli(&["hooks", "install"]).status.success()),
            "undo" => assert!(f.cli(&["claim", "01"]).status.success()),
            "reopen" => assert!(f
                .cli(&[
                    "cancel",
                    "01",
                    "--reason",
                    "obsolete",
                    "--note",
                    "Private cancelled fixture"
                ])
                .status
                .success()),
            "migrate" => {
                std::fs::write(f.root.join(".tickets/legacy.md"), "---\nstatus: open\ndeps: []\n---\n\n# Legacy fixture\n\nOwner migration specification\n").unwrap();
            }
            "validate-fix" | "lint" => {
                let path = f.root.join(".tickets/01-fixture.md");
                let text = std::fs::read_to_string(&path)
                    .unwrap()
                    .replace("id: \"01\"", "id : 01");
                std::fs::write(path, text).unwrap();
            }
            _ => (),
        }
        let marker = f.temp.path().join("hook-ran");
        if *name == "undo" {
            assert!(
                marker.exists(),
                "real claim did not exercise the hook control"
            );
            std::fs::remove_file(&marker).unwrap();
        }
        for prefix in ["config", "data"] {
            let dir = f.temp.path().join(prefix).join("tkt/telemetry");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("fixture.jsonl"), "existing local telemetry\n").unwrap();
        }
        let before = f.snapshot();
        let mut preview = vec!["--dry-run"];
        preview.extend_from_slice(args);
        let out = f.cli(&preview);
        let after = f.snapshot();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .to_lowercase();
        let expected = match *name {
            "plan-fix" => "plan-status-drift",
            "plan-generate" => "<!-- tkt:projection:begin -->",
            "migrate" => "migration plan",
            "native-check" => "installed native write nudge",
            "doctor" => "--fix is not yet implemented",
            _ => "would",
        };
        let changed: Vec<_> = before
            .keys()
            .chain(after.keys())
            .filter(|path| before.get(*path) != after.get(*path))
            .collect();
        if !out.status.success() || !changed.is_empty() || !text.contains(expected) {
            failures.push(format!(
                "{name}: status={} changed={changed:?}\n{}\n{}",
                out.status,
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
#[ignore = "subprocess entry point, invoked explicitly by the private hook control"]
fn preview_hook_probe() {
    std::fs::write(std::env::var_os("TKT_PREVIEW_MARKER").unwrap(), "hook ran").unwrap();
}

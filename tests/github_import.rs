use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    command
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_DEBUG", "0")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env_remove("TKT_AGENT")
        .env_remove("TKT_CONTEXT")
        .env_remove("NODE_OPTIONS");
    command
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = command("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn hook(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    provider: PathBuf,
    bin: PathBuf,
    preload: PathBuf,
    real_git: Option<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("tkt-github-import-")
            .tempdir()
            .unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.name", "Import fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        std::fs::create_dir(root.join(".tickets")).unwrap();
        std::fs::write(root.join(".tickets/config.toml"), "[push]\nenabled=false\n").unwrap();
        std::fs::write(root.join(".tickets/01-seed.md"),"---\nid: \"01\"\ntitle: \"Existing ticket\"\nstatus: open\nblocked_by: []\n---\n\nOwner specification\n").unwrap();
        std::fs::write(root.join("README.md"), "Owner source\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-qm", "seed"]);
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let node = command("node")
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
        let preload = temp.path().join("fake-gh.cjs");
        std::fs::write(&preload,r#"
const fs=require('node:fs');
const data=JSON.parse(fs.readFileSync(process.env.TKT_TEST_GH_DATA,'utf8'));
const args=process.argv.slice(1);
if(require('node:path').basename(args[0])!=='api'||args[args.indexOf('--method')+1]!=='GET'||args[args.indexOf('--hostname')+1]!=='github.com'||!args.includes('X-GitHub-Api-Version: 2026-03-10'))process.exit(71);
const endpoint=args.find(a=>a.startsWith('repos/'));
const fields=Object.fromEntries(args.flatMap((a,i)=>a==='--raw-field'?[args[i+1].split(/=(.*)/s).slice(0,2)]:[]));
fs.appendFileSync(process.env.TKT_TEST_GH_DATA+'.requests',JSON.stringify({endpoint,fields})+'\n');
if(!endpoint.endsWith('/issues')){process.stdout.write(JSON.stringify(data.repository));process.exit(0)}
if(fields.state!=='all'||fields.per_page!=='100')process.exit(72);
if(data.failPage){process.stderr.write('authentication failure secret-token-must-not-leak');process.exit(1)}
if(data.huge){process.stdout.write(' '.repeat(10*1024*1024));process.exit(0)}
if(!args.includes('--paginate'))process.exit(73);
process.stdout.write(data.pages.map(page=>JSON.stringify(page)).join('\n'));process.exit(0);
"#).unwrap();
        let provider = temp.path().join("source.json");
        let fixture = Self {
            _temp: temp,
            root,
            provider,
            bin,
            preload,
            real_git: None,
        };
        fixture.data(json!([
            issue(101, 1, "open"),
            issue(102, 2, "closed"),
            pull_request()
        ]));
        fixture
    }
    fn data(&self, rows: Value) {
        self.provider(json!({"repository":{"id":77,"full_name":"owner/project","html_url":"https://github.com/owner/project"},"pages":[rows]}));
    }
    fn provider(&self, value: Value) {
        std::fs::write(&self.provider, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    fn cli(&self, args: &[&str]) -> Command {
        let mut command = command(env!("CARGO_BIN_EXE_tkt"));
        let mut path = vec![self.bin.clone()];
        path.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
        command
            .current_dir(&self.root)
            .args(args)
            .env("PATH", std::env::join_paths(path).unwrap())
            .env(
                "NODE_OPTIONS",
                format!(
                    "--require=\"{}\"",
                    self.preload.to_string_lossy().replace('\\', "/")
                ),
            )
            .env("TKT_TEST_GH_DATA", &self.provider)
            .env("GH_TOKEN", "fixture-invalid-token")
            .env_remove("GITHUB_TOKEN")
            .env_remove("GH_ENTERPRISE_TOKEN")
            .env_remove("GITHUB_ENTERPRISE_TOKEN")
            .env("GH_CONFIG_DIR", self._temp.path().join("private-gh"))
            .env("XDG_CONFIG_HOME", self._temp.path().join("private-config"))
            .env("XDG_STATE_HOME", self._temp.path().join("private-state"));
        if let Some(real) = &self.real_git {
            command
                .env("TKT_TEST_REAL_GIT", real)
                .env("TKT_TEST_GIT_RACE", self._temp.path().join("race"))
                .env(
                    "TKT_TEST_CAS_MISMATCH",
                    self._temp.path().join("cas-mismatch"),
                );
        }
        command
    }
    fn import(&self, extra: &[&str]) -> Output {
        let mut args = vec![
            "import",
            "--github",
            "--repo",
            "owner/project",
            "-o",
            "json",
        ];
        args.extend_from_slice(extra);
        self.cli(&args).output().unwrap()
    }
    fn good(&self, extra: &[&str]) -> Value {
        let out = self.import(extra);
        assert!(out.status.success(), "{extra:?}: {out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn pair(&self, id: &str) -> Value {
        let ticket_path = format!(".tickets/{id}-github-101.md");
        let snapshot_path = ".tickets/imports/github-v1/101.json";
        json!({"source_id":101,"local_id":id,"ticket_path":ticket_path,"ticket":std::fs::read_to_string(self.root.join(&ticket_path)).unwrap(),"snapshot_path":snapshot_path,"snapshot":std::fs::read_to_string(self.root.join(snapshot_path)).unwrap()})
    }

    fn cas_probe(&mut self) {
        let name = if cfg!(windows) { "git.exe" } else { "git" };
        self.real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|dir| dir.join(name))
            .find(|file| file.is_file());
        assert!(self.real_git.is_some(), "Git fixture infrastructure");
        let output = command("rustc")
            .args(["--edition=2021"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/git_cas_probe.rs"))
            .arg("-o")
            .arg(self.bin.join(name))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Git CAS probe compilation: {output:?}"
        );
    }
}
fn issue(id: u64, number: u64, state: &str) -> Value {
    json!({"id":id,"number":number,"state":state,"title":"Quote \" and Ω\nnext line","body":"#999\n- [ ] Provider checkbox\n```\n## Resolution\n```","html_url":format!("https://github.com/owner/project/issues/{number}"),"repository_url":"https://api.github.com/repos/owner/project","labels":[{"name":"priority:urgent","unknown":true}],"assignees":[{"id":42}],"future":{"arbitrary":[null,true,1]}})
}
fn pull_request() -> Value {
    let mut value = issue(103, 3, "open");
    value["pull_request"] = json!({"url":"kept raw"});
    value["html_url"] = json!("https://github.com/owner/project/pull/3");
    value
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, path: &Path, map: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, map)
            } else {
                map.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut map = BTreeMap::new();
    walk(root, root, &mut map);
    map
}

#[test]
fn bounded_preview_mapping_and_create_only_replay_preserve_source_and_local_work() {
    let f = Fixture::new();
    std::fs::write(f.root.join("unrelated.txt"), "preserve me").unwrap();
    let before = snapshot(&f.root);
    let preview = f.good(&["--dry-run"]);
    assert_eq!(preview["imported"], json!(["02"]));
    assert_eq!(snapshot(&f.root), before);
    let result = f.good(&[]);
    assert_eq!(result["imported"], json!(["02"]));
    assert_eq!(result["skipped_closed"], 1);
    assert_eq!(result["skipped_pull_requests"], 1);
    let snapshot_value: Value = serde_json::from_slice(
        &std::fs::read(f.root.join(".tickets/imports/github-v1/101.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(snapshot_value["issue"], issue(101, 1, "open"));
    let path = f.root.join(".tickets/02-github-101.md");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("import_source: \"github.com:issue:101\""));
    assert!(text.contains("````markdown\n#999\n- [ ] Provider checkbox"));
    assert!(text.contains("tags: [\"import:github\"]"));
    assert!(!text.contains("tags: [\"priority:urgent\"]"));
    let changed = git(
        &f.root,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"],
    );
    assert_eq!(changed.lines().count(), 2);
    assert!(!changed.contains("unrelated.txt"));
    assert_eq!(
        std::fs::read_to_string(f.root.join("unrelated.txt")).unwrap(),
        "preserve me"
    );
    std::fs::write(&path, format!("{text}\nOwner note and evidence\n")).unwrap();
    git(&f.root, &["add", ".tickets/02-github-101.md"]);
    git(&f.root, &["commit", "-qm", "owner note"]);
    let mut changed = issue(101, 1, "open");
    changed["body"] = json!("Provider changed later");
    f.data(json!([changed]));
    let before = snapshot(&f.root);
    let result = f.good(&[]);
    assert_eq!(result["skipped_existing"], 1);
    assert_eq!(result["imported"], json!([]));
    assert_eq!(snapshot(&f.root), before);
    assert!(f
        .cli(&["query", "--all-context"])
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn invalid_or_incomplete_provider_reads_make_no_local_changes() {
    let f = Fixture::new();
    let mut missing = issue(101, 1, "open");
    missing.as_object_mut().unwrap().remove("id");
    let mut state = issue(101, 1, "open");
    state["state"] = json!("future-state");
    let mut provenance = issue(101, 1, "open");
    provenance["repository_url"] = json!("https://api.github.com/repos/other/project");
    for rows in [json!([missing]), json!([state]), json!([provenance])] {
        f.data(rows);
        let before = snapshot(&f.root);
        let out = f.import(&[]);
        assert!(!out.status.success(), "{out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
    let metadata =
        json!({"id":77,"full_name":"owner/project","html_url":"https://github.com/owner/project"});
    for provider in [
        json!({"repository":metadata,"pages":[],"failPage":1}),
        json!({"repository":metadata,"pages":[],"huge":true}),
        json!({"repository":metadata,"pages":vec![vec![issue(101,1,"closed");100];10]}),
    ] {
        f.provider(provider);
        let before = snapshot(&f.root);
        let out = f.import(&[]);
        assert!(!out.status.success(), "{out:?}");
        assert!(!String::from_utf8_lossy(&out.stderr).contains("secret-token-must-not-leak"));
        assert_eq!(snapshot(&f.root), before);
    }
}

#[test]
fn ownership_and_clean_index_conflicts_are_refused_before_writes() {
    let f = Fixture::new();
    std::fs::write(f.root.join("README.md"), "staged owner work").unwrap();
    git(&f.root, &["add", "README.md"]);
    let before = snapshot(&f.root);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    git(&f.root, &["reset", "--hard", "HEAD"]);
    f.good(&[]);
    let path = f.root.join(".tickets/imports/github-v1/101.json");
    let original = std::fs::read(&path).unwrap();
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    value["ticket_blob"] = json!("forged");
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "malformed owned receipt"]);
    let before = snapshot(&f.root);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn interrupted_pairs_resume_same_identity_without_duplicates() {
    for partial in ["none", "ticket", "snapshot", "both", "staged", "committed"] {
        let f = Fixture::new();
        f.data(json!([issue(101, 1, "open")]));
        let base = git(&f.root, &["rev-parse", "HEAD"]);
        f.good(&[]);
        let pair = f.pair("02");
        let commit = git(&f.root, &["rev-parse", "HEAD"]);
        if partial != "committed" {
            git(&f.root, &["reset", "--hard", &base]);
        }
        for (key, text_key) in [("ticket_path", "ticket"), ("snapshot_path", "snapshot")] {
            if partial == key.trim_end_matches("_path") || matches!(partial, "both" | "staged") {
                let path = f.root.join(pair[key].as_str().unwrap());
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, pair[text_key].as_str().unwrap()).unwrap();
            }
        }
        if partial == "staged" {
            git(&f.root, &["add", ".tickets"]);
        }
        let value = json!({"format":"tkt-github-import/v1","branch":"refs/heads/main","selector":{"repository":"owner/project","label":null,"validation_criteria":[]},"source":serde_json::from_slice::<Value>(&std::fs::read(&f.provider).unwrap()).unwrap(),"phase":if partial=="committed"{"applied"}else{"prepared"},"base":base,"commit":null,"pairs":[pair]});
        std::fs::write(
            f.root.join(".git/tkt-github-import.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        f.good(&[]);
        assert_eq!(f.pair("02"), pair, "{partial}");
        assert!(!f.root.join(".git/tkt-github-import.json").exists());
        assert!(!f.root.join(".git/tkt-allocation.lock").exists());
        assert_eq!(
            git(&f.root, &["ls-files", ".tickets/*github*.md"])
                .lines()
                .count(),
            1
        );
        if partial == "committed" {
            assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), commit);
        }
    }
}

#[test]
fn failed_commit_preserves_owned_pair_and_can_resume() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    std::fs::write(
        f.root.join(".tickets/config.toml"),
        "[push]\nenabled=false\n[commit]\nhooks=\"run\"\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "opt in"]);
    hook(&f.root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 1\n");
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), base);
    let pair = f.pair("02");
    assert!(f.root.join(".git/tkt-github-import.json").exists());
    std::fs::remove_file(f.root.join(".git/hooks/pre-commit")).unwrap();
    f.good(&[]);
    assert_eq!(f.pair("02"), pair);
    assert!(!f.root.join(".git/tkt-github-import.json").exists());
}

#[test]
fn labels_null_bodies_duplicate_pages_and_renames_follow_source_identity() {
    let f = Fixture::new();
    let mut row = issue(101, 1, "open");
    row["body"] = Value::Null;
    let mut other = issue(104, 4, "open");
    other["labels"] = json!([]);
    f.provider(json!({"repository":{"id":77,"full_name":"owner/project","html_url":"https://github.com/owner/project"},"pages":[[row.clone(),other],[row.clone()]]}));
    let result = f.good(&["--label", "priority:urgent"]);
    assert_eq!(result["imported"], json!(["02"]));
    assert_eq!(result["received_objects"], 3);
    assert_eq!(result["skipped_duplicates"], 1);
    let snapshot_value: Value = serde_json::from_slice(
        &std::fs::read(f.root.join(".tickets/imports/github-v1/101.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(snapshot_value["issue"]["body"], Value::Null);
    row["html_url"] = json!("https://github.com/owner/renamed/issues/1");
    row["repository_url"] = json!("https://api.github.com/repos/owner/renamed");
    f.provider(json!({"repository":{"id":77,"full_name":"owner/renamed","html_url":"https://github.com/owner/renamed"},"pages":[[row]]}));
    let before = snapshot(&f.root);
    let out = f
        .cli(&[
            "import",
            "--github",
            "--repo",
            "owner/renamed",
            "-o",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["skipped_existing"], 1);
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn pending_recovery_is_selector_bound_and_does_not_need_github() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    let source: Value = serde_json::from_slice(&std::fs::read(&f.provider).unwrap()).unwrap();
    f.good(&[]);
    let pair = f.pair("02");
    git(&f.root, &["reset", "--hard", &base]);
    let value = json!({"format":"tkt-github-import/v1","branch":"refs/heads/main","selector":{"repository":"owner/project","label":null,"validation_criteria":[]},"source":source,"phase":"prepared","base":base,"commit":null,"pairs":[pair]});
    std::fs::write(
        f.root.join(".git/tkt-github-import.json"),
        serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    f.provider(json!({"repository":null,"pages":[],"failPage":1}));
    let before = snapshot(&f.root);
    let out = f.import(&["--label", "different"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    let before_requests = std::fs::read(f.provider.with_extension("json.requests")).unwrap();
    let result = f.good(&[]);
    assert_eq!(result["recovered"], true);
    assert_eq!(f.pair("02"), pair);
    assert_eq!(
        std::fs::read(f.provider.with_extension("json.requests")).unwrap(),
        before_requests
    );
}

fn remote_race(f: &Fixture, advance_local: bool) -> PathBuf {
    let remote = f._temp.path().join("remote.git");
    let out = command("git")
        .args(["init", "--bare", "-q"])
        .arg(&remote)
        .output()
        .unwrap();
    assert!(out.status.success());
    std::fs::write(
        f.root.join(".tickets/config.toml"),
        "[push]\nenabled=true\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "push configuration"]);
    git(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.root, &["push", "-qu", "origin", "main"]);
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    let peer = f._temp.path().join("peer");
    let out = command("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&peer)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    git(&peer, &["checkout", "-q", "main"]);
    git(&peer, &["config", "user.name", "Peer fixture"]);
    git(&peer, &["config", "user.email", "peer@example.invalid"]);
    std::fs::write(peer.join(".tickets/02-peer.md"),"---\nid: \"02\"\ntitle: \"Peer allocation\"\nstatus: open\nblocked_by: []\n---\n\nPeer source\n").unwrap();
    git(&peer, &["add", "."]);
    git(&peer, &["commit", "-qm", "peer allocation"]);
    let advance = git(&peer, &["rev-parse", "HEAD"]);
    git(
        &peer,
        &["push", "-q", "origin", "HEAD:refs/heads/test-race-object"],
    );
    let remote = remote.to_string_lossy().replace('\\', "/");
    std::fs::write(
        f._temp.path().join("race"),
        format!(
            "{remote}\n{advance}\n{base}\n{}\n",
            if advance_local { "advanced" } else { "retry" }
        ),
    )
    .unwrap();
    PathBuf::from(remote)
}

#[test]
fn rejected_push_reallocates_from_the_same_snapshot_and_preserves_other_commits() {
    for mode in ["retry", "advanced", "cas-mismatch"] {
        let mut f = Fixture::new();
        f.data(json!([issue(101, 1, "open")]));
        let remote = remote_race(&f, mode == "advanced");
        f.cas_probe();
        if mode == "cas-mismatch" {
            std::fs::write(f._temp.path().join("cas-mismatch"), "once").unwrap();
        }
        let out = f.import(&[]);
        if mode == "retry" {
            assert!(out.status.success(), "{out:?}");
            let result: Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(result["imported"], json!(["03"]));
            assert!(f.root.join(".tickets/02-peer.md").is_file());
            assert!(!f.root.join(".tickets/02-github-101.md").exists());
            assert!(!f.root.join(".git/tkt-github-import.json").exists());
            assert_eq!(
                git(&f.root, &["rev-parse", "HEAD"]),
                git(&remote, &["rev-parse", "main"])
            );
            assert_eq!(
                std::fs::read_to_string(f.provider.with_extension("json.requests"))
                    .unwrap()
                    .lines()
                    .count(),
                2,
                "retry must reuse extraction"
            );
        } else {
            assert!(!out.status.success(), "{out:?}");
            assert!(std::fs::read_to_string(f.root.join("README.md"))
                .unwrap()
                .contains(if mode == "advanced" {
                    "Other local mutation"
                } else {
                    "Concurrent mutator data"
                }));
            assert!(f.root.join(".tickets/02-github-101.md").is_file());
            assert!(f.root.join(".git/tkt-github-import.json").exists());
            let before = snapshot(&f.root);
            let again = f.import(&[]);
            assert!(!again.status.success(), "{again:?}");
            assert_eq!(
                snapshot(&f.root),
                before,
                "unexpected concurrent commit must remain intact"
            );
        }
    }
}

#[test]
fn failed_push_keeps_a_durable_commit_and_recovery_retries_without_duplicates() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    let remote = remote_race(&f, false);
    std::fs::remove_file(f._temp.path().join("race")).unwrap();
    hook(&remote.join("hooks/pre-receive"), "#!/bin/sh\nexit 1\n");
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    let commit = git(&f.root, &["rev-parse", "HEAD"]);
    assert_ne!(commit, base);
    let pair = f.pair("02");
    assert!(f.root.join(".git/tkt-github-import.json").exists());
    std::fs::remove_file(remote.join("hooks/pre-receive")).unwrap();
    f.provider(json!({"repository":null,"pages":[],"failPage":1}));
    let result = f.good(&[]);
    assert_eq!(result["recovered"], true);
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), commit);
    assert_eq!(f.pair("02"), pair);
    assert_eq!(git(&remote, &["rev-parse", "main"]), commit);
    assert!(!f.root.join(".git/tkt-github-import.json").exists());
}

#[test]
fn rollback_journal_resumes_ref_index_and_file_boundaries_without_new_provider_reads() {
    for state in [
        "before-cas",
        "after-cas",
        "after-index",
        "partial-cleanup",
        "after-cleanup",
        "rolled-back",
    ] {
        let f = Fixture::new();
        f.data(json!([issue(101, 1, "open")]));
        let source: Value = serde_json::from_slice(&std::fs::read(&f.provider).unwrap()).unwrap();
        let base = git(&f.root, &["rev-parse", "HEAD"]);
        f.good(&[]);
        let pair = f.pair("02");
        let commit = git(&f.root, &["rev-parse", "HEAD"]);
        if state != "before-cas" {
            git(&f.root, &["update-ref", "refs/heads/main", &base, &commit]);
        }
        if matches!(
            state,
            "after-index" | "partial-cleanup" | "after-cleanup" | "rolled-back"
        ) {
            git(
                &f.root,
                &[
                    "reset",
                    &base,
                    "--",
                    pair["ticket_path"].as_str().unwrap(),
                    pair["snapshot_path"].as_str().unwrap(),
                ],
            );
        }
        if matches!(state, "partial-cleanup" | "after-cleanup" | "rolled-back") {
            std::fs::remove_file(f.root.join(pair["ticket_path"].as_str().unwrap())).unwrap();
        }
        if matches!(state, "after-cleanup" | "rolled-back") {
            std::fs::remove_file(f.root.join(pair["snapshot_path"].as_str().unwrap())).unwrap();
        }
        let value = json!({"format":"tkt-github-import/v1","branch":"refs/heads/main","selector":{"repository":"owner/project","label":null,"validation_criteria":[]},"source":source,"phase":if state=="rolled-back"{"rolled_back"}else{"rolling_back"},"base":base,"commit":commit,"pairs":[pair]});
        std::fs::write(
            f.root.join(".git/tkt-github-import.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        f.provider(json!({"repository":null,"pages":[],"failPage":1}));
        let requests = std::fs::read(f.provider.with_extension("json.requests")).unwrap();
        let result = f.good(&[]);
        assert_eq!(result["imported"], json!(["02"]), "{state}");
        assert_eq!(f.pair("02"), pair, "{state}");
        assert_eq!(
            std::fs::read(f.provider.with_extension("json.requests")).unwrap(),
            requests
        );
        assert!(!f.root.join(".git/tkt-github-import.json").exists());
    }
}

#[test]
fn changed_pair_index_and_unrelated_staged_data_block_recovery_without_cleanup() {
    for foreign in ["worktree", "index", "unrelated"] {
        let f = Fixture::new();
        f.data(json!([issue(101, 1, "open")]));
        let source: Value = serde_json::from_slice(&std::fs::read(&f.provider).unwrap()).unwrap();
        let base = git(&f.root, &["rev-parse", "HEAD"]);
        f.good(&[]);
        let pair = f.pair("02");
        let commit = git(&f.root, &["rev-parse", "HEAD"]);
        git(&f.root, &["update-ref", "refs/heads/main", &base, &commit]);
        if foreign == "unrelated" {
            std::fs::write(f.root.join("README.md"), "Other staged data\n").unwrap();
            git(&f.root, &["add", "README.md"]);
            std::fs::write(f.root.join("README.md"), "Owner source\n").unwrap();
        } else {
            std::fs::write(
                f.root.join(pair["ticket_path"].as_str().unwrap()),
                "User-owned changed data\n",
            )
            .unwrap();
            if foreign == "index" {
                git(&f.root, &["add", pair["ticket_path"].as_str().unwrap()]);
                std::fs::write(
                    f.root.join(pair["ticket_path"].as_str().unwrap()),
                    pair["ticket"].as_str().unwrap(),
                )
                .unwrap();
            }
        }
        let value = json!({"format":"tkt-github-import/v1","branch":"refs/heads/main","selector":{"repository":"owner/project","label":null,"validation_criteria":[]},"source":source,"phase":"rolling_back","base":base,"commit":commit,"pairs":[pair]});
        std::fs::write(
            f.root.join(".git/tkt-github-import.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let before = snapshot(&f.root);
        let out = f.import(&[]);
        assert!(!out.status.success(), "{foreign}: {out:?}");
        assert_eq!(snapshot(&f.root), before, "{foreign}");
    }
}

#[test]
fn new_batch_and_import_share_allocation_serialization_and_preview_does_not_lock() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    std::fs::write(
        f.root.join(".tickets/config.toml"),
        "[push]\nenabled=false\n[commit]\nhooks=\"run\"\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "opt in"]);
    hook(&f.root.join(".git/hooks/pre-commit"),"#!/bin/sh\ntouch .git/allocation-ready\ni=0\nwhile [ ! -f .git/allocation-release ]; do\n  i=$((i+1))\n  [ \"$i\" -lt 250 ] || exit 42\n  sleep 0.02\ndone\n");
    let mut first = f
        .cli(&[
            "new",
            "held",
            "--title",
            "Held allocation",
            "--body",
            "Independent source body",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
    while !f.root.join(".git/allocation-ready").exists() {
        if let Some(status) = first.try_wait().unwrap() {
            panic!("allocation fixture exited before hook: {status}");
        }
        if std::time::Instant::now() > deadline {
            first.kill().unwrap();
            panic!("allocation fixture did not reach hook");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let before = snapshot(&f.root);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("allocation operation owns"));
    assert_eq!(snapshot(&f.root), before);
    let out = f
        .cli(&["batch", "blocked:Blocked allocation"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    let out = f
        .cli(&[
            "new",
            "preview",
            "--title",
            "Preview allocation",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    std::fs::write(f.root.join(".git/allocation-release"), "release").unwrap();
    let out = first.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    std::fs::remove_file(f.root.join(".git/hooks/pre-commit")).unwrap();
    for marker in ["allocation-ready", "allocation-release"] {
        std::fs::remove_file(f.root.join(".git").join(marker)).unwrap();
    }
    let result = f.good(&[]);
    assert_eq!(result["imported"], json!(["03"]));
    assert!(!f.root.join(".git/tkt-allocation.lock").exists());
    git(
        &f.root,
        &[
            "remote",
            "add",
            "origin",
            "http://127.0.0.1:1/unreachable.git",
        ],
    );
    f.data(json!([issue(104, 4, "open")]));
    let result = f.good(&["--no-push"]);
    assert_eq!(result["imported"], json!(["04"]));
}

#[test]
fn imported_source_headings_and_checkboxes_never_become_local_close_sections() {
    let f = Fixture::new();
    let mut row = issue(101, 1, "open");
    let source="## Acceptance criteria\n\n- [ ] External source criterion\n\n## Resolution (2025-01-01)\n\nExternal historical resolution\n";
    row["body"] = json!(source);
    f.data(json!([row]));
    f.good(&[]);
    let path = f.root.join(".tickets/02-github-101.md");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "Define an observable result and how to verify it.",
        "Verify the local import contract independently.",
    );
    std::fs::write(&path, text).unwrap();
    git(&f.root, &["add", ".tickets/02-github-101.md"]);
    git(&f.root, &["commit", "-qm", "define local acceptance"]);
    let out = f
        .cli(&[
            "close",
            "02",
            "--check-all",
            "--resolution",
            "Actual local completion evidence",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains(source),
        "close changed the fenced provider source: {text}"
    );
    assert!(text.contains("- [x] Verify the local import contract independently."));
    assert!(
        text.contains("Actual local completion evidence"),
        "source resolution shadowed the local resolution"
    );
    let out = f.cli(&["--dry-run", "undo"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn rebase_failure_retains_the_retry_journal_until_rebase_and_replan_complete() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    let remote = remote_race(&f, false);
    std::fs::remove_file(f._temp.path().join("race")).unwrap();
    let source: Value = serde_json::from_slice(&std::fs::read(&f.provider).unwrap()).unwrap();
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    f.good(&[]);
    let pair = f.pair("02");
    git(&f.root, &["reset", "--hard", &base]);
    let value = json!({"format":"tkt-github-import/v1","branch":"refs/heads/main","selector":{"repository":"owner/project","label":null,"validation_criteria":[]},"source":source,"phase":"rolled_back","base":base,"commit":null,"pairs":[pair]});
    std::fs::write(
        f.root.join(".git/tkt-github-import.json"),
        serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    git(
        &f.root,
        &[
            "remote",
            "set-url",
            "origin",
            "http://127.0.0.1:1/unreachable.git",
        ],
    );
    f.provider(json!({"repository":null,"pages":[],"failPage":1}));
    let requests = std::fs::read(f.provider.with_extension("json.requests")).unwrap();
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    let pending: Value =
        serde_json::from_slice(&std::fs::read(f.root.join(".git/tkt-github-import.json")).unwrap())
            .unwrap();
    assert_eq!(pending["phase"], "rolled_back");
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), base);
    assert!(!f.root.join(".tickets/02-github-101.md").exists());
    assert_eq!(
        std::fs::read(f.provider.with_extension("json.requests")).unwrap(),
        requests
    );
    git(
        &f.root,
        &["remote", "set-url", "origin", remote.to_str().unwrap()],
    );
    let result = f.good(&[]);
    assert_eq!(result["recovered"], true);
    assert_eq!(result["imported"], json!([]));
    assert_eq!(result["skipped_existing"], 1);
    assert_eq!(f.pair("02"), pair);
    assert!(!f.root.join(".git/tkt-github-import.json").exists());
    assert_eq!(
        std::fs::read(f.provider.with_extension("json.requests")).unwrap(),
        requests
    );
}

#[test]
fn current_origin_and_missing_gh_have_explicit_public_behavior() {
    let f = Fixture::new();
    git(
        &f.root,
        &[
            "remote",
            "add",
            "origin",
            "git@github.com:owner/project.git",
        ],
    );
    let out = f
        .cli(&["import", "--github", "-o", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["repository"], "owner/project");
    assert_eq!(value["imported"], json!(["02"]));
    let f = Fixture::new();
    let path = f.bin.join(if cfg!(windows) { "gh.exe" } else { "gh" });
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, []).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = snapshot(&f.root);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("cannot execute gh"),
        "{out:?}"
    );
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn unmanaged_or_duplicate_ownership_and_closed_label_policies_never_write_imports() {
    for state in [
        "unmanaged",
        "missing-ticket",
        "duplicate",
        "policy",
        "foreign-index",
    ] {
        let f = Fixture::new();
        if state == "policy" {
            std::fs::write(
                f.root.join(".tickets/config.toml"),
                "[push]\nenabled=false\n[labels.import]\nvalues=[\"other\"]\n",
            )
            .unwrap();
            git(&f.root, &["add", "."]);
            git(&f.root, &["commit", "-qm", "label policy"]);
        } else if state == "unmanaged" {
            std::fs::create_dir_all(f.root.join(".tickets/imports/github-v1")).unwrap();
            std::fs::write(
                f.root.join(".tickets/imports/github-v1/101.json"),
                "User-owned data\n",
            )
            .unwrap();
        } else {
            f.good(&[]);
            if state == "missing-ticket" {
                git(&f.root, &["rm", ".tickets/02-github-101.md"]);
            } else if state == "duplicate" {
                let text = std::fs::read_to_string(f.root.join(".tickets/02-github-101.md"))
                    .unwrap()
                    .replace("id: \"02\"", "id: \"03\"");
                std::fs::write(f.root.join(".tickets/03-duplicate.md"), text).unwrap();
                git(&f.root, &["add", ".tickets/03-duplicate.md"]);
            } else {
                std::fs::write(f.root.join("README.md"), "Staged elsewhere\n").unwrap();
                git(&f.root, &["add", "README.md"]);
                std::fs::write(f.root.join("README.md"), "Owner source\n").unwrap();
            }
            if state != "foreign-index" {
                git(&f.root, &["commit", "-qm", "conflicting ownership"]);
            }
        }
        let before = snapshot(&f.root);
        let out = f.import(&[]);
        assert!(!out.status.success(), "{state}: {out:?}");
        assert_eq!(snapshot(&f.root), before, "{state}");
    }
}

#[test]
fn explicit_shared_validation_satisfies_creation_policy_without_inference() {
    let f = Fixture::new();
    std::fs::write(
        f.root.join(".tickets/config.toml"),
        "[push]\nenabled=false\n[close]\nrequire_validation_criteria=true\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "required validation"]);
    let before = snapshot(&f.root);
    let out = f.import(&[]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    let result = f.good(&[
        "--vc",
        "Verify each issue independently",
        "--validation",
        "Preserve source provenance",
    ]);
    assert_eq!(result["imported"], json!(["02"]));
    let value: Value = serde_json::from_slice(
        &std::fs::read(f.root.join(".tickets/imports/github-v1/101.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        value["validation_criteria"],
        json!([
            "Verify each issue independently",
            "Preserve source provenance"
        ])
    );
    let before = snapshot(&f.root);
    let result = f.good(&[]);
    assert_eq!(result["skipped_existing"], 1);
    assert_eq!(snapshot(&f.root), before);
    for text in [String::new(), "x".repeat(2001)] {
        let before = snapshot(&f.root);
        let out = f.import(&["--validation", &text]);
        assert!(!out.status.success(), "{out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
}

#[test]
fn linked_worktree_import_uses_its_git_metadata_and_preserves_the_base_worktree() {
    let f = Fixture::new();
    let linked = f._temp.path().join("linked");
    git(
        &f.root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            linked.to_str().unwrap(),
        ],
    );
    assert!(linked.join(".git").is_file());
    let base = git(&f.root, &["rev-parse", "HEAD"]);
    let original = std::fs::read(f.root.join(".tickets/01-seed.md")).unwrap();
    let out = f
        .cli(&[
            "import",
            "--github",
            "--repo",
            "owner/project",
            "-o",
            "json",
        ])
        .current_dir(&linked)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(linked.join(".tickets/02-github-101.md").exists());
    assert!(!f.root.join(".tickets/02-github-101.md").exists());
    assert_eq!(
        std::fs::read(f.root.join(".tickets/01-seed.md")).unwrap(),
        original
    );
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), base);
    for marker in ["tkt-allocation.lock", "tkt-github-import.json"] {
        let path = git(&linked, &["rev-parse", "--git-path", marker]);
        assert!(!Path::new(&path).exists());
    }
}

#[test]
fn issue_number_is_not_identity_and_a_transfer_is_an_explicit_conflict() {
    let f = Fixture::new();
    f.data(json!([issue(101, 1, "open")]));
    f.good(&[]);
    let mut row = issue(201, 1, "open");
    row["html_url"] = json!("https://github.com/other/project/issues/1");
    row["repository_url"] = json!("https://api.github.com/repos/other/project");
    f.provider(json!({"repository":{"id":88,"full_name":"other/project","html_url":"https://github.com/other/project"},"pages":[[row.clone()]]}));
    let out = f
        .cli(&[
            "import",
            "--github",
            "--repo",
            "other/project",
            "-o",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["imported"],
        json!(["03"])
    );
    row["id"] = json!(101);
    f.provider(json!({"repository":{"id":88,"full_name":"other/project","html_url":"https://github.com/other/project"},"pages":[[row]]}));
    let before = snapshot(&f.root);
    let out = f
        .cli(&[
            "import",
            "--github",
            "--repo",
            "other/project",
            "-o",
            "json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}

use std::path::Path;
use std::process::{Command, Output};

fn compile(source: &Path, output: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new("rustc");
    command.args(args).arg(source).arg("-o").arg(output);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.env("DO_NOT_TRACK", "1").output().unwrap();
    assert!(output.status.success(), "rustc: {output:?}");
    output
}

fn write(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
}

#[test]
fn do_not_track_prevents_update_request_and_cache_creation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let harness_source = root.join("harness.rs");
    let dirs_source = root.join("dirs.rs");
    let fake_curl_source = root.join("fake-curl.rs");
    let dirs_rlib = root.join("libdirs.rlib");
    let harness = root.join(if cfg!(windows) {
        "update-check-harness.exe"
    } else {
        "update-check-harness"
    });
    let fake_curl = root.join(if cfg!(windows) { "curl.exe" } else { "curl" });
    let marker = root.join("curl-invoked");
    let private_config = root.join("private-config");

    write(
        &dirs_source,
        "pub fn config_dir() -> Option<std::path::PathBuf> { std::env::var_os(\"TKT_TEST_CONFIG_DIR\").map(std::path::PathBuf::from) }\n",
    );
    write(
        &harness_source,
        &format!(
            "use std::sync::atomic::AtomicBool;\npub static QUIET: AtomicBool = AtomicBool::new(false);\npub static DRY_RUN: AtomicBool = AtomicBool::new(false);\n#[path = {:?}] mod update_check;\nfn main() {{ if std::env::var_os(\"TKT_TEST_CLEAR_DNT\").is_some() {{ std::env::remove_var(\"DO_NOT_TRACK\"); }} update_check::check_for_update(|| match std::env::var(\"TKT_TEST_INTERVAL\").as_deref() {{ Ok(\"never\") => Ok(None), Ok(\"invalid\") => Err(\"invalid test interval\".to_string()), Ok(value) => Ok(Some(value.parse().unwrap())), Err(_) => panic!(\"configuration callback invoked before suppression guard\"), }}); }}\n",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/update_check.rs")
        ),
    );
    write(
        &fake_curl_source,
        "fn main() { let marker = std::env::var_os(\"TKT_FAKE_CURL_MARKER\").unwrap(); std::fs::write(marker, std::env::args().skip(1).collect::<Vec<_>>().join(\"\\n\")).unwrap(); println!(\"{}\", r#\"{\"crate\":{\"max_stable_version\":\"99.99.0\"}}\"#); }\n",
    );

    compile(
        &dirs_source,
        &dirs_rlib,
        &[
            "--edition=2021",
            "--crate-name",
            "dirs",
            "--crate-type=rlib",
        ],
        &[],
    );
    compile(&fake_curl_source, &fake_curl, &["--edition=2021"], &[]);
    compile(
        &harness_source,
        &harness,
        &[
            "--edition=2021",
            "--extern",
            &format!("dirs={}", dirs_rlib.display()),
        ],
        &[("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))],
    );

    // Prove the platform's actual `curl` resolution reaches our fake before running the updater.
    let preflight = Command::new("curl")
        .arg("--preflight")
        .current_dir(root)
        .env("PATH", root)
        .env("TKT_FAKE_CURL_MARKER", &marker)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(
        preflight.status.success(),
        "fake curl preflight: {preflight:?}"
    );
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "--preflight");
    std::fs::remove_file(&marker).unwrap();

    // Every process starts with DNT=1. The isolated harness clears it internally only in
    // control cases so the updater's other guards can be checked independently.
    for (case, clear_dnt, ci, update_check) in [
        ("DNT", false, None, None),
        ("TKT_UPDATE_CHECK", true, None, Some("0")),
        ("CI", true, Some("true"), None),
    ] {
        let _ = std::fs::remove_dir_all(&private_config);
        let _ = std::fs::remove_file(&marker);
        let mut command = Command::new(&harness);
        command
            .current_dir(root)
            .env("PATH", root)
            .env("TKT_TEST_CONFIG_DIR", &private_config)
            .env("TKT_FAKE_CURL_MARKER", &marker)
            .env("DO_NOT_TRACK", "1")
            .env_remove("TKT_TEST_CLEAR_DNT")
            .env_remove("CI")
            .env_remove("TKT_UPDATE_CHECK");
        if clear_dnt {
            command.env("TKT_TEST_CLEAR_DNT", "1");
        }
        if let Some(value) = ci {
            command.env("CI", value);
        }
        if let Some(value) = update_check {
            command.env("TKT_UPDATE_CHECK", value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{case} update check harness: {output:?}"
        );
        assert!(!marker.exists(), "{case} guard did not suppress curl");
        assert!(
            !private_config.exists(),
            "{case} guard created the cache directory"
        );
    }
    for (case, interval, age, request) in [
        ("never", "never", None, false),
        ("invalid", "invalid", None, false),
        ("hour-expired", "3600", Some(7200), true),
        ("day-fresh", "86400", Some(7200), false),
        ("week-fresh", "604800", Some(172800), false),
        ("no-cache", "86400", None, true),
    ] {
        let config = root.join(case);
        let cache = config.join("tkt/update-check.txt");
        if let Some(age) = age {
            std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            write(&cache, &format!("{}\n99.99.0\n", now - age));
        }
        let before = std::fs::read(&cache).ok();
        let _ = std::fs::remove_file(&marker);
        let output = Command::new(&harness)
            .current_dir(root)
            .env("PATH", root)
            .env("DO_NOT_TRACK", "1")
            .env("TKT_TEST_CLEAR_DNT", "1")
            .env_remove("CI")
            .env_remove("TKT_UPDATE_CHECK")
            .env("TKT_TEST_CONFIG_DIR", &config)
            .env("TKT_FAKE_CURL_MARKER", &marker)
            .env("TKT_TEST_INTERVAL", interval)
            .output()
            .unwrap();
        assert!(output.status.success(), "{case}: {output:?}");
        assert_eq!(marker.exists(), request, "{case}: {output:?}");
        if request {
            assert!(cache.exists());
        } else {
            assert_eq!(std::fs::read(&cache).ok(), before, "{case}");
        }
        if age.is_none() && !request {
            assert!(!config.exists(), "{case} created cache root");
        }
    }
}

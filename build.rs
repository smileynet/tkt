use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command.current_dir(root).args(args);
    for (key, _) in env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command.env("GIT_CONFIG_NOSYSTEM", "1").env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    );
    let output = command.output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn short_hash(value: &str) -> Option<&str> {
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).then(|| &value[..7])
}

fn identity(root: &Path) -> Option<String> {
    let metadata = root.join(".cargo_vcs_info.json");
    match fs::read(&metadata) {
        Ok(bytes) => {
            println!("cargo:rerun-if-changed={}", metadata.display());
            // Cargo metadata describes packaging time; it is not proof of provenance.
            let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
            let git = value.get("git")?.as_object()?;
            let hash = short_hash(git.get("sha1")?.as_str()?)?;
            let dirty = match git.get("dirty") {
                Some(value) => value.as_bool()?,
                None => false,
            };
            return Some(format!("{}{}", hash, if dirty { "-dirty" } else { "" }));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return None,
    }
    let marker = root.join(".git");
    println!("cargo:rerun-if-changed={}", marker.display());
    // tkt owns its repository root. Never discover a repository through an ancestor.
    if !marker.exists()
        || Path::new(&git(root, &["rev-parse", "--show-toplevel"])?)
            .canonicalize()
            .ok()?
            != root.canonicalize().ok()?
    {
        return None;
    }
    for name in ["HEAD", "refs", "packed-refs", "reftable"] {
        let path = git(root, &["rev-parse", "--git-path", name])?;
        let path = root.join(path);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    short_hash(&git(root, &["rev-parse", "HEAD"])?).map(str::to_owned)
}

fn main() {
    let root = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the package directory");
    let hash = identity(Path::new(&root)).unwrap_or_default();
    println!("cargo:rustc-env=TKT_GIT_HASH={hash}");
}

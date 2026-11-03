//! Self-update check: notify when a newer version is available on crates.io.
//! Default cadence is 24 hours, configurable in user settings.
//! Never blocks longer than 3 seconds, prints to stderr, doesn't affect exit code.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const TIMEOUT_SECS: u64 = 3;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the update check (call at program start, after command execution).
/// Prints a notice to stderr if a newer version is available.
/// Never panics, never affects exit code.
pub fn check_for_update(interval: impl FnOnce() -> Result<Option<u64>, String>) {
    // Disabled by quiet mode
    if crate::QUIET.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // `--dry-run` must not create or refresh the update-check cache.
    if crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // Disabled by env
    if std::env::var("TKT_UPDATE_CHECK").as_deref() == Ok("0") {
        return;
    }
    if std::env::var("CI").is_ok() {
        return;
    }
    // Update checks use the network and write a per-user cache, so follow telemetry consent.
    if std::env::var("DO_NOT_TRACK").as_deref() == Ok("1") {
        return;
    }

    let interval = match interval() {
        Ok(Some(seconds)) => seconds,
        Ok(None) => return,
        Err(error) => {
            eprintln!("warning: {error}; update check skipped");
            return;
        }
    };
    let cache_path = match cache_file() {
        Some(p) => p,
        None => return,
    };

    // Check if we already checked recently
    if let Some(cached) = read_cache(&cache_path) {
        if cache_is_fresh(cached.timestamp, unix_now(), interval) {
            // Still fresh — show cached notice if applicable
            if let Some(ref latest) = cached.latest_version {
                if is_newer(latest, CURRENT_VERSION) {
                    print_notice(latest);
                }
            }
            return;
        }
    }

    // Time to check
    match fetch_latest_version() {
        Some(latest) => {
            write_cache(&cache_path, &latest);
            if is_newer(&latest, CURRENT_VERSION) {
                print_notice(&latest);
            }
        }
        None => {
            // Network failed — write cache with current version to avoid retrying immediately
            write_cache(&cache_path, CURRENT_VERSION);
        }
    }
}

fn cache_is_fresh(timestamp: u64, now: u64, interval: u64) -> bool {
    now.saturating_sub(timestamp) < interval
}

fn print_notice(latest: &str) {
    eprintln!(
        "\n  (tkt {} available — run `cargo install tkt` to update)",
        latest
    );
}

/// Fetch latest stable version from crates.io using curl.
fn fetch_latest_version() -> Option<String> {
    let output = std::process::Command::new("curl")
        .args([
            "--silent",
            "--max-time",
            &TIMEOUT_SECS.to_string(),
            "--proto",
            "=https",
            "--tlsv1.2",
            "-H",
            "User-Agent: tkt-update-check",
            "https://crates.io/api/v1/crates/tkt",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8(output.stdout).ok()?;
    // Parse max_stable_version from JSON without a dependency
    // Look for "max_stable_version":"X.Y.Z"
    let marker = "\"max_stable_version\":\"";
    let start = body.find(marker)? + marker.len();
    let end = start + body[start..].find('"')?;
    Some(body[start..end].to_string())
}

/// Compare versions: is `latest` newer than `current`?
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

// --- Cache file ---

struct CacheEntry {
    timestamp: u64,
    latest_version: Option<String>,
}

fn cache_file() -> Option<PathBuf> {
    let dir = dirs::config_dir()?.join("tkt");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("update-check.txt"))
}

fn read_cache(path: &PathBuf) -> Option<CacheEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut lines = content.lines();
    let timestamp: u64 = lines.next()?.parse().ok()?;
    let version = lines.next().map(|s| s.to_string());
    Some(CacheEntry {
        timestamp,
        latest_version: version.filter(|v| !v.is_empty()),
    })
}

fn write_cache(path: &PathBuf, version: &str) {
    let content = format!("{}\n{}\n", unix_now(), version);
    let _ = std::fs::write(path, content);
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_cadence_and_untrusted_cache_timestamps() {
        for (value, expected) in [
            ("1h", Some(3600)),
            ("24h", Some(86400)),
            ("7d", Some(604800)),
            ("never", None),
        ] {
            assert_eq!(
                crate::config::parse_update_interval(value).unwrap(),
                expected
            );
        }
        for value in ["", "0h", "-1h", "1.5h", "1m", "18446744073709551615d"] {
            assert!(
                crate::config::parse_update_interval(value).is_err(),
                "{value}"
            );
        }
        assert!(cache_is_fresh(100, 3699, 3600));
        assert!(!cache_is_fresh(100, 3700, 3600));
        assert!(cache_is_fresh(u64::MAX, 100, 3600));
    }

    #[test]
    fn version_comparison() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }
}

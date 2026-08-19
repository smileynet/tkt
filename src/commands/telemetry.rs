//! `tkt telemetry` — manage telemetry consent and inspect collected data.

use std::collections::HashMap;

use anyhow::Result;

use crate::telemetry;

pub fn run(
    enable: bool,
    disable: bool,
    status: bool,
    show: bool,
    all: bool,
    clear: bool,
) -> Result<i32> {
    if enable {
        telemetry::write_consent(true)
            .map_err(|e| anyhow::anyhow!("failed to write consent: {}", e))?;
        println!("telemetry enabled — events will be recorded locally");
        return Ok(0);
    }

    if disable {
        telemetry::write_consent(false)
            .map_err(|e| anyhow::anyhow!("failed to write consent: {}", e))?;
        println!("telemetry disabled");
        return Ok(0);
    }

    if clear {
        if let Some(dir) = telemetry::telemetry_dir() {
            if dir.is_dir() {
                std::fs::remove_dir_all(&dir)
                    .map_err(|e| anyhow::anyhow!("failed to clear telemetry: {}", e))?;
                println!("telemetry data cleared");
            } else {
                println!("no telemetry data to clear");
            }
        } else {
            println!("no telemetry data directory");
        }
        return Ok(0);
    }

    if show || all {
        return telemetry_show(all);
    }

    let _ = status;
    telemetry_status()
}

fn telemetry_status() -> Result<i32> {
    telemetry::cleanup_telemetry_dir();

    let (consent, reason) = telemetry::check_consent();
    let state_str = match consent {
        telemetry::Consent::Enabled => "enabled",
        telemetry::Consent::Disabled => "disabled",
    };
    let reason_str = match reason {
        telemetry::ConsentReason::DoNotTrack => "DO_NOT_TRACK=1",
        telemetry::ConsentReason::EnvVar => "TKT_TELEMETRY env var",
        telemetry::ConsentReason::CiDetected => "CI environment detected",
        telemetry::ConsentReason::ConfigFile => "consent.toml",
        telemetry::ConsentReason::Default => "default — never opted in",
    };
    println!("telemetry: {} ({})", state_str, reason_str);

    if let Some(dir) = telemetry::telemetry_dir() {
        if dir.is_dir() {
            let mut total_bytes: u64 = 0;
            let mut total_events: usize = 0;
            let mut projects: Vec<(String, usize, u64)> = Vec::new();

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "jsonl") {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let lines = std::fs::read_to_string(&path)
                            .map(|c| c.lines().filter(|l| !l.is_empty()).count())
                            .unwrap_or(0);
                        let name = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        total_bytes += size;
                        total_events += lines;
                        projects.push((name, lines, size));
                    }
                }
            }
            projects.sort_by_key(|p| std::cmp::Reverse(p.1));

            println!(
                "storage: {} ({} events across {} projects)",
                format_bytes(total_bytes),
                total_events,
                projects.len()
            );
            for (name, events, bytes) in &projects {
                println!("  {}: {} events ({})", name, events, format_bytes(*bytes));
            }
        } else {
            println!("storage: 0 bytes (no data)");
        }
    } else {
        println!("storage: unavailable (no data directory)");
    }

    if let Some(path) = telemetry::consent_file_path() {
        let exists = if path.is_file() { "found" } else { "not found" };
        println!("consent file: {} ({})", path.display(), exists);
    }

    let dnt = std::env::var("DO_NOT_TRACK").unwrap_or_else(|_| "unset".to_string());
    let tkt_tel = std::env::var("TKT_TELEMETRY").unwrap_or_else(|_| "unset".to_string());
    let ci = std::env::var("CI").unwrap_or_else(|_| "unset".to_string());
    println!(
        "env overrides: DO_NOT_TRACK={}, TKT_TELEMETRY={}, CI={}",
        dnt, tkt_tel, ci
    );

    Ok(0)
}

fn telemetry_show(show_all: bool) -> Result<i32> {
    let dir = match telemetry::telemetry_dir() {
        Some(d) if d.is_dir() => d,
        _ => {
            println!("no telemetry data");
            return Ok(0);
        }
    };

    let mut all_lines: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if !line.is_empty() {
                            all_lines.push(line.to_string());
                        }
                    }
                }
            }
        }
    }

    if all_lines.is_empty() {
        println!("no telemetry events recorded");
        return Ok(0);
    }

    all_lines.sort_by(|a, b| extract_ts(a.as_str()).cmp(extract_ts(b.as_str())));

    // --- Summary header ---
    print_summary(&all_lines);
    println!();

    // --- Event list ---
    let start = if show_all {
        0
    } else {
        all_lines.len().saturating_sub(20)
    };
    let showing = all_lines.len() - start;

    if show_all {
        println!("all events ({}):", all_lines.len());
    } else {
        println!(
            "recent events ({} total, showing last {}):",
            all_lines.len(),
            showing
        );
    }
    println!();
    for line in &all_lines[start..] {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let error_part = val["error_kind"]
                .as_str()
                .map(|k| format!(" err={}", k))
                .unwrap_or_default();
            println!(
                "  {} {} cmd={} exit={}{}  {}ms",
                val["ts"].as_str().unwrap_or("?"),
                val["project"].as_str().unwrap_or("?"),
                val["cmd"].as_str().unwrap_or("?"),
                val["exit_code"].as_i64().unwrap_or(-1),
                error_part,
                val["duration_ms"].as_u64().unwrap_or(0),
            );
        } else {
            println!("  {}", line);
        }
    }
    Ok(0)
}

/// Print a compact summary of telemetry data: command distribution, errors, slow commands.
fn print_summary(lines: &[String]) {
    let mut cmd_counts: HashMap<String, usize> = HashMap::new();
    let mut error_count: usize = 0;
    let mut slow_cmds: Vec<(String, String, u64)> = Vec::new(); // (ts, cmd, duration_ms)

    for line in lines {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let cmd = val["cmd"].as_str().unwrap_or("?").to_string();
            *cmd_counts.entry(cmd.clone()).or_default() += 1;

            if val["exit_code"].as_i64().unwrap_or(0) != 0 {
                error_count += 1;
            }

            let duration = val["duration_ms"].as_u64().unwrap_or(0);
            if duration > 2000 {
                let ts = val["ts"].as_str().unwrap_or("?").to_string();
                slow_cmds.push((ts, cmd, duration));
            }
        }
    }

    // Command distribution (sorted by count desc)
    let mut cmd_list: Vec<_> = cmd_counts.iter().collect();
    cmd_list.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    print!("commands:");
    for (cmd, count) in &cmd_list {
        print!(" {}:{}", cmd, count);
    }
    println!();

    // Error rate
    let total = lines.len();
    if error_count > 0 {
        println!(
            "errors: {}/{} ({:.0}%)",
            error_count,
            total,
            (error_count as f64 / total as f64) * 100.0
        );
    } else {
        println!("errors: 0");
    }

    // Slow commands (>2s)
    if !slow_cmds.is_empty() {
        println!("slow (>2s): {}", slow_cmds.len());
        for (ts, cmd, ms) in slow_cmds.iter().rev().take(5) {
            println!("  {} {} {:.1}s", ts, cmd, *ms as f64 / 1000.0);
        }
    }
}

fn extract_ts(line: &str) -> &str {
    if let Some(start) = line.find("\"ts\":\"") {
        let rest = &line[start + 6..];
        if let Some(end) = rest.find('"') {
            return &rest[..end];
        }
    }
    ""
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

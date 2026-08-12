//! `tkt doctor` — health check for single project or cross-project scan.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::color::{sym_err, sym_ok, sym_warn};
use crate::commands::common::is_quiet;
use crate::core::{self, Ticket};
use crate::findings;

pub fn run(path: Option<&str>, fix: bool) -> Result<i32> {
    if fix {
        eprintln!(
            "  {} --fix is not yet implemented for doctor (use `tkt validate --fix` per project)",
            sym_warn()
        );
    }
    match path {
        None => run_single_project(fix),
        Some(p) => run_cross_project(Path::new(p), fix),
    }
}

// --- Single-project doctor (no path arg) ---

fn run_single_project(fix: bool) -> Result<i32> {
    let mut issues = 0;

    // Check 1: git available
    match std::process::Command::new("git")
        .args(["--version"])
        .output()
    {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("  {} {}", sym_ok(), ver);
        }
        _ => {
            println!("  {} git not found on PATH", sym_err());
            issues += 1;
        }
    }

    // Check 2: inside a git repo
    let repo_root = match crate::git::repo_root_cwd() {
        Ok(r) => {
            println!("  {} git repo: {}", sym_ok(), r.display());
            Some(r)
        }
        Err(_) => {
            println!("  {} not inside a git repository", sym_err());
            issues += 1;
            None
        }
    };

    let Some(root) = repo_root else {
        println!("\n{} issues found (not in a git repo)", issues);
        return Ok(1);
    };

    // Check 3: .tickets/ exists
    let tickets_dir = root.join(".tickets");
    if tickets_dir.exists() {
        let count = ticket_file_count(&tickets_dir);
        println!("  {} .tickets/ exists ({} tickets)", sym_ok(), count);
    } else {
        println!("  {} .tickets/ not found — run: tkt init", sym_err());
        issues += 1;
        println!("\n{} issues found", issues);
        return Ok(1);
    }

    // Check 4: config.toml
    let config_path = tickets_dir.join("config.toml");
    if config_path.exists() {
        println!("  {} .tickets/config.toml present", sym_ok());
    } else {
        println!(
            "  {} .tickets/config.toml missing — run: tkt init",
            sym_warn()
        );
    }

    // Check 5: remote configured
    match crate::git::has_remote(&root) {
        Ok(true) => println!("  {} remote configured", sym_ok()),
        _ => println!(
            "  {} no remote configured (push will be skipped)",
            sym_warn()
        ),
    }

    // Check 6: validate (cycles, dangling deps, contract)
    match core::load_corpus(&tickets_dir) {
        Ok(corpus) => {
            let all_findings = run_validate_checks(&corpus, &tickets_dir, &root, fix);
            let errors = all_findings
                .iter()
                .filter(|f| f.severity == "error")
                .count();
            let warnings = all_findings
                .iter()
                .filter(|f| f.severity == "warning")
                .count();

            if errors == 0 && warnings == 0 {
                println!("  {} no validation issues", sym_ok());
            } else {
                if errors > 0 {
                    println!("  {} {} validation errors", sym_err(), errors);
                    issues += errors;
                }
                if warnings > 0 {
                    println!("  {} {} validation warnings", sym_warn(), warnings);
                }
            }
        }
        Err(e) => {
            println!("  {} failed to load corpus: {}", sym_err(), e);
            issues += 1;
        }
    }

    if !is_quiet() {
        println!();
        if issues == 0 {
            println!("All checks passed.");
        } else {
            println!("{} issue(s) found.", issues);
        }
    }

    Ok(if issues == 0 { 0 } else { 1 })
}

// --- Cross-project scan ---

fn run_cross_project(scan_path: &Path, fix: bool) -> Result<i32> {
    let projects = find_ticket_dirs(scan_path);

    if projects.is_empty() {
        println!(
            "No .tickets/ directories found under {}",
            scan_path.display()
        );
        return Ok(0);
    }

    if !is_quiet() {
        println!(
            "Scanning {} ({} projects found)\n",
            scan_path.display(),
            projects.len()
        );
    }

    let mut clean = 0;
    let mut fixable = 0;
    let mut broken = 0;

    for tickets_dir in &projects {
        let project_name = tickets_dir
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let count = ticket_file_count(tickets_dir);

        match core::load_corpus(tickets_dir) {
            Ok(corpus) => {
                let repo_root = tickets_dir.parent().unwrap_or(tickets_dir);
                let all_findings = run_validate_checks(&corpus, tickets_dir, repo_root, fix);
                let errors = all_findings
                    .iter()
                    .filter(|f| f.severity == "error")
                    .count();
                let warnings = all_findings
                    .iter()
                    .filter(|f| f.severity == "warning")
                    .count();

                if errors == 0 && warnings == 0 {
                    println!("  {} {} ({} tickets)", sym_ok(), project_name, count);
                    clean += 1;
                } else if errors == 0 {
                    println!(
                        "  {} {} ({} tickets, {} warnings)",
                        sym_warn(),
                        project_name,
                        count,
                        warnings
                    );
                    fixable += 1;
                } else {
                    println!(
                        "  {} {} ({} tickets, {} errors)",
                        sym_err(),
                        project_name,
                        count,
                        errors
                    );
                    broken += 1;
                }
            }
            Err(_) => {
                println!(
                    "  {} {} ({} tickets, parse errors)",
                    sym_err(),
                    project_name,
                    count
                );
                broken += 1;
            }
        }
    }

    if !is_quiet() {
        println!(
            "\nSummary: {} clean, {} fixable, {} broken",
            clean, fixable, broken
        );
    }

    Ok(if broken > 0 { 1 } else { 0 })
}

// --- Helpers ---

fn ticket_file_count(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .count()
        })
        .unwrap_or(0)
}

fn find_ticket_dirs(root: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    find_ticket_dirs_recursive(root, &mut results, 5);
    results.sort();
    results
}

fn find_ticket_dirs_recursive(dir: &Path, results: &mut Vec<PathBuf>, depth: u8) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // Skip hidden dirs (except .tickets) and common large dirs
        if name_str.starts_with('.') && name_str != ".tickets" {
            continue;
        }
        if matches!(
            name_str.as_ref(),
            "node_modules" | "target" | "dist" | "build" | ".git" | "vendor"
        ) {
            continue;
        }
        if name_str == ".tickets" {
            // Found a tickets dir
            results.push(path);
        } else {
            find_ticket_dirs_recursive(&path, results, depth - 1);
        }
    }
}

fn run_validate_checks(
    corpus: &[Ticket],
    _tickets_dir: &Path,
    _repo_root: &Path,
    _fix: bool,
) -> Vec<findings::Finding> {
    let mut all = Vec::new();
    all.extend(findings::check_status(corpus));
    all.extend(findings::check_env(corpus));
    all.extend(findings::check_id_filename(corpus));
    all.extend(findings::check_duplicate_ids(corpus));
    all.extend(findings::check_dangling_deps(corpus));
    all.extend(findings::check_cycles(corpus));
    all
}

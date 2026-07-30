//! Validation findings: rules, accumulation, and output formatting.
//!
//! Each check function takes a corpus and returns findings independently.
//! The validate command orchestrates them; sync-plan shares the output type.

use std::collections::{HashMap, HashSet};

use crate::core::{self, Env, Status, Ticket};

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct Finding {
    pub file: String,
    pub rule: String,
    pub message: String,
    pub severity: String,
}

// --- Output ---

/// Print findings as JSON (default) or brief human-readable.
pub fn print_findings(findings: &[Finding], brief: bool, status: &str) {
    if brief {
        for f in findings {
            println!("{}: {} [{}] {}", f.severity, f.file, f.rule, f.message);
        }
        println!("{} ({} finding(s))", status, findings.len());
    } else {
        println!(
            "{{\"status\":\"{}\",\"findings\":[{}]}}",
            status,
            findings
                .iter()
                .map(|f| format!(
                    "{{\"file\":\"{}\",\"rule\":\"{}\",\"message\":\"{}\",\"severity\":\"{}\"}}",
                    core::json_string_escape(&f.file),
                    core::json_string_escape(&f.rule),
                    core::json_string_escape(&f.message),
                    core::json_string_escape(&f.severity),
                ))
                .collect::<Vec<_>>()
                .join(",")
        );
    }
}

/// Determine pass/fail status from findings and strict mode.
pub fn status_from_findings(findings: &[Finding], strict: bool) -> &'static str {
    let has_errors = findings.iter().any(|f| f.severity == "error");
    let has_warnings = findings.iter().any(|f| f.severity == "warning");
    if has_errors || (strict && has_warnings) {
        "fail"
    } else {
        "pass"
    }
}

// --- Rules ---

/// Check that all ticket statuses are valid.
/// Note: with the new Ticket type, invalid statuses are rejected at parse time.
/// This check is retained for TicketFile-based validation where we want findings
/// instead of hard errors.
pub fn check_status(corpus: &[Ticket]) -> Vec<Finding> {
    // Status is validated at parse time now — tickets with invalid status
    // won't make it into the corpus. This returns empty for well-parsed tickets.
    corpus
        .iter()
        .filter(|t| !core::STATUS_VALUES.contains(&t.status.as_str()))
        .map(|t| Finding {
            file: filename(t),
            rule: "bad-status".into(),
            message: format!(
                "status {:?} not in {}",
                t.status.as_str(),
                core::STATUS_VALUES.join("/")
            ),
            severity: "error".into(),
        })
        .collect()
}

/// Check that all ticket env values are valid.
/// Note: with the new Ticket type, invalid env values are rejected at parse time.
pub fn check_env(corpus: &[Ticket]) -> Vec<Finding> {
    corpus
        .iter()
        .filter(|t| t.env != Env::Either && !core::ENV_VALUES.contains(&t.env.as_str()))
        .map(|t| Finding {
            file: filename(t),
            rule: "bad-env".into(),
            message: format!(
                "env {:?} not in {}",
                t.env.as_str(),
                core::ENV_VALUES.join("/")
            ),
            severity: "error".into(),
        })
        .collect()
}

/// Check that ticket IDs match their filenames.
pub fn check_id_filename(corpus: &[Ticket]) -> Vec<Finding> {
    corpus
        .iter()
        .filter(|t| {
            let name = filename(t);
            !name.starts_with(&format!("{}-", t.id))
        })
        .map(|t| Finding {
            file: filename(t),
            rule: "id-filename-mismatch".into(),
            message: format!("id {:?} vs filename", t.id),
            severity: "error".into(),
        })
        .collect()
}

/// Check for duplicate ticket IDs.
pub fn check_duplicate_ids(corpus: &[Ticket]) -> Vec<Finding> {
    let mut seen: HashMap<&str, String> = HashMap::new();
    let mut findings = Vec::new();
    for t in corpus {
        let name = filename(t);
        if let Some(existing) = seen.get(t.id.as_str()) {
            findings.push(Finding {
                file: name,
                rule: "duplicate-id".into(),
                message: format!("id {:?} also in {}", t.id, existing),
                severity: "error".into(),
            });
        } else {
            seen.insert(&t.id, name);
        }
    }
    findings
}

/// Check for dangling blocked_by references.
pub fn check_dangling_deps(corpus: &[Ticket]) -> Vec<Finding> {
    let known: HashSet<&str> = corpus.iter().map(|t| t.id.as_str()).collect();
    let mut findings = Vec::new();
    for t in corpus {
        for dep in &t.blocked_by {
            if !known.contains(dep.as_str()) {
                findings.push(Finding {
                    file: filename(t),
                    rule: "dangling-blocked-by".into(),
                    message: format!("ref {:?} has no ticket", dep),
                    severity: "error".into(),
                });
            }
        }
    }
    findings
}

/// Detect dependency cycles via DFS.
pub fn check_cycles(corpus: &[Ticket]) -> Vec<Finding> {
    let known: HashSet<&str> = corpus.iter().map(|t| t.id.as_str()).collect();

    // Build adjacency: id -> list of blocked_by ids (only known ones)
    let adj: HashMap<&str, Vec<&str>> = corpus
        .iter()
        .map(|t| {
            let deps: Vec<&str> = t
                .blocked_by
                .iter()
                .map(|d| d.as_str())
                .filter(|d| known.contains(d))
                .collect();
            (t.id.as_str(), deps)
        })
        .collect();

    // DFS states: 0=unvisited, 1=visiting (in current path), 2=visited (complete)
    let mut state: HashMap<&str, u8> = adj.keys().map(|&k| (k, 0u8)).collect();
    let mut path: Vec<&str> = Vec::new();
    let mut cycles: Vec<Vec<&str>> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, u8>,
        path: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<&'a str>>,
    ) {
        state.insert(node, 1);
        path.push(node);

        if let Some(deps) = adj.get(node) {
            for &dep in deps {
                match state.get(dep) {
                    Some(&1) => {
                        if let Some(pos) = path.iter().position(|&n| n == dep) {
                            let mut cycle: Vec<&str> = path[pos..].to_vec();
                            cycle.push(dep);
                            cycles.push(cycle);
                        }
                    }
                    Some(&0) => {
                        dfs(dep, adj, state, path, cycles);
                    }
                    _ => {}
                }
            }
        }

        path.pop();
        state.insert(node, 2);
    }

    // Sort keys for deterministic output
    let mut nodes: Vec<&str> = adj.keys().copied().collect();
    nodes.sort();
    for node in nodes {
        if state.get(node) == Some(&0) {
            dfs(node, &adj, &mut state, &mut path, &mut cycles);
        }
    }

    // Deduplicate: normalize by rotating to start at the smallest id
    let mut unique_cycles: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for cycle in &cycles {
        let path_part = &cycle[..cycle.len() - 1];
        if let Some(min_pos) = path_part
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| *v)
            .map(|(i, _)| i)
        {
            let mut normalized: Vec<&str> = path_part[min_pos..].to_vec();
            normalized.extend_from_slice(&path_part[..min_pos]);
            let key = normalized.join(" -> ");
            if seen.insert(key.clone()) {
                unique_cycles.push(format!("{} -> {}", key, normalized[0]));
            }
        }
    }

    // Map cycle to file for reporting
    let id_to_file: HashMap<&str, String> = corpus
        .iter()
        .map(|t| (t.id.as_str(), filename(t)))
        .collect();

    unique_cycles
        .iter()
        .map(|desc| {
            let first_id = desc.split(" -> ").next().unwrap_or("");
            Finding {
                file: id_to_file
                    .get(first_id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                rule: "cycle".into(),
                message: format!("dependency cycle: {}", desc),
                severity: "error".into(),
            }
        })
        .collect()
}

/// Check for unchecked acceptance criteria on done tickets.
pub fn check_unchecked_acs(corpus: &[Ticket]) -> Vec<Finding> {
    use regex::Regex;
    use std::sync::LazyLock;

    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());

    corpus
        .iter()
        .filter(|t| t.status == Status::Done)
        .filter_map(|t| {
            let count = RE.find_iter(&t.body).count();
            if count > 0 {
                Some(Finding {
                    file: filename(t),
                    rule: "unchecked-acs-on-done".into(),
                    message: format!("{} unchecked box(es)", count),
                    severity: "warning".into(),
                })
            } else {
                None
            }
        })
        .collect()
}

// --- Helpers ---

fn filename(t: &Ticket) -> String {
    t.path.file_name().unwrap().to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_ticket(content: &str) -> Ticket {
        Ticket::parse_str(content, Path::new("test.md")).unwrap()
    }

    #[test]
    fn cycles_detected_self() {
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# A\n",
        );
        let findings = check_cycles(&[t]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "cycle");
        assert!(findings[0].message.contains("01"));
    }

    #[test]
    fn cycles_detected_two_node() {
        let a = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# A\n",
        );
        let b = make_ticket(
            "---\nid: \"02\"\ntitle: \"B\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# B\n",
        );
        let findings = check_cycles(&[a, b]);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("01") && findings[0].message.contains("02"));
    }

    #[test]
    fn no_cycle_in_linear_chain() {
        let a = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: done\nblocked_by: []\n---\n\n# A\n",
        );
        let b = make_ticket(
            "---\nid: \"02\"\ntitle: \"B\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# B\n",
        );
        let c = make_ticket(
            "---\nid: \"03\"\ntitle: \"C\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# C\n",
        );
        let findings = check_cycles(&[a, b, c]);
        assert!(findings.is_empty());
    }

    #[test]
    fn dangling_dep_detected() {
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: [\"99\"]\n---\n\n# A\n",
        );
        let findings = check_dangling_deps(&[t]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "dangling-blocked-by");
    }

    #[test]
    fn bad_status_detected() {
        // With the new type system, invalid status is rejected at parse time.
        // This test verifies that by showing parse fails.
        let content =
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: invalid\nblocked_by: []\n---\n\n# A\n";
        let result = Ticket::parse_str(content, Path::new("test.md"));
        assert!(result.is_err());
    }
}

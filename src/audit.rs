//! Pure audit rules — each function takes a corpus (and optionally an injectable
//! dependency) and returns `Vec<Finding>`. No I/O, no git, no filesystem.

use std::path::Path;

use crate::core::{self, Status, Ticket};
use crate::findings::Finding;

/// Check resolution quality on done tickets:
/// - Missing `## Resolution` section entirely
/// - Resolution is empty or still "TBD"
pub fn check_resolution_quality(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        let fname = filename(t);

        if !t.body.contains("## Resolution") {
            findings.push(Finding {
                file: fname,
                rule: "missing-resolution".into(),
                message: "done ticket has no Resolution section".into(),
                severity: "warning".into(),
            });
            continue;
        }

        let has_content = t
            .body
            .split_once("## Resolution")
            .map(|(_, after)| {
                let text = after.lines().skip(1).collect::<Vec<_>>().join("\n");
                let trimmed = text.trim();
                !trimmed.is_empty() && trimmed != "TBD"
            })
            .unwrap_or(false);

        if !has_content {
            findings.push(Finding {
                file: fname,
                rule: "tbd-resolution".into(),
                message: "resolution is empty or still TBD".into(),
                severity: "warning".into(),
            });
        }
    }

    findings
}

/// Check AC completeness on done tickets:
/// - All AC boxes unchecked (none checked) on a done ticket
pub fn check_ac_completeness(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        let stats = t.file.ac_stats();
        if stats.unchecked > 0 && stats.checked == 0 {
            findings.push(Finding {
                file: filename(t),
                rule: "all-acs-unchecked-on-done".into(),
                message: format!("{} unchecked box(es), none checked", stats.unchecked),
                severity: "warning".into(),
            });
        }
    }

    findings
}

/// Check for stale WIP tickets — in_progress for more than 7 days.
///
/// `last_commit_ts` is an injectable function that returns the unix timestamp
/// of the last commit touching a given path, or `None` if unavailable.
pub fn check_stale_wip(
    corpus: &[Ticket],
    now: u64,
    last_commit_ts: impl Fn(&Path) -> Option<u64>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let threshold = 7 * 24 * 60 * 60; // 7 days

    for t in corpus.iter().filter(|t| t.status == Status::InProgress) {
        if let Some(ts) = last_commit_ts(&t.path) {
            if now > ts && (now - ts) > threshold {
                let days = (now - ts) / (24 * 60 * 60);
                findings.push(Finding {
                    file: filename(t),
                    rule: "stale-wip".into(),
                    message: format!("in_progress for {} days (last commit)", days),
                    severity: "info".into(),
                });
            }
        }
    }

    findings
}

/// Check frontier health — high-priority tickets still open on the frontier.
pub fn check_frontier_health(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    let frontier_ids: std::collections::HashSet<&str> = core::frontier(corpus)
        .iter()
        .map(|t| t.id.as_str())
        .collect();

    for t in corpus {
        if t.status == Status::Open && t.is_high_priority() && frontier_ids.contains(t.id.as_str())
        {
            findings.push(Finding {
                file: filename(t),
                rule: "high-priority-open".into(),
                message: "high-priority ticket still open".into(),
                severity: "info".into(),
            });
        }
    }

    findings
}

/// Flag done tickets that have validation_criteria but were closed without evidence.
pub fn check_validation_evidence(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        if t.validation_criteria.is_empty() {
            continue;
        }
        // Check if the body contains a Verification section with evidence
        let has_verification = t.body.contains("### Verification") && t.body.contains("✓");

        if !has_verification {
            findings.push(Finding {
                file: filename(t),
                rule: "low-evidence-closure".into(),
                message: format!(
                    "{} validation criteria defined but no evidence recorded",
                    t.validation_criteria.len()
                ),
                severity: "warning".into(),
            });
        }
    }

    findings
}

fn filename(t: &Ticket) -> String {
    t.path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

// --- Deep audit rules (--deep) ---

/// Generic phrases that indicate low-effort evidence.
const THIN_EVIDENCE_PHRASES: &[&str] = &[
    "done",
    "looks good",
    "works",
    "fixed",
    "lgtm",
    "completed",
    "should work",
    "all good",
    "tested",
    "verified",
    "ok",
    "pass",
    "passed",
];

/// Check evidence specificity on done tickets with validation_criteria.
/// Flags evidence strings that are too short or use generic phrases.
pub fn check_evidence_specificity(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        if t.validation_criteria.is_empty() {
            continue;
        }

        // Look for evidence in the Verification section
        let evidence_lines: Vec<&str> = t
            .body
            .lines()
            .skip_while(|l| !l.contains("### Verification"))
            .skip(1)
            .take_while(|l| !l.starts_with("## ") && !l.starts_with("### "))
            .filter(|l| l.starts_with("- ") || l.starts_with("✓"))
            .collect();

        for line in &evidence_lines {
            let text = line
                .trim_start_matches("- ")
                .trim_start_matches("✓ ")
                .trim();
            if text.len() < 15 {
                let is_generic = THIN_EVIDENCE_PHRASES
                    .iter()
                    .any(|p| text.eq_ignore_ascii_case(p));
                if is_generic || text.len() < 10 {
                    findings.push(Finding {
                        file: filename(t),
                        rule: "thin-evidence".into(),
                        message: format!("evidence too brief or generic: \"{}\"", text),
                        severity: "warning".into(),
                    });
                }
            }
        }

        // Check evidence count vs criteria count
        if !evidence_lines.is_empty() && evidence_lines.len() < t.validation_criteria.len() {
            findings.push(Finding {
                file: filename(t),
                rule: "evidence-count-mismatch".into(),
                message: format!(
                    "{} validation criteria but only {} evidence items",
                    t.validation_criteria.len(),
                    evidence_lines.len()
                ),
                severity: "warning".into(),
            });
        }
    }

    findings
}

/// Check for force-close without substantial resolution.
/// Looks for `--force` marker in the Resolution section (tkt appends it).
pub fn check_force_close_justification(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        let resolution_text = t
            .body
            .split_once("## Resolution")
            .map(|(_, after)| after.lines().skip(1).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default();

        let is_forced = resolution_text.contains("(forced)") || resolution_text.contains("--force");

        if is_forced {
            let substance = resolution_text
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.contains("(forced)"))
                .count();
            if substance < 2 {
                findings.push(Finding {
                    file: filename(t),
                    rule: "force-without-justification".into(),
                    message: "force-closed with little or no justification".into(),
                    severity: "warning".into(),
                });
            }
        }
    }

    findings
}

/// Check for template-only tickets (closed with no real content added).
pub fn check_template_only(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for t in corpus.iter().filter(|t| t.status == Status::Done) {
        let has_tbd_body =
            t.body.contains("## What to build\n\nTBD") || t.body.contains("## What to build\n\n\n");
        let has_tbd_ac = t.body.contains("- [ ] TBD");

        if has_tbd_body || has_tbd_ac {
            findings.push(Finding {
                file: filename(t),
                rule: "template-only-closure".into(),
                message: "closed with template placeholders still present".into(),
                severity: "warning".into(),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // --- Resolution quality ---

    #[test]
    fn resolution_quality_missing_section() {
        let corpus = vec![done_ticket("01", "auth", "# Auth\n\nSome body text.\n")];
        let findings = check_resolution_quality(&corpus);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "missing-resolution");
    }

    #[test]
    fn resolution_quality_tbd() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Resolution (2026-01-01)\n\nTBD\n",
        )];
        let findings = check_resolution_quality(&corpus);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "tbd-resolution");
    }

    #[test]
    fn resolution_quality_empty() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Resolution (2026-01-01)\n\n",
        )];
        let findings = check_resolution_quality(&corpus);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "tbd-resolution");
    }

    #[test]
    fn resolution_quality_good() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Resolution (2026-01-01)\n\nShipped JWT tokens.\n",
        )];
        let findings = check_resolution_quality(&corpus);
        assert!(findings.is_empty());
    }

    #[test]
    fn resolution_quality_skips_non_done() {
        let corpus = vec![open_ticket(
            "01",
            "auth",
            "# Auth\n\nNo resolution section.\n",
        )];
        let findings = check_resolution_quality(&corpus);
        assert!(findings.is_empty());
    }

    // --- AC completeness ---

    #[test]
    fn ac_completeness_all_unchecked() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Acceptance criteria\n\n- [ ] First\n- [ ] Second\n\n## Resolution (2026-01-01)\n\nDone\n",
        )];
        let findings = check_ac_completeness(&corpus);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "all-acs-unchecked-on-done");
        assert!(findings[0].message.contains("2 unchecked"));
    }

    #[test]
    fn ac_completeness_partially_checked_ok() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Acceptance criteria\n\n- [x] First\n- [ ] Second\n\n## Resolution (2026-01-01)\n\nDone\n",
        )];
        let findings = check_ac_completeness(&corpus);
        assert!(findings.is_empty());
    }

    #[test]
    fn ac_completeness_no_ac_section_ok() {
        let corpus = vec![done_ticket(
            "01",
            "auth",
            "# Auth\n\n## Resolution (2026-01-01)\n\nDone\n",
        )];
        let findings = check_ac_completeness(&corpus);
        assert!(findings.is_empty());
    }

    // --- Stale WIP ---

    #[test]
    fn stale_wip_detected() {
        let corpus = vec![wip_ticket("01", "auth")];
        let now = 1_000_000;
        let eight_days_ago = now - 8 * 24 * 60 * 60;
        let findings = check_stale_wip(&corpus, now, |_| Some(eight_days_ago));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "stale-wip");
        assert!(findings[0].message.contains("8 days"));
    }

    #[test]
    fn stale_wip_recent_ok() {
        let corpus = vec![wip_ticket("01", "auth")];
        let now = 1_000_000;
        let two_days_ago = now - 2 * 24 * 60 * 60;
        let findings = check_stale_wip(&corpus, now, |_| Some(two_days_ago));
        assert!(findings.is_empty());
    }

    #[test]
    fn stale_wip_no_timestamp_ok() {
        let corpus = vec![wip_ticket("01", "auth")];
        let findings = check_stale_wip(&corpus, 1_000_000, |_| None);
        assert!(findings.is_empty());
    }

    #[test]
    fn stale_wip_skips_non_wip() {
        let corpus = vec![open_ticket("01", "auth", "# Auth\n")];
        let old_ts = 0;
        let findings = check_stale_wip(&corpus, 1_000_000, |_| Some(old_ts));
        assert!(findings.is_empty());
    }

    // --- Frontier health ---

    #[test]
    fn frontier_health_high_priority_flagged() {
        let corpus = vec![open_ticket_with_priority("01", "auth", "high")];
        let findings = check_frontier_health(&corpus);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "high-priority-open");
    }

    #[test]
    fn frontier_health_medium_priority_ok() {
        let corpus = vec![open_ticket_with_priority("01", "auth", "medium")];
        let findings = check_frontier_health(&corpus);
        assert!(findings.is_empty());
    }

    #[test]
    fn frontier_health_blocked_high_not_flagged() {
        // High priority but blocked — not on frontier
        let corpus = vec![
            open_ticket("99", "blocker", "# Blocker\n"),
            blocked_high_ticket("01", "auth", "99"),
        ];
        let findings = check_frontier_health(&corpus);
        assert!(findings.is_empty());
    }

    // --- Test helpers ---

    fn done_ticket(id: &str, slug: &str, body: &str) -> Ticket {
        let content = format!(
            "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: done\nblocked_by: []\n---\n{}",
            id, slug, body
        );
        Ticket::parse_str(&content, &PathBuf::from(format!("{}-{}.md", id, slug))).unwrap()
    }

    fn open_ticket(id: &str, slug: &str, body: &str) -> Ticket {
        let content = format!(
            "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: open\nblocked_by: []\n---\n{}",
            id, slug, body
        );
        Ticket::parse_str(&content, &PathBuf::from(format!("{}-{}.md", id, slug))).unwrap()
    }

    fn wip_ticket(id: &str, slug: &str) -> Ticket {
        let content = format!(
            "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: in_progress\nblocked_by: []\n---\n\n# {}\n",
            id, slug, slug
        );
        Ticket::parse_str(&content, &PathBuf::from(format!("{}-{}.md", id, slug))).unwrap()
    }

    fn open_ticket_with_priority(id: &str, slug: &str, priority: &str) -> Ticket {
        let content = format!(
            "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: open\nblocked_by: []\npriority: {}\n---\n\n# {}\n",
            id, slug, priority, slug
        );
        Ticket::parse_str(&content, &PathBuf::from(format!("{}-{}.md", id, slug))).unwrap()
    }

    fn blocked_high_ticket(id: &str, slug: &str, blocked_by: &str) -> Ticket {
        let content = format!(
            "---\nid: \"{}\"\ntitle: \"{}\"\nstatus: open\nblocked_by: [\"{}\"]\npriority: high\n---\n\n# {}\n",
            id, slug, blocked_by, slug
        );
        Ticket::parse_str(&content, &PathBuf::from(format!("{}-{}.md", id, slug))).unwrap()
    }
}

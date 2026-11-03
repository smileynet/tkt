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

/// Collect every corpus finding for the `.tickets/` directory `dir`.
///
/// This is the single source of truth for `validate` and `doctor`: it owns the
/// per-file parse loop (so an unparseable ticket becomes a structured
/// `unparseable` **error** finding with file context instead of being silently
/// dropped by `load_corpus`) and runs every structural, AC, stub-body, and
/// closure-quality check. Both commands call this so their strict pass/fail
/// result and finding set can never disagree.
pub fn collect_corpus_findings(dir: &std::path::Path) -> anyhow::Result<Vec<Finding>> {
    let mut all_findings: Vec<Finding> = Vec::new();

    let mut corpus: Vec<Ticket> = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        match Ticket::parse(&entry.path()) {
            Ok(t) => corpus.push(t),
            Err(e) => all_findings.push(Finding {
                file: entry.file_name().to_string_lossy().to_string(),
                rule: "unparseable".to_string(),
                message: e.to_string(),
                severity: "error".to_string(),
            }),
        }
    }

    all_findings.extend(check_status(&corpus));
    all_findings.extend(check_env(&corpus));
    all_findings.extend(check_id_filename(&corpus));
    all_findings.extend(check_duplicate_ids(&corpus));
    all_findings.extend(check_dangling_deps(&corpus));
    all_findings.extend(check_cancelled_tickets(&corpus));
    all_findings.extend(check_blocked_by_cancelled(&corpus));
    all_findings.extend(check_cycles(&corpus));
    all_findings.extend(check_dangling_parent(&corpus));
    all_findings.extend(check_parent_cycles(&corpus));
    all_findings.extend(check_dangling_body_references(&corpus));
    all_findings.extend(check_unchecked_acs(&corpus));
    all_findings.extend(check_ac_transfers(&corpus));
    all_findings.extend(check_stub_body(&corpus));
    let cfg = crate::config::ProjectConfig::load(dir);
    if let Err(error) = cfg.validate() {
        all_findings.push(Finding {
            file: "config.toml".to_string(),
            rule: "label-config".to_string(),
            message: error.to_string(),
            severity: "error".to_string(),
        });
    } else {
        all_findings.extend(check_label_namespaces(&corpus, &cfg));
    }
    all_findings.extend(check_source_references(
        &corpus,
        cfg.validate_require_source_reference,
    ));
    let cutoff = cfg.validate_missing_resolution_before_id;
    let legacy_ids: HashSet<String> = cutoff
        .map(|cutoff| {
            corpus
                .iter()
                .filter(|ticket| ticket.id.parse::<u64>().is_ok_and(|id| id < cutoff))
                .map(|ticket| ticket.id.clone())
                .collect()
        })
        .unwrap_or_default();
    all_findings.extend(
        crate::audit::check_resolution_quality(&corpus, None)
            .into_iter()
            .filter(|finding| {
                finding.rule != "missing-resolution"
                    || !corpus.iter().any(|ticket| {
                        filename(ticket) == finding.file && legacy_ids.contains(&ticket.id)
                    })
            }),
    );

    Ok(all_findings)
}

// --- Rules ---

pub fn has_source_reference(ticket: &Ticket) -> bool {
    ticket.spec.as_ref().is_some_and(|s| !s.trim().is_empty())
        || !ticket.blocked_by.is_empty()
        || ticket.parent.as_ref().is_some_and(|p| !p.trim().is_empty())
        || crate::renumber::body_has_source_reference(&ticket.body)
}

pub fn check_source_references(corpus: &[Ticket], required: bool) -> Vec<Finding> {
    corpus.iter().filter(|t| matches!(t.status, Status::Open | Status::InProgress))
        .filter(|t| !has_source_reference(t)).map(|t| Finding {
            file: filename(t), rule: "no-source-reference".into(),
            message: "add a spec, prerequisite, parent, local ticket citation, documentation path or HTTP(S) source link".into(),
            severity: if required { "error" } else { "info" }.into(),
        }).collect()
}

pub fn check_ac_transfers(corpus: &[Ticket]) -> Vec<Finding> {
    core::ac_transfer::transfer_errors(corpus)
        .into_iter()
        .map(|(id, message)| Finding {
            file: filename(corpus.iter().find(|ticket| ticket.id == id).unwrap()),
            rule: "invalid-ac-transfer".into(),
            message,
            severity: "error".into(),
        })
        .collect()
}

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
            let slug_id_matches_stem = t
                .path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| stem == t.id);
            !slug_id_matches_stem && !name.starts_with(&format!("{}-", t.id))
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

/// Warn when an unqualified numeric body citation does not resolve in this corpus.
pub fn check_dangling_body_references(corpus: &[Ticket]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for ticket in corpus {
        for citation in crate::renumber::body_ticket_citations(&ticket.body) {
            if !corpus
                .iter()
                .any(|candidate| crate::renumber::numeric_id_eq(&candidate.id, &citation.id))
            {
                findings.push(Finding {
                    file: filename(ticket),
                    rule: "dangling-body-reference".into(),
                    message: format!(
                        "body reference {} on line {} has no matching local ticket ID; verify whether it is external",
                        citation.text, citation.line
                    ),
                    severity: "warning".into(),
                });
            }
        }
    }
    findings
}

pub fn check_cancelled_tickets(corpus: &[Ticket]) -> Vec<Finding> {
    let by_id: HashMap<&str, &Ticket> = corpus
        .iter()
        .map(|ticket| (ticket.id.as_str(), ticket))
        .collect();
    let mut findings = Vec::new();
    for ticket in corpus
        .iter()
        .filter(|ticket| ticket.status == Status::Cancelled)
    {
        let Some(record) = core::cancellation_record(&ticket.body) else {
            findings.push(Finding {
                file: filename(ticket),
                rule: "missing-cancellation".into(),
                message: "cancelled ticket has no Cancellation section".into(),
                severity: "error".into(),
            });
            continue;
        };
        if matches!(record.reason.as_deref(), Some("duplicate" | "superseded"))
            && record.by.is_none()
        {
            findings.push(Finding {
                file: filename(ticket),
                rule: "missing-cancellation-by".into(),
                message: "duplicate or superseded cancellation requires a By ticket".into(),
                severity: "error".into(),
            });
        }
        if let Some(by) = record.by.as_deref() {
            if !by_id.contains_key(by) {
                findings.push(Finding {
                    file: filename(ticket),
                    rule: "invalid-cancellation-by".into(),
                    message: format!("cancellation By ticket {} does not resolve", by),
                    severity: "error".into(),
                });
            }
        }
    }
    findings
}

pub fn check_blocked_by_cancelled(corpus: &[Ticket]) -> Vec<Finding> {
    let cancelled: HashSet<&str> = corpus
        .iter()
        .filter(|ticket| ticket.status == Status::Cancelled)
        .map(|ticket| ticket.id.as_str())
        .collect();
    corpus
        .iter()
        .flat_map(|ticket| {
            ticket
                .blocked_by
                .iter()
                .filter(|id| cancelled.contains(id.as_str()))
                .map(|id| Finding {
                    file: filename(ticket),
                    rule: "blocked-by-cancelled".into(),
                    message: format!("blocked_by references cancelled ticket {}", id),
                    severity: "error".into(),
                })
        })
        .collect()
}

/// Check namespaced tags against configured value sets and selection rules.
pub fn check_label_namespaces(
    corpus: &[Ticket],
    config: &crate::config::ProjectConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (name, namespace) in &config.label_namespaces {
        let prefix = format!("{name}:");
        for ticket in corpus {
            let matching: Vec<(&str, &str)> = ticket
                .tags
                .iter()
                .filter_map(|tag| tag.strip_prefix(&prefix).map(|value| (tag.as_str(), value)))
                .collect();
            if namespace.single_select && matching.len() > 1 {
                findings.push(Finding {
                    file: filename(ticket),
                    rule: "label-namespace-single-select".into(),
                    message: format!(
                        "single-select namespace {:?} appears {} times",
                        name,
                        matching.len()
                    ),
                    severity: "error".into(),
                });
            }
            for (tag, value) in matching {
                if !namespace.values.iter().any(|allowed| allowed == value) {
                    findings.push(Finding {
                        file: filename(ticket),
                        rule: "label-namespace-value".into(),
                        message: format!(
                            "tag {:?} is not an allowed value for namespace {:?}",
                            tag, name
                        ),
                        severity: "error".into(),
                    });
                }
            }
        }
    }
    findings
}

/// Check for dangling parent references.
pub fn check_dangling_parent(corpus: &[Ticket]) -> Vec<Finding> {
    let known: HashSet<&str> = corpus.iter().map(|t| t.id.as_str()).collect();
    corpus
        .iter()
        .filter_map(|ticket| {
            let parent = ticket.parent.as_deref()?;
            if known.contains(parent) {
                return None;
            }
            Some(Finding {
                file: filename(ticket),
                rule: "dangling-parent".into(),
                message: format!("parent {:?} has no ticket", parent),
                severity: "error".into(),
            })
        })
        .collect()
}

/// Check for self-parent references and cycles in parent chains.
pub fn check_parent_cycles(corpus: &[Ticket]) -> Vec<Finding> {
    let known: HashSet<&str> = corpus.iter().map(|t| t.id.as_str()).collect();
    let mut findings: Vec<Finding> = corpus
        .iter()
        .filter_map(|ticket| {
            let parent = ticket.parent.as_deref()?;
            core::validate::validate_parent_ref(&ticket.id, parent)
                .err()
                .map(|message| Finding {
                    file: filename(ticket),
                    rule: "self-parent".into(),
                    message,
                    severity: "error".into(),
                })
        })
        .collect();

    let parent_of: HashMap<&str, &str> = corpus
        .iter()
        .filter_map(|ticket| {
            let parent = ticket.parent.as_deref()?;
            (known.contains(parent) && ticket.id != parent).then_some((ticket.id.as_str(), parent))
        })
        .collect();
    let mut state: HashMap<&str, u8> = corpus
        .iter()
        .map(|ticket| (ticket.id.as_str(), 0))
        .collect();
    let mut path: Vec<&str> = Vec::new();
    let mut cycles: Vec<Vec<&str>> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        parent_of: &HashMap<&'a str, &'a str>,
        state: &mut HashMap<&'a str, u8>,
        path: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<&'a str>>,
    ) {
        state.insert(node, 1);
        path.push(node);
        if let Some(&parent) = parent_of.get(node) {
            match state.get(parent) {
                Some(1) => {
                    if let Some(pos) = path.iter().position(|&value| value == parent) {
                        let mut cycle = path[pos..].to_vec();
                        cycle.push(parent);
                        cycles.push(cycle);
                    }
                }
                Some(0) => dfs(parent, parent_of, state, path, cycles),
                _ => {}
            }
        }
        path.pop();
        state.insert(node, 2);
    }

    let mut nodes: Vec<&str> = parent_of.keys().copied().collect();
    nodes.sort();
    for node in nodes {
        if state.get(node) == Some(&0) {
            dfs(node, &parent_of, &mut state, &mut path, &mut cycles);
        }
    }

    let id_to_file: HashMap<&str, String> = corpus
        .iter()
        .map(|ticket| (ticket.id.as_str(), filename(ticket)))
        .collect();
    let mut seen = HashSet::new();
    for cycle in cycles {
        let path_part = &cycle[..cycle.len() - 1];
        let min_pos = path_part
            .iter()
            .enumerate()
            .min_by_key(|(_, id)| *id)
            .map(|(index, _)| index)
            .unwrap_or(0);
        let mut normalized = path_part[min_pos..].to_vec();
        normalized.extend_from_slice(&path_part[..min_pos]);
        let key = normalized.join(" -> ");
        if seen.insert(key.clone()) {
            let first_id = normalized[0];
            normalized.push(first_id);
            findings.push(Finding {
                file: id_to_file
                    .get(first_id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".into()),
                rule: "parent-cycle".into(),
                message: format!("parent cycle: {}", normalized.join(" -> ")),
                severity: "error".into(),
            });
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
            let section = match crate::core::ac_section_range(&t.body) {
                Some(range) => &t.body[range],
                None => return None,
            };
            let count = RE.find_iter(section).count();
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

/// Check for template placeholder ("stub") bodies on tickets still being worked
/// (open / in_progress). `tkt new` writes a `## What to build\n\nTBD` +
/// `- [ ] TBD` stub by default; this surfaces tickets that were never filled in,
/// before they reach `done` (the done-only counterpart is audit's
/// `template-only-closure`). Advisory: warning severity, so it only fails the
/// run under `--strict`.
///
/// Detection is CONTENT-based, not substring-based: it inspects the trimmed
/// content of the `## What to build` and `## Acceptance criteria` sections. A
/// ticket that merely *quotes* the template sentinel in otherwise-filled prose
/// (e.g. a ticket about the template itself) is not flagged, because its section
/// content is substantive. Only a section whose real content is empty or exactly
/// `TBD` (intent) or whose sole checkbox is `- [ ] TBD` (AC) counts as a stub.
pub fn check_stub_body(corpus: &[Ticket]) -> Vec<Finding> {
    corpus
        .iter()
        .filter(|t| matches!(t.status, Status::Open | Status::InProgress))
        .filter_map(|t| {
            if is_stub_body(&t.body) {
                Some(Finding {
                    file: filename(t),
                    rule: "stub-body".into(),
                    message: "body still has template placeholders (fill in before closing)".into(),
                    severity: "warning".into(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Classify real level-two template sections while excluding fenced examples.
/// Lifecycle policy remains explicit: completed tickets treat any exact
/// placeholder AC line as unfinished, even if it is checked or mixed with real ACs.
pub(crate) fn has_completed_template_content(body: &str) -> bool {
    let sections = template_sections(body);
    sections
        .intent
        .iter()
        .any(|section| section.is_empty() || is_placeholder_intent(&section.content))
        || sections.acceptance.iter().any(|section| {
            let lines: Vec<_> = section
                .content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect();
            (lines.is_empty() && section.is_empty())
                || lines.iter().any(|line| is_placeholder_ac_line(line))
        })
}

fn is_stub_body(body: &str) -> bool {
    let sections = template_sections(body);
    sections
        .intent
        .iter()
        .any(|section| section.is_empty() || is_placeholder_intent(&section.content))
        || sections
            .acceptance
            .iter()
            .any(|section| is_placeholder_ac(&section.content))
}

#[derive(Default)]
struct TemplateSection {
    content: String,
    has_fenced_content: bool,
}

impl TemplateSection {
    fn is_empty(&self) -> bool {
        self.content.trim().is_empty() && !self.has_fenced_content
    }
}

struct TemplateSections {
    intent: Vec<TemplateSection>,
    acceptance: Vec<TemplateSection>,
}

#[derive(Clone, Copy)]
enum TemplateTarget {
    Intent,
    Acceptance,
}

fn template_sections(body: &str) -> TemplateSections {
    let mut sections = TemplateSections {
        intent: Vec::new(),
        acceptance: Vec::new(),
    };
    let mut current: Option<TemplateTarget> = None;
    let mut fence: Option<(char, usize)> = None;
    for line in body.lines() {
        let trimmed = line.trim();
        let delimiter = trimmed.chars().next().filter(|c| *c == '`' || *c == '~');
        let marker_width = delimiter.map_or(0, |first| {
            trimmed.chars().take_while(|c| *c == first).count()
        });
        if marker_width >= 3 {
            let marker_rest = &trimmed[marker_width..];
            if let Some((open, width)) = fence {
                if delimiter == Some(open) && marker_width >= width && marker_rest.trim().is_empty()
                {
                    fence = None;
                }
            } else {
                match current {
                    Some(TemplateTarget::Intent) => mark_fenced_content(&mut sections.intent),
                    Some(TemplateTarget::Acceptance) => {
                        mark_fenced_content(&mut sections.acceptance)
                    }
                    _ => {}
                }
                fence = Some((delimiter.expect("fence delimiter"), marker_width));
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        match trimmed {
            "## What to build" => {
                sections.intent.push(TemplateSection::default());
                current = Some(TemplateTarget::Intent);
            }
            "## Acceptance criteria" => {
                sections.acceptance.push(TemplateSection::default());
                current = Some(TemplateTarget::Acceptance);
            }
            value if value.starts_with("## ") => current = None,
            _ => match current {
                Some(TemplateTarget::Intent) => append_section(&mut sections.intent, line),
                Some(TemplateTarget::Acceptance) => append_section(&mut sections.acceptance, line),
                _ => {}
            },
        }
    }
    sections
}

fn append_section(sections: &mut [TemplateSection], line: &str) {
    let content = &mut sections.last_mut().expect("active section").content;
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(line);
}

fn mark_fenced_content(sections: &mut [TemplateSection]) {
    sections
        .last_mut()
        .expect("active section")
        .has_fenced_content = true;
}

fn is_placeholder_intent(content: &str) -> bool {
    let content = content.trim();
    content == "TBD" || content == core::ticket::DEFAULT_INTENT_PROMPT
}

fn is_placeholder_ac(content: &str) -> bool {
    let checkboxes: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- ["))
        .collect();
    checkboxes == ["- [ ] TBD"]
        || checkboxes == [format!("- [ ] {}", core::ticket::DEFAULT_AC_PROMPT)]
}

fn is_placeholder_ac_line(line: &str) -> bool {
    ["- [ ] ", "- [x] ", "- [X] "].iter().any(|prefix| {
        line.strip_prefix(prefix)
            .is_some_and(|text| text == "TBD" || text == core::ticket::DEFAULT_AC_PROMPT)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_ticket(content: &str) -> Ticket {
        Ticket::parse_str(content, Path::new("test.md")).unwrap()
    }

    #[test]
    fn stub_body_flags_open_template() {
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: []\n---\n\n# A\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n",
        );
        let f = check_stub_body(&[t]);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, "stub-body");
        assert_eq!(f[0].severity, "warning");
    }

    #[test]
    fn stub_body_ignores_filled_open() {
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: []\n---\n\n# A\n\n## What to build\n\nReal content here.\n\n## Acceptance criteria\n\n- [ ] real criterion\n",
        );
        assert!(check_stub_body(&[t]).is_empty());
    }

    #[test]
    fn stub_body_ignores_done_tickets() {
        // Done tickets are the domain of audit's template-only-closure, not this check.
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: done\nblocked_by: []\n---\n\n# A\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n",
        );
        assert!(check_stub_body(&[t]).is_empty());
    }

    #[test]
    fn stub_body_ignores_ticket_that_quotes_the_sentinel() {
        // A ticket ABOUT the template quotes `## What to build`/`TBD` in filled
        // prose. Content-based detection must not flag it (regression: the old
        // substring match produced false positives on #152/#174/#176).
        let t = make_ticket(
            "---\nid: \"01\"\ntitle: \"A\"\nstatus: open\nblocked_by: []\n---\n\n# A\n\n## What to build\n\nThe template writes `## What to build\\n\\nTBD` and `- [ ] TBD` by default; replace it.\n\n## Acceptance criteria\n\n- [ ] detector ignores quoted sentinels\n",
        );
        assert!(
            check_stub_body(&[t]).is_empty(),
            "quoting the sentinel in filled prose must not be flagged"
        );
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

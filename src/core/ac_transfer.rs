//! Linked acceptance-criterion transfers, distinct from checked work.

use std::sync::LazyLock;

use regex::Regex;

use super::{Ticket, TicketFile};

static AC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^- \[([ xX>])\] ([^\r\n]*)\r?$").unwrap());
static MOVED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+) <!-- tkt-moved:([0-9]+) -->$").unwrap());
static FROM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+) <!-- tkt-from:([0-9]+):([0-9]+) -->$").unwrap());

pub struct Criterion<'a> {
    pub range: std::ops::Range<usize>,
    pub state: &'a str,
    pub text: &'a str,
}

pub fn criteria(file: &TicketFile) -> Vec<Criterion<'_>> {
    let Some(range) = section_range(&file.body) else {
        return Vec::new();
    };
    let mut fence = None;
    let mut offset = range.start;
    let mut result = Vec::new();
    for line in file.body[range].split_inclusive('\n') {
        if !fenced(line, &mut fence) {
            if let Some(capture) = AC.captures(line.trim_end_matches('\n')) {
                let matched = capture.get(0).unwrap();
                result.push(Criterion {
                    range: offset + matched.start()
                        ..offset + matched.end() - usize::from(matched.as_str().ends_with('\r')),
                    state: capture.get(1).unwrap().as_str(),
                    text: capture.get(2).unwrap().as_str(),
                });
            }
        }
        offset += line.len();
    }
    result
}

fn fenced(line: &str, fence: &mut Option<(char, usize)>) -> bool {
    let trimmed = line.trim();
    let delimiter = trimmed.chars().next().filter(|c| *c == '`' || *c == '~');
    let width = delimiter.map_or(0, |c| trimmed.chars().take_while(|v| *v == c).count());
    if let Some((open, size)) = *fence {
        if delimiter == Some(open) && width >= size && trimmed[width..].trim().is_empty() {
            *fence = None;
        }
        return true;
    }
    if width >= 3 {
        *fence = Some((delimiter.unwrap(), width));
        return true;
    }
    false
}

pub fn section_range(body: &str) -> Option<std::ops::Range<usize>> {
    let mut fence = None;
    let mut start = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        if !fenced(line, &mut fence) {
            let heading = line.trim_end();
            if start.is_none() && heading == "## Acceptance criteria" {
                start = Some(offset + line.len());
            } else if heading.starts_with("## ") {
                if let Some(start) = start {
                    return Some(start..offset);
                }
            }
        }
        offset += line.len();
    }
    start.map(|start| start..body.len())
}

pub fn transfer_errors(corpus: &[Ticket]) -> Vec<(String, String)> {
    let mut errors = Vec::new();
    for source in corpus {
        for (index, criterion) in criteria(&source.file).iter().enumerate() {
            if criterion.state != ">" {
                if criterion.text.contains("<!-- tkt-moved:") {
                    errors.push((
                        source.id.clone(),
                        format!(
                            "criterion {} has a moved link without the moved state",
                            index + 1
                        ),
                    ));
                }
                if let Some(from) = FROM.captures(criterion.text) {
                    let valid = corpus
                        .iter()
                        .find(|t| t.id == from[2])
                        .and_then(|t| {
                            criteria(&t.file)
                                .into_iter()
                                .nth(from[3].parse::<usize>().ok()?.checked_sub(1)?)
                        })
                        .is_some_and(|ac| {
                            ac.state == ">"
                                && ac.text
                                    == format!("{} <!-- tkt-moved:{} -->", &from[1], source.id)
                        });
                    if !valid {
                        errors.push((
                            source.id.clone(),
                            format!(
                                "receiving criterion {} has no matching moved source",
                                index + 1
                            ),
                        ));
                    }
                } else if criterion.text.contains("<!-- tkt-from:") {
                    errors.push((
                        source.id.clone(),
                        format!(
                            "criterion {} has an invalid source back-reference",
                            index + 1
                        ),
                    ));
                }
                continue;
            }
            let Some(marker) = MOVED.captures(criterion.text) else {
                errors.push((
                    source.id.clone(),
                    format!("criterion {} has an invalid moved marker", index + 1),
                ));
                continue;
            };
            let target = &marker[2];
            if marker[1].contains("<!-- tkt-") {
                errors.push((
                    source.id.clone(),
                    format!("criterion {} mixes transfer markers", index + 1),
                ));
                continue;
            }
            let suffix = format!(" <!-- tkt-from:{}:{} -->", source.id, index + 1);
            let matches = corpus
                .iter()
                .find(|t| {
                    t.id == target && t.id != source.id && t.status != super::Status::Cancelled
                })
                .map(|t| {
                    criteria(&t.file)
                        .into_iter()
                        .filter(|ac| {
                            ac.state != ">" && ac.text.strip_suffix(&suffix) == Some(&marker[1])
                        })
                        .count()
                })
                .unwrap_or(0);
            if matches != 1 {
                errors.push((
                    source.id.clone(),
                    format!(
                        "criterion {} needs exactly one matching receiving criterion in ticket {}",
                        index + 1,
                        target
                    ),
                ));
            }
        }
    }
    errors
}

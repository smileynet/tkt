//! Bounded read-only GitHub extraction and the immutable source snapshot.

use std::collections::BTreeMap;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;
use serde_json::Value;

use crate::{DomainError, ErrorKind};

pub const API_VERSION: &str = "2026-03-10";
pub const MAX_OBJECTS: usize = 1_000;
pub const MAX_BYTES: usize = 10 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> anyhow::Error {
    DomainError::new(ErrorKind::Validation, message.into()).into()
}

pub struct Repository {
    pub id: u64,
    pub name: String,
}

pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub url: String,
    pub raw: Value,
}

pub struct Extraction {
    pub repository: Repository,
    pub issues: Vec<Issue>,
    pub received: usize,
    pub closed: usize,
    pub pull_requests: usize,
    pub duplicates: usize,
    pub filtered: usize,
    pub raw: Value,
}

pub fn repository_name(value: &str) -> Result<&str> {
    let parts: Vec<_> = value.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || *part == "."
                || *part == ".."
                || !part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        })
    {
        return Err(invalid(
            "GitHub repository must be an explicit OWNER/NAME on github.com",
        ));
    }
    Ok(value)
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| invalid(format!("GitHub object requires string {key}")))
}

fn positive(value: &Value, key: &str) -> Result<u64> {
    value[key]
        .as_u64()
        .filter(|id| *id > 0)
        .ok_or_else(|| invalid(format!("GitHub object requires positive integer {key}")))
}

pub fn decode(
    repository: &Value,
    requested: &str,
    pages: &[Value],
    label: Option<&str>,
) -> Result<Extraction> {
    let raw = serde_json::json!({"repository":repository,"pages":pages});
    repository_name(requested)?;
    let name = repository_name(string(repository, "full_name")?)?.to_owned();
    if !name.eq_ignore_ascii_case(requested)
        || string(repository, "html_url")? != format!("https://github.com/{name}")
    {
        return Err(invalid(
            "GitHub repository metadata does not match the requested provenance",
        ));
    }
    let repository = Repository {
        id: positive(repository, "id")?,
        name,
    };
    let mut unique = BTreeMap::new();
    let mut received = 0;
    for page in pages {
        let rows = page
            .as_array()
            .ok_or_else(|| invalid("GitHub issue page must be an array"))?;
        received += rows.len();
        if received >= MAX_OBJECTS {
            return Err(invalid(
                "GitHub extraction reached the 1,000-object bound; input is incomplete",
            ));
        }
        for row in rows {
            let id = positive(row, "id")?;
            if let Some(previous) = unique.insert(id, row.clone()) {
                if previous != *row {
                    return Err(invalid(format!(
                        "GitHub issue {id} changed across extraction pages"
                    )));
                }
            }
        }
    }
    let duplicates = received - unique.len();
    let mut issues = Vec::new();
    let mut closed = 0;
    let mut pull_requests = 0;
    let mut filtered = 0;
    for (id, row) in unique {
        let number = positive(&row, "number")?;
        let title = string(&row, "title")?.to_owned();
        if title.trim().is_empty() {
            return Err(invalid(format!("GitHub issue {id} has an empty title")));
        }
        let state = string(&row, "state")?;
        if !matches!(state, "open" | "closed") {
            return Err(invalid(format!("GitHub issue {id} has unsupported state")));
        }
        let pr = row.get("pull_request").is_some();
        if pr && !row["pull_request"].is_object() {
            return Err(invalid(format!(
                "GitHub issue {id} has malformed pull_request metadata"
            )));
        }
        let url = string(&row, "html_url")?.to_owned();
        let kind = if pr { "pull" } else { "issues" };
        if string(&row, "repository_url")?
            != format!("https://api.github.com/repos/{}", repository.name)
            || url != format!("https://github.com/{}/{kind}/{number}", repository.name)
        {
            return Err(invalid(format!(
                "GitHub issue {id} has mismatched repository/URL provenance"
            )));
        }
        let body = match row.get("body") {
            Some(Value::Null) => None,
            Some(Value::String(body)) => Some(body.clone()),
            _ => {
                return Err(invalid(format!(
                    "GitHub issue {id} requires a string or null body"
                )))
            }
        };
        if !row["labels"].is_array() {
            return Err(invalid(format!(
                "GitHub issue {id} requires a labels array"
            )));
        }
        let selected = if let Some(label) = label.filter(|_| !pr && state == "open") {
            let names: Result<Vec<_>> = row["labels"]
                .as_array()
                .expect("validated labels array")
                .iter()
                .map(|value| {
                    value["name"]
                        .as_str()
                        .ok_or_else(|| invalid("GitHub label filter requires named source labels"))
                })
                .collect();
            names?.contains(&label)
        } else {
            true
        };
        if pr {
            pull_requests += 1;
        } else if state == "closed" {
            closed += 1;
        } else if selected {
            issues.push(Issue {
                id,
                number,
                title,
                body,
                url,
                raw: row,
            });
        } else {
            filtered += 1;
        }
    }
    Ok(Extraction {
        repository,
        issues,
        received,
        closed,
        pull_requests,
        duplicates,
        filtered,
        raw,
    })
}

struct BoundedReader<R> {
    inner: R,
    bytes: usize,
    limit: usize,
}
impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = self.limit.saturating_sub(self.bytes);
        if available == 0 {
            return Err(std::io::Error::other("raw response bound reached"));
        }
        let size = buf.len().min(available);
        let count = self.inner.read(&mut buf[..size])?;
        self.bytes += count;
        if self.bytes >= self.limit {
            return Err(std::io::Error::other("raw response bound reached"));
        }
        Ok(count)
    }
}

fn request(
    repo: &Path,
    endpoint: &str,
    remaining: usize,
    fields: &[(&str, &str)],
) -> Result<(Vec<Value>, usize)> {
    let paginated = endpoint.ends_with("/issues");
    let mut command = Command::new("gh");
    command
        .current_dir(repo)
        .args([
            "api",
            "--method",
            "GET",
            "--hostname",
            "github.com",
            "-H",
            "Accept: application/vnd.github.raw+json",
            "-H",
            &format!("X-GitHub-Api-Version: {API_VERSION}"),
            endpoint,
        ])
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_DEBUG", "")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if paginated {
        command.arg("--paginate");
    }
    for (name, value) in fields {
        command.args(["--raw-field", &format!("{name}={value}")]);
    }
    let mut child = command.spawn().map_err(|_| {
        invalid("cannot execute gh; install GitHub CLI and authenticate it for github.com")
    })?;
    let mut reader = BufReader::new(BoundedReader {
        inner: child.stdout.take().expect("piped stdout"),
        bytes: 0,
        limit: remaining,
    });
    let parsed = (|| -> Result<Vec<Value>> {
        let mut values = Vec::new();
        let mut received = 0;
        for value in serde_json::Deserializer::from_reader(&mut reader).into_iter::<Value>() {
            let value=value.map_err(|_|invalid("GitHub response is invalid/incomplete JSON or reached the 10 MiB raw JSON bound"))?;
            if paginated {
                let rows = value
                    .as_array()
                    .ok_or_else(|| invalid("GitHub issue page must be an array"))?;
                if rows.len() > 100 {
                    return Err(invalid("GitHub issue page exceeds the requested page size"));
                }
                received += rows.len();
                if received >= MAX_OBJECTS {
                    return Err(invalid(
                        "GitHub extraction reached the 1,000-object bound; input is incomplete",
                    ));
                }
            } else if !values.is_empty() {
                return Err(invalid(
                    "GitHub repository response contains multiple objects",
                ));
            }
            values.push(value);
        }
        if values.is_empty() {
            return Err(invalid(
                "GitHub returned an empty response; check authentication and network",
            ));
        }
        Ok(values)
    })();
    if parsed.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let values = parsed?;
    let status = child.wait()?;
    if !status.success() {
        return Err(invalid("GitHub read failed; check gh authentication, repository access, rate limits and network, then retry. No import was applied"));
    }
    Ok((values, reader.get_ref().bytes))
}

pub fn extract(repo: &Path, name: &str, label: Option<&str>) -> Result<Extraction> {
    repository_name(name)?;
    let (metadata, bytes) = request(repo, &format!("repos/{name}"), MAX_BYTES, &[])?;
    let mut fields = vec![("state", "all"), ("per_page", "100")];
    if let Some(label) = label {
        fields.push(("labels", label));
    }
    let (pages, _) = request(
        repo,
        &format!("repos/{name}/issues"),
        MAX_BYTES - bytes,
        &fields,
    )?;
    decode(&metadata[0], name, &pages, label)
}

pub mod ticket;
pub mod validate;

pub use ticket::{
    find_ticket, frontier, id_width, json_string_escape, load_corpus, max_id, new_ticket_text,
    yaml_scalar_escape, Env, Priority, Status, Ticket, TicketFile, ENV_VALUES, STATUS_VALUES,
};

/// Extract the byte range of the acceptance criteria section content from a ticket body.
/// Returns the range starting after the `## Acceptance criteria` heading line,
/// ending at the next `## ` heading or EOF. Returns None if no AC section exists.
pub fn ac_section_range(body: &str) -> Option<std::ops::Range<usize>> {
    let heading = "## Acceptance criteria";
    let start_idx = body.find(heading)?;
    let after_heading = start_idx + heading.len();
    let content_start = body[after_heading..]
        .find('\n')
        .map(|i| after_heading + i + 1)
        .unwrap_or(body.len());
    let content_end = body[content_start..]
        .find("\n## ")
        .map(|i| content_start + i)
        .unwrap_or(body.len());
    Some(content_start..content_end)
}

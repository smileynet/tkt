---
id: "17"
title: "separate Ticket (domain) from TicketFile (raw editor)"
status: open
blocked_by: ["14", "15", "16"]
---

# Separate Ticket (domain) from TicketFile (raw editor)

## What to build

The Ticket struct currently serves two roles: typed domain object (id, title, status queries) and raw frontmatter editor (set_field, serialize, write). This dual role causes id()/title() to allocate on every call (unescape), blocked_by to re-parse from raw on every access, and prevents validation at construction time.

### Changes needed

1. Rename current `Ticket` to `TicketFile` — owns raw frontmatter Vec<(String, String)> + body, provides set_field/remove_field/serialize/write
2. Create new `Ticket` struct with owned, validated fields:
   - `id: String` (unescaped at parse time)
   - `title: String` (unescaped at parse time)
   - `status: Status` (enum: Open, InProgress, Done)
   - `blocked_by: Vec<String>` (parsed once)
   - `env: Option<Env>` (enum)
   - `priority: Option<Priority>` (enum)
   - `spec: Option<String>`
   - `path: PathBuf`
   - `file: TicketFile` (for mutations)
3. `Ticket::parse(path) → Result<Ticket>` validates at construction — invalid status/env rejected here
4. Field access becomes `&self.id` (zero-cost borrow), not method call with allocation
5. Mutation: `ticket.file.set_field(...)` then `ticket.file.write()`

### Deletion test

If the Ticket/TicketFile split were deleted, every caller of id()/title() pays an allocation, blocked_by parses on every access, and validation logic must be duplicated between parse-time and query-time.

## Acceptance criteria

- [ ] `TicketFile` struct handles raw preservation + surgical edits
- [ ] `Ticket` struct has owned, validated, typed fields
- [ ] `id` and `title` access is &str (no allocation)
- [ ] `status` is an enum with exhaustive match
- [ ] Invalid status/env rejected at parse time (not scattered in accessors)
- [ ] All 39+ tests pass
- [ ] Frontier computation uses typed fields directly
- [ ] cargo clippy clean, cargo fmt clean

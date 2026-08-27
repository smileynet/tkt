---
id: "132"
title: "Fix common hand-edits ejecting tickets from corpus (BOM, comments, space-colon)"
status: done
blocked_by: []
priority: urgent
validation_criteria:
  - "UTF-8 BOM files parse successfully"
  - "comment lines in frontmatter do not hard-bail"
  - "space before colon in key: value tolerated"
---

# Fix common hand-edits ejecting tickets from corpus (BOM, comments, space-colon)

## What to build

Three common hand-edits cause `TicketFile::parse_str` to bail, and `load_corpus` then
silently skips the file (stderr warning only) — ejecting the ticket from frontier, query,
and any dependent's blocker resolution. Make the parser lenient for all three (aligned with
YAML 1.2 spec, which permits leading BOM, treats comments as non-data, and allows whitespace
before the colon).

The three edits collapse into two bail sites:
1. **UTF-8 BOM** → `is_fence(lines[0])` fails (`trim()` doesn't strip `\u{FEFF}`) → opening-fence bail
2. **Comment lines** (`# note`) → fail all parser branches → "unparseable frontmatter line" bail
3. **Space before colon** (`key : value`) → `RE_FM_KEY` rejects → same generic bail

## Context

- **Relevant files:** `src/core/ticket.rs` (parse_str, is_fence, RE_FM_KEY, load_corpus), `src/commands/lint.rs`
- **YAML 1.2 basis:** §5.2 permits leading BOM (parser strips); §6.6 comments are presentation-only (spec-compliant to drop); space *before* colon is legal (Examples 2.4/2.12/2.27) — only space *after* colon is required
- **Comment handling decision:** parse leniently (store as empty-key passthrough like blank lines) so the ticket is NOT ejected, but DROP on lint (spec-compliant; preserving through lint's canonical reordering would cost ~40 lines + position-drift risk for no data value)

## Acceptance criteria

- [x] UTF-8 BOM file parses successfully (appears in corpus)
- [x] `# comment` line in frontmatter parses (ticket not ejected); indented `  # comment` tolerated
- [x] `status : open` (space before colon) parses; key read correctly
- [x] `tkt lint` normalizes space-before-colon to `key: value` and drops comments
- [x] Regression: missing opening fence / missing closing fence / missing required field / genuinely-garbage line still bail
- [x] Integration: corpus with clean + BOM + commented + space-colon tickets → all appear in `tkt ready`
- [x] Integration: broken file (no closing fence) still skipped, stderr shows skip warning
- [x] All existing tests pass

## Out of scope (file as separate tickets)

- **doctor detection gap:** `doctor` uses load_corpus (silent skip) so it can report clean while a ticket is ejected; `validate` catches it. Inconsistent → separate ticket.
- **load_corpus diagnostics:** ejection is stderr-only, bypasses sym_warn/TKT_ASCII/JSON envelope → separate ticket.
- Comment round-trip preservation through lint (spec says comments are non-data)

## Resolution (2026-08-27)

parse_str strips leading BOM, tolerates comment lines (dropped on lint), and RE_FM_KEY relaxed for space-before-colon. Regression guards preserved. 6 unit + 2 integration tests.

### Verification
1. ✓ UTF-8 BOM files parse successfully — "test parse_tolerates_utf8_bom passes; e2e: BOM ticket appears in tkt ready"
2. ✓ comment lines in frontmatter do not hard-bail — "test parse_tolerates_comment_lines passes; e2e: comment ticket loads, dropped on lint"
3. ✓ space before colon in key: value tolerated — "test parse_tolerates_space_before_colon passes; e2e: lint normalizes id : to id:"

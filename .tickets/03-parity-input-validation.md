---
id: "03"
title: "R18 input validation for slugs and free text"
status: open
blocked_by: ["08"]
---

# R18 input validation for slugs and free text

## What to build

Centralized input validation for all user-provided values that enter frontmatter, filenames, or commit messages. Currently only slugs are validated (basic regex). Python validates more thoroughly.

### Validators needed

1. **Slug validator** (enhance existing):
   - Current: `^[a-z0-9][a-z0-9-]*$` ✓
   - Add: reject Windows reserved device names (`con`, `prn`, `aux`, `nul`, `com1`-`com9`, `lpt1`-`lpt9`) case-insensitively, with or without extensions
   - Add: max length (100 chars)

2. **Free-text validator** (new — for titles, specs, notes):
   - Reject: literal newlines, carriage returns
   - Reject: characters that would break YAML even after escaping: null bytes
   - Allow: quotes, backslashes (these get escaped by ticket 08's yaml_scalar_escape)
   - Max length: 200 chars for title, 100 for spec

3. **ID validator** (for --blocked-by values):
   - Must match `^\d+$`
   - Reject self-references (blocked_by contains own ID)

4. **Enum validators** (enforce in `new` and `batch`, not just `edit`):
   - `env`: must be one of `corp`, `personal`, `either`
   - `priority`: must be `high` or empty/absent

5. **Batch duplicate detection**:
   - Reject duplicate slugs within a single batch call
   - Reject slugs that would produce filenames already existing locally or on remote

### Architecture

Create `src/core/validate.rs` with pure functions. Call from every write path. Return structured errors with the field name and reason.

## Acceptance criteria

- [ ] Windows reserved names rejected as slugs (case-insensitive)
- [ ] Titles with newlines rejected with clear error
- [ ] Titles with null bytes rejected
- [ ] `--blocked-by` rejects non-numeric values
- [ ] Self-dependency rejected (blocked_by contains own ID)
- [ ] `new --env invalid` rejected (not just `edit`)
- [ ] `batch slug1 slug1` rejected (duplicate slugs)
- [ ] Validation errors include field name and reason
- [ ] Unit tests for each validator with boundary cases
- [ ] All existing tests still pass

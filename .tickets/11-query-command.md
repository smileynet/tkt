---
id: "11"
title: "implement query command for full corpus JSON dump"
status: done
blocked_by: ["08"]
---

# Implement query command for full corpus JSON dump

## What to build

Python tkt has a `query` command that emits every ticket as JSON Lines (one JSON object per ticket per line). The Rust implementation is missing this command entirely.

### Changes needed

1. Add `Query` variant to the Clap command enum (no arguments needed)
2. Load corpus, serialize each ticket as a complete JSON object (id, title, status, blocked_by, priority, env, spec — all fields)
3. Output one JSON object per line (JSON Lines format)
4. Use the `json_string_escape` helper from ticket 08
5. Exit 0 on success, 2 on corpus load failure

## Acceptance criteria

- [ ] `tkt query` outputs one JSON object per line
- [ ] Each object includes all frontmatter fields (id, title, status, blocked_by, priority, env, spec)
- [ ] Output is valid JSON Lines (each line parses independently)
- [ ] Empty corpus produces no output (exit 0)
- [ ] Corpus with adversarial titles produces valid JSON
- [ ] Integration test validates output against known corpus

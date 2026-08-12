---
id: "91"
title: "Agent close confirmation: formalize validation_criteria field + y/n gate"
status: open
blocked_by: []
priority: high
---

# Agent close confirmation: formalize validation_criteria field + evidence gate

## Design Decision

**No y/n prompts.** Agents always say yes (structurally incapable of honest self-assessment — 56% of agent-claimed completions had zero/negative real improvement per Park & Choi arXiv:2607.25152). Humans habituate to confirmation prompts (aviation checklist fatigue). Instead: require **evidence** — externally verifiable strings linked to specific criteria.

## What to build

### 1. `validation_criteria` frontmatter field

```yaml
---
id: "42"
title: "Implement auth"
status: open
validation_criteria:
  - "cargo test passes"
  - "login returns JWT on valid creds"
  - "invalid creds return 401"
---
```

List of strings. Declared at creation or edit time. Frozen before implementation (the work doesn't define its own success).

### 2. `--evidence` flag on close

Positional by default, named IDs optional:

```bash
# Positional (default) — maps by order
tkt close 42 \
  --evidence "49 passed, 0 failed" \
  --evidence "POST /login → 200 + JWT" \
  --evidence "POST /login bad → 401"

# Named (optional) — prefix with N=
tkt close 42 \
  --evidence "1=49 passed, 0 failed" \
  --evidence "3=POST /login bad → 401"
```

Parsing rule: if evidence starts with `N=` (digit + equals), it's keyed to criterion N. Otherwise positional. Mixed is allowed.

Count check: if criteria exist and evidence count doesn't match criteria count (for positional), behavior depends on config.

### 3. Resolution section records the pairing

```markdown
## Resolution (2026-08-12)

All criteria verified via integration test run.

### Verification
1. ✓ cargo test passes — "49 passed, 0 failed"
2. ✓ login returns JWT on valid creds — "POST /login → 200 + JWT"
3. ✓ invalid creds return 401 — "POST /login bad → 401"
```

### 4. Configuration

```toml
[close]
# Require validation_criteria field to exist on tickets being closed
# off by default — warn when missing
require_validation_criteria = false

# Require --evidence when validation_criteria is present
# warn by default — shows warning but allows close
require_validation_evidence = "warn"   # false | "warn" | true
```

| Scenario | require_validation_criteria=false | require_validation_criteria=true |
|----------|----------------------------------|----------------------------------|
| No VC field, bare close | ✓ close | ⚠ warn, close |
| No VC field, bare close (strict) | ✓ close | ✗ block |

| Scenario | evidence=false | evidence="warn" (default) | evidence=true |
|----------|---------------|---------------------------|---------------|
| VC present, no evidence | ✓ close | ⚠ warn, close | ✗ block |
| VC present, partial evidence | ✓ close | ⚠ warn (list gaps), close | ✗ block |
| VC present, full evidence | ✓ close | ✓ close | ✓ close |
| `--force` on any | ✓ close | ✓ close | ✓ close (logged) |

### 5. Our instance config

```toml
# .tickets/config.toml for tkt itself
[close]
require_validation_criteria = true
require_validation_evidence = true
```

### 6. CLI flags for setting criteria

```bash
tkt new auth --title "..." --vc "cargo test passes" --vc "login returns JWT"
tkt edit 42 --vc "cargo test passes" --vc "login returns JWT" --vc "401 on bad creds"
```

`--vc` is repeatable. Replaces the full list (not append — same semantics as `--blocked-by`).

### 7. Audit integration

`tkt audit` reports on evidence quality:
```
Low-evidence closures: 2/12
  09  3 criteria, closed with --force (no evidence)
  11  2 criteria, 1 evidence provided (gap: criterion 2)
```

## Acceptance criteria

- [ ] `validation_criteria` field parsed and preserved in frontmatter
- [ ] `tkt new --vc "..."` sets criteria (repeatable flag)
- [ ] `tkt edit --vc "..."` replaces criteria list
- [ ] `--evidence` flag on close, positional by default
- [ ] Named evidence (`N=...`) supported, mixed with positional
- [ ] Count mismatch behavior respects config (false/warn/true)
- [ ] `require_validation_criteria` config: false (default) / true
- [ ] `require_validation_evidence` config: "warn" (default) / false / true
- [ ] `--force` bypasses all gates (with audit trail)
- [ ] Resolution section records criterion + evidence pairs
- [ ] `tkt audit` flags low-evidence closures
- [ ] Backward compatible: tickets without VC close normally
- [ ] Our .tickets/config.toml set to require both

# Agent close confirmation: formalize validation_criteria field + y/n gate

## What to build

TBD

## Acceptance criteria

- [ ] TBD

---
id: "59"
title: "Resolve color/symbol spec vs implementation (F9)"
status: in_progress
blocked_by: []
---

# Resolve color/symbol spec vs implementation (F9)

## Origin

Review ticket #38, finding F9.

## Problem

Ticket #31 specifies:
- "✓ (green) for success, ✗ (red) for domain errors, ⚠ (yellow) for warnings"
- "Respect NO_COLOR=1 — degrade to plain ✓/✗/⚠ without ANSI codes"

Implementation emits plain UTF-8 glyphs with no color. The `NO_COLOR` AC passes vacuously. Additionally, ✓/✗/⚠/→ are non-ASCII — legacy Windows consoles may mangle them.

## What to build

Research and implement **graceful, optional** color support:

### Approach: opt-in color, safe defaults

1. **Default: plain UTF-8 glyphs** (current behavior) — no ANSI codes, works everywhere
2. **Opt-in color** via `--color=always` flag or `TKT_COLOR=1` env var
3. **Auto-detect** when stdout is a tty AND `NO_COLOR` is unset → color on
4. **`NO_COLOR=1`** or `--color=never` → plain glyphs (current behavior)
5. **ASCII fallback** via `TKT_ASCII=1` for legacy terminals that mangle Unicode: `✓→[ok]`, `✗→[err]`, `⚠→[warn]`

### Research needed

- What crate to use? (`owo-colors`, `colored`, `yansi` — prefer zero-alloc, respects NO_COLOR natively)
- Does the chosen crate handle tty detection or do we need `atty`/`is-terminal`?
- How does `cargo` handle this? (good reference for a single-binary CLI)
- Windows console: do modern terminals (Windows Terminal, VS Code) handle UTF-8 + ANSI fine? Is the legacy concern still real?

### Dependency budget

tkt currently has minimal deps. Adding a color crate is acceptable if it's small and well-maintained. Prefer one that handles NO_COLOR + tty detection in one package.

Related: the error prefix changed from `tkt: <msg>` to `✗ <msg>`, dropping program identification from stderr. Consider keeping `tkt:` prefix for pipeline diagnostics.

## Acceptance criteria

- [ ] Decision documented (implement color OR drop requirement)
- [ ] Color crate selected with rationale (research output)
- [ ] Color active on tty when NO_COLOR unset; off otherwise
- [ ] `--color=always|never|auto` flag (matches cargo/git convention)
- [ ] `NO_COLOR=1` disables (https://no-color.org/)
- [ ] `TKT_ASCII=1` degrades to ASCII symbols (optional, stretch)
- [ ] Error output includes program name for pipeline diagnostics
- [ ] Behavior consistent across commands
- [ ] No color in `--json` output regardless of settings

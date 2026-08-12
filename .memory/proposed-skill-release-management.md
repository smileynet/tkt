# Release Management

Manage the full release lifecycle: version decisions, changelog curation, gated release execution, and post-release verification. Use when cutting a release, deciding version bumps, writing changelog entries, or troubleshooting release failures.

Trigger: release, version bump, changelog, ship, publish, tag, cut a release, what version, prepare release, release failed.

## Tools

| Tool | Role | Install |
|------|------|---------|
| cargo-release | Version bump + tag + push orchestrator | `cargo install cargo-release` |
| git-cliff | Changelog generation from conventional commits | `cargo install git-cliff` |
| cargo-dist | Cross-platform binary builds on tag push | `cargo install cargo-dist` |

## Release Flow

### 1. Decide the version bump

| Change type | Bump | Example |
|-------------|------|---------|
| Breaking CLI change (remove flag, rename command, change exit code, break JSON schema) | Major | `tkt edit` → `tkt modify` |
| New feature (command, flag, config key) backward-compatible | Minor | Add `tkt blocked` |
| Bug fix, perf improvement, docs | Patch | Fix crash on empty corpus |

Pre-1.0: treat minor as "new features" and patch as "fixes". Major = "interface contract broken."

### 2. Gate check

Before any release, all gates must pass:

```bash
cargo fmt --check        # no formatting drift
cargo clippy --all-targets -- -D warnings  # zero warnings
cargo test               # all tests pass
```

If any gate fails, fix first. Never release red.

### 3. Preview (dry-run)

```bash
cargo release patch          # dry-run by default
# or: mise run release -- patch
```

Verify: correct version, changelog looks right, tag name correct.

### 4. Execute

```bash
cargo release patch --execute
# or: mise run release -- patch --execute
```

This will:
1. Bump version in Cargo.toml
2. Run git-cliff → update CHANGELOG.md
3. Commit: "chore(release): vX.Y.Z"
4. Create tag: vX.Y.Z
5. Push commit + tag to remote
6. (If publish=true) cargo publish to crates.io

### 5. Verify

- GitHub Actions triggered by tag → builds binaries → creates GitHub Release
- Check: GitHub Releases page has artifacts for all platforms
- Check: `cargo install <crate>` works (if published to crates.io)

## Changelog Discipline

Every changelog entry must pass these tests:

### The user-value test
> "Would a user of this tool care about this change?"

If no → don't include it. Internal refactoring, test changes, CI config, dev dependency bumps are invisible to users.

### The technology-replacement test
> "Would this entry be true regardless of implementation?"

"Reduce page load time by 60%" ✓. "Refactor auth to use middleware pattern" ✗.

### The file-naming test
> "Does this entry name a source file, function, or internal module?"

If yes → rewrite to describe the user-visible effect instead.

### Entry format

- **One line per entry.** If it takes two lines to explain, you're describing mechanics not value.
- Start with what changed, not how: "`tkt doctor` command to verify your setup is correct"
- No sub-bullets, no indented details, no implementation notes
- No config key names, internal module names, or technical jargon
- Specific enough to be useful: "Fix crash when closing tickets with no AC section" not "Fix bug"
- Ask: "Would someone who uses tkt but doesn't build it understand this?" If no, rewrite.

### Six categories only

| Category | What belongs |
|----------|-------------|
| Added | New features users can use |
| Changed | Altered behavior (mark breaking with **Breaking:**) |
| Deprecated | Features announced for future removal |
| Removed | Features removed this release |
| Fixed | Bug fixes |
| Security | Vulnerability patches (CVE when applicable) |

Never: "Dependencies", "Internal", "Performance" (describe the user effect under the right category).

## Configuration Files

### cliff.toml (git-cliff)

Controls how conventional commits map to changelog categories. Key sections:
- `[git].commit_parsers`: regex → group (feat→Added, fix→Fixed)
- Skip patterns: `chore(release)`, `chore(tickets)`, `chore(deps)`, `test`
- `protect_breaking_commits = true`: never skip breaking changes

### release.toml (cargo-release)

Controls the release orchestration:
- `allow-branch`: only release from main
- `pre-release-hook`: run git-cliff before committing
- `tag-prefix = "v"`: standard Rust ecosystem convention
- `push = true`: push immediately (push = done)

### mise.toml

Task runner entry point: `mise run release` gates on fmt+clippy+test before calling cargo-release.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| Tag already exists | `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z` (only if not published) |
| Version mismatch | Tag must exactly match Cargo.toml version |
| Push rejected | Someone else pushed; pull, rebase, re-tag |
| cargo publish fails | Check: token valid? email verified? dry-run passes? |
| CI didn't trigger | Check tag pattern in workflow: `v[0-9].*` |

## Anti-patterns

- Never delete a published tag (fix forward with a patch release)
- Never include commit log dumps in CHANGELOG
- Never release without running the gate check
- Never batch unrelated changes into one version bump
- Never use "misc improvements" or "various fixes" as entries

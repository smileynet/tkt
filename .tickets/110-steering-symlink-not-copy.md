---
id: "110"
title: "deploy-skills.sh must symlink steering, not copy (crew prune deletes copies)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "deploy-skills.sh symlinks steering/*.md into ~/.kiro/steering/"
  - "a crew-research deploy reports 'kept (symlink): frontier-work.md' and the file survives"
  - "kiro-cli still loads the steering content through the symlink"
---

# deploy-skills.sh must symlink steering, not copy

## Problem

Ticket 100 deployed tkt's `frontier-work.md` steering by **copying** it into
`~/.kiro/steering/` ("copy, not symlink — steering is body-only"). That decision
collides with crew-research's deploy contract, which nobody cross-checked at the time.

crew-research's `init.sh` prunes the shared `~/.kiro/steering/` directory on every
deploy: it deletes any regular `.md` file it did not deploy itself, but **preserves
symlinks** (init.sh line ~275: `[[ -L "$f" ]] && kept`). Because tkt copies its steering
as a regular file, every crew-research deploy deletes `frontier-work.md`. It only
reappears when tkt's deploy runs again — so on any machine that runs both tools, the
frontier-work steering silently vanishes until the next `mise run deploy` in tkt.

Observed 2026-08-18 on a corp machine: after a crew-research full-tier deploy, doctor
reported `⚠️ unmanaged steering file: frontier-work.md — next deploy will PRUNE it`.

This is the same convention archwright and recall already follow — they symlink their
self-deployed content specifically so crew's prune keeps it. tkt is the outlier.

## What to build

Change `tools/deploy-skills.sh` steering deployment from `cp` to `ln -sf` (matching how
it already deploys the skill directory). One loop, two lines.

## Acceptance criteria

- [x] `deploy-skills.sh` symlinks `steering/*.md` into `~/.kiro/steering/` (was `cp`)
- [x] `--dry-run` says "would symlink" for steering
- [x] A crew-research deploy afterward reports `kept (symlink): frontier-work.md` and the
      file survives the prune
- [x] kiro-cli reads the steering content through the symlink (resolves to repo source)
- [x] Comment in the script explains WHY (crew prune contract) so it is not reverted

## Resolution (2026-08-18)

Changed the steering loop from `cp` to `rm -f` + `ln -sf`, mirroring the skill-symlink
loop. Added a comment citing crew's prune contract so a future edit does not revert it.

### Verification
1. ✓ symlinks steering — `deploy-skills.sh --dry-run` shows "would symlink → ~/.kiro/steering/frontier-work.md"
2. ✓ survives crew prune — crew-research `mise run init` reported "kept (symlink): frontier-work.md"; file present afterward
3. ✓ kiro loads through link — `readlink -f` resolves to `~/code/tkt/steering/frontier-work.md`; `head` shows real content
4. ✓ doctor clean — crew doctor no longer reports the unmanaged-steering warning

## Out of scope

- Auto-running tkt's deploy after crew's deploy (ordering is the user's; symlink makes order irrelevant)
- codex/crush steering (tkt only deploys steering to kiro — steering is a kiro-cli concept)

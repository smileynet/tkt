---
id: "37"
title: "research skill distribution: reference copies vs deployment source"
status: open
blocked_by: []
---

# Research skill distribution: reference copies vs deployment source

## Problem

Global skills are deployed through crew-research (the canonical source), but project-specific context (like tkt's exact command set and workflow patterns) lives in this repo. We need a pattern for:
- Keeping a reference copy of tkt-related skills here (for relevance and updating)
- Deploying the canonical version through crew-research
- Syncing changes between the two without manual coordination

## Research questions

1. **How does Gas Town's beads project manage distributed artifacts?** (reference copies, deployment source, sync mechanism)
2. **What's the crew-research pattern for project-contributed skills?** (does a project author a skill locally then PR it to crew-research? or does crew-research own all skills?)
3. **Should tkt own a `.skills/` directory** with reference copies that crew-research pulls from?
4. **What's the update lifecycle?** When tkt adds a new command, how does the skill get updated and redeployed?

## Proposed patterns to evaluate

| Pattern | How it works | Pros | Cons |
|---------|-------------|------|------|
| **Source-of-truth in crew-research** | tkt PRs skill changes to crew-research; no local copy | Single source | Friction to update, disconnected from implementation |
| **Source-of-truth in tkt** | tkt owns `.skills/tkt-workflow/SKILL.md`; crew-research pulls/symlinks | Close to implementation | Distribution requires sync |
| **Dual maintenance** | Both repos have copies; manual sync | Independence | Drift risk |
| **Template/generator** | tkt exports skill content via `tkt capabilities --json`; crew-research generates from it | Always fresh | Complex machinery |

## Acceptance criteria

- [ ] Research Gas Town beads and other prior art for artifact distribution
- [ ] Decide: where does the tkt-workflow skill live canonically?
- [ ] Document the chosen pattern in an ADR
- [ ] Implement the chosen sync mechanism (if any)

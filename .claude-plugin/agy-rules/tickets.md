---
trigger: glob
globs: ".tickets/*.md"
---

# Ticket status rule

Never hand-edit `status: done` in `.tickets/*.md` files. Hand-editing skips close gates (validation criteria, evidence, and `docs/plan.md` synchronization). Always close tickets with `tkt close <id> --check-all --evidence "..." --resolution "..."`.

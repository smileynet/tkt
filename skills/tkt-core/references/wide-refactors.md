# Wide Refactors — Expand-Contract Sequencing

When a change touches many files (rename a column, retype a shared symbol, migrate an API), vertical slicing doesn't work — each slice would break everything else.

Use **expand-contract** instead:

## Pattern

```
1. EXPAND  — add the new form beside the old (nothing breaks)
2. MIGRATE — move callers in batches (each batch its own ticket)
3. CONTRACT — delete old form once no callers remain
```

## Ticket Structure

```
01-expand-add-new-type     → introduces NewType alongside OldType
02-migrate-package-auth    → auth/ callers switch to NewType
03-migrate-package-api     → api/ callers switch to NewType  [P]
04-migrate-package-core    → core/ callers switch to NewType [P]
05-contract-remove-old     → delete OldType (blocked by 02, 03, 04)
```

## Rules

- Expand ticket is ALWAYS first (creates the new form without breaking anything)
- Contract ticket is ALWAYS last (removes old form, blocked by all migrations)
- Migration tickets are parallelizable (`[P]`) — each handles one package/directory
- Each migration ticket must leave the codebase GREEN (all tests pass)
- If even batches can't stay green alone: use an integration branch with a final verify ticket

## When to Use

- Renaming a type/function used in 10+ files
- Changing a shared interface's shape
- Migrating from one dependency to another
- Database schema migrations with application code changes

## When NOT to Use

- Changes that only touch 1-3 files → just do it in one ticket
- New features (no "old form" exists) → use vertical slices

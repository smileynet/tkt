# Parents, invocation ownership and label namespaces

These optional fields and project settings extend existing tickets. Tickets
without a parent, assignee or declared label namespaces remain valid. Parent
hierarchy is independent of `blocked_by`; neither replaces the other.

In a new Git repository, run `tkt init`, configure a Git author and use
`tkt config --project --set push.enabled=false` for a local-only walkthrough.
Add this declaration to `.tickets/config.toml`:

```toml
[labels.triage]
single_select = true
values = ["needs-info", "ready"]
```

The values array must be on one line, double-quoted and without a trailing comma.
This is the supported narrow configuration syntax, not general TOML parsing.
Declare the work, replacing body/evidence text with the project's actual scope:

```text
tkt new effort --title "Delivery effort" --body @parent-body.md --validation "Effort accepted"
tkt new child --title "Deliver one child" --parent 01 --tags triage:ready --body @child-body.md --validation "Child delivery verified"
tkt ready
tkt --agent codex claim 02
tkt query --parent 01
tkt --agent codex close 02 --check-all --resolution "Child accepted independently" --evidence "Describe the actual child verification"
tkt children 01 -o json
```

Create both referenced body files before running `new`, with the specific scope
and acceptance for each. For example, `child-body.md` can start with:

```markdown
## What to build

Deliver the child feature described by the project specification.

## Acceptance criteria

- [ ] The specified child behavior is verified.
```

Fill the parent's body with its own effort-level acceptance. IDs `01` and `02`
assume an empty ticket corpus. `ready` shows the child,
not the parent container. Claim records `assignee: "codex"`; it does not change
agent eligibility. Other invocation identities cannot claim that owned ticket
without the explicit scheduling override `--force`. Tags scope work streams;
`requires` remains all-of machine capability matching.

After closing the child, the parent reports one done child out of one total and
retains its own stored status. Total includes cancelled children, reported
separately; cancellation is not done. Parent progress counts direct children,
including those outside the active tag context. Close the parent explicitly
after independently verifying its own criteria and evidence.

Validation rejects undeclared `triage` values and multiple `triage` tags on one
ticket, even though a normal tag edit can store them. Run `tkt validate --strict
--brief` to check policy. Undeclared namespaces and ordinary tags remain free-form.
`tkt capabilities -o json` describes `ticket_relations`, label namespace syntax
under `config.label_namespaces`, and each command's flags. The six instruction
targets share these semantics; `tkt init --agent-only --target TARGET` writes its
target file, while plain `tkt init --agent-only` prints the default snippet.

For a session working one map, use `tkt ready --parent 01` instead of the global
frontier. It selects direct ready children, preserves all normal eligibility and
ordering rules, and intersects active tag context. This is native-parent-only;
older epic-tag groups retain normal tag-context filtering without a new alias.

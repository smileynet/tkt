# Assess ticket migration support

Use this reference when ticket files are foreign, detection is uncertain, or the
requested source has no built-in adapter. First inspect the installed interface:

```sh
tkt --version
tkt capabilities -o json
tkt migrate --help
```

The current built-in converter is `tk`. There is no executable custom-map engine:
`--map`, `--generate-map` and `--apply` are not supported migration flags. A proposed
mapping document records analysis; it does not configure the converter. If local
help/capabilities differ from these examples, resolve that mismatch before writes.
No repository checkout or additional workflow package is needed for this process.

## Inspect the source before choosing a converter

Read representative source files and inventory the complete source directory.
Record its format/version, filename and ID conventions, status values, dependency
references, title/body representation and fields that must survive. Distinguish
observed values from assumptions. Preserve originals and identify the intended
migration scope before changing files.

- Native tkt files already use `id`, `title`, `status` and `blocked_by`; do not
  force a native or mixed corpus through a foreign converter.
- The tk adapter uses source filename order for new numeric IDs, extracts titles
  from body headings, remaps `deps`, and translates known status/priority values.
  Review the actual preview rather than assuming a custom policy was applied.
- Unknown tk status values currently become `open`. Stop if that would erase
  required meaning, such as a different retirement or blocked state.
- The converter writes its supported fields. Unknown source frontmatter is not
  automatically retained in converted tickets, although backups preserve original
  bytes. Record unsupported information and stop if the reviewed conversion would
  lose required meaning. There is no configurable preserve/drop map.
- Current tkt has invocation `assignee`, agent eligibility, parent and tags. External
  people/assignees are not automatically invocation identities; source labels are
  not automatically machine capabilities or scheduling policy. Inspect current
  capabilities before proposing those mappings.

## Supported tk conversion

```sh
tkt migrate --detect
tkt migrate --from tk --dry-run
tkt migrate --from tk --confirm
```

Detection is a signal, not proof that every source field is supported. Preview
first: the preview displays filename-to-ID/title mappings and orphan warnings.
It does not print mapped status, priority, dependencies or dropped frontmatter.
Manually compare those source fields with the documented conversion rules and
inspect the resulting files after conversion. Use `--confirm`
only within the authorized migration scope after that review. It is the CLI's
explicit apply action; a map document does not authorize additional transformations.

Preview writes no marker, backup or converted ticket. Preparation rejects unreadable
or changed source snapshots, existing destinations and conflicting backup data.
Backups remain in `.tickets.bak/`; matching complete backups can be reused, while
different bytes are preserved as a conflict. New files are published completely
without overwriting arrivals, and originals are removed only after successful
publication. The filesystem must support hard links for new-file publication.

This is per-file safety, not a batch transaction or external-editor lock. Coordinate
writers, retain backups and inspect reported paths if a later operation fails.
Earlier complete outputs/backups may remain. Do not blindly delete a conflicting
backup or foreign destination to make the command succeed.

After conversion, inspect the resulting files and run:

```sh
tkt validate
tkt query --all-context
```

Read findings and skip diagnostics as well as exit status. Check source-specific
information that validation cannot infer. Do not claim Python parity, a generic
adapter or completed native evidence merely because converted fields parse.

## Unsupported sources or custom policy

Keep analysis separate from executable support. Record the source/version,
representative inputs, identity/replay policy, lifecycle and relation mappings,
unsupported values, preservation strategy and verification cases in a proposed
adapter brief. Mark it unimplemented until an actual command is available and
verified. Do not invent flags, apply a design file as configuration, silently drop
fields, or substitute hand-edited lifecycle completion for a supported conversion.

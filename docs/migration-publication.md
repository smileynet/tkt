# Migration publication and recovery

A tk migration prepares readable originals and complete converted text before any
backup or ticket write. Preview validates the same source, destination and backup
preconditions without creating a review marker or directory. Apply rechecks the
prepared snapshots before effects.

New outputs and backups use the shared complete-file, no-overwrite publication
primitive. Existing destinations and different old backup bytes are conflicts;
matching complete backups can be reused after an interrupted attempt. Sources are
removed only after their converted output is published successfully. Sources,
backups and the backup root must be regular, unlinked files/directories; malformed
or changed inputs are rejected instead of turning a read failure into empty text.

The existing tk command remains `tkt migrate --from tk --dry-run`, followed by
`tkt migrate --from tk --confirm` after review. Backups remain in `.tickets.bak/`.
No Git commit or push is added. 
This is per-file safety, not an atomic batch transaction. Earlier converted outputs
and backups can remain if a later operation fails; inspect the reported paths and
retain the backup before cleanup or retry. Coordinate external writers and
namespace changes: snapshot checks are not an editor lock. No-overwrite creation
requires hard-link support and uses inert `.tkt-create-*` staging files; unsupported
filesystems fail explicitly. Existing replacement still uses sibling-temp/rename.


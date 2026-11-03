# Lifecycle, identifier and config-input repairs

Independent critic accepted implementations and the final guidance/capability
delta with no blocking findings. Separate workers implemented #241 and #164;
the coordinator implemented #140.

`mise run check`: pass. Rust 322 tests passed, none failed/ignored. Node 42
passed, one existing Windows relative-link skip, no failures. Formatting clean;
Clippy zero warnings. Git fixture LF/CRLF notices are informational.

## #241

Before repair, independently executed integration fixtures confirmed edit, new
and batch each accepted done and wrote it without close gates. All three now
return structured Validation with a close hint before mutation. Three public
integration tests cover normal/dry-run rejection, ticket bytes and index/HEAD
preservation, compatible statuses, failed close gates and successful close with
Resolution. Creation still supports in_progress; capabilities now documents it
alongside open/backlog. All generated instruction targets and workflow guidance
describe the close-only completion rule. No cancellation policy changed.

## #164

The exact-stem slug-ID fixture was red before repair and green after. Two public
fixtures verify imported cr-axcq/cr-axcq.md compatibility and genuine numeric
mismatch detection. ADR 0005 records bounded existing-file validation support,
without promising new allocation or universal mutation support. The named
historical project trees are absent under D:/code; catalyst has no ticket corpus.
Their complete current corpuses were not executed. Acceptance concerns removal
of this mismatch rule's false positive, not absence of unrelated findings.

## #140

Malformed config --set returned exit 2 before the change (observed red assertion).
It now returns exit 1 and the ordinary structured Validation envelope under JSON,
including dry-run. Two public tests verify missing assignment does not write a
user config and valid debug=false still writes successfully. Known-value policy
validation remains #190's separate scope. Existing JSON errors can include human
diagnostics before the final envelope; that output contract was preserved.

Temporary fixtures self-clean. Gate log results are promoted here; the shared
pre-existing target cache remains for subsequent work. Windows local execution
does not establish other-platform execution or dashboard rendering.

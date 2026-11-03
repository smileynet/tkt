# ADR 0005: Validate imported exact-stem identifiers

- Status: accepted
- Date: 2026-11-02
- Source: ticket #164

## Context

Imported and hand-authored corpuses use identifiers such as `cr-axcq` in
`cr-axcq.md`. Their identifiers already match their filenames, but validation
previously required the numeric-prefix naming convention and reported a mismatch.
Renumbering these cited identifiers would break references.

## Decision

Validation accepts an identifier equal to the complete filename stem, alongside
the existing numeric-prefix convention. Genuine mismatches such as ID `04` in
`07-example.md` continue to be findings. This is a bounded compatibility rule
for existing files, not a new identifier allocator or a promise that every mutation
supports every imported identifier scheme. Numeric allocation, renumbering and
dependency-reference rules remain governed by their existing contracts.

## Evidence and limits

Public CLI fixtures exercise exact-stem imported IDs and real numeric mismatches.
The historical named project directories were unavailable in this checkout's
Windows workspace; no claim is made about their complete corpuses or unrelated
validation findings.

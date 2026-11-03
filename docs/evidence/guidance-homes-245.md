# Prepared guidance homes and frozen coverage

Frozen source is AGENTS.md at `9e944bda0dbc79361976907730564b969aa033d0`.
All 355 source lines partition into 45 units in the JSON/Markdown coverage map.
The source runbook is unchanged in this stage. Plugin, architecture, ticket/CLI
contract, Windows and maintainer-tooling references preserve their assigned
source text; the review-dispatch trap is in session operations. Only relative
Markdown links were rebased. Temporary duplication ends in ticket 246.

- Pass: `node tools/check-guidance-coverage.mjs`: 45 units cover 355 lines;
  frozen source/unit hashes, exhaustive partition, home text, anchors and local
  links pass. `node --check` passes.
- Pass: removing a mapped architecture rule in a controlled temporary mutation
  makes the checker exit 1 with Missing rule text; restoring the exact bytes
  returns to pass. No damaged reference was committed.
- Pass: `git diff 9e944bda -- AGENTS.md` is empty; scoped `git diff --check` and
  deployment precheck pass. No Rust/product code changed or rebuild was needed.
- Independent critic exhaustively accepted the prepared scope, including full
  local verification, commit/push/deploy cadence, mutation safety, open-ticket
  release restriction and separate publication authority.
- Accepted SHA-256 identities: coverage JSON
  C23C7F072D8A886F2E20D23ACBE66F3AB0154F341A10A5E260DD72B354E3F6B3;
  checker AB2C6BD9EC597B832481CFC6E71F1380A88DBE33B7B709B792675A7C6BF3A974;
  architecture 9C5053BD439B4E94BD9BEB06F7EB561DA142F6F0AD42D8C6E21CC530C74AB50C;
  contract A0FBFC90346126425FBD82C78C2B905FCEFE5B51CB121B608EF910AE766A2EB3.

An intentional home refinement groups cargo-dist/generated-workflow traps with
other maintainer-tooling rules, rather than copying them into release-staging.
Release guidance/staging documents still control workflow mechanics and authority;
ticket 246 will make the cross-pointer explicit. The critic accepted this as a
nonblocking refinement. The human map is reviewed here; ticket 246 can add a
mechanical JSON-to-table consistency check without changing the approved rules.

Prepared references and coverage/checker are promoted; disposable preparation and
verification helpers go to OS trash. No build output was created. Existing shared
target cache remains for the final CLI installation; unknown-owned scratch stays.

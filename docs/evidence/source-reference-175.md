# Source-reference advisory verification

Open/in-progress tickets without a nonempty spec, prerequisite, parent or prose citation/documentation/HTTP(S) link receive info/no-source-reference. Strict mode does not promote it; validate.require_source_reference=true explicitly makes it an error. Backlog and terminal tickets are exempt. Target integrity is checked separately. Shared fence-aware prose iteration preserves renumber citation behavior; inline documentation paths remain usable.

PASS: final mise run check ran 403 Rust tests and 42 Node fixtures, with clean formatting and zero clippy warnings. SKIP: one existing Windows relative-link fixture. ASSERTION FAILURE, repaired: the first test builder removed required blocked_by fields. The corrected fixture retained empty dependencies and the final full gate passed.

The public regression covers eight reference alternatives, default strict-mode advice, the explicit error policy, fenced examples and lifecycle exclusions. Independent critic review accepted the final source and corrected fixture. Guidance coverage and diff checks passed. No frontier code changed and the new policy was not enabled here.

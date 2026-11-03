# Source-reference advisory verification

Open/in-progress tickets lacking a spec, prerequisite, parent or prose citation/documentation/HTTP(S) link receive info/no-source-reference. It remains advisory under strict mode; explicit validate.require_source_reference=true makes it error. Backlog/terminal tickets are exempt. Source existence/relationship validity remains checked separately. Shared fenced prose iteration preserves existing renumber/citation behavior; inline documentation paths remain usable.

PASS: final mise run check: 403 Rust tests,42 Node fixtures, formatter clean and clippy zero warnings. SKIP: one existing Windows relative-link fixture. ASSERTION FAILURE, repaired: first integration fixture removed required blocked_by; builder corrected, final full gate rerun passed. No product failure was hidden. PASS: independent critic accepted final product source and corrected integration hash49B45E36…8A83B5C1. PASS: guidance coverage45units/355frozenlines and git diff --check.

Public regression covers eight union members, default strict advisory, explicit hard policy, fenced examples and lifecycle exclusions. No frontier membership or sort code changed. The config is optional and was not enabled in this repository.

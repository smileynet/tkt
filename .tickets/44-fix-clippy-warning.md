---
id: "44"
title: "Fix clippy unnecessary_sort_by warning (F6)"
status: done
blocked_by: []
---

# Fix clippy unnecessary_sort_by warning (F6)

## Origin

Review ticket #38, finding F6.

## Problem

`src/cli.rs:1444` has `projects.sort_by(|a, b| b.1.cmp(&a.1))` which triggers `clippy::unnecessary_sort_by`. AGENTS.md requires zero clippy warnings.

Pre-existing (from 2026-07-30), possibly surfaced by a clippy upgrade.

## What to build

Replace with `projects.sort_by_key(|p| std::cmp::Reverse(p.1))`.

## Acceptance criteria

- [x] `cargo clippy --all-targets` produces 0 warnings
- [x] Sort behavior unchanged (descending by line count)

## Resolution (2026-08-05)

Replaced `projects.sort_by(|a, b| b.1.cmp(&a.1))` with `projects.sort_by_key(|p| std::cmp::Reverse(p.1))`. Zero clippy warnings on `--all-targets`.

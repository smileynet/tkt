#!/bin/sh
# tkt-managed-advisory-pre-commit-v1
if command -v tkt >/dev/null 2>&1; then
    DO_NOT_TRACK=1 tkt hooks run >&2 || printf '%s\n' 'tkt advisory hook could not inspect tickets; commit continues.' >&2
else
    printf '%s\n' 'tkt advisory hook: tkt unavailable; commit continues.' >&2
fi
exit 0

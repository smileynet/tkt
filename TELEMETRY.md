# Telemetry

tkt includes optional, local-only telemetry that records command usage to help identify bugs and improve the tool. **No data leaves your machine** — telemetry is stored as local files you can inspect, export, or delete at any time.

## Status: Local-Only (v1)

Telemetry in v1 is **entirely local**. Events are written to files on your disk. There is no upload, no phone-home, no network calls. This may change in a future version, but only with explicit opt-in and a versioned consent update.

## What Is Collected

Each command invocation records one JSONL line with these fields:

| Field | Example | Purpose |
|-------|---------|---------|
| `ts` | `2026-07-30T10:00:01Z` | When the command ran (second precision) |
| `session` | `019fb35f8242-de38` | Groups all events from one CLI invocation |
| `project` | `tkt` | Which project (repo directory name) |
| `cmd` | `ready` | Which subcommand was run |
| `exit_code` | `0` | Whether it succeeded (0), failed (1), or crashed (2) |
| `duration_ms` | `150` | How long it took |
| `version` | `0.1.0` | Which tkt version |
| `os` | `windows` | Operating system |
| `arch` | `x86_64` | CPU architecture |

### Example event

```json
{"ts":"2026-07-30T10:00:01Z","session":"019fb35f8242-de38","project":"tkt","cmd":"ready","exit_code":0,"duration_ms":150,"version":"0.1.0","os":"windows","arch":"x86_64"}
```

## What Is Never Collected

- File paths or directory names (beyond the repo root basename used as project slug)
- Ticket content, titles, or IDs
- Git remote URLs, branch names, or commit hashes
- Environment variable values
- Command argument values (only the subcommand name)
- Full error messages or stack traces
- IP addresses or network identifiers
- Any data that could identify you personally

## Where Data Is Stored

| Platform | Path |
|----------|------|
| **Windows** | `%APPDATA%\tkt\telemetry\` |
| **macOS** | `~/Library/Application Support/tkt/telemetry/` |
| **Linux** | `~/.local/share/tkt/telemetry/` |

Files are named by project: `{project-slug}.jsonl` (one file per project).

### Consent file

| Platform | Path |
|----------|------|
| **Windows** | `%APPDATA%\tkt\consent.toml` |
| **macOS** | `~/Library/Application Support/tkt/consent.toml` |
| **Linux** | `~/.config/tkt/consent.toml` |

## How to Opt In / Out

Telemetry is **disabled by default**. You must explicitly opt in.

### Enable

```bash
tkt telemetry --enable
```

### Disable

```bash
tkt telemetry --disable
```

### Environment variables (override config file)

| Variable | Effect |
|----------|--------|
| `DO_NOT_TRACK=1` | Disables telemetry (universal standard) |
| `TKT_TELEMETRY=off` | Disables telemetry |
| `TKT_TELEMETRY=on` | Enables telemetry |
| `CI=true` | Disables telemetry (CI environments) |

Priority: `DO_NOT_TRACK` > `TKT_TELEMETRY` > `CI` > config file > default (off).

## How to Inspect Local Data

```bash
# See current status and storage summary
tkt telemetry --status

# Print recent events in human-readable format
tkt telemetry --show
```

You can also read the JSONL files directly — they're plain text, one JSON object per line.

## How to Delete Local Data

```bash
# Delete all telemetry files
tkt telemetry --clear
```

Or manually delete the telemetry directory listed above.

## Storage Limits

Telemetry files are automatically managed to prevent unbounded growth:

| Limit | Value |
|-------|-------|
| Max file size before rotation | 5 MB |
| Max rotated files per project | 5 |
| Max age for rotated files | 30 days |
| Hard cap per project | ~30 MB (5 MB × 6 files) |

Rotation happens automatically. Old rotated files are deleted on each invocation. Session boundaries are preserved during pruning — partial sessions are never created.

## Debug Mode

For real-time diagnostics (not persisted, does not require consent):

```bash
TKT_DEBUG=1 tkt ready       # Human-readable trace to stderr
TKT_DEBUG=json tkt ready    # JSONL trace to stderr
```

Debug output goes to stderr only and is never written to disk.

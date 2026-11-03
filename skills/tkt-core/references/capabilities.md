# Capabilities manifest

Run `tkt capabilities -o json` to discover the command surface as one JSON object.
The inventory and argument metadata are derived from the same Clap definitions
that produce `--help`; handler-specific rules describe the supported workflow.
No repository is needed to print the manifest.

## Structure

- `version`: the CLI package version.
- `commands`: every public command, including the built-in `help` path.
- `global_flags`: root options; each command's schema includes its inherited flags.
- `workflows`, `config`, `errors`, `output`: workflow examples, resolved config
  locations, error kinds and common output conventions.

Each command declares `description`, `effects`, `effects_description`,
`inputSchema` and `output`. `effects` is conservative across its modes:
`read_only`, `idempotent` or `non_idempotent`. For example, `validate` can write
under `--fix` and `config` can write under `--set` or `--unset`; consult
`effects_description` before treating an invocation as read-only. This describes
command behavior, not authorization to mutate files or publish changes. Shared
optional debug/telemetry/update behavior is independent of command effects.

`output.default` describes the default wire format (`text`, `json` or
`json_lines`), not a new value for the `--output` flag. That flag still accepts
`json` or `text`. Read mode flags and descriptions: audit/validate/plan findings
use JSON by default and `--brief` selects text. Query always emits JSON Lines.
Capabilities always emits one JSON object. Quiet/output combinations retain
their current command behavior; the manifest does not normalize their output.

## Argument schemas

Property keys use the long option with hyphens replaced by underscores, or the
positional argument ID: `blocked_by` maps to `--blocked-by`. Each property can
declare `flag`, `short`, `position`, `description`, `enum` and `default`.

- Arrays have `items` and `repeatable`; `value_delimiter` records comma-separated
  inputs where supported. Numeric acceptance indices have integer items.
- `validation` is the canonical property for `--validation`. The compatible `vc`
  property has `alias_of: "validation"`; aliases are also listed on the property.
  Pass one spelling, rather than duplicating the same input under both names.
- `required` lists required positional/option properties. `conflicts_with` lists
  mutually exclusive property names (for example, agents and exclude_agents).
  It is metadata, not a substitute for the CLI's full validation.
- `value_optional` marks a flag such as `init --write`: a string supplies its
  value, while boolean `true` means pass the flag without a value.
- Agent identity documents `TKT_AGENT`; the flag takes precedence over that
  environment variable. Root help/version options need not be inherited by
  every subcommand; inspect that command's properties.
- Built-in help takes an optional `command` path, as in `tkt help new`.

Schemas describe supported workflow values, including empty strings for fields
that can be cleared. Creation advertises open/backlog/in_progress. Edit advertises
open/in_progress/backlog; use `close` for done so resolution, acceptance and
evidence gates run. Generic status mutations reject done.
Project policy, existing ticket state and filesystem/Git conditions still require
runtime validation even when an input conforms to the schema.

# Configuration

English | [日本語](configuration.ja.md) | [简体中文](configuration.zh-CN.md)

ccstatuskit reads `$XDG_CONFIG_HOME/ccstatuskit/config.toml`
(`~/.config/ccstatuskit/config.toml` by default).
Set the `CCSTATUSKIT_CONFIG` environment variable to use another path.

A missing config falls back to built-in defaults. A *broken* config also
falls back to defaults — the statusline never goes blank over a
configuration mistake, but the layout resets until the TOML parses again
(a warning goes to stderr).

## Layout: the `format` template

```toml
format = """
$model $directory $git
$ctx $usage
"""
```

- `$name` and `${name}` reference builtin modules; `${custom.x}` references
  a `[custom.x]` entry (the dot requires braces).
- Every template line is a statusline row. Rows are declared, not
  positioned — there are no line numbers.
- A hidden module collapses out of its row; a row whose modules all hid
  (and whose literals are whitespace) disappears entirely.
- `\$` produces a literal dollar, `\\` a literal backslash. Unknown module
  names hide silently, so a config written for a newer version degrades
  gracefully.
- Default when unset:
  `$model $directory $memory $ctx $time $git`,
  `$package $unity $node $rust $go $python $dotnet $ruby $java $kotlin $php $swift`,
  `$usage $cost $lines $agent $pr $worktree`

## Builtin modules

| Module | Segment | Notes |
| --- | --- | --- |
| `model` | Model name with family color | Hidden when stdin has no model info |
| `directory` | Project-relative path | Falls back to the cwd basename |
| `git` | Branch, dirty/staged/untracked/renamed, conflicts, stash, merge/rebase, ahead/behind | Runs `git` with `GIT_OPTIONAL_LOCKS=0`; hidden outside a repository |
| `memory` | System RAM used/total in GB | Colored at >60% / >80% |
| `ctx` | Context-window usage percent | Colored at ≥40% / ≥60%; hidden until Claude Code reports it |
| `time` | Clock plus session duration | Duration from `cost.total_duration_ms` |
| `usage` | Claude quota windows | See below |
| `cost` | Session cost in USD | From `cost.total_cost_usd`; colored at ≥$1 / ≥$5 |
| `lines` | Lines added/removed this session (`+120 -45`) | From `cost.total_lines_added`/`total_lines_removed`; hidden when both are zero |
| `agent` | Active subagent name | From `agent.name`; hidden outside a subagent run |
| `pr` | Open PR number | From `pr.number`; colored by `pr.review_state` (approved/changes_requested/pending) |
| `worktree` | Worktree name (and branch, if it differs) | From `worktree.name`/`worktree.branch`; hidden in the primary checkout |
| `package` | The project's own declared version | starship-style: reads the `version` field from `Cargo.toml`, `package.json`, `pyproject.toml`, `composer.json`, or `pom.xml` (first match wins) |
| `unity`, `node`, `rust`, `go`, `python`, `dotnet`, `ruby`, `java`, `kotlin`, `php`, `swift` | Language icon + toolchain version | One module per language, detected independently from marker files; a repo can match more than one (`Cargo.toml` + `package.json` shows both `$rust` and `$node`). The version is the *toolchain's*, from the real binary (`rustc --version`, `go version`, `python3 --version`, …), falling back to project files (`go.mod` directive, `.python-version`, …) when the binary isn't available. Exceptions: `unity` shows the editor version, `dotnet` the target framework, and `node`'s React/Vue/Next/TypeScript variants that framework's dependency version. `dotnet` hides inside a Unity project (Unity generates its own `.csproj` files — use `$unity` there) |

Every builtin accepts two common options:

```toml
[git]
disabled = true              # remove the module entirely

[model]
style = "bold fg:#FF79C6"    # override the default color
```

## Styles and the palette

Style strings follow starship's format: whitespace-separated tokens.

- Attributes: `bold`, `dimmed`, `italic`, `underline`, `inverted`
- Colors: `fg:` / `bg:` with `#rrggbb`, a 0-255 ANSI index, or a named
  ANSI color (`black`, `red`, `green`, `yellow`, `blue`, `purple`,
  `cyan`, `white`, plus `bright-` variants; `magenta` is an alias of `purple`)

Named colors come from the `[palette]` table:

```toml
[palette]
hot = "#FF5555"

[ctx]
style = "bold fg:hot"
```

## Usage quotas

```toml
[usage]
scoped = true            # also fetch per-model quotas (unofficial API, opt-in)
only = ["fable"]         # filter which windows to show
cache_ttl = 120          # scoped cache freshness in seconds
```

Without `scoped`, the module renders the official `rate_limits` data
Claude Code pipes in (5-hour and 7-day windows, Pro/Max plans) and never
touches the network.

With `scoped = true`, per-model weekly quotas (e.g. Fable) come from the
same unofficial endpoint Claude Code itself uses. A detached
`ccstatuskit --refresh-usage` process refreshes a stale-while-revalidate
disk cache; renders only ever read the disk. **The scoped source is
unofficial and may break without notice.**

`only` entries: `"five_hour"` (aliases `"5h"`, `"session"`),
`"seven_day"` (aliases `"wk"`, `"weekly"`), or a case-insensitive
model-name substring such as `"fable"`. An empty list shows everything.

Advanced: `auto_refresh = false` disables the background refresh spawn,
and `cache_dir` overrides the cache location (defaults to
`$XDG_CACHE_HOME/ccstatuskit`).

## Custom modules

Any executable becomes a module through the
[module contract](module-contract.md):

```toml
format = "$model ${custom.pr}"

[custom.pr]
command = "jq -r '.pr.number // empty' | sed 's/^/#/'"
style = "fg:cyan"
when_dir = [".git"]      # run only when one of these exists in the project dir
# shell = ["bash", "-c"] # default: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# disabled = true
```

The child receives the raw stdin JSON plus `CCSK_*` env vars; its first
stdout line becomes the segment, and empty output hides it. A module
whose only job is a side effect (write a marker file, ping a socket) is
legal: print nothing and it never appears.

## Global options

```toml
command_timeout = 500    # wall-clock budget in ms shared by all modules
```

Set the `NO_COLOR` environment variable to strip all styling.

## Diagnosing problems

The render path hides mistakes by design. When something doesn't show
up, ask the doctor:

```sh
ccstatuskit doctor
```

It reports the resolved config path, TOML and format-template errors,
invalid styles and palette entries, unknown module names, custom
references without tables, and the scoped-usage credential/cache state.
Errors exit 1; warnings (things that merely hide) exit 0.

## Behavior on errors

| Event | Result |
| --- | --- |
| Module panics, errors, or times out | Its segment is omitted; the row survives |
| Malformed stdin JSON | Modules render what they can; exit code stays 0 |
| Broken config TOML | Defaults are used; warning on stderr |
| Unknown module name in `format` | Hidden silently |

## Full example

```toml
# Row 1: session state / Row 2: Claude info / Row 3: dev environment.
# Row 3 disappears automatically outside recognized projects.
format = """
$directory $memory $time $git
$model $ctx $usage
$rust $unity ${custom.unilyze}
"""

[usage]
scoped = true

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]
```

Changes take effect on the next render — every render is a fresh
process, so there is nothing to restart.

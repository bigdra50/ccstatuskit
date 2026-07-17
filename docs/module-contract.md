# External Module Contract (v1)

English | [日本語](module-contract.ja.md) | [简体中文](module-contract.zh-CN.md)

A ccstatuskit module is any executable. Declare it under `[custom.<name>]`
and place `${custom.<name>}` in your `format` template.

```toml
format = "$model $git ${custom.unilyze}"

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]          # run only when one of these exists in the project dir
# shell = ["bash", "-c"]       # default: ["sh", "-c"] (unix) / ["cmd", "/C"] (windows)
# style = "bold fg:#A6E22E"    # optional; wraps plain-text output
# disabled = true
```

## Input

| Channel | Content |
| --- | --- |
| stdin | The Claude Code statusline JSON, byte-for-byte as ccstatuskit received it. Fields you don't know about may appear; ignore them. |
| `CCSK_CONTRACT` | Contract version, currently `1`. Bump = breaking change. |
| `CCSK_MODEL` | Model display name (e.g. `Opus`). Unset if unknown. |
| `CCSK_MODEL_ID` | Model id (e.g. `claude-opus-4-8`). |
| `CCSK_CWD` | Claude Code's current working directory. |
| `CCSK_CURRENT_DIR` | `workspace.current_dir`. |
| `CCSK_PROJECT_DIR` | `workspace.project_dir`. |
| `CCSK_SESSION_ID` | Stable per session. |
| `CCSK_TRANSCRIPT_PATH` | Path to the session transcript JSONL. |
| `CCSK_VERSION` | Claude Code version. |
| `CCSK_CTX_PCT` | Context window usage, integer percent. |

Every `CCSK_*` var is set only when the underlying JSON field is present.
Prefer the env vars for simple modules; parse stdin when you need more.

## Output

- **Your first stdout line becomes the segment.** Further lines are ignored.
- **Empty output means "hide me".** This is the idiomatic way to be
  conditional — print nothing when you have nothing to say.
- ANSI escape codes are allowed. If the config sets `style`, ccstatuskit
  wraps your (plain) output in it; don't combine both.
- Write UTF-8. Keep it to one short segment — this is a statusline.

## Failure semantics (fail-soft)

| Event | Result |
| --- | --- |
| Nonzero exit | Segment hidden (stdout discarded) |
| No output within `command_timeout` (default 250 ms) | Process killed, segment hidden |
| Crash / unspawnable command | Segment hidden |

A module can lose its segment; it can never break the statusline or leak a
process past the deadline. If your data source is slow, cache it yourself
(stale-while-revalidate works well) and return the cached value fast.

## Compatibility rules

1. stdin JSON is passed through untouched — schema drift in Claude Code
   reaches you unfiltered. Parse defensively.
2. New `CCSK_*` vars may be added within contract v1; existing ones will not
   change meaning or disappear.
3. `CCSK_CONTRACT` increments on any breaking change to the rules above.

## Example: a minimal bash module

```bash
#!/usr/bin/env bash
# Shows the PR number when the session has an open PR.
number=$(jq -r '.pr.number // empty' 2>/dev/null)   # reads stdin
[ -n "$number" ] && printf '#%s' "$number"           # empty output = hidden
```

Any language works the same way: read stdin or `CCSK_*`, print one line.

## Testing your module

Run your command through the built-in harness to see exactly what a
render would do with it:

```sh
ccstatuskit test-module './my-module.sh'
```

It executes two scenarios (a full context and an empty context) with the
JSON on stdin and the `CCSK_*` env set, then reports the resulting
segment (or why it hid), the timing against the render budget, and any
stderr — which a real render discards.

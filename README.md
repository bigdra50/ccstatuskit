# ccstatuskit

[![CI](https://github.com/bigdra50/ccstatuskit/actions/workflows/ci.yml/badge.svg)](https://github.com/bigdra50/ccstatuskit/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ccstatuskit.svg)](https://crates.io/crates/ccstatuskit)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange.svg)](Cargo.toml)

English | [日本語](README.ja.md) | [简体中文](README.zh-CN.md)

A modular, starship-inspired statusline kit for [Claude Code](https://code.claude.com).

Compose your statusline from builtin modules and external commands, laid out with a TOML template. One Rust binary, no runtime dependencies.

![ccstatuskit rendering a three-row statusline in a Rust repo and in a Unity project](https://raw.githubusercontent.com/bigdra50/ccstatuskit/main/assets/statusline.png)

```toml
# ~/.config/ccstatuskit/config.toml
format = """
$directory $memory $time $git
$model $ctx $usage
$rust $unity ${custom.unilyze}
"""

[usage]
scoped = true
only = ["fable"]          # show just "Fable 14%" — pick exactly what you want

[custom.unilyze]
command = "unilyze statusline"
when_dir = ["Assets"]     # Unity projects only
```

## Why

Claude Code runs exactly one statusline command. Every statusline out there is a monolith: to adopt someone's usage meter you adopt their entire script. ccstatuskit splits the problem:

- **Builtin modules** run in-process and in parallel, each with its own timeout.
- **A module contract** (`[custom.x]`) runs *any executable* as a module — stdin gets the Claude Code JSON, `CCSK_*` env vars carry the parsed basics, the first stdout line becomes a segment, empty output hides it. Write modules in bash, Python, anything. See [docs/module-contract.md](docs/module-contract.md).
- **Fail-soft everywhere**: a broken or slow module drops its segment, never the line. Malformed input, broken config, schema drift — the statusline always renders and always exits 0.

Rows are declared as template lines, not absolute positions: modules that hide collapse away, and a row whose modules all hid disappears entirely.

## Install

```sh
cargo install ccstatuskit          # or grab a binary from GitHub Releases
```

Then point Claude Code at it in `~/.claude/settings.json`:

```json
{
  "statusLine": { "type": "command", "command": "ccstatuskit" }
}
```

See [docs/configuration.md](docs/configuration.md) for the full configuration reference, and [docs/module-contract.md](docs/module-contract.md) for writing your own modules.

## Builtin modules

| Module | Shows | Source |
| --- | --- | --- |
| `model` | Model name with family color | stdin |
| `directory` | Project-relative path | stdin |
| `git` | Branch, dirty/staged/untracked/renamed, conflicts, stash, merge/rebase, ahead/behind | `git` CLI |
| `memory` | System RAM used/total | procfs / sysctl / WinAPI |
| `ctx` | Context-window usage % with thresholds | stdin |
| `time` | Clock + session duration | system clock |
| `usage` | Claude quota windows (5h / weekly / per-model) | stdin (+ optional API) |
| `cost` | Session cost in USD, colored by spend | stdin |
| `lines` | Lines added/removed this session | stdin |
| `agent` | Active subagent name | stdin |
| `pr` | Open PR number, colored by review state | stdin |
| `worktree` | Worktree name/branch when not the primary checkout | stdin |
| `package` | The project's own declared version (starship-style) | `Cargo.toml` / `package.json` / `pyproject.toml` / `composer.json` / `pom.xml` |
| `unity`, `node`, `rust`, `go`, `python`, `dotnet`, `ruby`, `java`, `kotlin`, `php`, `swift` | Per-language icon + toolchain version, one module per language | real binaries (`rustc --version`, `go version`, …), falling back to project files |

Every module accepts `disabled = true` and `style = "bold fg:#A6E22E"` (starship-style strings; `[palette]` defines named colors). The language modules detect independently — a repo with both `Cargo.toml` and `package.json` shows both `$rust` and `$node`.

## Usage quotas

By default `$usage` renders the official `rate_limits` data Claude Code pipes in (5-hour and 7-day windows, Pro/Max plans). With:

```toml
[usage]
scoped = true
```

it also shows per-model weekly quotas (e.g. Fable) from the same unofficial endpoint Claude Code itself uses, refreshed in the background with a stale-while-revalidate disk cache — renders never wait on the network, and your OAuth token never appears in output or logs. **The scoped source is unofficial and may break without notice.** `only = [...]` filters windows: `"five_hour"`, `"seven_day"`, or a model-name substring like `"fable"`.

## Requirements

- A [Nerd Font](https://www.nerdfonts.com/) **v3** for the default icons. Without one, nothing breaks: icons degrade to replacement boxes while every piece of information stays readable as text (model name, path, percentages, branch). State marks like ✓ ✎ ⚠ are standard Unicode and render with any modern font.
- Windows: works out of the box; `[custom.x]` modules default to `cmd /C` there — set `shell = ["sh", "-c"]` if you have Git Bash

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

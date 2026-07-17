# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-07-18

### Added

- `ccstatuskit doctor`: diagnoses what the fail-soft render hides — config
  resolution, TOML and format-template errors, invalid styles and palette
  entries, unknown module names, custom references without tables, shell
  resolution, and scoped-usage credential/cache state. Errors exit 1
- `ccstatuskit test-module '<cmd>'`: exercises a command against the module
  contract with full- and empty-context scenarios, reporting the resulting
  segment (or why it hid), timing against the render budget, and stderr
- `--version` / `--help`; an interactive invocation prints help instead of
  waiting on stdin
- Japanese and Simplified Chinese documentation with CI-enforced structural
  sync, a README screenshot, and `mise run install` / `mise run test` tasks

### Fixed

- The Vue project icon uses the Nerd Fonts v3 codepoint; the old v2-range
  glyph rendered as an Arabic ligature or a replacement box on current fonts
- The release workflow creates the GitHub Release before uploading binaries

## [0.1.0] - 2026-07-16

### Added

- `format` TOML template with `$module` placeholders; template lines are
  statusline rows, and rows collapse automatically when all their modules hide
- Eight builtin modules: `model`, `directory`, `git`, `memory`, `ctx`,
  `time`, `project`, `usage`
- `usage` module: official stdin `rate_limits` (5-hour / 7-day) by default,
  opt-in per-model quotas (`scoped = true`) via a stale-while-revalidate
  disk cache refreshed by a detached `--refresh-usage` process
- External module contract v1 (`[custom.x]`): raw stdin JSON passthrough,
  `CCSK_*` env vars, first-stdout-line segments, empty output hides
- starship-style `style` strings with a named `[palette]`; `NO_COLOR` support
- Parallel module execution with a global deadline and fail-soft semantics:
  the binary always renders and always exits 0
- Windows support, including process-tree kill on custom-module timeout and
  non-inheritable std handles so children can never hold the statusline pipe
- Release binaries for Linux (gnu/musl), macOS (x64/arm64), and Windows (msvc)

[0.2.0]: https://github.com/bigdra50/ccstatuskit/releases/tag/v0.2.0
[0.1.0]: https://github.com/bigdra50/ccstatuskit/releases/tag/v0.1.0

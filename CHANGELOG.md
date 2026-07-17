# Changelog

All notable changes to this project are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/).

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

[0.1.0]: https://github.com/bigdra50/ccstatuskit/releases/tag/v0.1.0

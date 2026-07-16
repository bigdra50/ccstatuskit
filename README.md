# ccstatuskit

A modular, starship-inspired statusline kit for [Claude Code](https://code.claude.com).

Compose your statusline from builtin modules and external commands, laid out with a TOML format template.

> Status: pre-release — v0.1 in development.

## Design

- Single Rust binary: reads Claude Code's statusline JSON on stdin, prints styled rows on stdout
- `format` template with `$module` placeholders; literal newlines define statusline rows
- Builtin modules run in-process, in parallel, each with its own timeout
- Fail-soft: a broken or slow module drops its segment — never the whole line
- `[custom.x]` runs any executable via a documented module contract (stdin JSON pass-through + `CCSK_*` env vars), so modules can be written in any language

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

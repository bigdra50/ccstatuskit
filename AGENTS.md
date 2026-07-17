# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project

ccstatuskit is a modular statusline for Claude Code: a Rust binary that reads
the statusline JSON on stdin, renders styled rows on stdout, and always exits 0.

## Commands

```sh
mise run test      # cargo test + clippy -D warnings + fmt check — run before committing
mise run install   # build release and install to ~/.local/bin
```

Plain cargo works too: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.

## Architecture

- `src/context.rs` — lenient serde parse of the stdin JSON. Every top-level
  field degrades to `None` on type mismatch, and the raw bytes are kept for
  custom-module passthrough. Never add `deny_unknown_fields`.
- `src/format.rs` — `$module` template parser; template lines are statusline rows.
- `src/engine.rs` — one thread per module with a global deadline.
- `src/modules/` — builtin modules (enum dispatch). All OS access goes through
  the `src/probes/` trait so tests inject fakes.
- `src/modules/custom.rs` — `[custom.x]` execution per `docs/module-contract.md`
  (stdin passthrough + `CCSK_*` env).

## Invariants

- Fail-soft over erroring: a broken module loses its segment, never the row;
  the binary always exits 0 and never blocks past its deadline.
- `docs/module-contract.md` is a versioned public contract (`CCSK_CONTRACT`).
  Breaking it requires a contract version bump, not a silent change.
- TDD: write the failing test first. Golden fixtures live in `tests/fixtures/`
  and double as the schema-compatibility corpus.

## Documentation languages

- English files are the source of truth: `README.md`, `docs/*.md`.
- Every documentation change MUST update the translated counterparts in the
  same change: `README.ja.md`, `README.zh-CN.md`, `docs/*.ja.md`,
  `docs/*.zh-CN.md`. Keep section structure 1:1 across languages.
- New documents get all three languages plus the language-switcher line at the top.

## Commits

English, Conventional Commits with a gitmoji prefix
(`✨ feat` / `🐛 fix` / `📝 docs` / `♻️ refactor` / `🔧 chore` / `✅ test` / `🔖 release`).

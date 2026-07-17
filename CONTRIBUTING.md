# Contributing

Thanks for your interest in ccstatuskit.

## Development setup

Stable Rust is the only requirement. With [mise](https://mise.jdx.dev/):

```sh
mise run test      # cargo test + clippy -D warnings + fmt check
mise run install   # build release and install to ~/.local/bin
```

Plain cargo equivalents work too.

## Workflow

- **TDD**: write the failing test first, then make it pass, then refactor.
  Golden fixtures live in `tests/fixtures/` and double as the
  schema-compatibility corpus for Claude Code's stdin JSON.
- **Invariants** (see [AGENTS.md](AGENTS.md)): the binary always exits 0,
  never blocks past its deadline, and a broken module loses only its own
  segment. Changes that trade these away will not be merged.
- **Module contract**: `docs/module-contract.md` is a versioned public
  interface (`CCSK_CONTRACT`). Breaking changes require a version bump and
  a changelog entry, never a silent edit.

## Documentation

English files are the source of truth. Any change to `README.md` or
`docs/*.md` must update the `.ja.md` and `.zh-CN.md` counterparts in the
same pull request, keeping the section structure 1:1.
`scripts/validate-docs.sh` enforces this in CI.

## Commits and pull requests

- Commit messages: English, Conventional Commits with a gitmoji prefix
  (`✨ feat` / `🐛 fix` / `📝 docs` / `♻️ refactor` / `🔧 chore` / `✅ test`)
- CI (3-OS test matrix + docs validation) must be green

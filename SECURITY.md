# Security Policy

## Supported versions

Only the latest release receives security fixes.

## How ccstatuskit handles credentials

The optional scoped-usage feature (`[usage] scoped = true`) reads the
Claude Code OAuth token to query the usage endpoint:

- The token is read at fetch time from `~/.claude/.credentials.json` or,
  on macOS, the `Claude Code-credentials` Keychain item — the same places
  Claude Code itself stores it.
- Fetches happen only in the detached `--refresh-usage` process; renders
  never touch the network.
- The token is never written to the cache, the statusline output, logs,
  or error messages. The disk cache stores only the API response body.
- With `scoped` unset (the default), ccstatuskit reads no credentials at
  all and performs no network I/O.

Custom modules (`[custom.x]`) run arbitrary user-configured commands with
the user's own privileges. ccstatuskit executes only what the user wrote
in their own config file.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting
(Security tab → "Report a vulnerability") on this repository.
Do not open a public issue for security reports.

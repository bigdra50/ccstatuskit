---
name: tmux-pr-screenshot
description: Capture a real tmux/terminal rendering of the ccstatuskit binary (with Nerd Font icons and ANSI colors intact) as a PNG, and embed it in a GitHub PR or issue comment. Use this whenever a change to a builtin module, the git status logic, an icon, or the default format should be shown visually during code review — not just described in text or covered by unit tests — including phrases like "screenshot this", "show me what it looks like", "paste it into the PR", "render this in tmux", or "let's see it visually" for this repo. Also use it as the reference recipe whenever Nerd Font icon codepoints need to be rendered or verified in a sandbox that has no real Nerd Font installed and no network access to github.com/ryanoasis/nerd-fonts.
---

# tmux PR screenshot

ccstatuskit's whole value proposition is visual (icons, color, layout), so
"does this look right" is a real review question that plain-text terminal
output or `cargo test` can't answer. This skill drives the actual binary in
a real tmux pane, captures the exact bytes it wrote (ANSI codes and all),
and turns that into a PNG that can go straight into a PR comment.

The hard part — rendering Nerd Font icon codepoints without a real Nerd
Font and without network access to fetch one — is already solved below.
Don't re-derive it; use the recipe.

## Why this is needed (read once, skip if you already know it)

ccstatuskit's icons live in Unicode Private Use Areas that only a Nerd
Font patched font defines. A cloud sandbox for this repo typically has:
- no `ryanoasis/nerd-fonts` access (GitHub access here is scoped to the
  repos explicitly added to the session; that one usually isn't in scope,
  and even if it were, downloading arbitrary release assets isn't what
  `add_repo` is for)
- no Nerd Font pre-installed

But `registry.npmjs.org`, `pypi.org`, and `files.pythonhosted.org` are
reachable from anywhere (they're excluded from the GitHub-scoped proxy
restriction), and it turns out **`qtawesome`, a PyPI package for Qt icon
fonts, vendors plain `.ttf` files for several modern icon sets** — not a
Nerd Font itself, but several of its fonts happen to share codepoints with
what Nerd Fonts v3 assigned when it merged those same upstream sources.
Combined with a few `apt` packages for the older/legacy overlaps, this
covers a solid majority of ccstatuskit's icons with **verified-correct**
glyphs — good enough for an honest, mostly-accurate preview. Run
`scripts/setup_fonts.sh` once per environment; it's idempotent.

## What's covered, what isn't

Only trust a codepoint match if it's been visually verified (a font's
`cmap` containing a codepoint does NOT mean the glyph is the right
picture — icon libraries reuse PUA slots for unrelated icons, and MDI's
own codepoint assignments have churned across versions). Verified so far:

| Source in ccstatuskit | Codepoint | Font family after setup | Status |
| --- | --- | --- | --- |
| `model` (robot) | U+F06A9 | Material Design Icons | ✅ verified |
| `directory` (folder) | U+F0256 | Material Design Icons | ✅ verified |
| `time` (clock) | U+F0954 | Material Design Icons | ✅ verified |
| `memory` (chip) | U+F035B | Material Design Icons | ✅ verified |
| `node`'s vue icon | U+F0844 | Material Design Icons | ✅ verified |
| `python` | U+F0320 | Material Design Icons | ✅ verified |
| `dotnet` (.NET wordmark) | U+F0AAE | Material Design Icons | ✅ verified |
| `ruby` (gem) | U+F0D2D | Material Design Icons | ✅ verified |
| `php` (wordmark) | U+F031F | Material Design Icons | ✅ verified |
| `cost` (dollar) | U+F155 | FontAwesome (apt) / Font Awesome 6 Free Solid | ✅ verified |
| `pr` (git-pull-request) | U+EA64 | codicon | ✅ verified |
| `lines`'s conceptual diff icon | U+EA98 | codicon | ✅ verified |
| `worktree` (repo-forked) | U+EA63 | codicon | ✅ verified |
| `git` branch glyph | U+E0A0 | PowerlineSymbols | ✅ verified |
| `node`'s nextjs icon | U+F0E01 | Material Design Icons | ❌ renders as a thermometer-alert icon — do NOT use in a demo |
| `kotlin` | U+F10FE | Material Design Icons | ❌ renders as a kubernetes-helm icon — do NOT use in a demo |
| `rust`, `go`, `node`'s plain/react/typescript icons, `java`, `swift`, `unity` | U+E600–E7FF (devicon/Seti-UI range) | none found | tofu box (□) — expected, matches the project's documented no-Nerd-Font degrade-gracefully behavior |
| `agent` (robot, FA6-extension slot) | U+EE0D | none found | tofu box — expected |

When you add a new icon to ccstatuskit or design a demo scenario, check
this table first. If a codepoint isn't listed, verify it yourself before
trusting the rendered glyph (see "Verifying a new codepoint" below) rather
than assuming it's fine.

## Step-by-step

### 1. Set up fonts (once per environment)

```sh
bash .claude/skills/tmux-pr-screenshot/scripts/setup_fonts.sh
```

Installs `fonts-jetbrains-mono fonts-font-awesome fonts-octicons
fonts-powerline` via apt, removes the conflicting old
`fonts-materialdesignicons-webfont` apt package (it squats on the same
font-family name with incompatible codepoints), downloads `qtawesome` from
PyPI and lifts its bundled Material Design Icons 6 / Font Awesome 6 Solid
/ codicon `.ttf` files into `/usr/local/share/fonts/`, then `fc-cache -f`s.
Safe to re-run; it no-ops if already installed.

### 2. Build a scenario that exercises what you're reviewing

Design a small temp directory (and a temp git repo if you're demoing `git`
module changes) that puts the feature in a realistic, honest state — don't
fake data through the JSON that the real code path wouldn't produce.

- **Language modules**: create marker files for languages whose icons are
  ✅ verified above (e.g. `pyproject.toml`+`.python-version`,
  `package.json` with a `vue` dependency, `Gemfile`+`.ruby-version`,
  `composer.json`) so multiple modules fire in the same screenshot,
  demonstrating that they detect independently. Including `Cargo.toml`
  (rust) is fine even though its icon is a tofu box — this repo is itself
  a Rust crate, so the version text next to the box is honest and
  relevant.
- **git module**: a real merge conflict plus a few other real, live
  changes shows the most icons at once. This sequence produces conflicted
  + renamed + staged + modified + deleted + untracked + stash, all live:
  ```sh
  git init -q -b main && git config user.email you@example.com && git config user.name you
  # commit files: conflict.txt, rename_me.txt, delete_me.txt, staged_edit.txt, unstaged_edit.txt, stash_file.txt
  git add -A && git commit -q -m initial
  echo "stash change" > stash_file.txt && git stash push -q -m wip   # -> 1 stash
  git checkout -q -b feature && echo "feature change" > conflict.txt && git commit -qam wip
  git checkout -q main && echo "main change" > conflict.txt && git commit -qam wip2
  git merge feature -q || true                                       # -> conflict + in-progress merge
  git mv rename_me.txt renamed.txt                                    # -> staged rename
  rm delete_me.txt                                                    # -> unstaged delete
  echo x > untracked_new.txt                                          # -> untracked
  echo x > staged_edit.txt && git add staged_edit.txt                 # -> staged
  echo x > unstaged_edit.txt                                          # -> modified
  ```
  Order matters: create the stash *before* the merge (git stash is
  finicky mid-conflict), and only touch files unrelated to the conflicting
  path after the merge starts (git allows that; it won't let you merge
  over already-dirty files that the merge itself needs to touch).
- **Other stdin-driven modules** (`cost`, `agent`, `pr`, `worktree`,
  `context_window`, etc.): just write the JSON directly — these don't
  depend on real filesystem or git state, only on what you pipe to stdin.

### 3. Run the real binary in tmux and capture its exact output

```sh
tmux new-session -d -s ccdemo -x 220 -y 12
tmux send-keys -t ccdemo "clear; cat context.json | CCSTATUSKIT_CONFIG=/nonexistent /path/to/target/debug/ccstatuskit" Enter
sleep 1.5   # give the render + pane a moment; capturing too early grabs an empty pane
tmux capture-pane -t ccdemo -e -p > capture.ansi   # -e is required to keep ANSI codes; plain -p strips them
tmux kill-session -t ccdemo
```

Build/point at a debug build (`cargo build`) unless you specifically need
release behavior — it's faster to iterate on.

### 4. Convert the capture to a screenshot

```sh
python3 .claude/skills/tmux-pr-screenshot/scripts/render_screenshot.py \
  capture.ansi out.png \
  --lines 3 \
  --title "you@vm: ~/demo-repo — tmux" \
  --tmux-window "0:ccstatuskit*" \
  --tmux-right '"vm" 12:34 2026-01-01'
```

`--lines` should match how many rows your `format` template actually
produces (the tmux pane has blank padding below the real output — this
trims it). The script auto-detects the pre-installed Chromium under
`/opt/pw-browsers/chromium-*/chrome-linux/chrome` via Playwright
(`pip install playwright` if it isn't already available; no need to run
`playwright install`, the browser is already there).

Look at the result yourself (the Read tool renders PNGs) before sending it
anywhere — check for tofu boxes you didn't expect, and cross-reference any
icon you're not sure about against the codepoint table above.

### 5. Get it into the PR

There's no direct "upload an image attachment" API available through this
session's GitHub tools — `add_issue_comment` only takes a markdown string.
The way to make an image actually render in a PR/issue comment is to
commit the PNG into the repo and link to it by commit SHA:

```sh
mkdir -p .github/pr-preview
cp out.png .github/pr-preview/pr<N>-<short-description>.png
git add .github/pr-preview/pr<N>-<short-description>.png
git commit -m "📝 docs: add PR preview screenshot"
git push
```

Then comment with:

```markdown
![description](https://raw.githubusercontent.com/<owner>/<repo>/<commit-sha>/.github/pr-preview/pr<N>-<short-description>.png)
```

Pin to the **commit SHA** you just pushed, not the branch name — a later
force-push or rebase would otherwise break the link. Mention in the
comment that it's a review-only asset and can be dropped before merge if
the maintainer would rather not keep it in history.

## Verifying a new codepoint

If you need an icon that isn't in the table above (a new module, or a
language icon in the unresolved devicon range), don't guess:

```python
from fontTools.ttLib import TTFont
f = TTFont("/path/to/font.ttf", fontNumber=0, lazy=True)
0xXXXX in f.getBestCmap()   # True just means *a* glyph exists there — not that it's the right one
```

Then actually look at it — `cmap` membership is necessary but not
sufficient:

```python
from PIL import Image, ImageDraw, ImageFont
img = Image.new("RGB", (200, 200), "white")
ImageDraw.Draw(img).text((10, 10), chr(0xXXXX), fill="black",
                          font=ImageFont.truetype("/path/to/font.ttf", 80))
img.save("/tmp/glyphcheck.png")
```

Read the PNG back and eyeball it before trusting the codepoint in a demo
or adding it to the table above.

## Files in this skill

- `scripts/setup_fonts.sh` — one-time, idempotent font installation
- `scripts/ansi2html.py` — SGR-to-HTML converter (also importable)
- `scripts/render_screenshot.py` — capture → HTML → Playwright screenshot
- `assets/term_template.html` — the terminal-window mockup (traffic-light
  titlebar + dark terminal body + tmux status bar), with placeholders
  `{{WIDTH}}`, `{{TITLE}}`, `{{BODY}}`, `{{TMUX_WINDOW}}`,
  `{{TMUX_STATUS_RIGHT}}`

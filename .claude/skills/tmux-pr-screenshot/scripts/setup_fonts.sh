#!/usr/bin/env bash
# Idempotent font setup for rendering ccstatuskit's Nerd Font icon codepoints
# in a sandbox that can't reach github.com/ryanoasis/nerd-fonts (repo-scoped
# proxy) and has no real Nerd Font installed.
#
# Recipe: apt gives us a base monospace font plus a few legacy icon fonts
# whose codepoints happen to still match Nerd Fonts (FontAwesome 4's own
# PUA range was kept unchanged when Nerd Fonts merged it, and Powerline's
# branch glyph likewise). pip's `qtawesome` package is not a Nerd Font
# consumer at all, but it vendors *plain font files* for several modern
# icon sets as installable data, including a current Material Design Icons
# build whose astral-plane codepoints (U+F0000+) match what Nerd Fonts v3
# assigns for `nf-md-*` icons, and a `codicon` build whose U+EA60+ range
# matches Nerd Fonts' octicon-derived git icons. See SKILL.md's codepoint
# table for what's verified-good vs. known-mismatched vs. unavailable.
set -euo pipefail

FONT_DIR=/usr/local/share/fonts/tmux-pr-screenshot
MARKER="$FONT_DIR/.installed"

if [ -f "$MARKER" ]; then
    echo "fonts already set up ($FONT_DIR); skipping. Delete $MARKER to force a re-run."
    exit 0
fi

echo "== apt: base monospace + legacy icon fonts =="
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    fonts-jetbrains-mono fonts-font-awesome fonts-octicons fonts-powerline

# The apt package for Material Design Icons registers under the exact same
# family name ("Material Design Icons") as the modern one we install below,
# but with old/incompatible codepoints. Remove it so fontconfig can't pick
# the wrong one.
apt-get remove -y fonts-materialdesignicons-webfont 2>/dev/null || true

echo "== pip: extract modern icon fonts bundled inside qtawesome =="
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
pip download --no-deps -d "$WORK" qtawesome
python3 - "$WORK" <<'PY'
import sys, zipfile, glob
work = sys.argv[1]
whl = glob.glob(f"{work}/qtawesome-*.whl")[0]
with zipfile.ZipFile(whl) as z:
    z.extractall(f"{work}/extracted")
PY

mkdir -p "$FONT_DIR"
FONTS_SRC="$WORK/extracted/qtawesome/fonts"
cp "$FONTS_SRC"/materialdesignicons6-webfont-*.ttf "$FONT_DIR/"
cp "$FONTS_SRC"/fontawesome6-solid-webfont-*.ttf "$FONT_DIR/"
cp "$FONTS_SRC"/codicon-*.ttf "$FONT_DIR/"

fc-cache -f >/dev/null
touch "$MARKER"
echo "done. Installed to $FONT_DIR:"
ls "$FONT_DIR"

#!/usr/bin/env python3
"""Turn a tmux ANSI capture into a terminal-window screenshot PNG.

Usage:
    python3 render_screenshot.py CAPTURE.ansi OUT.png \
        [--lines N] [--title "user@host: ~/repo — tmux"] \
        [--tmux-window "0:mysession*"] [--tmux-right '"vm" 12:34'] \
        [--width 1180]

Reads the raw tmux `capture-pane -e -p` output (ANSI escape codes intact),
converts it to styled HTML with ansi2html.py, drops it into the terminal-
window mockup in ../assets/term_template.html, and screenshots just the
`.window` element with headless Chromium via Playwright.
"""
import argparse
import glob
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from ansi2html import ansi_to_html_body  # noqa: E402

SKILL_DIR = Path(__file__).parent.parent


def find_chromium():
    candidates = sorted(glob.glob("/opt/pw-browsers/chromium-*/chrome-linux/chrome"))
    if not candidates:
        candidates = sorted(glob.glob("/opt/pw-browsers/chromium/chrome-linux/chrome"))
    if not candidates:
        raise SystemExit(
            "No pre-installed Chromium found under /opt/pw-browsers. "
            "Run `ls /opt/pw-browsers` to see what's there and adjust the path."
        )
    return candidates[-1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("capture", help="path to a tmux `capture-pane -e -p` dump")
    ap.add_argument("out_png")
    ap.add_argument("--lines", type=int, default=None, help="keep only the first N lines")
    ap.add_argument("--title", default="demo@vm: ~ — tmux")
    ap.add_argument("--tmux-window", default="0:demo*")
    ap.add_argument("--tmux-right", default="")
    ap.add_argument("--width", type=int, default=1180)
    ap.add_argument("--scale", type=float, default=2.0, help="device scale factor (crispness)")
    args = ap.parse_args()

    raw = Path(args.capture).read_text()
    body = ansi_to_html_body(raw, args.lines)

    template = (SKILL_DIR / "assets" / "term_template.html").read_text()
    html_out = (
        template.replace("{{WIDTH}}", str(args.width))
        .replace("{{TITLE}}", args.title)
        .replace("{{BODY}}", body)
        .replace("{{TMUX_WINDOW}}", args.tmux_window)
        .replace("{{TMUX_STATUS_RIGHT}}", args.tmux_right)
    )
    tmp_html = Path(args.out_png).with_suffix(".preview.html")
    tmp_html.write_text(html_out, encoding="utf-8")

    from playwright.sync_api import sync_playwright

    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=find_chromium())
        page = browser.new_page(
            viewport={"width": args.width + 80, "height": 500},
            device_scale_factor=args.scale,
        )
        page.goto(f"file://{tmp_html.resolve()}")
        page.wait_for_timeout(150)
        el = page.query_selector(".window")
        el.screenshot(path=args.out_png)
        browser.close()

    print(f"wrote {args.out_png} (preview html at {tmp_html})")


if __name__ == "__main__":
    main()

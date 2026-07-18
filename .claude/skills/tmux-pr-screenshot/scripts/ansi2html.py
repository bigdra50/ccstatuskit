#!/usr/bin/env python3
"""Convert a captured ANSI terminal buffer into HTML spans.

Usage: python3 ansi2html.py [num_lines] < capture.ansi > body.html

Parses the general SGR grammar (CSI ... m) but ccstatuskit's own output
only ever emits truecolor foreground (`ESC[38;2;r;g;bm`) and a reset
(`ESC[0m` or `ESC[39m`), so that's the part worth trusting completely.
"""
import html
import re
import sys

CSI_RE = re.compile(r"\x1b\[([0-9;]*)m")


def parse_sgr_stream(text):
    """Yields (plain_text_chunk, style_dict) pairs."""
    style = {"fg": None, "bold": False, "italic": False, "underline": False, "dim": False}
    pos = 0
    for m in CSI_RE.finditer(text):
        chunk = text[pos:m.start()]
        if chunk:
            yield chunk, dict(style)
        pos = m.end()
        codes = [c for c in m.group(1).split(";") if c != ""] or ["0"]
        i = 0
        while i < len(codes):
            code = codes[i]
            if code == "0":
                style = {"fg": None, "bold": False, "italic": False, "underline": False, "dim": False}
            elif code == "1":
                style["bold"] = True
            elif code == "2":
                style["dim"] = True
            elif code == "3":
                style["italic"] = True
            elif code == "4":
                style["underline"] = True
            elif code == "39":
                style["fg"] = None
            elif code == "38" and i + 4 < len(codes) and codes[i + 1] == "2":
                r, g, b = codes[i + 2], codes[i + 3], codes[i + 4]
                style["fg"] = f"rgb({r},{g},{b})"
                i += 4
            i += 1
    chunk = text[pos:]
    if chunk:
        yield chunk, dict(style)


def line_to_html(line):
    out = []
    for chunk, style in parse_sgr_stream(line):
        if not chunk:
            continue
        css = []
        if style["fg"]:
            css.append(f"color:{style['fg']}")
        if style["bold"]:
            css.append("font-weight:bold")
        if style["italic"]:
            css.append("font-style:italic")
        if style["underline"]:
            css.append("text-decoration:underline")
        if style["dim"]:
            css.append("opacity:0.7")
        text = html.escape(chunk)
        out.append(f'<span style="{";".join(css)}">{text}</span>' if css else text)
    return "".join(out)


def ansi_to_html_body(raw, num_lines=None):
    lines = raw.split("\n")
    if num_lines is not None:
        lines = lines[:num_lines]
    return "\n".join(line_to_html(line) for line in lines)


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else None
    print(ansi_to_html_body(sys.stdin.read(), n))


if __name__ == "__main__":
    main()

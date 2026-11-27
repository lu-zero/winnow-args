#!/usr/bin/env python3
"""Write the performance graph: performance.py docs/performance.svg

The times are those of benchmarks/README.md, in nanoseconds."""
import math, sys
from svg_theme import STYLE
FONT = "ui-sans-serif,system-ui,Segoe UI,Helvetica,Arial,sans-serif"
TEAL, GREY = "accent", "muted"
COLOURS = {"winnow-args": "green", "usage 7": "partial", "clap 4": "warn", "bpaf 0.9": "bad"}
PANELS = [
    ("example -v --path /tmp/x a b c", [("winnow-args", 207, "207 ns"), ("usage 7", 494, "494 ns"), ("clap 4", 15000, "15.0 µs"), ("bpaf 0.9", 14900, "14.9 µs")]),
    ("mise use -g node@20, 211 commands", [("winnow-args", 365, "365 ns"), ("usage 7", 782, "782 ns"), ("clap 4", 755000, "755 µs"), ("bpaf 0.9", 2630000, "2.63 ms")]),
]
TICKS = ["100 ns", "1 µs", "10 µs", "100 µs", "1 ms", "10 ms"]
W, X0, DECADE, ROW, BAR, SIZE = 720, 104, 112, 26, 16, 13
out = []

def text(x, y, s, anchor="start", colour=GREY, weight=None, centred=False):
    w = f' font-weight="{weight}"' if weight else ""
    c = ' dominant-baseline="central"' if centred else ""
    out.append(f'<text x="{x:.1f}" y="{y:.1f}" font-family="{FONT}" font-size="{SIZE}" class="wa-{colour}" text-anchor="{anchor}"{c}{w}>{s}</text>')

y = 0
for title, rows in PANELS:
    text(10, y + 18, title, colour=TEAL, weight=600)
    top, bottom = y + 30, y + 30 + ROW * len(rows)
    for i, tick in enumerate(TICKS):
        x = X0 + i * DECADE
        out.append(f'<line x1="{x}" y1="{top}" x2="{x}" y2="{bottom}" class="wa-grid" stroke-width="1"/>')
        text(x, bottom + 17, tick, "middle")
    for n, (name, ns, label) in enumerate(rows):
        ry = top + n * ROW + (ROW - BAR) // 2
        width = math.log10(ns / 100) * DECADE
        text(X0 - 8, ry + BAR / 2, name, "end", centred=True)
        out.append(f'<rect x="{X0}" y="{ry}" width="{width:.1f}" height="{BAR}" rx="2" class="wa-{COLOURS[name]}"/>')
        text(X0 + width + 6, ry + BAR / 2, label, centred=True)
    y = bottom + 34
text(X0 + DECADE * 2.5, y + 4, "time per parse, log scale", "middle")
H = y + 14
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" class="wa-figure" width="1024" height="{round(H * 1024 / W)}" viewBox="0 0 {W} {H}" role="img" '
       f'aria-label="Time per parse: winnow-args against usage, clap and bpaf.">\n{STYLE}\n'
       f'<rect width="{W}" height="{H}" fill="none"/>\n' + "\n".join(out) + "\n</svg>\n")
open(sys.argv[1], "w").write(svg)

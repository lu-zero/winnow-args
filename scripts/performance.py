#!/usr/bin/env python3
"""Write the performance graph: performance.py docs/performance.svg

The times are those of benchmarks/README.md, in nanoseconds."""
import math, sys
FONT = "ui-sans-serif,system-ui,Segoe UI,Helvetica,Arial,sans-serif"
TEAL, GREY = "#5fafaf", "#8a8a8a"
COLOURS = {"winnow-args": "#5faf5f", "usage 7": "#87afaf", "clap 4": "#d78787", "bpaf 0.9": "#d75f5f"}
PANELS = [
    ("example -v --path /tmp/x a b c", [("winnow-args", 195, "195 ns"), ("usage 7", 504, "504 ns"), ("clap 4", 15100, "15.1 µs"), ("bpaf 0.9", 15000, "15.0 µs")]),
    ("mise use -g node@20, 211 commands", [("winnow-args", 334, "334 ns"), ("usage 7", 770, "770 ns"), ("clap 4", 760000, "760 µs"), ("bpaf 0.9", 2630000, "2.63 ms")]),
]
TICKS = ["100 ns", "1 µs", "10 µs", "100 µs", "1 ms", "10 ms"]
W, X0, DECADE, ROW, BAR, SIZE = 720, 104, 112, 26, 16, 13
out = []

def text(x, y, s, anchor="start", fill=GREY, weight=None):
    w = f' font-weight="{weight}"' if weight else ""
    out.append(f'<text x="{x:.1f}" y="{y}" font-family="{FONT}" font-size="{SIZE}" fill="{fill}" text-anchor="{anchor}"{w}>{s}</text>')

y = 0
for title, rows in PANELS:
    text(10, y + 18, title, fill=TEAL, weight=600)
    top, bottom = y + 30, y + 30 + ROW * len(rows)
    for i, tick in enumerate(TICKS):
        x = X0 + i * DECADE
        out.append(f'<line x1="{x}" y1="{top}" x2="{x}" y2="{bottom}" stroke="{GREY}" stroke-opacity="0.35" stroke-width="1"/>')
        text(x, bottom + 17, tick, "middle")
    for n, (name, ns, label) in enumerate(rows):
        ry = top + n * ROW + (ROW - BAR) // 2
        width = math.log10(ns / 100) * DECADE
        text(X0 - 8, ry + BAR - 3, name, "end")
        out.append(f'<rect x="{X0}" y="{ry}" width="{width:.1f}" height="{BAR}" rx="2" fill="{COLOURS[name]}"/>')
        text(X0 + width + 6, ry + BAR - 3, label)
    y = bottom + 34
text(X0 + DECADE * 2.5, y + 4, "time per parse, log scale", "middle")
H = y + 14
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="{round(H * 1024 / W)}" viewBox="0 0 {W} {H}" role="img" '
       f'aria-label="Time per parse: winnow-args against usage, clap and bpaf.">\n<rect width="{W}" height="{H}" fill="none"/>\n' + "\n".join(out) + "\n</svg>\n")
open(sys.argv[1], "w").write(svg)

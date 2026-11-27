#!/usr/bin/env python3
"""Write the feature-coverage graph: features.py docs/features.svg

The counts come from the tables in docs/COMPARISON.md."""
import pathlib
import sys
from svg_theme import STYLE

FONT = "ui-sans-serif,system-ui,Segoe UI,Helvetica,Arial,sans-serif"
INK, MUTED, ACCENT = "ink", "muted", "accent"
COLOURS = {"built": "green", "partial": "partial", "missing": "bad"}
NAMES = ["winnow-args", "usage", "bpaf", "clap"]
MARKS = {"✓": "built", "~": "partial", "–": "missing"}
W, X0, BAR, ROW, SIZE = 760, 120, 520, 44, 13
out = []


def text(x, y, value, anchor="start", colour=MUTED, weight=None, centred=False):
    w = f' font-weight="{weight}"' if weight else ""
    c = ' dominant-baseline="central"' if centred else ""
    out.append(
        f'<text x="{x:.1f}" y="{y:.1f}" font-family="{FONT}" font-size="{SIZE}" '
        f'class="wa-{colour}" text-anchor="{anchor}"{c}{w}>{value}</text>'
    )


def counts(path):
    result = {name: dict.fromkeys(COLOURS, 0) for name in NAMES}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 5 or set(cells[0]) <= set("-:") or not cells[0]:
            continue
        for name, cell in zip(NAMES, cells[1:]):
            result[name][MARKS.get(cell[:1], "missing")] += 1
    return result


data = counts(pathlib.Path(__file__).resolve().parent.parent / "docs" / "COMPARISON.md")
total = sum(data["winnow-args"].values())
text(10, 22, "Features matched", colour=ACCENT, weight=600)
text(10, 40, f"{total} features from the full comparison", colour=MUTED)

for n, name in enumerate(NAMES):
    y = 62 + n * ROW
    text(
        X0 - 10, y + 9, name, "end",
        INK if n == 0 else MUTED, 600 if n == 0 else None, centred=True,
    )
    x = X0
    for kind in COLOURS:
        width = data[name][kind] / total * BAR
        if width:
            out.append(
                f'<rect x="{x:.1f}" y="{y}" width="{width:.1f}" height="18" '
                f'class="wa-{COLOURS[kind]}"/>'
            )
            x += width
    text(
        X0 + BAR + 12, y + 9, " · ".join(str(data[name][k]) for k in COLOURS),
        colour=INK, centred=True,
    )

legend_y = 62 + len(NAMES) * ROW + 14
for i, (kind, label) in enumerate(
    [("built", "built in"), ("partial", "with some work"), ("missing", "not offered")]
):
    x = 10 + i * 205
    out.append(f'<rect x="{x}" y="{legend_y - 10}" width="11" height="11" class="wa-{COLOURS[kind]}"/>')
    text(x + 18, legend_y - 4.5, label, centred=True)

H = legend_y + 18
svg = (
    f'<svg xmlns="http://www.w3.org/2000/svg" class="wa-figure" width="1024" height="{round(H * 1024 / W)}" '
    f'viewBox="0 0 {W} {H}" role="img" aria-label="Feature coverage: winnow-args {total}, '
    f'{data["winnow-args"]["built"]} built in, {data["winnow-args"]["missing"]} missing.">\n'
    f'{STYLE}\n'
    f'<rect width="{W}" height="{H}" fill="none"/>\n' + "\n".join(out) + "\n</svg>\n"
)
open(sys.argv[1], "w", encoding="utf-8").write(svg)

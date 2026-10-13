#!/usr/bin/env python3
"""Write the architecture diagram: architecture.py docs/architecture.svg"""
import sys
FONT = "ui-sans-serif,system-ui,Segoe UI,Helvetica,Arial,sans-serif"
TEAL, GREEN, GREY = "#5fafaf", "#5faf5f", "#8a8a8a"
W, H = 720, 770
out = []

def text(x, y, s, size=12.5, fill=GREY, weight=None, anchor="middle", style=None):
    extra = (f' font-weight="{weight}"' if weight else "") + (f' font-style="{style}"' if style else "")
    s = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    out.append(f'<text x="{x}" y="{y}" font-family="{FONT}" font-size="{size}" fill="{fill}" text-anchor="{anchor}"{extra}>{s}</text>')

def box(x, y, w, h, title, sub="", colour=TEAL, dashed=False):
    dash = ' stroke-dasharray="5 3"' if dashed else ""
    out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{colour}" fill-opacity="0.08" stroke="{colour}" stroke-width="1.5"{dash}/>')
    cy = y + h / 2
    if sub:
        text(x + w / 2, cy - 4, title, 14, colour, 600)
        text(x + w / 2, cy + 14, sub)
    else:
        text(x + w / 2, cy + 5, title, 14, colour, 600)

def path(points, colour=GREY, dashed=False, arrow=True):
    d = "M" + " L".join(f"{x} {y}" for x, y in points)
    dash = ' stroke-dasharray="5 3"' if dashed else ""
    marker = f' marker-end="url(#arrow-{ "g" if colour == GREEN else "n" })"' if arrow else ""
    out.append(f'<path d="{d}" fill="none" stroke="{colour}" stroke-width="1.5"{dash}{marker}/>')

def lane(y, label):
    text(20, y, label.upper(), 13, TEAL, 600, "start")
    out.append(f'<line x1="20" y1="{y + 6}" x2="{W - 20}" y2="{y + 6}" stroke="{GREY}" stroke-opacity="0.35" stroke-width="1"/>')

# --- build time
lane(22, "Build time")
box(20, 44, 210, 60, "your struct or enum", "#[derive(Args)]")
box(270, 44, 210, 60, "winnow-args-derive", "the proc macro")
box(20, 140, 206, 60, "fragments", "with WINNOW_ARGS_SPEC", GREEN, dashed=True)
box(262, 140, 210, 60, "help data", "static, read only", GREEN)
box(494, 140, 206, 60, "a parser", "a match over the flags", GREEN)
path([(230, 74), (268, 74)])
path([(375, 104), (375, 138)])
path([(375, 122), (123, 122), (123, 138)])
path([(375, 122), (597, 122), (597, 138)])

# --- run time
lane(232, "Run time")
box(20, 262, 170, 60, "argv", "words, borrowed")
box(225, 262, 215, 60, "token", "-v  --long=value  word  --")
box(485, 262, 215, 60, "the parser", "generated, or written by hand")
path([(190, 292), (223, 292)])
path([(440, 292), (483, 292)])
path([(597, 200), (597, 260)], GREEN, dashed=True)
text(604, 222, "is", anchor="start")

box(20, 358, 195, 60, "FromArg", "words to values")
box(250, 358, 190, 60, "finish", "env, defaults, rules")
box(590, 350, 110, 34, "Ok(Cli)", colour=GREEN)
box(590, 392, 110, 34, "Err(Error)", colour=GREEN)
path([(597, 322), (597, 340), (117, 340), (117, 356)])
path([(215, 388), (248, 388)])
path([(440, 380), (588, 367)])
path([(440, 396), (588, 409)])

box(20, 462, 210, 60, "complete", "answers the shell's script")
box(255, 462, 190, 60, "help", "rendered when asked for")
box(510, 462, 190, 60, "report", "error, tip, exit status")
path([(645, 426), (645, 460)])
path([(510, 492), (447, 492)])
text(478, 485, "--help", 11)
# help data feeds the three
path([(462, 200), (462, 444)], GREEN, dashed=True, arrow=False)
path([(462, 444), (125, 444), (125, 460)], GREEN, dashed=True)
path([(350, 444), (350, 460)], GREEN, dashed=True)
path([(462, 444), (575, 444), (575, 460)], GREEN, dashed=True)
text(468, 222, "read by", anchor="start")

# --- documentation
lane(560, "Man and documentation generation")
box(20, 582, 206, 60, "fragments", "one file per derived type", GREEN, dashed=True)
box(262, 582, 438, 60, "winnow-args-spec", "stitches them into a command tree")
box(20, 686, 380, 60, "winnow-args-man, winnow-args-markdown", "one page per command")
box(440, 686, 260, 60, "pages", "tool.1  tool-use.1  tool.md", GREEN)
path([(20, 170), (8, 170), (8, 612), (18, 612)], GREEN, dashed=True)
path([(226, 612), (260, 612)])
path([(481, 642), (481, 664), (210, 664), (210, 684)])
path([(400, 716), (438, 716)])

defs = f'''<defs>
<marker id="arrow-n" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" fill="{GREY}"/></marker>
<marker id="arrow-g" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" fill="{GREEN}"/></marker>
</defs>'''
svg = (f'<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="{round(H * 1024 / W)}" viewBox="0 0 {W} {H}" role="img" '
       f'aria-label="winnow-args: build time, run time, and man and documentation generation.">\n'
       f'<rect width="1024" height="{round(H * 1024 / W)}" fill="none"/>\n{defs}\n' + "\n".join(out) + "\n</svg>\n")
open(sys.argv[1], "w").write(svg)

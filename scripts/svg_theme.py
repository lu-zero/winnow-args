"""Palette for the generated figures, scoped to the figure so it works inline or as an image."""

STYLE = """<style>
.wa-figure {
    --ink: #17181a;
    --muted: #8a8a8a;
    --accent: #5fafaf;
    --green: #5faf5f;
    --partial: #87afaf;
    --bad: #d75f5f;
    --warn: #d78787;
}
@media (prefers-color-scheme: dark) {
    .wa-figure {
        --ink: #e6e6e6;
        --muted: #a8a8a8;
        --accent: #7fd1d1;
        --green: #7fd17f;
        --partial: #afd0d0;
        --bad: #ef7f7f;
        --warn: #e7a0a0;
    }
}
.wa-ink { fill: var(--ink); }
.wa-muted { fill: var(--muted); }
.wa-accent { fill: var(--accent); }
.wa-green { fill: var(--green); }
.wa-partial { fill: var(--partial); }
.wa-bad { fill: var(--bad); }
.wa-warn { fill: var(--warn); }
.wa-box-accent { fill: var(--accent); fill-opacity: 0.08; stroke: var(--accent); }
.wa-box-green { fill: var(--green); fill-opacity: 0.08; stroke: var(--green); }
.wa-line-muted { fill: none; stroke: var(--muted); }
.wa-line-green { fill: none; stroke: var(--green); }
.wa-grid { stroke: var(--muted); stroke-opacity: 0.35; }
.wa-arrow-muted { fill: var(--muted); }
.wa-arrow-green { fill: var(--green); }
</style>"""

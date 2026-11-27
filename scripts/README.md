# Scripts

The three figures under `docs/` are generated. Edit the script, then run it:

```sh
scripts/architecture.py docs/architecture.svg
scripts/features.py docs/features.svg
scripts/performance.py docs/performance.svg
```

- `architecture.py`: the diagram of `docs/ARCHITECTURE.md`. Boxes and arrows
  are placed by hand, in a 720 × 770 drawing.
- `features.py`: the coverage graph in the README. It counts the marks in
  [`docs/COMPARISON.md`](../docs/COMPARISON.md), so update that table first.
- `performance.py`: the graph in the README. The times are copied from
  [`benchmarks/README.md`](../benchmarks/README.md); update `PANELS` after a
  new measurement.

The colors for both themes live in `svg_theme.py`, scoped to the figure's root
so the same SVG works as an image and inlined in the site.

All need only Python 3.

## Release image

`docs/release-template.svg` is the 1600 × 900 image for a release post, in the
colors of `Palette::DEFAULT_256`. Replace `{{VERSION}}`, rewrite the four
feature rows and the mock `--help` to match the changelog, then render it:

```sh
sed 's/{{VERSION}}/0.6.5/' docs/release-template.svg | rsvg-convert -w 1600 -o release.png
```

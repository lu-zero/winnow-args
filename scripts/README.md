# Scripts

The two figures under `docs/` are generated. Edit the script, then run it:

```sh
scripts/architecture.py docs/architecture.svg
scripts/performance.py docs/performance.svg
```

- `architecture.py`: the diagram of `docs/ARCHITECTURE.md`. Boxes and arrows
  are placed by hand, in a 720 × 770 drawing.
- `performance.py`: the graph in the README. The times are copied from
  [`benchmarks/README.md`](../benchmarks/README.md); update `PANELS` after a
  new measurement.

Both need only Python 3.

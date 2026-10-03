# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers, with a derive. How to use it is in
[`winnow-args/README.md`](./winnow-args/README.md) and the crate docs; this
page is about the repository.

## Crates

- `winnow-args/`: the runtime, published.
- `winnow-args-derive/`: the derives, published; used through `winnow-args`.
- `bench/` (unpublished): the same command lines in winnow-args, usage, bpaf
  and clap, for `just perf`.
- `xtask/` (unpublished): the generators for the mold port, the examples and
  the mise shadow.

## Examples

```
cargo run --example example -- -vp /tmp
cargo run --example help -- --help
cargo run --example brush_builtins -- set -eu +x -o pipefail a b
cargo run --example ld -- -shared -o out.so --as-needed -lc a.o -z now
```

`brush_builtins` and `ld` print what they parse, and say what to expect with
`--help`. They are generated (`just gen examples BRUSH_DIR MOLD_DIR`) from two ports:

- [brush](https://github.com/reubeno/brush), a bash-compatible shell: every
  builtin and the shell's own command line. Its compatibility suite is
  unchanged; a builtin call costs at most 11 µs over an empty loop, where
  clap's takes up to 127 (`docs/PERF.md`, step 50).
- [mold](https://github.com/rui314/mold), a linker: its whole option set
  behind a feature, its test suite unchanged; a link line parses in a sixth of
  the instructions of mold's own parser.

## Documents

- [`docs/DESIGN.md`](./docs/DESIGN.md): why it is built this way.
- [`docs/CHECKLIST.md`](./docs/CHECKLIST.md): what is done, what is not, and
  where we chose a side.
- [`docs/PERF.md`](./docs/PERF.md): measurements, one entry per feature.
- [`CONTRIBUTING.md`](./CONTRIBUTING.md): building, testing, measuring.

## License

MIT or Apache-2.0, at your option: [`LICENSE-MIT`](./LICENSE-MIT),
[`LICENSE-APACHE`](./LICENSE-APACHE).

# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers, with a derive. How to use it is in
[`winnow-args/README.md`](./winnow-args/README.md) and the crate docs; this
page is about the repository.

## Crates

- `winnow-args/`: the runtime, published.
- `winnow-args-derive/`: the derives, published; used through `winnow-args`.
- `benchmarks/` (unpublished): the same command lines in winnow-args, usage, bpaf
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
  builtin and the shell's own command line, its compatibility suite unchanged.
- [mold](https://github.com/rui314/mold), a linker: its whole option set
  behind a feature, its test suite unchanged.

## Performance

Parse time comes first here: a feature costs nothing to a program that does
not use it, and each one was measured as it was added. The numbers, the host,
and where bpaf is skipped on a line it cannot represent are in
[`benchmarks/`](./benchmarks/README.md). [`docs/PERF.md`](./docs/PERF.md) is
the history of each feature. `just bench-shell` and `just bench-ld` measure
the brush and mold ports.

## Documents

- [`docs/DESIGN.md`](./docs/DESIGN.md): why it is built this way.
- [`docs/CHECKLIST.md`](./docs/CHECKLIST.md): what is done, what is not, and
  where we chose a side.
- [`docs/PERF.md`](./docs/PERF.md): measurements, one entry per feature.
- [`docs/COMPARISON.md`](./docs/COMPARISON.md): features next to usage's,
  bpaf's and clap's.
- [`CONTRIBUTING.md`](./CONTRIBUTING.md): building, testing, measuring.

## License

MIT or Apache-2.0, at your option: [`LICENSE-MIT`](./LICENSE-MIT),
[`LICENSE-APACHE`](./LICENSE-APACHE).

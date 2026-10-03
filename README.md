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
not use it, and each one was measured as it was added. One cold parse, in
instructions, and the warm time of one, on an aarch64 server:

| | `example -v --path /tmp/x a b c` | `mise use -g node@20` (211 commands) |
|---|---:|---:|
| winnow-args | 2 939, 196 ns | 4 012, 328 ns |
| usage | 5 683, 503 ns | 7 720, 782 ns |
| clap | 136 514, 15.3 µs | 4 943 837, 753 µs |
| bpaf 0.9 | 142 994, 15.0 µs | 21 966 400, 2.65 ms |
| bpaf 0.10, pre-release (`844357f`) | 144 538, 17.3 µs | 1 196 466, 166 µs |

The same lines are checked to parse to the same result in all four.
[`benchmarks/`](./benchmarks/README.md) says how this is measured and how to
run it; [`docs/PERF.md`](./docs/PERF.md) keeps the history.

In the ports: a brush builtin call costs at most 11 µs over an empty loop
where clap's takes up to 127, and mold's 2 496-word link line parses in 554
instructions a word against 3 333 for mold's own parser.

## Documents

- [`docs/DESIGN.md`](./docs/DESIGN.md): why it is built this way.
- [`docs/CHECKLIST.md`](./docs/CHECKLIST.md): what is done, what is not, and
  where we chose a side.
- [`docs/PERF.md`](./docs/PERF.md): measurements, one entry per feature.
- [`CONTRIBUTING.md`](./CONTRIBUTING.md): building, testing, measuring.

## License

MIT or Apache-2.0, at your option: [`LICENSE-MIT`](./LICENSE-MIT),
[`LICENSE-APACHE`](./LICENSE-APACHE).

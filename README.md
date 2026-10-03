# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers, with a derive.

winnow-args parses a command line faster than usage, bpaf and clap. It covers
most of what their derives offer, and adds what none of them has built in: the
conventions of shell builtins and of GNU `ld`.

How to use it is in [`winnow-args/README.md`](./winnow-args/README.md) and the
crate docs; this page is about the repository.

## Features

| | winnow-args | usage | clap |
|---|:-:|:-:|:-:|
| Flags, values, positionals, subcommands, global flags, `flatten` | ✓ | ✓ | ✓ |
| Conflicts, requirements, groups | ✓ | ✓ | ✓ |
| Help from doc comments | ✓ | ✓ | ✓ |
| Completion scripts | ✓ | ✓ | ~ |
| Parser generated at compile time, words borrowed | ✓ | ✓ | – |
| Combinators, without a macro | ✓ | – | ~ |
| `+x` options, as a shell builtin takes | ✓ | – | – |
| Long options with one dash, `-z` keywords, as GNU `ld` takes | ✓ | – | – |
| Flags kept in command-line order | ✓ | ~ | ~ |
| Flag spellings and rule selectors checked at compile time | ✓ | – | – |
| Help prose left out of the binary by a feature | ✓ | – | – |
| A CLI built at runtime, external subcommands | – | ✓ | ✓ |
| Conditional requirements (`required_if`) | – | ✓ | ✓ |
| Man pages | – | ✓ | ~ |
| "Did you mean" suggestions | – | – | ✓ |

`✓` built in · `~` with some work, or in a companion crate · `–` not offered.
[`docs/COMPARISON.md`](./docs/COMPARISON.md) has every row, and bpaf.

## Performance

Two command lines: `example -v --path /tmp/x a b c`, and `mise use -g node@20`
with mise's 211 commands declared. Instructions for one parse in a fresh
process, and time per parse in a hot loop, on an Ampere-1a, release build, one
core.

| | example, instructions | time | mise, instructions | time |
|---|---:|---:|---:|---:|
| winnow-args | 2 939 | 196 ns | 4 012 | 328 ns |
| usage | 5 683 | 503 ns | 7 720 | 782 ns |
| clap | 136 514 | 15.3 µs | 4 943 837 | 753 µs |
| bpaf 0.9 | 142 994 | 15.0 µs | 21 966 400 | 2.65 ms |

usage and winnow-args generate their parser at compile time; clap and bpaf
build theirs at each start, which grows with the number of commands. Parse
time comes first here: a feature costs nothing to a program that does not use
it, and each one was measured as it was added.

The method, the sizes and where bpaf is skipped on a line it cannot represent
are in [`benchmarks/`](./benchmarks/README.md). [`docs/PERF.md`](./docs/PERF.md)
is the history of each feature. `just bench-shell` and `just bench-ld` measure
the brush and mold ports.

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

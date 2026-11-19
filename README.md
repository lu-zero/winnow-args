# winnow-args

[![LICENSE](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Crates.io](https://img.shields.io/crates/v/winnow-args.svg)](https://crates.io/crates/winnow-args)
[![docs.rs](https://docs.rs/winnow-args/badge.svg)](https://docs.rs/winnow-args)
[![CI](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml/badge.svg)](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/lu-zero/winnow-args/status.svg)](https://deps.rs/repo/github/lu-zero/winnow-args)

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers, with a derive.

winnow-args parses a command line faster than usage, bpaf and clap. It covers
most of what their derives offer, and adds what none of them has built in: the
conventions of shell builtins and of GNU `ld`.

How to use it is in [`winnow-args/README.md`](./winnow-args/README.md) and the
crate docs; this page is about the repository.

## Features

The [crate README](./winnow-args/README.md) lists what the derive and the
combinators support. [`docs/COMPARISON.md`](./docs/COMPARISON.md) has every row
of that matrix, including bpaf.

## Performance

Two command lines: `example -v --path /tmp/x a b c`, for
[a small CLI](./benchmarks/src/lib.rs) written once per framework, and
`mise use -g node@20` with [mise's 211 commands](./benchmarks/shadows/)
declared. Instructions for one parse in a fresh process, and time per parse in
a hot loop.

Test hardware: an Ampere-1a desktop (aarch64, 128 cores, 3.4 GHz), `release`
profile, pinned to one core, rustc/cargo 1.100.0.

![Time per parse for the two command lines, log scale](./docs/performance.svg)

| | example, instructions | time | mise, instructions | time |
|---|---:|---:|---:|---:|
| winnow-args | 2 962 | 207 ns | 4 164 | 365 ns |
| usage 7 | 5 233 | 494 ns | 7 408 | 782 ns |
| clap 4 | 134 882 | 15.0 µs | 4 886 044 | 755 µs |
| bpaf 0.9 | 142 110 | 14.9 µs | 21 866 995 | 2.63 ms |

Parse time comes first here: a feature costs nothing to a program that does
not use it, and each one was measured as it was added.

The method, the sizes and why clap and bpaf build their parser at each start
are in [`benchmarks/`](./benchmarks/README.md), which also says where bpaf is
skipped on a line it cannot represent. [`docs/PERF.md`](./docs/PERF.md) is the
history of each feature. `just bench-shell` and `just bench-ld` measure the
brush and mold ports.

## Crates

- `winnow-args/`: the runtime, published.
- `winnow-args-derive/`: the derives, published; used through `winnow-args`.
- `winnow-args-spec/`, `winnow-args-man/`, `winnow-args-markdown/`: a derived
  command read back from what the derive writes at compile time, and its man
  and markdown pages. [`examples/`](./examples/README.md) shows the generator.
- `benchmarks/` (unpublished): the same command lines in winnow-args, usage, bpaf
  and clap, for `just perf`.
- `xtask/` (unpublished): the generators for the mold port, the examples and
  the mise shadow.
- `scripts/`: the generators of the two figures under `docs/`.

[`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) shows how they fit together.

## Examples

```
cargo run --example example -- -vp /tmp
cargo run --example help -- --help
cargo run --example brush_builtins -- set -eu +x -o pipefail a b
cargo run --example ld -- -shared -o out.so --as-needed -lc a.o -z now
```

The sources are in [`examples/`](./examples/). `brush_builtins` and `ld` print
what they parse, and say what to expect with `--help`. They are generated
(`just gen examples BRUSH_DIR MOLD_DIR`) from two ports:

- [brush](https://github.com/reubeno/brush), a bash-compatible shell: every
  builtin and the shell's own command line, its compatibility suite unchanged.
- [mold](https://github.com/rui314/mold), a linker: its whole option set
  behind a feature, its test suite unchanged.

## Documents

- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md): an overview of the code, with a
  diagram.
- [`docs/DESIGN.md`](./docs/DESIGN.md): why it is built this way.
- [`docs/CHECKLIST.md`](./docs/CHECKLIST.md): what is done, what is not, and
  where we chose a side.
- [`docs/PERF.md`](./docs/PERF.md): measurements, one entry per feature.
- [`docs/COMPARISON.md`](./docs/COMPARISON.md): features next to usage's,
  bpaf's and clap's.
- [`CHANGELOG.md`](./CHANGELOG.md): what changed in each release.
- [`CONTRIBUTING.md`](./CONTRIBUTING.md): building, testing, measuring.

## License

MIT or Apache-2.0, at your option: [`LICENSE-MIT`](./LICENSE-MIT),
[`LICENSE-APACHE`](./LICENSE-APACHE).

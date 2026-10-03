# Benchmarks

What a command line costs to parse in winnow-args, [usage](https://github.com/jdx/usage),
[bpaf](https://github.com/pacak/bpaf) and [clap](https://github.com/clap-rs/clap),
measured the way usage measures its own parser. The package is `bench`,
unpublished.

## What is compared

- **A small CLI**, `example`, with a bit of everything: counts, values,
  negation, optional values, a subcommand, positionals. It is written once per
  framework in `src/lib.rs`: the winnow-args derive (`wa`), a hand-written
  `dispatch!` (`wa-disp`), the combinators (`wa-comb`), usage, bpaf and clap.
  The lines parsed are `argv.txt`.
- **mise's whole CLI**, 211 commands, as usage's generator declares it for
  each framework (`shadows/`, vendored from usage with its license; ours,
  `mise-wa`, is translated from usage's by `just gen mise-shadow`). The lines
  are `mise-argv.txt`.

`tests/` holds every framework to the same answer on every line, so the
numbers compare parsers that do the same thing. Where one cannot (bpaf has no
hyphen values, and cannot route words after `--` past a greedy positional),
the test skips it on those lines and says why.

## What is measured

Each framework has a `parse-n-*` binary that parses its arguments `PARSE_N`
times. A CLI parses once per process, so setup counts: clap and bpaf build
their parser inside the loop.

- **Instructions, cold**: the binary run with `PARSE_N=1` minus `PARSE_N=0`,
  one parse in a fresh process. Counted by cachegrind where valgrind runs, by
  `perf stat` (median of 31 runs) otherwise.
- **Time, cold**: the first parse of a fresh process, median of 31 processes.
- **Time, warm**: per parse in a hot loop, minimum and median over 2 000
  rounds (`time-sweep`); noise only ever adds time, so the minimum is the
  estimate.
- **Size**: the binary, stripped.

Instruction counts are the stable measure; times move with the machine.

## Running

```
just perf                           # every line of argv.txt, one table each
just perf -v --path /tmp/x a b c    # one line
SUITE=mise just perf                # mise's CLI, every line of mise-argv.txt
PROFILE=release-lto just perf       # one codegen unit and fat LTO
BPAF010=1 just perf                 # with the unreleased bpaf 0.10 as well
numactl -N 3 -m 3 taskset -c 96 just perf   # pinned, as the results below
```

`bpaf010/` is a crate of its own, outside the workspace: bpaf 0.10 is not
released and comes from git, pinned to commit `844357f`, so only `BPAF010=1`
builds it.

`just bench-examples`, `bench-shell` and `bench-ld` measure the two examples
and the brush and mold ports; they are described in the `justfile`.

## Results

An Ampere-1a (aarch64), `release` profile, pinned to one core, `perf stat`,
winnow 1.0.4, bpaf 0.9, clap 4. Cold instructions, warm time, stripped size.

`example -v --path /tmp/x a b c`:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 2 939 | 0.5 | 196 ns | 377 KB |
| winnow-args, `dispatch!` | 3 276 | 0.6 | 251 ns | 376 KB |
| winnow-args, combinators | 4 574 | 0.8 | 396 ns | 385 KB |
| usage | 5 683 | 1 | 503 ns | 394 KB |
| clap | 136 514 | 24 | 15.3 µs | 763 KB |
| bpaf 0.9 | 142 994 | 25 | 15.0 µs | 582 KB |
| bpaf 0.10, pre-release (`844357f`) | 144 538 | 25 | 17.3 µs | 825 KB |

`mise use -g node@20`, 211 commands declared:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 4 012 | 0.5 | 328 ns | 1.72 MB |
| usage | 7 720 | 1 | 782 ns | 1.18 MB |
| bpaf 0.10, pre-release (`844357f`) | 1 196 466 | 154 | 166 µs | 2.97 MB |
| clap | 4 943 837 | 640 | 753 µs | 2.24 MB |
| bpaf 0.9 | 21 966 400 | 2 845 | 2.65 ms | 1.95 MB |

usage and winnow-args generate their parser at compile time; clap and bpaf
build theirs at each start, which is what grows with the number of commands.
winnow-args' binary is larger than usage's at mise's scale because each
struct's loop is its own code.

[`docs/PERF.md`](../docs/PERF.md) has every line, and what each feature cost
when it was added.

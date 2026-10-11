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

Test hardware: an Ampere-1a desktop (aarch64, 128 cores, 3.4 GHz). `release`
profile, pinned to one core, `perf stat`, winnow 1.0.4, usage 7.0, bpaf 0.9, clap 4. Cold
instructions, warm time, stripped size.

`example -v --path /tmp/x a b c`:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 2 896 | 0.5 | 187 ns | 373 KB |
| winnow-args, `dispatch!` | 3 279 | 0.6 | 252 ns | 369 KB |
| winnow-args, combinators | 4 491 | 0.8 | 391 ns | 377 KB |
| usage | 5 682 | 1 | 516 ns | 382 KB |
| clap 4 | 134 820 | 24 | 15.0 µs | 752 KB |
| bpaf 0.9 | 141 614 | 25 | 14.8 µs | 574 KB |
| bpaf 0.10, pre-release (`844357f`) | 143 770 | 25 | 17.2 µs | 812 KB |

`mise use -g node@20`, 211 commands declared:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 3 997 | 0.5 | 327 ns | 1.74 MB |
| usage | 7 391 | 1 | 767 ns | 1.16 MB |
| bpaf 0.10, pre-release (`844357f`) | 1 187 954 | 161 | 164 µs | 2.95 MB |
| clap 4 | 4 883 364 | 661 | 757 µs | 2.23 MB |
| bpaf 0.9 | 21 878 285 | 2 960 | 2.65 ms | 1.94 MB |

usage and winnow-args generate their parser at compile time; clap and bpaf
build theirs at each start, which is what grows with the number of commands.
winnow-args' binary is larger than usage's at mise's scale because each
struct's loop is its own code.

[`docs/PERF.md`](../docs/PERF.md) has every line, and what each feature cost
when it was added.

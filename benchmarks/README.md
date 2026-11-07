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
- **A CLI with rules**, `export`: requirements that depend on another flag
  (`required_if`), one that depends on two (`required_unless_all`), a default
  chosen by another flag (`default_if`), and a check of the parsed values
  (`validate`). It is `src/rules.rs`, in winnow-args, usage and clap; bpaf has
  none of these rules built in. The lines are `rules-argv.txt`.

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
SUITE=rules just perf               # the CLI with rules, every line of rules-argv.txt
PROFILE=release-lto just perf       # one codegen unit and fat LTO
BPAF010=1 just perf                 # with the unreleased bpaf 0.10 as well
numactl -N 2 -m 2 taskset -c 64 just perf   # pinned, as the results below
```

`bpaf010/` is a crate of its own, outside the workspace: bpaf 0.10 is not
released and comes from git, pinned to commit `844357f`, so only `BPAF010=1`
builds it.

`just bench-examples`, `bench-shell` and `bench-ld` measure the two examples
and the brush and mold ports; they are described in the `justfile`.

## Results

Test hardware: an Ampere-1a desktop (aarch64, 128 cores, 3.4 GHz). `release`
profile, pinned to one core, `perf stat`, winnow 1.0.4, usage 7.3, bpaf 0.9, clap 4.6. Cold
instructions, warm time, stripped size.

`example -v --path /tmp/x a b c`:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 2 896 | 0.6 | 184 ns | 380 KB |
| winnow-args, `dispatch!` | 3 171 | 0.6 | 249 ns | 370 KB |
| winnow-args, combinators | 4 428 | 0.8 | 399 ns | 379 KB |
| usage | 5 227 | 1 | 505 ns | 371 KB |
| clap 4 | 135 612 | 26 | 15.2 µs | 751 KB |
| bpaf 0.9 | 141 555 | 27 | 14.7 µs | 571 KB |
| bpaf 0.10, pre-release (`844357f`) | 143 786 | 28 | 17.2 µs | 812 KB |

`mise use -g node@20`, 211 commands declared:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 3 968 | 0.5 | 322 ns | 1.72 MB |
| usage | 7 306 | 1 | 826 ns | 1.14 MB |
| bpaf 0.10, pre-release (`844357f`) | 1 187 794 | 163 | 164 µs | 2.95 MB |
| clap 4 | 4 883 517 | 668 | 760 µs | 2.23 MB |
| bpaf 0.9 | 21 878 142 | 2 995 | 2.66 ms | 1.94 MB |

usage and winnow-args generate their parser at compile time; clap and bpaf
build theirs at each start, which is what grows with the number of commands.
winnow-args' binary is larger than usage's at mise's scale because each
struct's loop is its own code.

`export --format json --schema s.json --target t`, the rules CLI:

| framework | instructions | × usage | warm | size |
|---|---:|---:|---:|---:|
| winnow-args, derive | 1 378 | 0.3 | 139 ns | 370 KB |
| usage | 4 036 | 1 | 343 ns | 359 KB |
| clap 4 | 76 273 | 19 | 8.25 µs | 708 KB |

[`docs/PERF.md`](../docs/PERF.md) has every line, and what each feature cost
when it was added.

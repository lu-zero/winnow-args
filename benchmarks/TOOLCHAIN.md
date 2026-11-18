# Re-measuring with rustc 1.100.0

The tables in [README.md](./README.md) were last refreshed for usage 7.3 with
rustc/cargo 1.99.1. This note records the re-measurement with the default
toolchain, rustc/cargo 1.100.0, on the same host and with the same dependency
versions (winnow 1.0.4, usage 7.3, bpaf 0.9, clap 4.6.7), then the update of
usage to 7.4.0 and clap to 4.6.8 on the same toolchain. The published numbers
are those of the last run; `docs/PERF.md` keeps the older per-feature
history as it was measured at the time.

## Method

All runs use the command the results section documents:

```
numactl -N 2 -m 2 taskset -c 64 just perf -v --path /tmp/x a b c
SUITE=mise numactl -N 2 -m 2 taskset -c 64 just perf use -g node@20
SUITE=rules numactl -N 2 -m 2 taskset -c 64 just perf --format json --schema s.json --target t
```

Each number is cold instructions (`PARSE_N=1` minus `PARSE_N=0`, `perf stat`
median of 31) and warm time (minimum over 2 000 rounds). To isolate the
compiler, the same line was also measured with
`RUSTUP_TOOLCHAIN=1.99.1 numactl -N 2 -m 2 taskset -c 64 just perf …`, twice per
toolchain, on the current tree.

## What changed

`example -v --path /tmp/x a b c`, warm time on one pinned core:

| framework | 1.99.1 instr | 1.100.0 instr | 1.99.1 warm | 1.100.0 warm | change |
|---|---:|---:|---:|---:|---:|
| usage | 5 150–5 292 | 5 205–5 265 | 515–521 ns | 504–505 ns | flat |
| winnow-args, derive | 2 813–2 873 | 2 875–2 900 | 185–186 ns | 194–195 ns | +9 ns |
| winnow-args, `dispatch!` | 3 125–3 181 | 3 332–3 339 | 247–248 ns | 271–274 ns | +25 ns |
| winnow-args, combinators | 4 385–4 445 | 4 498–4 541 | 393–402 ns | 417–426 ns | +24 ns |
| bpaf 0.9 | ~141 500 | ~142 000 | 14.9 µs | 15.0 µs | flat |
| clap 4 | ~135 600 | ~134 900 | 15.4 µs | 15.1 µs | flat |

Instruction counts stay within the recorded ±150 noise. The warm shifts are
limited to the three winnow-args parsers, where the generated loop is larger
than the others: the new compiler emits code that runs a few percent slower
there. usage, bpaf and clap are unchanged within run-to-run variation, so this
is a code-generation effect rather than a change in what any parser does.

An A/B of `v0.6.3` against the 0.6.4 work, both built with 1.100.0 in the same
target directory, was flat or slightly faster for winnow-args (2 900 / 201 ns
before, 2 868 / 195 ns after). None of the 0.6.4 commits caused the warm-time
shift.

## Competitor update

usage 7.3.0 → 7.4.0 and clap 4.6.7 → 4.6.8, with rustc 1.100.0 and winnow
1.0.4 in both runs, built in the same target directory. Instructions, warm
time:

| line | framework | before | after |
|---|---|---:|---:|
| example | usage | 5 341, 494 ns | 5 233, 494 ns |
| example | clap | 134 907, 14.9 µs | 134 882, 15.0 µs |
| mise | usage | 7 412, 782 ns | 7 408, 782 ns |
| mise | clap | 4 886 088, 754 µs | 4 886 044, 755 µs |
| rules | usage | 4 044, 347 ns | 3 994, 349 ns |
| rules | clap | 75 904, 8.21 µs | 75 826, 8.25 µs |

Both are flat within the noise, usage's example line aside, which is 2% fewer
instructions. The winnow-args times of this run are a few percent above those
of the 1.100.0 run, which the host being busy building probably explains, so
read the times as an upper bound. The published tables are from this run.

## New published results

`example -v --path /tmp/x a b c`:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 2 896 | 2 962 | 184 ns | 207 ns |
| winnow-args, `dispatch!` | 3 171 | 3 360 | 249 ns | 276 ns |
| winnow-args, combinators | 4 428 | 4 641 | 399 ns | 428 ns |
| usage | 5 227 | 5 233 | 505 ns | 494 ns |
| clap 4 | 135 612 | 134 882 | 15.2 µs | 15.0 µs |
| bpaf 0.9 | 141 555 | 142 110 | 14.7 µs | 14.9 µs |
| bpaf 0.10, pre-release (`844357f`) | 143 786 | 143 656 | 17.2 µs | 17.2 µs |

`mise use -g node@20`, 211 commands declared:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 3 968 | 4 164 | 322 ns | 365 ns |
| usage | 7 306 | 7 408 | 826 ns | 782 ns |
| bpaf 0.10, pre-release (`844357f`) | 1 187 794 | 1 187 739 | 164 µs | 164 µs |
| clap 4 | 4 883 517 | 4 886 044 | 760 µs | 755 µs |
| bpaf 0.9 | 21 878 142 | 21 866 995 | 2.66 ms | 2.63 ms |

`export --format json --schema s.json --target t`, the rules CLI:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 1 378 | 1 529 | 139 ns | 145 ns |
| usage | 4 036 | 3 994 | 343 ns | 349 ns |
| clap 4 | 76 273 | 75 826 | 8.25 µs | 8.25 µs |

The ratios and stripped sizes in [README.md](./README.md) are from the same
last run. Times move with the machine and the compiler; quote both when a
number is compared later.

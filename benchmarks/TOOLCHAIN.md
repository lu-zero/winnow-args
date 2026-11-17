# Re-measuring with rustc 1.100.0

The tables in [README.md](./README.md) were last refreshed for usage 7.3 with
rustc/cargo 1.99.1. This note records the re-measurement with the default
toolchain, rustc/cargo 1.100.0, on the same host and with the same dependency
versions (winnow 1.0.4, usage 7.3, bpaf 0.9, clap 4.6). The published numbers
are updated to the 1.100.0 run; `docs/PERF.md` keeps the older per-feature
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

## New published results

`example -v --path /tmp/x a b c`:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 2 896 | 2 875 | 184 ns | 195 ns |
| winnow-args, `dispatch!` | 3 171 | 3 339 | 249 ns | 274 ns |
| winnow-args, combinators | 4 428 | 4 498 | 399 ns | 417 ns |
| usage | 5 227 | 5 265 | 505 ns | 504 ns |
| clap 4 | 135 612 | 134 855 | 15.2 µs | 15.1 µs |
| bpaf 0.9 | 141 555 | 142 090 | 14.7 µs | 15.0 µs |
| bpaf 0.10, pre-release (`844357f`) | 143 786 | 143 656 | 17.2 µs | 17.2 µs |

`mise use -g node@20`, 211 commands declared:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 3 968 | 3 919 | 322 ns | 334 ns |
| usage | 7 306 | 7 197 | 826 ns | 770 ns |
| bpaf 0.10, pre-release (`844357f`) | 1 187 794 | 1 187 739 | 164 µs | 164 µs |
| clap 4 | 4 883 517 | 4 884 524 | 760 µs | 760 µs |
| bpaf 0.9 | 21 878 142 | 21 866 919 | 2.66 ms | 2.63 ms |

`export --format json --schema s.json --target t`, the rules CLI:

| framework | old instr | new instr | old warm | new warm |
|---|---:|---:|---:|---:|
| winnow-args, derive | 1 378 | 1 479 | 139 ns | 147 ns |
| usage | 4 036 | 4 001 | 343 ns | 338 ns |
| clap 4 | 76 273 | 75 804 | 8.25 µs | 8.23 µs |

The ratios and stripped sizes in [README.md](./README.md) are from the same
1.100.0 run. Times move with the machine and the compiler; quote both when a
number is compared later.

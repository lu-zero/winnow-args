# Performance log

One entry per feature. Each adds the feature to all five bench CLIs, re-runs
every line in `bench/argv.txt`, and keeps the earlier lines so the cost of
merely *carrying* a feature shows up next to the cost of *using* it.
`bench/tests/agree.rs` checks every framework parses every line to the same fields.

Method (`tasks/perf.sh`, aarch64 host):

- **instr**: one cold parse, `PARSE_N=1` minus `PARSE_N=0`, `perf stat -e
  instructions:u` median of 31 runs (valgrind can't run here); ±~150 noise.
- **cold ns**: first parse in a fresh process, median of 31 processes; ±~300 noise.
- **warm ns**: in-process min over 2000 rounds (`time-sweep`); ±~5 noise.

Compare ratios and deltas larger than the noise, not individual digits.

## 0. Flags: `-v/--verbose -p/--path=PATH`

`-v --path /tmp/x`

| framework | instr | cold ns | warm ns |
|-----------|------:|--------:|--------:|
| usage     | 1360  | 3080    | 142     |
| wa        | 598   | 1200    | 46      |
| wa-comb   | 1156  | 1460    | 169     |
| bpaf 0.10 | 31304 | 46761   | 3436    |
| clap 4    | 24535 | 22120   | 2558    |

## 1. Positionals: `[FILE]...`

Required, optional and one trailing `Vec` positional, in declaration order,
interleaved with flags. The derive tracks the next positional with one counter
and matches words against it.

`-v --path /tmp/x` (carrying the feature, not using it)

| framework | instr | cold ns | warm ns | Δ warm vs 0 |
|-----------|------:|--------:|--------:|------------:|
| usage     | 1410  | 2440    | 143     | +1          |
| wa        | 442   | 1340    | 45      | −1          |
| wa-comb   | 1346  | 1840    | 183     | +14         |
| bpaf 0.10 | 34778 | 76201   | 3866    | +430        |
| clap 4    | 26402 | 26660   | 2835    | +277        |

`-v --path /tmp/x a b c`

| framework | instr | cold ns | warm ns |
|-----------|------:|--------:|--------:|
| usage     | 3943  | 4180    | 328     |
| wa        | 1900  | 2420    | 135     |
| wa-comb   | 3494  | 2220    | 422     |
| bpaf 0.10 | 47725 | 70501   | 5335    |
| clap 4    | 38740 | 30920   | 3882    |

The derive pays nothing to carry positionals. The combinators pay ~14 ns per
parse for one more `alt` branch, which every word and the final end-of-line
attempt try. Using them costs about 30 ns per `PathBuf` word for wa, most of
it the allocation; usage pays about 60 ns.

## 2. Counting switch: `-v/--verbose...`

`verbose` becomes a count in every framework (`#[arg(count)]`, clap `ArgAction::Count`,
bpaf `req_flag(()), count`, usage `count`). The derive's slot is the integer
itself, incremented with `saturating_add`; the combinators fold `switch()`.

| framework | `-v --path /tmp/x` | Δ vs 1 | `… a b c` | Δ vs 1 | `-vvv --path /tmp/x` |
|-----------|-------------------:|-------:|----------:|-------:|---------------------:|
| usage     | 143 | 0    | 330  | +2   | 201  |
| wa        | 49  | +4   | 131  | −4   | 64   |
| wa-comb   | 181 | −2   | 418  | −4   | 212  |
| bpaf 0.10 | 4645| +779 | 6117 | +782 | 6842 |
| clap 4    | 2735| −100 | 3942 | +60  | 3510 |

Warm ns (min). Instructions and cold times are in the `tasks/perf.sh` output
and track the same way: wa 553 / 1840 / 726 instr against usage's 1382 / 3938 / 1953.

Only bpaf pays to carry a count (~780 ns, its `req_flag(()).count()` repeat
machinery); the rest is noise. Each extra letter in `-vvv` costs wa ~8 ns,
the combinators ~16, usage ~29, clap ~390, bpaf ~1100.

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

## 3. Repeatable options: `-I/--include=DIR...`

A `Vec<T>` flag collects one value per occurrence, in order (clap and bpaf
infer it from `Vec`; usage says `var`). The derive pushes into the slot; the
combinators fold `argument_as()` into a `Vec`.

| framework | `-v --path /tmp/x` | Δ vs 2 | `… a b c` | Δ vs 2 | `-vvv …` | `… -I a -I b -I c` |
|-----------|------:|-----:|-----:|-----:|-----:|-----:|
| usage     | 157   | +14  | 342  | +12  | 210  | 360  |
| wa        | 53    | +4   | 136  | +5   | 69   | 165  |
| wa-comb   | 226   | +45  | 567  | +149 | 254  | 525  |
| bpaf 0.10 | 5518  | +873 | 6917 | +800 | 7658 | 7869 |
| clap 4    | 2978  | +243 | 4049 | +107 | 3858 | 5196 |

Warm ns (min); three repeated sweeps agreed within 1 ns, so the carrying costs
are real, not drift. Instructions track the same way (wa 631 / 1905 / 812 / 2188
against usage's 1441 / 3997 / 1985 / 4437).

The combinators now show their structural cost: every word goes through each
flag branch of the `alt`, each of which lexes, fails, and resets, before
reaching the positional. `… a b c` got 149 ns slower from one more flag,
~50 ns per word. The derive's `match` does not grow with the flag count.

## 4. Combinators: `dispatch!` on the item kind

Not a feature: the combinator CLI now classifies each item once with
`token::kind` (a peek, no consumption) and `dispatch!`es it, so a word only
meets the positional parser and a flag only the flag `alt`.

| line | wa-comb before (3) | wa-comb after | Δ | wa derive |
|------|------:|------:|------:|------:|
| `-v --path /tmp/x`             | 226 | 134 | −92  | 52  |
| `… a b c`                      | 567 | 230 | −337 | 141 |
| `-vvv --path /tmp/x`           | 254 | 180 | −74  | 68  |
| `… -I a -I b -I c`             | 525 | 455 | −70  | 167 |

Warm ns (min). Instructions: 1140 / 2705 / 1603 / 4120 (were 1510 / 4340 / 1934 / 4341).

Words are now cheap. Flags still pay per preceding branch: every `-I` lexes and
fails `VERBOSE` and `PATH` before `INCLUDE` matches, which is why the last line
barely moved. The derive remains ~2.5× cheaper there.

## 5. Full dispatch on flag names, and three layout fixes

`token::arg` now returns an `Arg` whose variants carry their offset and whose
fields are plain (`LongFlag { name: &[u8], .. }`, `ShortFlag { letter, .. }`),
so one `dispatch!` can match both spellings of a flag as a pattern (`wa-disp`).
Its first version made **everything slower despite executing fewer
instructions**; `stall_backend` showed why, and three fixes followed:

| change | wa derive, `… a b c` | cycles / parse | backend stalls / parse |
|--------|------:|------:|------:|
| before (step 4)                                   | 130 | 452 | 19  |
| `Arg` grows to 48 bytes, `arg` not `#[inline]`    | 187 | 662 | 207 |
| `#[inline]` on `token::arg`                       | 145 | ~520 | ~100 |
| continuations as `&self` methods, not closures    | 117 | ~395 | ~23 |
| `Argv` counters `u32`: 48 → 32 bytes              | 116 | ~430 | ~32 |

Warm ns (min); cycles and stalls from `perf stat` over 1M warm parses, ±15%.
The stalls are store-to-load forwarding failures: a value written field by field
(the `Result<Arg, Error>` returned through memory, a closure capturing the
enum, an `Argv` checkpoint) read back at once with wider loads. The last row
mostly helps the combinators, whose `repeat`/`alt` copy `Argv` per item
(stalls ~136 → ~65).

| framework | `-v --path /tmp/x` | `… a b c` | `-vvv …` | `… -I a -I b -I c` |
|-----------|------:|------:|------:|------:|
| usage     | 157   | 341   | 209   | 366   |
| wa        | 46    | 115   | 57    | 130   |
| wa-disp   | 99    | 196   | 131   | 230   |
| wa-comb   | 125   | 216   | 175   | 415   |
| bpaf 0.10 | 5449  | 6872  | 7594  | 7752  |
| clap 4    | 3031  | 4140  | 3789  | 5238  |

Warm ns (min). Against step 4 the derive gained 6–37 ns per line. `wa-disp`
costs the same per flag however many flags there are; `wa-comb` still pays per
preceding flag (the `-I` line). What separates `wa-disp` from the derive is
winnow's `repeat`/`alt` bookkeeping in `args`, not the matching.

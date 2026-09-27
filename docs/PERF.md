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

## 6. Subcommands: `[use -g/--global [TOOL]...]`

`#[arg(subcommand)]` holds a `#[derive(Subcommand)]` enum; a word selects it
only before any positional is filled and never after `--`, and the child parses
the rest of the line. bpaf 0.10 needed the command field *before* its greedy
`FILE` positional, or it took `use` as a file.

| framework | `-v --path /tmp/x` | Δ vs 5 | `… a b c` | Δ vs 5 | `-vvv …` | `-I …` | `… use -g node@20` |
|-----------|------:|------:|------:|------:|------:|------:|------:|
| usage     | 160   | +3    | 353   | +12   | 216   | 371   | 344   |
| wa        | 49    | +3    | 119   | +4    | 59    | 134   | 109   |
| wa-disp   | 106   | +7    | 205   | +9    | 144   | 243   | 220   |
| wa-comb   | 133   | +8    | 254   | +38   | 188   | 430   | 251   |
| bpaf 0.10 | 6169  | +720  | 7872  | +1000 | 8326  | 8501  | 9633  |
| clap 4    | 4191  | +1160 | 5225  | +1085 | 5041  | 6335  | 7137  |

Warm ns (min). Cold ns was unusually noisy this run for everyone (usage
3.9–8.6 µs), so this entry leans on warm time and instructions (wa 487 / 1664
/ 593 / 1955 / 1386 against usage's 1518 / 4100 / 2027 / 4415 / 3680).

The first measurement showed wa ~30 ns slower on every line *with or without*
the subcommand field. Adding `Word::after_separator` had pushed `token::arg`
past rustc's inlining threshold, which brought back the step-5 stall. It is now
`#[inline(always)]`; warm per-parse on `… a b c` is 1315 instructions, ~412
cycles and ~24 stall cycles, against 1282 / ~434 / ~35 before the feature.
Routing itself costs the derive ~15–30 instructions per parse: one inlined
`matches!` of the word against the enum's names (`Subcommand::has`).

The combinators' `cond(files.is_empty(), command("use", …))` branch is tried
by every word, hence `wa-comb`'s +38 on `… a b c`; `wa-disp` does the same
check in a match guard.

## 7. Global flags: `-v/--verbose` becomes `global`

A parent's `global` flag is accepted after its subcommand word at any depth,
and a subcommand's own declaration of the same name wins. The parent hands its
subcommand a `Globals` handler (a closure over its global fields, then its own
parent's handler); the subcommand offers it only flags it does not declare, so
recognised flags never touch it. bpaf's derive has no `global`, so its field is
`external(short('v')…count().global())`.

| framework | `-v --path /tmp/x` | `… a b c` | `-vvv …` | `-I …` | `… use -g node@20` | Δ vs 6 | `--path /tmp/x use -v -g node@20` |
|-----------|------:|------:|------:|------:|------:|------:|------:|
| usage     | 160   | 362   | 215   | 366   | 351   | +7    | 377   |
| wa        | 48    | 123   | 56    | 135   | 118   | +9    | 118   |
| wa-disp   | 104   | 204   | 142   | 244   | 231   | +11   | 230   |
| wa-comb   | 133   | 272   | 185   | 425   | 261   | +10   | 281   |
| bpaf 0.10 | 6080  | 7863  | 8147  | 8570  | 9684  | +51   | 9728  |
| clap 4    | 4749  | 5822  | 5482  | 6789  | 8736  | +1599 | 8688  |

Warm ns (min); three repeated sweeps of the `use` line agreed at 118.

Lines without a subcommand did not move. Entering one costs wa ~9 ns more:
the child's `parse_argv_with` now takes the handler and stays out of line
(inlining every subcommand into its parent would not scale to mise's 211).
Binding a global through the handler is free next to that: the global line
costs the same as the local one. clap pays ~0.6–1.6 µs for `global = true`
everywhere, since it propagates the argument into every subcommand at build time.

## 8. Aliases: `--path` / `--dir`, `use` / `u`

Long flags and subcommands take `alias = "…"` or `alias("…", …)`; mise has 77
aliased names. The derive adds them to the same `match` arm as or-patterns
(`b"path" | b"dir" =>`); the combinators take `short('p').longs(["path", "dir"])`
and `command(["use", "u"], …)`.

| framework | `-v --path /tmp/x` | Δ vs 7 | `-v --dir /tmp/x` | `… use -g node@20` | `… u -g node@20` | `-I …` |
|-----------|------:|-----:|------:|------:|------:|------:|
| usage     | 159   | −1   | 158   | 342   | 343   | 362   |
| wa        | 48    | 0    | 47    | 116   | 114   | 138   |
| wa-disp   | 104   | 0    | 104   | 231   | 232   | 246   |
| wa-comb   | 134   | +1   | 133   | 265   | 261   | 411   |
| bpaf 0.10 | 6431  | +351 | 6353  | 10049 | 10007 | 8924  |
| clap 4    | 4782  | +33  | 4755  | 8959  | 8961  | 6905  |

Warm ns (min). An alias costs exactly what the name costs, everywhere.

The first combinator version put the aliases in `Named` as a slice, taking it
from 24 to 40 bytes, and `wa-comb` got 20–25 % slower on every line (`-I …`
425 → 526 ns, backend stalls ~300 → ~600 per parse): `dispatch!` rebuilds its
arm parsers per item, each copying `Named`. A 32-byte enum layout did not help
(the separately written tag is its own forwarding hazard); only the old
24-byte layout did. `Named<const N: usize = 1>` keeps it: a plain `Named` is
the old 24 bytes, and only a flag with aliases is a larger `Named<2>`.

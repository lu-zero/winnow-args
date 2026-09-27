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

## 9. Value delimiters: `-I/--include=DIR[,DIR]...`

`delimiter = ','` on a `Vec` flag or positional splits each value; every piece
is converted on its own and a positional piece keeps its own offset. The derive
splits inline and pushes into the field; `Named::arguments_as(b',')` returns a
`Vec` per occurrence; a `dispatch!` arm can loop over `token::split` instead.
bpaf has no delimiter, so its field splits after `many()`.

| framework | `-v --path /tmp/x` | `-vvv …` | `-I a -I b -I c` | Δ vs 8 | `-I a,b,c` | `… use -g node@20` |
|-----------|------:|------:|------:|------:|------:|------:|
| usage     | 161   | 212   | 378   | +16   | 286   | 364   |
| wa        | 55    | 67    | 136   | −2    | 122   | 123   |
| wa-disp   | 103   | 140   | 243   | −3    | 200   | 228   |
| wa-comb   | 134   | 188   | 545   | +134  | 340   | 258   |
| bpaf 0.10 | 6536  | 8537  | 9383  | +459  | 7511  | 9993  |
| clap 4    | 4832  | 5723  | 7236  | +331  | 6164  | 9056  |

Warm ns (min). One word with three values is cheaper than three occurrences
for everyone.

Two things moved that the feature did not ask for:

- The larger generated loop pushed `Arg::check_switch` and `Arg::read_value`
  out of line, so every letter of `-vvv` became a call (56 → 69 ns). Both are
  `#[inline(always)]` now, like `token::arg`.
- After that, the flags-only line still reads ~6 ns above step 8 with the same
  instructions (529 vs 526) and backend stalls (~32) but ~10 % more cycles:
  code placement, not work. Deltas under ~10 % between builds are not signal
  without instructions and stalls agreeing.

`wa-comb`'s `-I a -I b -I c` got 134 ns slower because `arguments_as` allocates
a `Vec` per occurrence; `wa-disp`'s loop over `split` does not.

## 10. Choices: `--color <auto|always|never>`

`#[derive(ValueEnum)]` implements `FromArg` as a `match` on the value's bytes
(kebab-case names, `name`/`alias` overrides), so it composes with `Option`,
`Vec`, delimiters, positionals and `argument_as::<E>()` unchanged. String
fields take `choices("a", "b")`, checked before conversion. Both fail with
`invalid_choice` listing the names. bpaf has no value enum; its `Color` is a
`FromStr`.

| framework | `-v --path /tmp/x` | Δ vs 9 | `… --color always` | Δ vs flags-only |
|-----------|------:|------:|------:|------:|
| usage     | 166   | +5    | 277   | +111  |
| wa        | 47    | −8    | 62    | +15   |
| wa-disp   | 106   | +3    | 132   | +26   |
| wa-comb   | 138   | +4    | 232   | +94   |
| bpaf 0.10 | 6962  | +426  | 7342  | +380  |
| clap 4    | 5269  | +437  | 6097  | +828  |

Warm ns (min); wa's −8 is the code-placement wobble noted in step 9 (instructions
513 against 573). One more flag and an enum match cost the derive 15 ns; the
`Named`/`alt` form pays per preceding branch again, since `--color` is tried
last.

## 11. `env` and `default`: `-j/--jobs <N>`, `$EXAMPLE_JOBS`, default 4

The command line wins, then the environment, then the default; a switch from
the environment is true unless `""`, `0`, `false`, `no` or `off`, as in usage.
Environment reads go through `winnow_args::env::var`, which `with_env` can point
at a fixed set for tests. The variable is unset while measuring, so every line
without `-j` pays one lookup and a default conversion.

| framework | `-v --path /tmp/x` | Δ vs 10 | `-v --path /tmp/x -j 8` | `… a b c` | `… use -g node@20` |
|-----------|------:|------:|------:|------:|------:|
| usage     | 238   | +72   | 239   | 480   | 444   |
| wa        | 120   | +73   | 69    | 198   | 183   |
| wa-disp   | 162   | +56   | 125   | 251   | 273   |
| wa-comb   | 205   | +67   | 259   | 334   | 318   |
| bpaf 0.10 | 7878  | +916  | 8214  | 9577  | 11599 |
| clap 4    | 6097  | +828  | 6607  | 7757  | 10399 |

Warm ns (min). One environment lookup costs everyone ~70 ns: `std::env::var_os`
takes the environment lock, scans every variable and copies the match. usage
pays exactly the same, since that is the whole cost; with `-j 8` the derive
skips the lookup and is back to 69 ns. It scales with the size of the
environment, so these numbers depend on the shell that ran them.

## 12. Constraints: `conflicts`, `overrides`, `requires`, `group`, `required`, `required_unless`

After the loop: environment fallbacks, then exclusivity (`conflicts`, at-most-one
groups) on what was *supplied*, then defaults, then requiredness (`required`,
`required_unless`, required groups, `requires` targets) on what *has a value*;
`overrides` unsets the loser while binding and keeps the environment and default
from refilling it. Selectors resolve to fields at compile time, so each check is
a couple of slot tests. The bench adds `-q` conflicting with `--verbose`,
`--json`/`--toml` in an at-most-one group and `--strict` requiring `--json`;
clap uses `conflicts_with`/`ArgGroup`/`requires`, bpaf a struct-level `guard`,
the combinators plain Rust after the parse. Every framework was checked to
reject each violation.

| framework | `-v --path /tmp/x` | Δ vs 11 | `… a b c` | `-vvv …` | `--json --strict` |
|-----------|------:|------:|------:|------:|------:|
| usage     | 251   | +13   | 486   | 308   | 366   |
| wa        | 120   | 0     | 193   | 129   | 136   |
| wa-disp   | 185   | +23   | 303   | 232   | 238   |
| wa-comb   | 213   | +8    | 329   | 271   | 590   |
| bpaf 0.10 | 9439  | +1561 | 11128 | 11303 | 9939  |
| clap 4    | 9944  | +3847 | 11610 | 10788 | 11642 |

Warm ns (min). Carrying the constraints costs the derive nothing measurable;
clap pays ~3.8 µs per start building the groups and conflict tables into its
command tree. `wa-comb`'s `--json --strict` pays for being the 9th and 11th
branches of its `alt`.

## 13. `double_dash`: `[-- CMD...]`

`double_dash = "required"` takes a positional out of the ordinary sequence:
every word after `--` goes to it (past a greedy `Vec` before it), and a word
that would reach it before `--` is `arg_requires_double_dash`.
`double_dash = "automatic"` calls `Argv::stop_flags()` once the argument has a
value. The bench's root gains mise-`exec`'s shape: `[FILE]... [-- CMD...]`
(clap `last = true`). **bpaf 0.10 cannot express it**: a `strict` positional
errors on a plain word instead of missing it, so its `FILE` also takes the words
after `--`; its number on that line is not the same work, and the agreement
test skips bpaf there.

**Measured under heavy load** (load average ~115 on 128 cores): times moved
10–25 ns for every framework on lines this step does not touch, usage included.
Warm instructions per parse against the step-12 build decide it instead:
1291 → 1299 on `-v --path /tmp/x`, 2109 → 2121 on `… a b c` — one more branch
per word.

| framework | `-v --path /tmp/x` | `… a b c` | `… a -- node app.js -v` |
|-----------|------:|------:|------:|
| usage     | 271   | 500   | 581   |
| wa        | 134   | 210   | 272   |
| wa-disp   | 186   | 302   | 393   |
| wa-comb   | 219   | 400   | 452   |
| bpaf 0.10 | 10005 | 12077 | (12533, different parse) |
| clap 4    | 10394 | 12088 | 13625 |

Warm ns (min), under load; compare within a row, not against step 12.

## 14. `default_missing` (`-w/--write[=PATH]`) and `restart_token`

`default_missing = "…"` (with or without `value_optional`) makes a value-taking
flag's value optional, per usage's corpus: a bare `--write`, or one followed by
a flag-like word, gives the missing default; `--write=x`, `-wx` and `--write x`
give `x`. It is `Arg::read_value_or` — `read_value` with the missing default
where it would report a missing value — and `Named::argument_or` for the
combinators. clap spells it `num_args = 0..=1, default_missing_value`; bpaf's
combinator `on_missing_value`. A struct-level `restart_token = ":::"` restarts
positionals and resumes flags; it is not in the bench, since mise's `run` has no
static positionals for it to act on.

Measured on a quiet machine (load ~3).

| framework | `-v --path /tmp/x` | `… --write` | `-v -w ./x --path /tmp/x` | `… a b c` |
|-----------|------:|------:|------:|------:|
| usage     | 271   | 351   | 335   | 484   |
| wa        | 130   | 168   | 164   | 201   |
| wa-disp   | 186   | 240   | 237   | 298   |
| wa-comb   | 226   | 506   | 508   | 416   |
| bpaf 0.10 | 10761 | 11213 | 11234 | 12916 |
| clap 4    | 10403 | 11523 | 11616 | 12154 |

Warm ns (min). `wa-comb` now needs nested `alt`s: winnow's `alt` takes at most
9 parsers, and the root has 10 flags. `--write` is the last branch of the second
one, which is why it costs `wa-comb` ~280 ns more than the flags-only line.

## 15. `default_subcommand` and `arg_required_else_help`

`#[arg(default_subcommand = "run")]` on a struct: a word naming no subcommand
selects `run`, which reads that word again as its own (`lint` → `run lint`);
flags before the word stay with the parent, words after `--` do not route, and
only the struct that declares it has it. Tests mirror usage's
`09-default-subcommand.json` vectors, except where winnow-args is strict about
unknown flags. `#[arg(arg_required_else_help)]` makes a bare invocation return
`ErrorKind::HelpRequested`.

Not benchmarked: clap and bpaf 0.10 cannot express a default subcommand, and
declaring one on the bench root would change what every positional line means
for usage and winnow-args but not for them. When unused it costs nothing —
warm instructions are unchanged from step 14 (1308 on `-v --path /tmp/x`,
1952 on `… use -g node@20`); when used, the derive copies the 32-byte `Argv` once
per item to re-read the word.

## 16. Help and version

The derive emits each command's help as `static` data (`Args::HELP`: doc
comments, `help`/`long_help`/`help_heading`/`hide`, value names, `[env]`,
`[default]`, `[possible values]`, visible aliases, subcommands with their own
help). `-h`/`--help`/`-V`/`--version` are extra arms of the generated `match`
(skipped where the command declares those names or disables them), as is the
`help <cmd…>` word, which walks the help data like usage's parser does — clap and
bpaf 0.10 supply it too. Help comes back as `ErrorKind::HelpRequested`; each
subcommand level adds its name for the usage line; `report` prints it.

| | `-v --path /tmp/x` | `… use -g node@20` | stripped bytes |
|---|------:|------:|------:|
| wa, step 15 | 1308 instr, 130 ns | 1952 instr | 368 984 |
| wa, with help | 1300 instr, 131 ns | 1971 instr | 375 696 |
| usage | 274 ns | 485 ns | 390 104 |

Warm instructions per parse and warm ns (min). Carrying help costs a parse
nothing measurable; the `+19` on the subcommand line is the `map_err` that
records the subcommand's name. The text costs 6.7 KB of binary.

## 17. mise at full scale

`tasks/gen-mise-shadow.py` translates usage's shadow of `mise.usage.kdl` (211
commands, 35 subcommand enums, 23 value enums) into winnow-args' vocabulary,
dropping only metadata that does not affect parsing (listed in the checklist)
and dropping no selector. It compiled after four fixes the toy CLI had not
needed: a repeated `long` as another spelling, `help` on a variant,
`required` on a switch, and `double_dash = "automatic"` next to `"required"`
(mise's `run`), which needed `Mode::Values` so an automatic stop is not
mistaken for a `--`. `SUITE=mise tasks/perf.sh` runs it against usage's own
shadows, unmodified. `bench/tests/mise.rs` checks that the benchmark line binds
the same fields in all four, and that usage and winnow-args accept and reject
the same 19 varied lines.

| `mise …` | usage | wa | bpaf 0.10 | clap 4 |
|---|------:|------:|------:|------:|
| `use -g node@20` instr | 7 549 | 3 810 (0.5×) | 1 195 054 (158×) | 4 944 348 (654×) |
| cold ns | 13 980 | 9 920 | 444 766 | 1 498 040 |
| warm ns | 783 | 320 | 166 360 | 766 525 |
| `-C /tmp install node@20 python@3.12` warm ns | 921 | 442 | 167 029 | 770 596 |
| `ls --json` warm ns | 598 | 281 | 167 604 | 765 835 |
| `settings set color false` warm ns | 700 | 278 | 171 928 | 802 250 |
| stripped bytes | 1 174 488 | 1 611 408 | 2 951 704 | 2 243 536 |

Load ~6 of 128. The ratio against usage holds from the toy CLI (0.4–0.6× on
instructions, ~0.4× warm): the derive's cost stays per command in scope, not per
command in the CLI. The binary is 37 % larger than usage's: each struct's loop
inlines the lexer and its continuations (`#[inline(always)]`, steps 5, 9, 14),
where usage walks shared static tables with one parser. Cold times, which
include first-touch page faults, gain less than warm ones for the same reason.
`run build` is rejected by every shadow: mise's tasks arrive through `mount`,
which none of them model.

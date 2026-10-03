# Performance log

One entry per feature. Each adds the feature to all five bench CLIs, re-runs
every line in `bench/argv.txt`, and keeps the earlier lines so the cost of
merely *carrying* a feature shows up next to the cost of *using* it.
`bench/tests/agree.rs` checks every framework parses every line to the same fields.

Method (`just perf`, aarch64 host):

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

Warm ns (min). Instructions and cold times are in the `just perf` output
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

`default_missing = "…"` makes a value-taking
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

`xtask gen mise-shadow` translates usage's shadow of `mise.usage.kdl` (211
commands, 40 subcommand enums, 23 value enums) into winnow-args' vocabulary,
dropping only metadata that does not affect parsing (listed in the checklist)
and dropping no selector. It compiled after four fixes the toy CLI had not
needed: a repeated `long` as another spelling, `help` on a variant,
`required` on a switch, and `double_dash = "automatic"` next to `"required"`
(mise's `run`), which needed `Mode::Values` so an automatic stop is not
mistaken for a `--`. `SUITE=mise just perf` runs it against usage's own
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

## 18. `allow_negative_numbers`: `--offset -3`

A flag or positional opts in; a negative number is digits, at most one `.` and
an optional exponent (usage's rule, so `-inf` and `-1x` stay flag-like). A flag
takes it as a detached value through `ValueOptions`, which the next grammar
steps extend; a positional takes it when it is the next to fill, through a check
the derive emits only for structs that have such a positional; a declared digit
short (`-0`) still wins. `Named::allow_negative_numbers` keeps `Named` at 24
bytes (it sits in padding). bpaf uses `negative_lit`, clap `allow_negative_numbers`.

| framework | `-v --path /tmp/x` | `… a b c` | `… --offset -3` |
|-----------|------:|------:|------:|
| usage     | 286   | 503   | 369   |
| wa        | 133   | 215   | 159   |
| wa-disp   | 186   | 300   | 238   |
| wa-comb   | 229   | 432   | 517   |
| bpaf 0.10 | 11151 | 13366 | 11723 |
| clap 4    | 10786 | 12727 | 12006 |

Warm ns (min), load rising 5 → 11 during the run. Warm instructions against
step 17 are +10 and +19 on the first two lines — the bench CLI's new `--offset`
slot and arm — so the rest of the drift is load.

## 19. `allow_hyphen_values`: `--args -destroy`

A flag declared `allow_hyphen_values` takes the next word whatever it looks
like, `--` included (usage's corpus; the `--args -- -x` vector then needs
lenient unknown flags, which winnow-args does not have yet). It is a second
`ValueOptions` field, so derive, `Named::allow_hyphen_values` and `dispatch!`
arms share it, and `Named` stays 24 bytes. clap says `allow_hyphen_values`;
bpaf 0.10 has no equivalent — pairing `literal` with `any` broke `optional()` —
so its field refuses `-destroy` and the agreement test skips bpaf there.

| framework | `-v --path /tmp/x` | `… a b c` | `… --args -destroy` |
|-----------|------:|------:|------:|
| usage     | 280   | 492   | 363   |
| wa        | 139   | 225   | 179   |
| wa-disp   | 185   | 300   | 238   |
| wa-comb   | 231   | 426   | 544   |
| bpaf 0.10 | 11995 | 14136 | —     |
| clap 4    | 10794 | 12750 | —     |

Warm ns (min) under heavy load (average 22 → 29); `perf.sh` stopped after bpaf
refused the last line, so clap has no number there. Warm instructions are the
evidence: 1326 and 2158 per parse against step 18's 1323 and 2167.

## 20. `require_equals`: `--inspect a`

A flag declared `require_equals` binds only an attached value (`--inspect=9229`,
`-i9229`, `-i=9229`); a detached one is a missing value, and with
`default_missing` (aube's `--inspect[=PORT]`) the next word stays a positional.
It is a third `ValueOptions` field; `Named::argument_or` and `arguments_as` now
honour a `Named`'s options too, and `Arg::value_or_with` joins the `dispatch!`
forms. clap says `require_equals = true, num_args = 0..=1`; bpaf 0.10's
`adjacent` reports `--inspect a` as not adjacent rather than missing, so
`on_missing_value` never fires (and loops until bpaf's no-progress check
panics); an `adjacent` argument or a bare `req_flag("9229")` spells it instead.

| framework | `-v --path /tmp/x` | `… a b c` | `… --inspect a` |
|-----------|------:|------:|------:|
| usage     | 283   | 495   | 483   |
| wa        | 130   | 199   | 190   |
| wa-disp   | 189   | 300   | 287   |
| wa-comb   | 240   | 435   | 679   |
| bpaf 0.10 | 13858 | 15963 | 15274 |
| clap 4    | 11058 | 13340 | 13742 |

Warm ns (min), load average ~13–18. `wa-comb` tries `--inspect` eighth in its
flag `alt`, and `argument_or` reads the value before failing the other branches:
the linear `alt` cost again. Warm instructions per parse against step 19:
wa 1326 → 1329 and 2158 → 2158; wa-disp 1626 → 1626 and 2685 → 2688.

## 21. `negate`: `--no-cache`

usage's `negate = "--no-color"`: a second long spelling that sets a `bool`
false. `#[arg(long, negate)]` derives `--no-<long>`; the last spelling given
wins, and a `default` ("true" or "false") or `env` fills only what neither
spelling set, so the slot is an `Option<bool>` until the struct is built. Help
lists `--color / --no-color`. Combinators: `Named::negated_by(no)` gives `true`
or `false`; `dispatch!` needs only two arms. clap has no negation — the usual
two flags with `overrides_with` each other; bpaf 0.10 takes two `req_flag`s,
`last()`, `fallback(true)`.

| framework | `-v --path /tmp/x` | `… a b c` | `… --no-cache` |
|-----------|------:|------:|------:|
| usage     | 295   | 500   | 386   |
| wa        | 129   | 199   | 139   |
| wa-disp   | 188   | 303   | 222   |
| wa-comb   | 236   | 428   | 626   |
| bpaf 0.10 | 15526 | 17413 | 17292 |
| clap 4    | 12911 | 15187 | 13557 |

Warm ns (min), load average ~4–8. Warm instructions per parse against step 20:
wa 1329 → 1338 and 2158 → 2167, the default filled into the new slot and
unwrapped on every parse; wa-disp 1626 → 1628 and 2688 → 2690.

## 22. `double_dash = "preserve"`

usage's fourth `double_dash` mode: when a `--` arrives while a `preserve`
positional is next to fill, the `--` is one of its values and flags go on
being flags (`wrap npm -- -v` gives `ARGS = ["--"]` and `-v`). The derive
emits the check in its `Separator` arm only for a struct that declares one,
and rejects `preserve` beside a `double_dash = "required"` positional, whose
`--` it would keep. Combinators take `token::separator_word`, `--` as a word.

Neither clap nor bpaf 0.10 keeps `--` as a value, mise does not use it, and the
benchmark CLI's trailing `CMD` needs its `--`, so no line is added. What a CLI
without `preserve` pays is nothing: warm instructions per parse are identical
to step 21 (wa 1338 and 2167, wa-disp 1628 and 2690), and `… a -- node app.js
-v` runs in 252 ns for wa against usage's 573.

## 23. `unknown_flags = "value"`: lenient unknown flags

usage's default: a flag-like word that names no flag is offered to the
positionals whole (`wrap --wat keep`), is never a subcommand word, and with no
positional left to take it is an unexpected argument. winnow-args stays strict
by default and takes `#[arg(unknown_flags = "value")]` per struct (not
inherited). A long word or a single letter falls through the flag `match`
and any globals, then goes to the positionals as the word saved before lexing.
A bundle of two or more letters is checked whole before any letter binds, as
usage does; inherited letters are known through a new `Globals::short`, which
the derive answers with `__wa::inherit` (its globals, then its parent's).
Combinators take `token::flag_word` as the last `alt` branch (whole words
only).

The mise shadow is now lenient in all 211 structs, as usage's is, and the
agreement test gains six lines with unknown flags (four accepted by both, two
rejected by both). Warm instructions per parse:

| line | step 22 | `inherit` (strict) | lenient |
|------|------:|------:|------:|
| `use -g node@20` | 3556 | 3571 | 3594 |
| `-C /tmp install node@20 python@3.12` | 4859 | 4874 | 4905 |
| `ls --json` | 3131 | 3146 | 3168 |
| `settings set color false` | 3097 | 3117 | 3146 |
| example `-v --path /tmp/x` | 1338 | 1337 | 1337 |

The first cut checked every short word, single letters too, with
`str::from_utf8`: +118 on `use -g`. An ASCII fast path took it to +86, and
leaving single letters to the fallback arm (nothing can be half applied) to
+23. mise timings, warm ns (min), load average ~4:

| framework | `use -g node@20` | `-C … install …` | `ls --json` | `settings set …` |
|-----------|------:|------:|------:|------:|
| usage     | 774    | 924    | 598    | 710    |
| wa        | 331    | 450    | 279    | 291    |
| bpaf 0.10 | 164930 | 163889 | 163684 | 167548 |
| clap 4    | 762532 | 764966 | 774887 | 797042 |

(wa measured before the two fixes, so it is 60–90 instructions slower here
than the final build.)

## 24. Help wraps to the terminal width

`help::render` wraps to `help::width()`: `COLUMNS` when set, else 100 (clap's
width when it cannot ask the terminal; probing the terminal would need a
dependency). Short help keeps descriptions beside their item, the column capped
at two fifths of the page as usage does, and an item wider than that has its
description on the next line; long help wraps its indented blocks; the about and
after text wrap too. `render_width` takes an explicit width.

Help costs nothing while parsing: parse instructions and the parse binaries'
stripped size are unchanged. A new `help-n-mise-wa` renders what its command
line asks for, so the help steps have a number. Warm instructions per render:

| line | unwrapped | first cut | final |
|------|------:|------:|------:|
| `--help` | 137 692 | 517 568 | 256 384 |
| `-h`     | 134 794 | 385 518 | 206 491 |
| `use -h` |  47 132 | 132 635 |  79 592 |

The first cut collected every description into owned lines. The final one
iterates borrowed slices, one scan per line that fits, and writes straight into
the output. It is 4.7 KB more binary (1 844 800 → 1 849 568 stripped, for a
binary that renders help).

## 25. `[possible values]` for `ValueEnum` fields

`FromArg` gains `const CHOICES: &'static [&'static str] = &[]`, which
`#[derive(ValueEnum)]` fills with its visible variants' names, and a field's
help item takes `<T as FromArg>::CHOICES` when it declares no `choices`. So
usage's `value_enum` marker is not needed: mise's `bootstrap remote --only` and
`--skip` now list their seventeen parts.

Parsing is unchanged (3594 warm instructions on `use -g node@20`). Rendering
`bootstrap remote -h` goes from 125 573 to 154 821 instructions for the two
extra lists; `--help` is unchanged at 256 384. Both binaries grow 512 bytes: the
lists are `static` help data, linked even where help is never rendered, which
is what the next step is about.

## 26. `help-text`: leave help prose out

A cargo feature of winnow-args, on by default. The derive wraps every piece
of prose (doc comments, `help`, `about`, `after_help`, …) in
`winnow_args::__text!`, which expands to the literal with the feature and to
`""` without; winnow-args picks the definition, since a proc macro cannot see
its features. Help without prose still has its usage line, commands, flags,
value names, defaults, env, possible values and version. `bench` has a
default `help-text` feature, so `cargo build -p bench --no-default-features`
builds every winnow-args binary without prose.

| stripped bytes | with prose | without |
|----------------|------:|------:|
| `parse-n-mise-wa` | 1 832 552 | 1 611 632 |
| `help-n-mise-wa`  | 1 850 080 | 1 629 160 |
| `parse-n-wa` (no doc comments) | 380 400 | 380 400 |
| usage's mise shadow, for reference | 1 174 488 | — |

mise's prose is 221 KB, 12 % of the binary. (Correction, step 27: these
sizes move ±110 KB with codegen-unit partitioning; with one codegen unit the
prose is 318 KB, 19 %.) Without it winnow-args' mise
is still larger than usage's (which keeps its prose), so the size item
stays open. Parsing is unaffected (`use -g node@20`: 3594 warm instructions
with, 3570 without, within layout noise); rendering `--help` drops from
256 384 to 85 620 instructions with less text to lay out.

## 27. `terminal-size`: wrap to the terminal; codegen units and size

An optional `terminal-size` feature (off by default) adds the `terminal_size`
0.4 crate, as clap's `wrap_help` does. `help::width()` is `COLUMNS` first (so a
user or a test can say what to assume, as in usage), then the terminal on
standard output, then 100. Under `with_env` the terminal is not asked, so
pinned tests stay pinned. The example now reports help through `report`:
`cargo run --example example --features terminal-size -- -h`. (A pty with 0
rows, as `script` makes one without `stty rows`, reads as no terminal:
`terminal_size` wants both dimensions.)

**Codegen units make default-profile sizes unreliable.** Building with the
feature made `parse-n-mise-wa` 112 KB *smaller*, though it never renders help:
changing winnow-args' features changes its crate hash, which moves code
between the 16 codegen units and changes cross-unit inlining (`.text`
−77 KB, `.eh_frame` −21 KB, `.gcc_except_table` −11 KB; some
`parse_argv_with` bodies shrank while `main`'s closure grew 14 KB). With
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` both builds are identical:

| stripped bytes, one codegen unit | default | `terminal-size` | no `help-text` |
|---|------:|------:|------:|
| `parse-n-mise-wa` | 1 671 664 | 1 671 664 | 1 353 464 |
| `help-n-mise-wa`  | 1 688 032 | 1 689 400 | 1 369 832 |
| `parse-n-wa`      |   374 624 |         — |   374 624 |
| usage's mise shadow | 1 163 168 | | |

So `terminal-size` costs 1 368 bytes where help is rendered and nothing
elsewhere; rendering `--help` is unchanged (256 282 instructions, plus one
`ioctl`). The `help-text` saving of step 26 was really 318 KB (19 %), not
221 KB. Against usage with one codegen unit, winnow-args' mise is 44 % larger
with its prose and 16 % larger without. Size comparisons from here on use the
`release-lto` profile (step 28).

## 28. A `release-lto` profile: one codegen unit, fat LTO

`[profile.release-lto]` inherits `release` with `codegen-units = 1` and
`lto = "fat"`; `PROFILE=release-lto just perf` builds and measures with it
(the header line names the profile). `release` stays the default, so earlier
entries remain comparable with new default runs.

Example CLI (load average ~3):

| framework | instr | warm ns | `… a b c` instr | warm ns | stripped |
|-----------|------:|------:|------:|------:|------:|
| usage     |   2457 |   239 |   4877 |   436 |   341 432 |
| wa        |   1443 |   122 |   2605 |   182 |   342 544 |
| wa-disp   |   1494 |   134 |   2691 |   197 |   337 336 |
| wa-comb   |   2139 |   203 |   3866 |   348 |   344 168 |
| bpaf 0.10 | 122015 | 14477 | 137535 | 16421 |   710 072 |
| clap 4    | 111099 | 12223 | 128439 | 14473 |   618 344 |

mise (cold instructions / warm ns min):

| framework | `use -g node@20` | `-C … install …` | `ls --json` | `settings set …` | stripped |
|-----------|------:|------:|------:|------:|------:|
| usage     | 6863 / 697 | 8147 / 825 | 5300 / 545 | 6133 / 610 | 988 064 |
| wa        | 3661 / 308 | 5356 / 422 | 3364 / 268 | 3509 / 276 | 1 694 272 |
| bpaf 0.10 | 1.13 M / 160 049 | 1.12 M / 158 057 | 1.12 M / 158 512 | 1.15 M / 161 564 | 2 609 008 |
| clap 4    | 4.43 M / 721 864 | 4.45 M / 741 379 | 4.46 M / 723 829 | 4.70 M / 770 355 | 2 645 320 |

LTO helps the combinators most: `wa-disp` goes from 1886 to 1494 cold
instructions and `wa-comb` from 2610 to 2139, both now close to the derive,
since their generic parsers get inlined across crates. It shrinks usage's mise
by 175 KB (1 163 168 → 988 064) but grows winnow-args' by 23 KB
(1 671 664 → 1 694 272), so the size gap is 71 %; without help prose
winnow-args' mise is 1 375 864 bytes, still 39 % above usage with its prose.
The per-struct match loops, inlined whole, are what the size work has to
attack.

## 29. Color

`help::Style` holds one SGR sequence per role; `Style::CLAP` has clap 4's
codes, measured from clap's own help and errors for the bench CLI (bold
underlined headers, bold literals — program, each flag spelling, subcommand
names — plain placeholders, bold red `error:`, yellow for what was typed,
green for what is missing, bold `--help` in the tip). `Style::auto` follows
usage and clap: a non-empty `NO_COLOR` refuses, `CLICOLOR_FORCE` other than `0`
forces, else color when the stream is a terminal; under `with_env` it never is.
`report` paints help on stdout and errors on stderr accordingly;
`render_help` stays plain and `render_help_styled`/`Error::render` take a
style. A test holds the painted page, escapes stripped, equal to the plain one
at two widths.

`release-lto`, against step 28:

| | before | after |
|---|------:|------:|
| `help-n-mise-wa` stripped | 1 710 072 | 1 715 016 |
| `parse-n-mise-wa` stripped | 1 694 272 | 1 694 272 |
| render `--help` (plain) | 253 725 | 254 288 |
| render `bootstrap remote -h` (plain) | 154 526 | 170 843 |
| parse `use -g node@20` | 3 403 | 3 403 |

Color costs 4.9 KB where help is rendered and nothing elsewhere. Plain
rendering pays up to 10 % for building each flag's cell piece by piece.

## 30. Default colors: usage's palette, cyan values

`Style::COLORED`, now what `Style::auto` picks, is usage's help palette (bold
yellow `1;33` headings, bold green `1;32` flags and subcommands, the program
plain) with bold cyan `1;36` where usage has magenta for value names; `[NAME]`
paints only the name, as usage does, `<NAME>` all of it. Errors keep the codes
clap and usage's diagnostics share. `Style::CLAP` stays, with a `program` role
for clap's bold program name. A test pins the codes; the strip-equals-plain
test runs under both styles. Nothing changes where help is not rendered, and
the plain page is byte for byte the same.

## 31. Size: stores out of line, cold error paths

`cargo bloated` (`release` with one codegen unit: it links std dynamically,
which fat LTO refuses) shows both mise binaries are almost all generated code:
824 KiB for winnow-args' shadow, 458 KiB for usage's, 8 KiB for our runtime.
A throwaway crate with 20 identical fields per kind gave the cost per field:

| field kind | bytes/field before | after |
|---|---:|---:|
| `bool` | 16 | 14 |
| `Option<u32>`, `Option<ValueEnum>` | 21 | 86 |
| `Option<String>` | 320 | 207 |
| `Vec<String>` | 455 | 328 |
| `Option<String>` + `default` | 570 | 276 |
| `Option<String>` + `env` | 715 | 335 |

Converting an owned value and dropping the one it replaces was inlined into
every arm and every fallback. `winnow_args::store` now has `#[inline(never)]`
generic `set`/`push` (flags), `set_word`/`push_word` (positionals) and
`set_from`/`push_from` (defaults, env): one copy per value type. Fields with
`choices` keep the inline path, since the check comes before conversion.
`read_value`/`check_switch` build their errors in `#[cold]` out-of-line
functions, which were otherwise inlined into every arm. Cheap types got worse
per field (their identical arms had merged whole; now the inlined
`read_value` fast path stays per arm), but mise is mostly strings.

`release-lto`:

| | before | after |
|---|------:|------:|
| `parse-n-mise-wa` stripped | 1 694 272 | 1 565 472 (−7.6 %) |
| `parse-n-wa` stripped | 342 544 | 339 496 |
| warm `use -g node@20` | 3403 | 3449 (+1.4 %) |
| warm `-C /tmp install …` | 4648 | 4776 (+2.8 %) |
| warm `ls --json` | 3022 | 3030 (+0.3 %) |
| warm `settings set color false` | 2934 | 3010 (+2.6 %) |
| warm `-v --path /tmp/x` | 1156 | 1187 (+2.7 %) |
| warm `… a b c` | 1833 | 1928 (+5.2 %) |

The call per value is the cost; `a b c` pays it for each positional word.
usage's mise is 988 128 bytes, so the gap is now 58 % (was 71 %).

## 32. Speed first: out-of-line stores reverted, cold errors kept

Size only counts where speed does not pay for it: a leaner binary does not
beat a faster parse, and LTO can already fold shared code when several CLIs
link into one binary (a multi-call build), which usage's per-command tables
cannot. Warm time (`time-sweep`, min ns, `release-lto`, three rounds each)
judged step 31's changes:

| line | before | out-of-line stores | cold errors only |
|---|---:|---:|---:|
| `use -g node@20` | 313–320 | 321–323 | 312–315 |
| `-C /tmp install …` | 427–430 | 448–451 | 427–435 |
| `settings set color false` | 267–276 | 287–289 | 266–272 |
| `-v --path /tmp/x` | 122–124 | 131–133 | 123–124 |
| `… a b c` | 179–185 | 191–193 | 181–187 |

The out-of-line `store` helpers cost 2.5–8 % and are reverted. The `#[cold]`
error constructors in `read_value`/`check_switch` are free and stay: mise is
1 629 104 bytes stripped (−3.8 % against 1 694 272). Also tried and dropped:
an out-of-line lexer (`token::arg` not inlined) saved 147 KB more but cost
5–15 % (`use -g` 320 → 343 ns, `a b c` 191 → 219 ns), the store-forwarding
stall of step 5.

## 33. Color depth and themes

`color::Depth::detect` tells 16, 256 and 24-bit terminals apart from the
environment (`COLORTERM`, `TERM`, `TERM_PROGRAM`, with
`NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE` and `FORCE_COLOR` levels), as
`supports-color` and `anstyle-query` do; under `with_env` a stream is never a
terminal. `color::Theme` holds a palette per depth; `report_with` paints with
one, `report` with `Theme::DEFAULT` (the 16-color default palette, suited to
any background). Colors deeper than the terminal map to the nearest one it has.
No dependency: the escapes are written by hand.

Help rendering is not on the hot path; parse binaries are unchanged
(`parse-n-mise-wa` 1 629 104 bytes stripped, `release-lto`).

## 34. Default palettes: greens and teals, no yellow

Yellow was too loud, and xterm's yellow and cyan are nearly unreadable on a
light background (contrast 1.47 and 1.71 against `#eeeeee`). The defaults now
keep to greens for what is typed and cyans/teals for structure and values:

| role | 16 colors | 256 colors | contrast on `#262626` / `#eeeeee` |
|---|---|---|---|
| headings | bold cyan | bold teal 73 `#5fafaf` | 5.94 / 2.20 |
| flags, subcommands | bold green | bold green 71 `#5faf5f` | 5.61 / 2.33 |
| value names | cyan | slate teal 109 `#87afaf` | 6.33 / 2.06 |
| `error:` | bold red | bold red 167 `#d75f5f` | 4.10 / 3.18 |
| what was typed wrong | red | rose 174 `#d78787` | 5.55 / 2.35 |
| what is missing | green | sage 108 `#87af87` | 6.13 / 2.13 |

`Theme::DEFAULT` uses the 256-color palette on 256-color and 24-bit terminals.
No mid-tone reaches 4.5 on both backgrounds, and the palette cannot know which
one it is on; these read well on dark and far better than before on light. The
16-color palette's look is the terminal theme's. `Palette::CLAP` keeps clap's
own error colors, yellow included. Rendering only; parsing is untouched.

## 35. Default and possible values highlighted

A seventh palette role, `value`, paints the value in `[default: …]` and each
value in `[possible values: …]` (plain green in 16 colors, green 71 in 256;
unpainted in `Palette::CLAP`, as in clap). Values are painted one by one, so
the spaces between them stay plain and a wrapped line never breaks inside a
painted span; `Wrap` now skips SGR sequences when counting columns. The
strip-equals-plain test covers wrapping with the new escapes at 50 and 100
columns. Rendering only.

## 36. Env, default and possible values each their own color

The `value` role splits into `env` (cyan; teal 37 `#00afaf`, 5.59/2.34),
`default` (green; sea green 72 `#5faf87`, 5.74/2.27) and `choice` (bright
green; green 71 `#5faf5f`, 5.61/2.33), contrast against `#262626`/`#eeeeee`.
Unpainted in `Palette::CLAP`. Rendering only.

## 37. Palette roles: flag/command, dim, code

13 roles now: `header`, `program`, `flag` and `command` (split from `literal`,
as usage separates `option` and `command`; both bold green by default),
`placeholder`, `env`, `default`, `choice`, `dim`, `code`, `error`, `invalid`,
`valid`. `dim` frames the rest — the `[env: `, `[default: `,
`[possible values: ` labels and their `]`, the brackets of an optional
`[NAME]`, `...` — with the dim attribute in 16 colors and gray 245 in 256.
`code` paints `` `quoted` `` spans in descriptions and subcommand summaries,
bold, backticks kept so the strip-equals-plain test still holds; each word of a
span is painted on its own, and an unpaired backtick is left alone. In the
usage line the trailing positional's `[--` is now a dim `[` and a flag `--`.

Correction to steps 29–30: clap 4's default styles use no color in help, only
bold and underline; color appears only in errors (red, yellow, green), and only
with clap's `color` feature. `Palette::CLAP` always reproduced that, but the
docs called it "clap's colors". Rendering only.

## 38. `long_only`: GNU's dash rules (ld phase 1)

`#[arg(long_only)]` makes a single-dash word whose name (up to `=`) is one of
the struct's long names a long option, tried before short letters, as GNU's
`getopt_long_only` and ld do: `-entry=main` is `--entry=main`, not
`-e ntry=main`. `#[arg(two_dashes)]` keeps a name to `--` (`--omagic`, since
`-omagic` is `-o magic`); `#[arg(short = 'l', prefix)]` keeps a letter that
always takes the rest of its word (`-lazy` is `-l azy`). `token::long_only` is
the combinator form. Tests port mold's `ArgCursor` vectors.

The default grammar pays nothing: `parse-n-wa` and `parse-n-mise-wa` build to
byte-identical binaries. In a `long_only` struct (11 fields), a 16-word
ld-shaped line costs 4 621 warm instructions against 4 393 without it (+5 %,
about 14 a word: the check leaves at the first byte for everything but a
single-dash word).

## 39. ld's values (ld phase 2)

`value::CInt<T>` reads C integers (`0x` hex, leading-zero octal, a sign for
signed types) as mold's `parse_c_number` does; `value::KeyValue<K, V>` splits
at the first `=` (`--defsym=sym=a=b`, `--section-start=.text=0x1000`);
`#[arg(values = N)]` makes each occurrence of a `Vec` flag read N words
(`-platform_version macos 11.0 12.0`), the words after the first via
`Arg::read_next`, whatever they look like. `-l:libfoo.a` already worked.

Nothing is linked unless used: warm instructions on the mise lines are
unchanged (3417, 4691, 3032, 2945 under `release-lto`), stripped sizes are
unchanged, and `parse-n-wa`'s `.text` is byte-identical. mise's `.text`
differs only in symbol names (the impl blocks of `value` renumbered) and a
few `.rodata` addresses that moved with them.

## 40. `+` options (brush phase 3)

`#[arg(plus_options)]` lexes `+abc` as a bundle (`token::arg_plus`), letters
reported with `ShortFlag::plus`; `#[arg(plus = 'x')]` gives a field its `+x`
spelling: on an `Option<bool>` with the same `short`, `-x` is `Some(true)` and
`+x` `Some(false)`, last wins; on a value flag, `+o NAME`. `#[arg(skip)]`
leaves a field at its default (declaration builtins fill theirs afterwards).

First cut: a `PlusBundle` stream mode. `take_word`, run for every word, then
tested two modes, and the default path paid for it: +15–25 warm instructions
a parse (`-v --path /tmp/x` 1087 → 1111, mise `use -g` 3417 → 3442) and 19.5 KB
on mise. Final: the `+` bit lives in `Argv`'s padding (still 32 bytes, now a
compile-time assertion), the mode stays `Bundle`, and only a bundle's start
writes the bit:

| | before | first cut | final |
|---|---:|---:|---:|
| `-v --path /tmp/x` | 1087 | 1111 | 1088 |
| `… a b c` | 1791 | 1812 | 1792 |
| `-vvv -p x` | 1178 | 1200 | 1180 |
| mise `use -g node@20` | 3417 | 3442 | 3420 |
| mise `settings set color false` | 2945 | 2964 | 2955 |
| `parse-n-mise-wa` stripped | 1 629 104 | 1 648 672 | 1 636 336 |

`release-lto`, warm instructions per parse.

## 41. `-z` keywords (ld phase 3)

`#[arg(keywords)]` on a field whose type derives `Args`: the flag's values are
collected as they come, and at the end parsed as that type's `--WORD` flags,
in order, so `now`/`lazy` (`negate = "lazy"`) resolve last-wins and a
lenient keyword type gathers unknown keywords for a warning. Errors become an
invalid value of `-z` naming the keyword. Nothing changes for fields without
it; the keyword parse runs once per command line, after the main loop.

## 42. Order as meaning: `Occurrence` and `sequence` (ld phase 4)

`#[derive(Occurrence)]` on an enum makes each variant a flag (a unit variant a
switch, a one-field variant a value) or the positional; an `Args` struct's
`#[arg(sequence)] items: Vec<Item>` field collects them in command-line order.
The struct's own arms come first; a flag they miss is offered to the enum
before globals and "unknown", a word before the struct's positionals.

On a 15-word ld-shaped line (`--as-needed`, `-lm`, `--whole-archive`, inputs),
the ordered version costs 4 465 warm instructions against 4 180 for the same
options as plain fields (+7 %), which lose which inputs each state applies to:
enum values pushed into one `Vec`, and flags reaching the enum only after the
struct's `match`. Nothing is generated without a `sequence` field.

## 43. Response files (ld phase 5)

`response::expand` replaces `@file` words with the file's words, as GNU's
`buildargv` splits them (whitespace, `'…'`, `"…"`, `\`), recursing up to 10
files deep as mold does. The files are read into a `ResponseFiles` that the
returned words borrow from, so a word is copied only when quotes or
backslashes were removed; the words feed `Argv::new` unchanged. A 4 MB file of
200 000 words (a tenth of them quoted) expands in 9.2 ms, read included:
46 ns per word. Nothing else changes: parsing never sees the `@`.

## 44. Unknown and ignored options (ld phase 6)

`#[arg(unknown)] unknown: Vec<T>` collects the flag-like words no field
declares, whole and in order, apart from the positionals; the linker warns
about those on its ignore list and reports the rest together (wild's
`unrecognized_options`). Silently ignored flags are the aliases of one unread
switch. `long_only` now combines with it and with `unknown_flags = "value"`:
the unknown-bundle check runs only once no single-dash long name matched, and
an `Occurrence` enum's short letters (`Occurrence::short`) count as known.

On the 15-word ld line, 4 832 warm instructions with an `unknown` field
against 4 681 without (+3 %): the bundle check reads each multi-letter
single-dash word (`-lc`, `-lpthread`) once more. Nothing is generated without
the field or `unknown_flags = "value"`.

## 45. `flatten`

`#[arg(flatten)] common: T` parses `T`'s flags as if declared in the parent.
The derive gives every flags-only struct a hidden `Flatten` impl: a slot
struct, `bind` (its own arms, one lexed flag at a time, returning whether it
took it), `short`/`is_long` for bundle checks and `long_only`, and `finish`
(environment, defaults, rules, required, the value). The parent keeps the
slots and offers each flag its own arms do not take, after a `sequence` and
before globals; its help lists `T`'s items after its own, joined at compile
time (`concat_items`). Nesting works the same way.

`bind` moves the slots out and back around its `match`, so a flattened flag
costs a few moves more than one declared in place. Nothing changes for a
struct that flattens nothing: warm instructions per parse are unchanged
(2 185 and 3 491 on the bench lines) and the `release-lto` binaries within a
few hundred bytes; the unused impls are not linked.

Not yet: `T`'s names in the parent's duplicate check (the parent's own arms
win), rules naming `T`'s flags, and a flattened struct with positionals,
a subcommand, keywords or `global` flags.

## 46. A positional `Vec` sized once

A `Vec` positional reserves room for every word left (`Argv::words_left`) on
its first value, instead of growing from empty: one allocation of the right
size for the usual tail of operands. On brush's `printf -v x FMT A a 3`
parse, 20 instructions fewer (2 247 → 2 227 with the bench's setup). Against
usage-port's hand-written `printf` parser, what is left is about 100
instructions a word of UTF-8 validation (`String::from_arg`), which a parser
over `String`s skips and winnow-args, over bytes and without `unsafe`, does
not; and the word list itself, which brush's adapter now keeps on the stack
for up to 16 words.

## 47. A whole linker's command line as one sequence (mold)

mold's options carry order everywhere (last one wins across different
spellings, `--as-needed` and friends apply to what follows), so its port is
one `Occurrence` enum with a variant per spelling, `#[arg(sequence, unknown)]`
on the only field, and mold's own handling folded over the items. What it
needed: value options on variants (`allow_hyphen_values` for every value,
`require_equals` and `default_missing` for `--build-id[=x]`), `keep_equals`
(`-L=dir`), `skip` variants for `-z` keywords, an `unknown` variant that keeps
`--lto-O3` in place, and `Spanned` values to tell `-zfoo` from `-z foo` in a
message. The sequence `Vec` is sized from the words left (one item a word):
on mold's input-heavy lines, 15 instructions a word fewer.

Over a bare `--version`, instructions a word: a 2 496-word link line 3 333
with mold's parser against 554; options alone 11 520 against 505, since
winnow-args dispatches a name once where mold tries up to 290 matchers in
turn; input files alone 388 against 516, the value being copied out of the
command line into an owned `OsString` (`FromArg` returns owned values).

## 48. The examples: brush's builtins and mold's command line

`examples/brush_builtins.rs` (17 of brush's builtins as subcommands) and
`examples/ld.rs` (mold's whole option set as one `Occurrence` sequence, with
`@file`) print what they parse; `xtask gen examples` regenerates them from the
two ports. With `PARSE_N=n` they parse `n` times and print nothing. Warm
instructions a parse, `release`, subcommand dispatch included:

| line | instructions |
|---|---|
| `pwd -P` | 347 |
| `cd -P /tmp` | 519 |
| `unset -fv x` | 822 |
| `declare -i +x n=1` | 947 |
| `test -n x` | 972 |
| `set -eu +x -o pipefail` | 1 094 |
| `kill -s TERM 1234` | 1 196 |
| `echo -n a b c` | 1 242 |
| `printf -v x %s a` | 1 246 |
| `read -rp prompt -t 2.5 a b` | 1 699 |
| `ld -shared -o out.so a.o` | 1 067 (266 a word) |
| `ld`, 18 words of options, libraries and inputs | 5 868 (326 a word) |
| `ld`, a 2 495-word link line | 524 a word |

A sequence enum's variants are now listed in help (`Occurrence::ITEMS`),
after the struct's own flags.

## 49. A review pass

Two adversarial reviews (the runtime, and the derive's generated code) found
what follows; each is fixed with a test in `tests/interactions.rs`,
`tests/complete.rs` or `tests/response.rs`.

- A `+x` word reached a flattened struct's `-x` arm; `+x` global flags never
  reached a subcommand.
- The built-in `-h`/`--help`/`-V`/`--version` shadowed a flattened struct's or
  a sequence enum's own (ld's `-h SONAME`): with either, they are now tried
  after them.
- A `bundle` item was only reported when the enum took the word's first letter.
- A flattened `prefix` letter was unknown to a `long_only` parent.
- A tristate `Option<bool>` flag counted as taking a value in bundle checks.
- A flag declared by a struct and by what it flattens, or by two variants of
  an `Occurrence` enum, and a struct positional beside a positional variant,
  are now compile errors (`help::items_clash`, checked in a `const`).
- Unit subcommands had no `--help`; a subcommand enum nested in another did
  not take global flags before its word; a declared `--help` was still listed.
- Completion: aliases, `require_equals`, global flags below their command,
  bash right after `--name=`, PowerShell's missing trailing space, elvish's
  unquoted words (`--words`); response files are capped at 4 096 in all.
- The `#[cfg]` handling in the derive was redundant (rustc strips the
  field first) and is gone.

The measured lines are unchanged: 2 186 and 3 476 warm instructions.

## 50. A full pass, after the review

At `a6f3e92`, pinned to one core (node 3), `release` unless said.

**Frameworks** (`just perf`; instructions for one cold parse, and warm
nanoseconds a parse). winnow-args takes about half of usage's instructions on
every line of both suites:

| line | usage | winnow-args | bpaf | clap |
|---|---|---|---|---|
| `-v --path /tmp/x` | 3 108 / 288 ns | 1 687 / 130 ns | 127 919 / 15 505 ns | 118 262 / 12 986 ns |
| `-v --path /tmp/x a b c` | 5 723 / 528 | 2 921 / 205 | 144 708 / 17 531 | 136 585 / 15 218 |
| `-v --path /tmp/x -I a -I b -I c` | 6 620 / 580 | 3 117 / 216 | 150 419 / 18 311 | 148 884 / 16 185 |
| `-v --path /tmp/x use -g node@20` | 5 198 / 505 | 2 532 / 203 | 157 816 / 19 521 | 157 148 / 17 917 |
| `-v --path /tmp/x -j 8` | 3 151 / 304 | 729 / 73 | 130 380 / 15 822 | 126 593 / 13 801 |
| mise: `use -g node@20` | 7 702 / 797 | 4 045 / 345 | 1 195 104 / 165 320 | 4 943 626 / 753 161 |
| mise: `-C /tmp install node@20 python@3.12` | 9 447 / 941 | 5 845 / 457 | 1 178 933 / 163 963 | 4 959 812 / 755 825 |
| mise: `ls --json` | 6 244 / 622 | 3 647 / 295 | 1 183 121 / 164 622 | 4 972 815 / 752 448 |
| mise: `settings set color false` | 6 720 / 726 | 3 822 / 288 | 1 206 373 / 168 281 | 5 235 116 / 790 096 |

Stripped mise binaries: usage 1 175 192, winnow-args 1 698 184, clap
2 237 312, bpaf 2 969 592 bytes.

**brush** (`winnow-port` at `14a60423` on this library, against its clap base
and usage-port). Binaries: clap 6 904 560, usage 6 733 808, winnow 6 475 280
bytes. Scripts, wall time over 15 samples and instructions averaged over four
paths:

| script | bash | clap | usage | winnow |
|---|---|---|---|---|
| startup | 2.39 ms | 4.55 | 4.47 | 4.44 |
| config-lint-500 | 24.6 ms | 115.5 (735 M) | 73.1 (426 M) | 71.7 (410 M) |
| deploy-sim | 20.9 ms | 55.5 (35 M) | 54.5 (26 M) | 54.1 (26 M) |
| wordops | 303 ms | 159.1 (506 M) | 157.3 (498 M) | 157.4 (497 M) |
| interp-loop | 148 ms | 292.3 (1 997 M) | 263.6 (1 815 M) | 260.1 (1 808 M) |

Per call, µs, loop included (`:` is the loop):

| command | bash | clap | usage | winnow |
|---|---|---|---|---|
| `:` | 2.3 | 4.4 | 4.0 | 3.9 |
| `set -f +f` | 3.1 | 131.2 | 82.0 | 5.8 |
| `declare -i n=1` | 3.2 | 31.4 | 6.8 | 6.6 |
| `local` | 2.4 | 27.8 | 4.6 | 4.4 |
| `compgen -W "a b" a` | 4.3 | 25.9 | 9.8 | 9.1 |
| `ulimit -n` | 3.3 | 18.1 | 6.6 | 6.5 |
| `shopt -q extglob` | 2.9 | 12.5 | 6.5 | 6.3 |
| `unset -v x` | 2.9 | 11.2 | 6.2 | 6.0 |
| `getopts ab o -a` | 3.2 | 11.0 | 7.7 | 7.3 |
| `cd .` | 5.6 | 11.0 | 6.5 | 6.6 |
| `kill -0 $$` | 3.3 | 10.6 | 7.4 | 7.0 |
| `command true` | 2.7 | 10.3 | 6.4 | 6.3 |
| `[ a = a ]` | 3.0 | 9.4 | 7.7 | 7.5 |
| `printf %s x` | 3.2 | 8.7 | 6.5 | 6.5 |
| `echo -n` | 2.6 | 8.1 | 5.3 | 5.2 |
| `export E=1` | 2.8 | 7.9 | 5.3 | 5.2 |
| `trap -p` | 2.9 | 7.8 | 5.2 | 5.1 |
| `test -n x` | 2.9 | 7.6 | 6.2 | 6.0 |
| `shift 0` | 2.6 | 6.7 | 5.1 | 5.1 |
| `type -t ls` | 10.6 | 20.5 | 14.8 | 14.5 |
| `read -r v <<<x` | 11.1 | 18.8 | 12.6 | 12.3 |

**mold** (`winnow-args-cmdline` at `6b5e54ad`). Binaries: 63 798 752 built-in,
63 879 464 with the feature (+79 KiB). Instructions a word over a bare
`--version`, and the whole run (parse, then exit at `--version`):

| line | built-in | winnow-args |
|---|---|---|
| 2 496-word link line | 3 333, 1.73 ms | 558, 0.99 ms |
| 2 751 option words | 11 521, 3.83 ms | 516, 1.20 ms |
| 2 501 input files | 388, about 1.0 ms | 517, about 1.0 ms |

The run times are a millisecond of process, within noise of each other for
the input files; the instruction counts are the measure. `just bench-ld`
and `just bench-shell` reproduce the mold and brush tables.

**The examples** (`PARSE_N`, warm instructions a parse): `pwd -P` 347,
`cd -P /tmp` 519, `unset -fv x` 824, `declare -i +x n=1` 947, `test -n x` 971,
`set -eu +x -o pipefail` 1 094, `kill -s TERM 1234` 1 196, `echo -n a b c`
1 241, `printf -v x %s a` 1 246, `read -rp prompt -t 2.5 a b` 1 699;
`ld -shared -o out.so a.o` 1 064, 18 mixed `ld` words 5 837 (324 a word), a
2 495-word link line 526 a word.

## 51. Help spells every form

A flag's row now shows all its spellings and how its value is given: `-e, +e`
for an `Option<bool>` with `plus`, `+o <OPT>` for a `+`-only flag (it was
hidden), `-l, -L` for a second `short`, `--inspect=<PORT>` under
`require_equals`, `--pager [<WHEN>]` and `--build-id[=<KIND>]` with
`default_missing`, `--platform <V> <V> <V>` for `values = 3`
(`help::Item::{more_shorts, plus, optional_value, values}`).

The derives also read an enum variant's value where they build it (a
`Subcommand` payload, an `Occurrence` value), as they do struct fields, so one
that is only printed is not dead code. Help is off the parse path, and the
read is a borrow the optimizer drops: the bench lines measure 2 165 and
3 434 warm instructions (2 186 and 3 476 before).

## 52. The brush port, phase by phase

Kept from the brush checklist when the checklists became one. Phases 1 to 6
were a fourth engine in brush's `abstract-arg-parsing` layout, since dropped;
phase 7 is `winnow-port`, on brush's engine-neutral contracts. Per call is
100 000 calls in a function, loop included (`:` is the loop alone), µs, clap
→ winnow-args; scripts are `benchmarks/three-way.py`, 15 samples, pinned.

| phase | builtins | per call | scripts |
|---|---|---|---|
| 1 | `pwd`, `read`, `mapfile`, … | (none timed) | within noise of clap |
| 2 | `echo`, `printf`, `exec`, `eval` | `printf -v x %s y` 9.9 → 7.2, `echo -n` 7.2 → 4.5, `read -r x <<< a` 18.2 → 11.7 | −1 % to +3 % |
| 3 | `set`, `declare`, `complete` | `set -f +f` 120 → 5.2, `declare -i n=1` 30 → 5.5, `compgen -W "a b" a` 26.2 → 10.6 | config-lint-500 107 → 70 ms, interp-loop 285 → 232 |
| 4 | `kill`, `ulimit`, `trap`, `shift` | `ulimit -n` 20.0 → 8.4, `kill -0` 10.4 → 7.6, `trap -p` 8.6 → 5.6 | — |
| 5 | `test`, `getopts`, `help` | `[ a = a ]` 9.1 → 7.3, `getopts ab o -a` 10.7 → 7.7 | — |
| 6 | the rest | — | config-lint-500 103 → 59 ms, interp-loop 286 → 246 |

Phase 6's binary was 65 KiB under clap's; phase 7's 256 KiB under usage-port's
(6 472 000 bytes), level with it on the scripts and 0.1 to 0.6 µs faster a
call, `set -f +f` 77.8 → 5.0 µs. Step 50 has the current numbers.

## 53. After the API and documentation rework

`d964cec` (public `BStr`, `value_optional` gone) through the doc fixes
(hidden `ValueEnum` variants out of errors, `Shell: FromArg`, role fields
checked). Pinned (node 3, core 96), `perf instructions:u`. Warm instructions
per parse on the bench lines, before (`1157798`) / after:

| line | release | release-lto |
|---|---|---|
| `-v --path /tmp/x a b c` | 2 165 / 2 173 | 1 765 / 1 765 |
| mise `use -g node@20` | 3 548 / 3 548 | 3 430 / 3 455 |

Each move arrives with a commit that changes no parse path (release `wa` with
the doc fixes, release-lto mise with the API commit): layout, not work. The
`just perf` tables agree with step 50 within the cold counter's noise; warm
times are equal or lower on every line (mise `use -g node@20` 321 ns, 345 at
step 50). The examples: brush builtins 338 to 1 697, ld 265 and 324 a word,
as in step 48.

## 54. On releases only: winnow 1.0.4, bpaf 0.9

Every number above was taken with winnow patched to a local checkout of its
`main` and bpaf 0.10 from git. Both are gone: the tree builds on crates.io
releases alone. Pinned as in step 53, warm instructions per parse, patched
winnow / released:

| line | release | release-lto |
|---|---|---|
| `-v --path /tmp/x a b c` | 2 173 / 2 143 | 1 765 / 1 773 |
| mise `use -g node@20` | 3 548 / 3 596 | 3 455 / 3 407 |

Within 1.4 % either way; the ratios to usage hold (`-v --path /tmp/x a b c`:
2 850 cold instructions and 196 ns warm against usage's 5 595 and 506; mise
`use -g node@20`: 4 015 and 326 ns against 7 702 and 789).

bpaf is now the released 0.9, with usage's own 0.9 shadow for mise (the same
source as its 0.10 one). On the small CLI it is where 0.10 was (143 031
instructions, 14.9 µs warm, clap 136 568 and 15.3 µs) in a smaller binary
(581 552 bytes stripped against 825 416). At mise's scale it is far slower:
22.0 M instructions and 2.65 ms a parse, against 1.2 M and 165 µs for 0.10 and
4.9 M and 758 µs for clap. Earlier steps' bpaf columns are 0.10's.

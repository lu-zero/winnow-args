# winnow-args design notes (phase 1)

## What the references taught

- **usage** (`../usage/argv`) is the one to beat. Its derive emits `static`
  tables, the runtime walks argv once, yields events borrowed as `&[u8]`, and
  allocates nothing. Its bench (`benches/gate`, `tasks/perf-shadow.sh`) measures
  a *cold* parse: the same binary run with `PARSE_N=0` and `PARSE_N=1`, the
  difference in instructions. Construction counts, because a CLI parses once.
- **clap** builds its command tree at runtime on every start, so the tree is
  most of its cost.
- **bpaf 0.10** has the nicest composition vocabulary —
  `short('p').long("path").argument("PATH")`, `construct!`, one-occurrence
  primitives folded by a product — but its runtime runs every parser as a
  cooperative task (`Future`s, `Rc`, `BTreeMap`). The parser is also rebuilt
  per run, and on this bench that makes it the slowest of the four.

The goal: bpaf's composition shape, winnow's combinators, usage's cost.

## The stream: `&[&BStr]`, one token per word

`Argv` borrows argv as `&[&BStr]` and is a winnow `Stream` whose tokens are
words. Nothing is copied or joined, a word is never re-split (`"a b"` stays
one value), and any byte, NUL included, may appear in a word. On Unix,
`&OsStr` → `&BStr` is free (`as_encoded_bytes`), so a caller that already
holds `&[&OsStr]`, as usage's harness does, parses with no allocation.

Earlier drafts flattened argv into one NUL-terminated buffer so that every
winnow byte parser applied directly. That cost a copy per parse (most of the
remaining cost) and relied on NUL never appearing in a word; it is gone.

Offsets are counted as if each word were followed by one separator. So a
position inside a short bundle is still one number, which shrinks as the lexer
reads a letter. `repeat`'s "parser must consume" check and error offsets
therefore work inside `-vq` too. Generic winnow token parsers (`any`, `take`)
see whole words; `next_slice` must not split a word.

## Mode lives in the stream, and the checkpoint is the whole state

A word position alone can't say whether we are part-way through `-vp` or past
`--`. `Argv` carries the byte position inside the current word and
`Mode::{Word, Bundle, Stopped}`, and its checkpoint is a copy of the whole
(small, `Copy`) struct.

Two winnow facts drove this:

- `Stateful<I, S>` does not restore `S` on `reset`, so it's unsound under `alt`.
- `winnow::stream::Checkpoint::new` is `pub(crate)`, so a custom stream needs its
  own checkpoint type. `Stream::Checkpoint` only asks for `Offset + Clone + Debug`,
  which `Argv` itself satisfies.

## Lexer, then continuation

`token::arg` reads exactly one item: `Long { name, value }`, `Short(char)`,
`Word`, or `Separator`. It never decides whether a flag takes a value. The
caller knows the flag and finishes it:

- `token::value`: attached `=value`, the rest of a bundle (minus one `=`), or
  the next word if that word isn't flag-like.
- `token::no_value`: rejects `--switch=x`.

This split lets one lexer serve both composition styles.

## Error type without `ErrMode`

winnow 1.0 lets the error type be modal itself. `alt` asks
`ParserError::is_backtrack`, and `cut_err` needs `ModalError`. `Error` implements
both with a `cut` flag.

A failed `alt` branch builds a 24-byte value with no heap. The token and
message are boxed only on committed errors. Error classes follow usage's
grammar codes (`unknown_flag`, `missing_flag_value`, …).

## Why not `unordered_seq!`

winnow's permutation macro applies each parser *exactly once*. Options are
0..n with per-field policies (absent → default, repeated → last wins or count
or collect). So accumulation is a loop plus a fold, which is how bpaf's
product and usage's `Partial` work too.

## Two composition styles over the same primitives

1. **Combinators.** `args(alt((VERBOSE.switch().map(|()| v = true), …)))`.
   `Named` implements `Parser<Argv, Arg, Error>` for one occurrence. `args`
   repeats the item and then reports leftovers precisely. Each `alt` branch
   re-lexes the token, so cost grows with the number of flags.
2. **Derive.** One `while` loop with `match arg { Long{name} => match &**name
   { b"verbose" => … }, Short(c) => match c { 'v' => … } }`. rustc compiles the
   lookup, nothing is re-lexed, and fields are locals. `Cli::parse_argv` is
   still a plain winnow parser.

## Numbers (aarch64, `tasks/perf.sh`)

Three measurements per framework, all on the same argv:

- **instr**: instructions for one cold parse, `PARSE_N=1` minus `PARSE_N=0`.
  This host's glibc uses an instruction valgrind can't decode, so it's the
  `perf stat -e instructions:u` median of 31 runs (±~100 noise).
- **cold ns**: wall time of the first parse in a fresh process, median over 31
  processes. This includes first-touch page faults, and is what a CLI pays.
- **warm ns**: in-process min / median over 2000 short rounds (`time-sweep`).

| argv | framework | instr | cold ns | warm ns (min) |
|------|-----------|------:|--------:|--------------:|
| `-v --path /tmp/x` | usage | 1428 | 3280 | 140 |
| | wa derive | 570 | 1460 | 46 |
| | wa combinators | 1216 | 1660 | 157 |
| | bpaf 0.10 | 31280 | 56781 | 3455 |
| | clap 4 | 24413 | 26961 | 2557 |
| `-vp/tmp/x` | usage | 1266 | 2900 | 125 |
| | wa derive | 509 | 1020 | 39 |
| | wa combinators | 1101 | 1700 | 139 |
| | bpaf 0.10 | 36032 | 73421 | 4104 |
| | clap 4 | 22865 | 21521 | 2394 |
| `--verbose --path=/tmp/x` | usage | 1557 | 2360 | 153 |
| | wa derive | 611 | 1600 | 49 |
| | wa combinators | 1263 | 1960 | 160 |
| | bpaf 0.10 | 31347 | 58941 | 3491 |
| | clap 4 | 23800 | 32060 | 2522 |

Cold ns varies a few hundred ns between runs; compare ratios, not digits.
Warm, the combinators cost ~3× the derive because each `alt` branch re-lexes
the token.

Caveat: this is a two-flag CLI. usage's static tables are built for mise scale
(211 commands), and it also does work we skip (help/version flags, spec
metadata). The real comparison is a `mise-winnow-args` shadow once we have
subcommands and positionals.

## Next steps

See `CHECKLIST.md`. In order: positionals, `Vec`/count occurrences,
subcommands (`enum` derive, and a combinator the parent loop delegates to),
help/version, then the shadow generator and usage's corpus.

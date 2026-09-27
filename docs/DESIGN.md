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

## The stream: one `BStr`, words terminated by NUL

`ArgvBuf` copies argv into one buffer, `word\0word\0…`. NUL can't appear in a
real argv on Unix or Windows, so it can mark word ends. `Argv` wraps the
`&BStr` and implements winnow's `Stream` by delegation.

Alternatives considered:

- **`TokenSlice<&BStr>` (a stream of words).** Zero-copy, but every word-level
  parser has to run a byte-level sub-parser on the word, and winnow's
  `ParserError<I>` is keyed by input type. That means two error worlds and
  explicit bridging at every flag.
- **Pre-lex into a `Vec<Token>`.** A bundle can't be lexed without the
  table: `-pfoo` is `-p foo` if `p` takes a value and `-p -f -o -o` if it
  doesn't. So a lexer can't be context-free here.

The flat buffer costs one allocation per parse (~40 bytes for typical lines).
Offsets are byte positions, so errors can point at the letter inside `-vx`.

## Mode lives in the stream, and the checkpoint saves it

A flat position alone can't say whether the next `p` starts a word or is the
next letter of `-vp`, or whether we are past `--`. `Argv` carries
`Mode::{Word, Bundle, Stopped}`, and `ArgvCheckpoint` saves it together with
the byte position.

Two winnow facts drove this:

- `Stateful<I, S>` does not restore `S` on `reset`, so it's unsound under `alt`.
- `winnow::stream::Checkpoint::new` is `pub(crate)`, so a custom stream needs its
  own checkpoint type. `Stream::Checkpoint` only asks for `Offset + Clone + Debug`.

Every mode change also consumes bytes, so `repeat`'s "parser must consume"
guard still works.

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

This host's glibc uses an instruction valgrind can't decode, so counts come
from `perf stat -e instructions:u`, the median of 31 runs (±~100 noise).

| argv                      | usage | wa derive | wa comb | bpaf 0.10 | clap 4 |
|---------------------------|------:|----------:|--------:|----------:|-------:|
| `-v --path /tmp/x`        | 1563  | 1368      | 1939    | 31351     | 24315  |
| `-vp/tmp/x`               | 1300  | 1038      | 1378    | 35994     | 22796  |
| `--verbose --path=/tmp/x` | 1589  | 1544      | 2078    | 31441     | 23674  |

Wall clock (min of 2000 rounds): usage ~140 ns, wa derive ~136 ns, wa comb
~217 ns, clap ~2.5 µs, bpaf ~3.5 µs.

Caveat: this is a two-flag CLI. usage's static tables are built for mise scale
(211 commands), and it also does work we skip (help/version flags, spec
metadata). The real comparison is a `mise-winnow-args` shadow once we have
subcommands and positionals.

## Next steps

See `CHECKLIST.md`. In order: positionals, `Vec`/count occurrences,
subcommands (`enum` derive, and a combinator the parent loop delegates to),
help/version, then the shadow generator and usage's corpus.

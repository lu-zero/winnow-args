# winnow-args design notes

Why winnow-args is built the way it is. What each piece does is in its rustdoc;
what it costs is in [PERF.md](./PERF.md).

## What the references taught

- **usage** is the one to beat. Its derive emits `static` tables, its runtime
  walks argv once, yields events borrowed as `&[u8]` and allocates nothing.
  Its bench measures a *cold* parse (`PARSE_N=1` minus `PARSE_N=0`, in
  instructions), because a CLI parses once and construction counts.
- **clap** builds its command tree at runtime on every start, so the tree is
  most of its cost.
- **bpaf** has the nicest composition vocabulary
  (`short('p').long("path").argument("PATH")`), but builds its tree of parsers
  at runtime on each run, as clap does.

The goal: bpaf's composition shape, winnow's combinators, usage's cost.

## The stream: `&[&BStr]`, one token per word

A word is never copied, joined or re-split, and any byte may appear in it; on
Unix `&OsStr` → `&BStr` is free, so the only allocation is the list of words,
which a caller holding `&[&BStr]` skips. Offsets count one separator after each
word, so a position inside a bundle is still one number that shrinks as
letters are read: `repeat`'s progress check and error offsets work inside `-vq`.

## The checkpoint is the whole state

A word position alone cannot say whether we are inside `-vp` or past `--`, so
`Argv` carries the byte inside the word and a `Mode`, and its checkpoint is a
copy of the whole 32-byte struct. Two winnow facts forced this: `Stateful` does
not restore its state on `reset` (unsound under `alt`), and `Checkpoint::new`
is private, so a custom stream brings its own checkpoint type.

## Lexer, then continuation

`token::arg` reads one item (long flag, short letter, word, `--`) and never
decides whether a flag takes a value: the caller knows the flag and finishes it
with `Arg::read_value` or `Arg::check_switch`. One lexer then serves the
combinators, a hand-written `dispatch!` and the derive.

## An error that is its own mode

winnow 1.0 lets the error type be modal: `alt` asks `is_backtrack`, `cut_err`
needs `ModalError`. `Error` implements both with a flag, so a failed `alt`
branch is a 24-byte value with no heap; the token and message are boxed only
when the error is final.

## A loop and a fold, not `unordered_seq!`

winnow's permutation macro applies each parser exactly once. Options occur 0 to
n times, each with its own policy (default, last wins, count, collect), so
accumulation is a loop plus a fold, as in bpaf's product and usage's `Partial`.

## Why the derive is the fast path

The derive lexes each item once, looks its name up in a `match` rustc compiles,
and keeps every field in a local of one function. Combinators pay winnow's
`repeat`/`alt` bookkeeping per item, and `Named` re-lexes and rewinds per flag
that does not match. Layout matters as much as instruction count: `token::arg`
is `#[inline(always)]` so its `Result` never goes through memory (PERF.md step 5).

A feature costs nothing to a struct that does not use it: the derive emits
only the arms and locals the struct declares, which each feature's PERF entry
checks against the bench lines.

## Order as meaning

A struct of fields keeps how often a flag came, not where. A linker needs
`--as-needed a.o --no-as-needed b.o` in order, so `#[derive(Occurrence)]` makes
an enum with one variant per flag or input, and `#[arg(sequence)]` collects
them in order for the program to fold; ordinary fields hold the rest. mold's
whole command line is one such enum.

## Flatten, checked at compile time

A flattened struct's flags are parsed in the parent's loop. Every struct's help
items are `const`, and a `const` assertion in the parent compares their
spellings, so two fields spelling the same flag fail the build rather than the
parse. The flattened struct is flags only, which keeps one positional counter
and one subcommand per loop.

## Help is data, off the parse path

Help is `static` data (`help::Command`) emitted beside the parser and read only
when help or an error is printed. A proc macro cannot see winnow-args'
features, so prose goes through `winnow_args::__text!`, which winnow-args
defines by its `help-text` feature: the literal, or `""`. The structure stays.

Colour is a palette per terminal depth (`color::Theme`); escapes are constants,
`std::io::IsTerminal` answers the rest, and widths are measured without them.
No dependency. The default 16-colour palette uses only the basic colours, which
the terminal's own theme defines, so it suits light and dark backgrounds.

## Completion asks the program

A completion script only forwards the line to `PROG __complete_word__`, and the
program answers from its own help data. The script stays a few lines per shell
and never goes stale: what can be completed is what the binary parses.

## Generated code's contract

Generated code calls what users should not through `winnow_args::__private`,
and builds `help::Item`, `help::Command` and `token::ValueOptions` by struct
literal. Those types can gain fields in a release; the derive of the same
release fills them, so the two crates are released together (`=` version).

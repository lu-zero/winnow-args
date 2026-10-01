# winnow-args design notes

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

- `Arg::read_value`: attached `=value`, the rest of a bundle (minus one `=`), or
  the next word if that word isn't flag-like.
- `Arg::check_switch`: rejects `--switch=x`.

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

## Three composition styles over the same primitives

1. **Named combinators.** `args(alt((VERBOSE.switch().map(|()| v = true), …)))`.
   `Named` implements `Parser<Argv, Arg, Error>` for one occurrence; `args`
   repeats the item and then reports leftovers precisely. `dispatch!` on
   `token::kind` keeps words away from the flag branches. Inside the flag
   `alt`, each branch still re-lexes, so flag cost grows with the flag count. winnow's `alt` takes at most 9
   parsers; a larger flag set nests them.
2. **Name dispatch.** One `dispatch!` on `token::arg`, with arms such as
   `a @ (Arg::Long(LongFlag { name: b"path", .. }) | Arg::Short(ShortFlag { letter: 'p', .. }))`.
   rustc compiles the names into a `match`, and each item is lexed once.
   Nesting `dispatch!` (on `--` and then on the name) does not work: each
   level is a `move` closure, and the inner one would move the outer's
   `&mut` captures.
3. **Derive.** The same `match`, generated, in one plain loop with locals.

Why the derive is fastest: it lexes each item once, looks names up in a
compiled `match`, and keeps every field in a local of one function. It stops
on `is_empty()` and handles `--` as one more arm. The combinators add
winnow's `repeat`/`alt` bookkeeping per item: a checkpoint copy, the
progress check, and a separator branch. `Named` also adds a re-lex and a
rewind per non-matching flag.

Layout matters as much as instruction count here (see PERF.md step 5):
`token::arg` is `#[inline]` so its `Result<Arg, Error>` never goes through
memory. `Arg`'s continuations are `&self` methods. `Argv` is kept at 32
bytes because the combinators copy it on every checkpoint.

## Numbers

Measurements, method and per-feature deltas live in [PERF.md](./PERF.md).
Headline, flags only: the derive is at ~0.3–0.4× usage's instructions and
~⅓ of its warm time; the combinators are about level with usage, 3–4× slower
than the derive warm, because each `alt` branch re-lexes the token.

At mise's full scale (211 commands, `docs/PERF.md` step 17) the derive runs
`mise use -g node@20` in 3 810 instructions and ~320 ns warm, against usage's
7 549 and ~783 ns; clap needs 4.9 M instructions. Its binary is 71 % larger
than usage's under the `release-lto` profile (PERF.md step 28).

The mise shadow is generated from usage's (`tasks/gen-mise-shadow.py`), so
both parse the same 211-command CLI, and `bench/tests/mise.rs` holds them to
accepting the same lines.

## Help

Help is `static` data (`help::Command`) the derive emits beside the parser.
Nothing reads it during a successful parse: `-h`/`--help` return it inside an
error and `help::render` lays it out only then. Rendering wraps to
`help::width()` (`COLUMNS`, else 100, clap's fallback) with usage's rule for
the column (at most two fifths of the page; a wider item has its description
on the next line). The wrapping borrows slices of the text, so a line that
fits costs one scan.

A proc macro cannot see winnow-args' features, and a `#[cfg]` in generated
code would test the user's crate. So the derive wraps each piece of prose in
`winnow_args::__text!`, a `macro_rules!` whose definition winnow-args picks by
its `help-text` feature: the literal, or `""`. The structure (names, values,
defaults, choices, the version) stays either way.

Color is a `help::Style`: one paint per role — `header`, `program`, `flag`,
`command`, `placeholder`, `env`, `default`, `choice`, `dim` (annotation labels,
the brackets of an optional `[NAME]`, `...`), `code` (`` `quoted` `` spans in
descriptions, backticks kept), and for errors `error`, `invalid`, `valid`. The default keeps to greens for what is
typed and cyans/teals for structure and values, with no yellow: in 16 colors
bold cyan headings, bold green flags and subcommands, cyan value names; in 256
colors the tamer teal 73, green 71 and slate teal 109, with red 167, rose 174
and sage 108 for errors (chosen for contrast on dark and light backgrounds,
docs/PERF.md step 34). Annotations paint their values, not their labels: the
environment variable cyan (teal 37), the default green (sea green 72), each
possible value bright green (green 71), each on its own; wrapping skips escape
sequences when counting columns. It follows usage's layout rules: the program
plain, and only the name painted in `[NAME]`. `Palette::CLAP` reproduces clap
4's default styles: no color in help (bold and underline only), color only in
errors, and without clap's `color` feature nothing at all.

Terminals differ in how many colors they show, so a `help::Style` is a
`color::Palette` plus a `color::Depth` (none, 16, 256, 24-bit). `Depth::detect`
reads the environment the way `supports-color` and `anstyle-query` do. A
`color::Theme` holds a palette for each depth, and the richest one the terminal
can show is used. A color deeper than the terminal is mapped to the nearest one
it has (the 256-color cube or gray ramp, then the basic 16 by weighted
distance). The default theme uses only the 16 basic colors, whose look is the
terminal theme's, so it suits light and dark backgrounds. Each paint is written
as one SGR sequence (`1;33`), so clap's `1` then `4` becomes `1;4`: the same on
screen. Windows is out of scope for now: no console API, no Windows-only
detection rules. Rendering
builds each left-hand cell as painted text plus its visible width, so padding
and wrapping never count escapes, and the painted page with its escapes
stripped is the plain page. `render` and `render_help` stay plain strings;
`report`, which knows the stream, picks `Style::auto` (`NO_COLOR`, then
`CLICOLOR_FORCE`, then whether it is a terminal). No dependency: the escapes are
constants and `std::io::IsTerminal` answers the rest.

## Next steps

See `CHECKLIST.md`: binary size (without help prose mise's shadow is still
39 % larger than usage's with its prose, `release-lto`, `docs/PERF.md` step 28), usage's conformance corpus, then consolidating the
draft history.

# winnow-args

[![LICENSE](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Crates.io](https://img.shields.io/crates/v/winnow-args.svg)](https://crates.io/crates/winnow-args)
[![docs.rs](https://docs.rs/winnow-args/badge.svg)](https://docs.rs/winnow-args)
[![CI](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml/badge.svg)](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml)

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow) parsers.

winnow-args parses a program's arguments into typed values, typically a
struct with one field per flag or positional. It distinguishes:

- **flags**, long (`--verbose`) or short (`-v`); short flags can be bundled,
  `-vq` for `-v -q`;
- **values** of flags, given as the next word (`--path /tmp`, `-p /tmp`) or
  attached (`--path=/tmp`, `-p/tmp`);
- **positionals**, words identified by their position: `a` and `b` in
  `cp a b`;
- **subcommands**, words that select a command with arguments of its own:
  `add` in `git add -p`;
- `--`, after which every word is a positional.

A failed parse returns an `Error` with its kind, the word at fault and its
offset.

There are two ways to say which of these a program takes:

- `#[derive(Args)]` on a struct. A field is a flag or a positional, its type
  says how many values it takes and what they convert to, and its doc comment
  is its help. This is what most programs want, and the faster of the two.
- By hand, with `winnow_args::combinator`. A flag is a winnow parser, and
  `alt`, `repeat` and the rest of winnow combine flags into a command line.

Either way the words are read where the shell left them: nothing is copied or
re-split, and the bytes need not be UTF-8 until a value's type asks for text.
Both also cover command lines older than `--long`: `+x` options as in a bash
builtin, and flags whose order matters as in `ld`.

```sh
cargo add winnow-args
```

```rust
use std::path::PathBuf;
use winnow_args::Args;

#[derive(Args, Debug)]
struct Cli {
    /// Say more.
    #[arg(short, long)]
    verbose: bool,
    /// Where to look.
    #[arg(short, long)]
    path: Option<PathBuf>,
    #[arg(positional)]
    files: Vec<String>,
}

fn main() {
    let cli = Cli::parse(); // prints help or the error, and exits, if it must
    println!("{cli:?}");
}
```

A test calls `Cli::parse_from(["-v", "a"])`, without the program name, and
gets a `Result`. The same command line written by hand, with
`winnow_args::combinator`, is next to this derive in
[`examples/example.rs`](https://github.com/lu-zero/winnow-args/blob/HEAD/examples/example.rs).
The [crate docs](https://docs.rs/winnow-args) list the entry points, and each
derive's page lists every attribute it takes.

## Features

| | winnow-args | usage | clap |
|---|:-:|:-:|:-:|
| Flags, values, positionals, subcommands, global flags, `flatten` | ✓ | ✓ | ✓ |
| Conflicts, requirements, groups | ✓ | ✓ | ✓ |
| Help from doc comments | ✓ | ✓ | ✓ |
| Completion scripts | ✓ | ✓ | ~ |
| Parser generated at compile time, words borrowed | ✓ | ✓ | – |
| Combinators, without a macro | ✓ | – | ~ |
| `+x` options, as a shell builtin takes | ✓ | – | – |
| Long options with one dash, `-z` keywords, as GNU `ld` takes | ✓ | – | – |
| Flags kept in command-line order | ✓ | ~ | ~ |
| Flag constraints checked at compile time | ✓ | – | – |
| Help prose left out of the binary by a feature | ✓ | ~ | – |
| A CLI built at runtime, external subcommands | – | ✓ | ✓ |
| Conditional requirements (`required_if`) | – | ✓ | ✓ |
| Man pages, markdown | ~ | ✓ | ~ |
| "Did you mean" suggestions | ✓ | ✓ | ✓ |

`✓` built in · `~` with some work, or in a companion crate · `–` not offered.
[`docs/COMPARISON.md`](https://github.com/lu-zero/winnow-args/blob/HEAD/docs/COMPARISON.md) has every row, and bpaf.

Man and Markdown pages are made by
[`winnow-args-man`](https://crates.io/crates/winnow-args-man) and
[`winnow-args-markdown`](https://crates.io/crates/winnow-args-markdown).

## Performance

The parser is generated at compile time, so a parse costs the same however
many commands the program declares; clap and bpaf build theirs at each start.

![Time per parse for two command lines, log scale: winnow-args against usage, clap and bpaf](https://raw.githubusercontent.com/lu-zero/winnow-args/HEAD/docs/performance.svg)

The measurements and the method are in the repository's
[benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

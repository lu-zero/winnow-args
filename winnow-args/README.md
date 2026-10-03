# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow) parsers.

A command line reaches the program as a list of words, already split by the
shell. winnow-args turns that list into a value of the program's own type, or
into an error that says which word is wrong. A word is one of:

- a **flag**: `--verbose`, or `-v`; short flags bundle, so `-vq` is `-v -q`;
- a flag's **value**: `--path /tmp`, `--path=/tmp`, `-p /tmp` or `-p/tmp`;
- a **positional**: a plain word that means what its position says, as the
  two files of `cp a b`;
- a **subcommand**: a word that names what the rest of the line is for, as
  `add` in `git add -p`;
- `--`: every word after it is a positional, whatever it looks like.

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
[`examples/example.rs`](https://github.com/lu-zero/winnow-args/blob/HEAD/winnow-args/examples/example.rs).
The [crate docs](https://docs.rs/winnow-args) list the entry points, and each
derive's page lists every attribute it takes.

## What it covers

- **The conventions clap and usage share**: short and long flags, bundles,
  aliases, counts, repeated and delimited values, optional values,
  positionals and the ways `--` treats them, subcommands with global flags,
  conflicts, requirements and groups, environment variables and defaults,
  `flatten`. A small extension: `--no-x` pairs, which clap has only in part.
  Unknown flags are errors unless `unknown_flags = "value"`.
- **Help and completion**: help with colour and wrapping; completion
  scripts for bash, zsh, fish, elvish and PowerShell that ask the program
  itself.
- **A shell's builtins**: `+x` options, unknown flags kept as words, options
  that stop at the first operand.
- **A linker's command line**: GNU ld's dash rules (`long_only`), `-z`
  keywords, options whose order matters kept as one sequence (`Occurrence`),
  `@file` response files, C-syntax numbers.
- **Without the derive**: the lexer (`token`) and bpaf-style combinators
  (`combinator`) the derive is built on.

## Performance

The parser is generated at compile time, so a parse costs the same however
many commands the program declares; clap and bpaf build theirs at each start.
Cold instructions and warm time for one parse, on an Ampere-1a, release
build, one core:

| | `example -v --path /tmp/x a b c` | | `mise use -g node@20`, 211 commands | |
|---|---:|---:|---:|---:|
| derive | 2 939 | 196 ns | 4 012 | 328 ns |
| combinators | 4 574 | 396 ns | | |
| usage | 5 683 | 503 ns | 7 720 | 782 ns |
| clap | 136 514 | 15.3 µs | 4 943 837 | 753 µs |
| bpaf 0.9 | 142 994 | 15.0 µs | 21 966 400 | 2.65 ms |

The full tables and the method are in the repository's
[benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

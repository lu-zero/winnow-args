# winnow-args

Command line parsing built on [winnow](https://github.com/winnow-rs/winnow).
Describe the command line as a struct and the derive writes its parser: one
loop that reads each word where it is and looks flags up in a `match`. Nothing
is copied, and an argument that is not UTF-8 is no special case.

```
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

Tests call `Cli::try_parse_from(["-v", "a"])`, without the program name. The
crate docs list the entry points and map clap's attributes to these; each
derive's page lists every attribute it takes.

## What it covers

- **The usual**: short and long flags, bundles, aliases, counts, repeated and
  delimited values, optional values, `--no-x` pairs, positionals and the ways
  `--` treats them, subcommands with global flags, conflicts, requirements
  and groups, environment variables and defaults, `flatten`.
- **Help and completion**: clap-like help with colour and wrapping; completion
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

The parser is generated at compile time and reads the words in place, so a
parse is short however many flags and subcommands the program has. One cold
parse, in instructions, and its warm time:

| | a small CLI | mise's 211 commands |
|---|---:|---:|
| winnow-args | 2 939, 196 ns | 4 012, 328 ns |
| usage | 5 683, 503 ns | 7 720, 782 ns |
| clap | 136 514, 15.3 µs | 4 943 837, 753 µs |
| bpaf 0.9 | 142 994, 15.0 µs | 21 966 400, 2.65 ms |

Measured on an aarch64 server; the method, the lines and how to run it are in
the repository's [benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

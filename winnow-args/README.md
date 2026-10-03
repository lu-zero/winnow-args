# winnow-args

Command line parsing built on [winnow](https://github.com/winnow-rs/winnow).
Describe the command line as a struct and the derive writes its parser: one
loop that reads each word where it is and looks flags up in a `match`. A word
is never copied or re-split, and it need not be UTF-8 until a value type
converts it.

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

Tests call `Cli::parse_from(["-v", "a"])`, without the program name. The
same command line by hand is `winnow_args::combinator`, shown next to this
derive in `examples/example.rs`. The crate docs list the entry points, and
each derive's page lists every attribute it takes.

## What it covers

- **The usual**: short and long flags, bundles, aliases, counts, repeated and
  delimited values, optional values, `--no-x` pairs, positionals and the ways
  `--` treats them, subcommands with global flags, conflicts, requirements
  and groups, environment variables and defaults, `flatten`. Unknown flags
  are errors unless `unknown_flags = "value"`.
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

The parser is generated at compile time and reads the words in place. What a
parse costs, on which host, and how to reproduce it are in the repository's
[benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers. The command line is a slice of byte-string words, so nothing is
copied or joined and a non-UTF-8 argument is no special case; a derived parser
is one loop with a `match` on each flag's name.

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

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

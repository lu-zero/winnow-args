# winnow-args

[winnow](https://github.com/winnow-rs/winnow) parsers for a program's command line.

A flag or a word is a winnow parser (`winnow_args::combinator`). It reads the
word where the shell left it: nothing is copied or re-split, and the bytes
need not be UTF-8 until a value type converts them. `alt`, `repeat` and the
rest of winnow combine those parsers. That covers `--long` and `-l` flags,
positionals and subcommands, and older command lines: `+x` options as in a
bash builtin, and flags whose order matters as in `ld`.

A derive crate is provided to make this easier. It fills a struct from the
arguments.

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

The library exists because usage, bpaf and clap are not both this flexible
and this fast. Cold instructions and warm time on an Ampere-1a, release
build, one core.

`example -v --path /tmp/x a b c`: derive 2 939 instructions and 196 ns,
combinators 4 574 and 396 ns, usage 5 683 and 503 ns, clap 136 514 and
15.3 µs, bpaf 0.9 takes 142 994 and 15.0 µs.

mise's 211 commands, `mise use -g node@20`: derive 4 012 and 328 ns, usage
7 720 and 782 ns, clap 4 943 837 and 753 µs, bpaf 0.9 takes 21 966 400 and
2.65 ms.

The tables and the method are in the repository's
[benchmarks](https://github.com/lu-zero/winnow-args/tree/HEAD/benchmarks).

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## License

MIT or Apache-2.0, at your option.

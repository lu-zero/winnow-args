# winnow-args

Command line parsing built from [winnow](https://github.com/winnow-rs/winnow)
parsers. The command line is a slice of byte-string words (`Argv`), so nothing
is copied or joined and a non-UTF-8 argument is no special case; a derived
parser is one loop with a `match` on each flag's name.

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

## What it covers

- **Three layers**: a lexer (`token`), bpaf-style combinators (`combinator`),
  and the derives (`Args`, `Subcommand`, `ValueEnum`, `Occurrence`), whose rustdoc
  lists every `#[arg(...)]` attribute.
- **The usual**: short and long flags, bundles, aliases, counts, repeated and
  delimited values, optional values, `--no-x` pairs, positionals and the ways
  `--` treats them, subcommands with global flags, conflicts, requirements
  and groups, environment variables and defaults, `flatten`.
- **Help and completion**: clap-like help with colour and wrapping; completion
  scripts for bash, zsh, fish, elvish and PowerShell that ask the program
  itself (`complete`).
- **A shell's builtins**: `+x` options, unknown flags kept as words, options
  that stop at the first operand.
- **A linker's command line**: GNU ld's dash rules (`long_only`), `-z`
  keywords, options whose order matters kept as one sequence (`Occurrence`),
  `@file` response files (`response`), C-syntax numbers.

`docs/CHECKLIST.md` lists what is done and what is not; `docs/CHECKLIST-brush.md`
and `docs/CHECKLIST-ld.md` follow the two ports below.

## Examples

```
cargo run --example example -- -vp /tmp
cargo run --example help -- --help
cargo run --example brush_builtins -- set -eu +x -o pipefail a b
cargo run --example ld -- -shared -o out.so --as-needed -lc a.o -z now
```

`brush_builtins` and `ld` print what they parse, and say what to expect with
`--help`. They are generated (`cargo run -p xtask -- gen examples`) from two ports:

- [brush](https://github.com/reubeno/brush), a bash-compatible shell: every
  builtin and the shell's own command line. Its compatibility suite is
  unchanged; a builtin's arguments cost 1 to 5 µs to handle where clap takes 4
  to 130.
- [mold](https://github.com/rui314/mold), a linker: its whole option set
  behind a feature, its test suite unchanged; a link line parses in a sixth of
  the instructions of mold's own parser.

## Layout

- `winnow-args/`: the runtime. `winnow-args-derive/`: the derives.
- `bench/`: the same command lines in winnow-args, usage, bpaf and clap;
  `tasks/perf.sh [argv...]` runs the comparison.
- `xtask/`: the generators for the mold port, the examples and the mise shadow.
- `docs/DESIGN.md`: why it is built this way; `docs/PERF.md`: measurements,
  one entry per feature.

## Cargo features

- `derive` (default): the derives.
- `help-text` (default): the prose of derived help. Without it, help still
  lists commands, flags, values and defaults, and the binary is smaller.
- `terminal-size`: wrap help to the terminal's width when `COLUMNS` is unset.

## Development

```
cargo test
tasks/check.sh    # every feature set and profile, clippy, docs, tests
```

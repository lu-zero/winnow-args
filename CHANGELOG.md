# Changelog

What changed in each release of `winnow-args`, `winnow-args-derive`,
`winnow-args-spec`, `winnow-args-man` and `winnow-args-markdown`, which are
released together. Versions follow [semver](https://semver.org).

## 0.2.0 (2026-10-11)

- **Added**: an unknown flag or subcommand, or a value that is not one of
  its choices, is reported with what was probably meant: `tip: a similar flag
  exists: --force`. `Error::similar` has it, `Error::tip` the line, and
  `Error::suggested` looks for it for a parse that does not go through
  `parse_words`. Only an error pays for it.
- **Added**: completion of a value that is no file. `hint = "dir"`,
  `"command"` or `"none"` on a field says what the shell is asked for, and
  `complete = function` has the program answer, given what is typed. With no
  hint, the type says (`FromArg::HINT`): a `PathBuf`, a `String` or an
  `OsString` is a file. A value attached to its flag, `--dir=sr`, is
  completed too.
- **Changed**: a number, or a type that is not a path or a word, is no
  longer completed as a file. 0.1 offered files for every value with no
  choices; a type of the program that holds a path now needs
  `hint = "file"`, and a `String` that is no file can say `hint = "none"`.
- **Changed**: `help::Item` has two more fields, `hint` and `complete`, and
  `complete::Completions` has `hint` and `before` where it had `files`.
- **Added**: `*emphasized*` and `**strong**` spans of a description are
  painted in help, italic and bold by default, as `` `quoted` `` ones are. The
  command's own description is painted too, and a span of a heading is
  painted as the heading and as what its marks say.
- **Changed**: `Palette` has two more fields, `emphasis` and `strong`. A
  palette written out in full names them; one that ends in
  `..Palette::DEFAULT` is unaffected.
- **Added**: man and markdown pages for a derived command. With
  `WINNOW_ARGS_SPEC` set to an absolute directory, each derive also writes a
  TOML description of its type; `winnow-args-spec` stitches them into a
  command, `winnow-args-man` and `winnow-args-markdown` render one page per
  command. The program is only checked, so pages can describe a target that
  is installed and cannot be run. Unset, nothing is written and the generated
  parser is unchanged.
- **Added**: `spec = "Name"` on a derived type, and on a field or a variant
  that holds one: the name the documentation files the type under, for two
  types of one name that their source files do not tell apart.
- **Added**: `winnow_args::complete::Shell` as a field's type has its choices
  in the pages.
- **Added**: a hidden `SPEC` constant on `Args`, `Subcommand`, `Occurrence`
  and `FromArg`, which the derives set: a documentation build uses it to
  refuse a field that names a type other than as it is filed, or a value
  whose choices it cannot find.
- **Changed**: the MSRV is 1.88, where a derive can tell which source file a
  type is in.
- **Changed**: the examples are in `examples/` at the top of the repository,
  with a guide and two documentation generators, an xtask and a justfile.

## 0.1.2 (2026-10-07)

- **Added**: an `Option` field with `require_equals` holds `None` for a bare
  flag — getopt's `optional_argument`, without the `"\0"` sentinel a
  `default_missing` needed. Help shows the value as optional, the combinators
  gain `argument_opt` and `argument_opt_as`, the token layer
  `Arg::read_value_opt_with`. A detached `Option` field still refuses a bare
  flag, as usage's corpus has it.
- **Documentation**: a graph of the performance table.

## 0.1.1 (2026-10-03)

- **Fixed**: the crate did not build for a 32-bit target, wasm32 included. A
  compile-time assertion required `Argv` to be exactly 32 bytes, which holds
  only where pointers are 64 bits; it now requires at most 32.

## 0.1.0 (2026-10-03)

The first release.

- **Parsing**: short and long flags, bundles, values attached or in the next
  word, positionals, `--`, subcommands with global flags, `flatten`. Words are
  borrowed, never copied or re-split, and need not be UTF-8.
- **The derives**: `Args`, `Subcommand`, `ValueEnum` and `Occurrence`. The
  parser is generated at compile time; conflicts, requirements and groups are
  declared on the fields, and their flag names are checked at compile time.
- **By hand**: the lexer (`token`) and the combinators (`combinator`) the
  derive is built on, as winnow parsers.
- **Values**: `FromArg`, with `Parsed<T>` for any `FromStr` type, C-syntax
  integers, `key=value`, environment variables and defaults.
- **Shell and linker conventions**: `+x` options, unknown flags kept as
  words, long options with one dash, `-z` keywords, flags kept in
  command-line order, `@file` response files.
- **Help and completion**: help from doc comments, wrapped and coloured;
  completion scripts for bash, zsh, fish, elvish and PowerShell.
- **Cargo features**: `derive` and `help-text` (both default), and
  `terminal-size`.

The MSRV is 1.85.

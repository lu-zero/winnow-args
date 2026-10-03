# Changelog

What changed in each release of `winnow-args` and `winnow-args-derive`, which
are released together. Versions follow [semver](https://semver.org).

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

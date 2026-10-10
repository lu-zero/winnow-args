# winnow-args checklist

What winnow-args does, what it does not yet, and where we chose a side. The
grammar follows usage's argv spec (`docs/spec/argv.md` in
[usage](https://github.com/jdx/usage)) unless a **decision** says otherwise.
Two programs were ported to check it: [brush](https://github.com/reubeno/brush)'s
bash builtins and [mold](https://github.com/rui314/mold)'s GNU `ld` command
line. Measurements are in [PERF.md](./PERF.md), not here.

Legend: `[x]` done, covered by a test · `[~]` partly · `[ ]` not yet.

## Done

### Foundations

- [x] Input is `&[&BStr]`: words borrowed, never copied, joined or re-split;
      they may hold spaces, NUL or non-UTF-8 bytes
- [x] `Argv` is a winnow `Stream` whose checkpoint is the whole lexer state
      (word start, inside a bundle, after `--`, flags stopped), so `alt`
      backtracking is sound; offsets count bytes plus one per word
- [x] `Error` implements `ParserError` + `ModalError` itself; backtracking
      errors do not allocate
- [x] No `unsafe`

### Flags

- [x] `--name`, `--name=value` (first `=` splits), `--name value`, `--name=`
      (present but empty); exact names, no abbreviation
- [x] A detached value is not flag-like; `-` alone is a value; a missing value
      at the end is an error
- [x] `-v`, bundles `-vq`, `-p value`, `-pvalue`, `-p=value`; a value letter
      ends the bundle; multi-byte letters
- [x] Several names: `alias`, a second `short` or `long`
- [x] `allow_negative_numbers`, `allow_hyphen_values`, `require_equals`,
      `keep_equals`, `prefix` (`-lfoo`), `two_dashes`
- [x] `--no-name` negation (`negate`): last spelling wins
- [x] `+x` options (`plus_options`, `plus`): `Option<bool>` tri-state, bundles,
      `+o NAME`
- [x] GNU dash rules (`long_only`): one dash or two, a single-dash word tried
      as a long name before a short takes the rest of it
- [x] Unknown flags kept as words (`unknown_flags = "value"`), collected
      whole (`unknown`), or errors (the default)

### Values

- [x] `FromArg`: `String`, `PathBuf`, `OsString` (lossless on Unix), integers,
      floats, `char`; `Parsed<T>` (any `FromStr`), `CInt<T>` (C-syntax
      numbers), `KeyValue<K, V>`, `Spanned<T>` (with its offset)
- [x] Switch, `count` (saturating), `Option<T>`, required `T`, `Vec<T>` one
      value per occurrence; a repeated single value keeps the last
- [x] `delimiter`, `values = N`, `values = 1..` / `a..=b` with `value_terminator`,
      `choices(…)`, `ValueEnum` (`name`, `alias`,
      `alias_hidden`, `hide`, `rename_all`)
- [x] Command line > `env` > `default_if` > `default` or `default_fn` (with
      `default_note`); `default_missing` for an optional value; an `Option`
      field with `require_equals` holds `None` for a bare flag
- [x] `keywords`: `-z now` is `--now` of a nested `Args` type, in order
- [x] `@file` response files (`response`): nested 10 deep, 4096 files, GNU quoting

### Positionals and subcommands

- [x] `--` stops flags (only the first); positionals in declaration order,
      interleaved with flags; required, optional, then one trailing `Vec`
- [x] `double_dash = "required" | "automatic" | "preserve"`, `stop_flags`
- [x] `restart_token`: positionals restart, flags keep their values
- [x] `Subcommand` enums, nested, `Option<E>`, unit and `Box<T>` variants,
      aliases (visible or hidden); `default_subcommand`, `arg_required_else_help`
- [x] `global` flags at any depth, bundles included
- [x] `flatten` (flags-only structs, nested; a clash is a compile error)
- [x] `sequence` of an `Occurrence` enum: flags and words kept in order, with
      `unknown` and `bundle` variants

### Rules

- [x] `conflicts`, `overrides`, `requires`, `required`, `required_unless`,
      struct-level `group("name", required, multiple)`

### Help, errors, completion

- [x] `-h/--help`, `-V/--version`, `help <cmd…>`, each can be disabled
- [x] Help from doc comments and `help`, `long_help`, `help_heading`, `hide`,
      `about`, `long_about`, `after_help`, `after_long_help`; `[possible values]`,
      defaults and env shown
- [x] Wrapping to `COLUMNS`, else the terminal (`terminal-size`), else 100
- [x] Colour: `help::Style`, `color::Theme` per depth (16, 256, 24-bit),
      chosen by `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE` and the stream
- [x] `help-text` feature: help keeps its structure without the prose
- [x] Error kinds after usage's codes, with the offending word's offset;
      `report` maps them to output and exit status
- [x] Completion scripts for bash, zsh, fish, elvish, PowerShell, answered by
      the program from its help data; the answers are tested, the bash and zsh
      scripts tried by hand
- [x] Man and markdown pages, one per command, from TOML fragments the derive
      writes when `WINNOW_ARGS_SPEC` is set: the program is checked, not run,
      so the pages describe any installed target
- [ ] In those pages: choices of a hand-written `FromArg` or through a type
      alias, a program whose top level is a `Subcommand` enum, two types of
      one target under one name
- [x] The derive's compile errors pinned by compile-fail cases (`tests/ui/`)
- [x] Derives read every built field, so a flag accepted and ignored is not
      dead code in the user's crate

### Conformance and the ports

- [x] `benchmarks/`: one CLI and mise's whole CLI in winnow-args, usage, bpaf and
      clap, measured as usage measures (`just perf`); mise's results checked
      for agreement (`benchmarks/tests/mise.rs`)
- [x] brush (local branch `winnow-port`, on brush's engine-neutral builtin
      contracts): every builtin and the shell's command line, no clap code in
      the binary. The compat suite passes what the clap and usage builds pass,
      0 failed. Covered there: option zones that stop at the first operand
      (`echo`, `printf`), `--` kept as data, `+x` bundles (`set`, `declare`),
      declaration operands, negative numbers (`exit -1`, `history -d -1`),
      signal options (`kill -TERM`), `ulimit`'s optional values, `test` and
      `getopts` taking their words raw
- [x] mold (local branch `winnow-args-cmdline`, behind a feature): every
      option spelling one variant of an `Occurrence` enum, folded in order
      through mold's own handling; generated by `just gen mold`. Its test suite
      gives the same results either way. Covered there: GNU dash rules, `-z`
      keywords and pairs, order-dependent state (`--as-needed`,
      `--push-state`), response files, `--lto-*` forwarding, grouped shorts
      accepted with GNU ld's warning

## Decisions

- **A bare `require_equals` flag on an `Option` field** is `None`, not a
  missing-value error (usage's corpus has no optional-argument flag; this is
  getopt's `optional_argument`, which mold's port needs).
- **`--verbose=x` on a switch** is an error (clap, bpaf), not dropped (usage).
- **An unknown letter in a bundle** fails the whole bundle before any letter
  applies; lenient structs check a bundle of two or more letters whole.
- **Positionals** are required, then optional, then one `Vec`, checked at
  compile time; usage lets an optional before a required one reserve the last
  word, which needs lookahead.
- **Unknown flags are errors** by default (usage treats them as values).
- **Help and errors** use winnow-args' own layout and wording, not bash's or
  GNU ld's.
- **Windows** is out of scope: no console API, no Windows-only detection.
- **brush keeps clap in its dependency graph** through uucore (`printf`'s
  formatter), which requires it; none of it reaches the binary.

## Open

### Grammar and values

- [ ] A variadic positional that is not last; `allow_missing_positional`
- [ ] A negative number routed into a default subcommand's positional
      (usage corpus `default-takes-an-opted-negative-number`)
- [ ] `bool_value` (`--flag=false`), `Option<Option<T>>`, `value_names`
- [ ] `choices_strict = false`: choices as suggestions only
- [ ] Custom value parsers in the derive (`parse_with`)
- [ ] Lossless non-UTF-8 on Windows without `unsafe`; non-UTF-8 flag names
      rejected cleanly
- [ ] GNU ld's extras: errors spelling a single-dash long option as written,
      unambiguous abbreviations (`--whole-arch`), an unreadable `@file` kept
      as a word

### Defaults and rules

- [ ] Bare `env`, `env_fallback`, `deprecated_env`
- [ ] Repeated `default` for a `Vec`
- [ ] `required_if`, `required_if_eq(_any/_all)`, `required_unless_all`,
      `requires_if`, `exclusive`
- [ ] A group as an enum (usage's `ArgGroup`); `validate` expressions; a hook
      on the built struct
- [ ] Selectors naming an ancestor's global flag: ours resolve at compile
      time, within the struct; usage resolves them at runtime
- [ ] Relations on positionals and across `flatten`

### Commands

- [ ] `args_override_self = false`, `subcommand_negates_reqs`,
      `args_conflicts_with_subcommands`, `subcommand_precedence_over_arg`,
      `dont_delimit_trailing_values`
- [ ] `default_subcommand_on_empty`, `default_subcommand_flags`
- [ ] External subcommands (`cargo foo`), multicall (`argv[0]` selects),
      executable views
- [ ] Sigil arguments (`+node@22`), clauses (repeated groups split by `:::`)
- [ ] Mounts: subcommands from a plugin's spec at runtime
- [ ] Deprecation: `deprecated` flags, commands and variables, with a warnings
      channel and `deprecated_warn_at` / `deprecated_remove_at`

### Help and errors

- [ ] A subcommand's help lists the global flags it inherits
- [ ] `long_version`, `usage = "…"`, `before_help`, `display_order`,
      `verbatim_doc_comment`, `note` / `warning`, examples, a logo, `help <topic>`
- [ ] `hide_default_value`, `hide_env`
- [ ] Rendered diagnostics with the command line and a caret; "did you mean"
- [ ] A user palette from the environment (as `GCC_COLORS`)
- [ ] Help width counts characters, not terminal columns

### Completion

- [ ] The fish, elvish and PowerShell scripts run in their shells
- [ ] Value hints (`FilePath`, `DirPath`, extensions), a completion function per
      value, descriptions on value candidates

### Program surface

- [ ] `update_from`: merge another command line into a held value
- [ ] Generated dispatch, testing helpers, declared outputs and exit codes
- [ ] Config-file settings (`setting = "key"`), between env and default
- [ ] Emit usage's KDL spec, so its tools (SDKs, spec diff, completion from
      a spec) work from a winnow-args program
- [ ] Metadata usage carries: `effect`, `author`, `surface`, `available_if`
- [ ] `no_std` + `alloc`

### Conformance, size, ports

- [ ] usage's `binding` corpus run as tests
- [ ] Binary size: 45 % over usage's for mise; share the lexer and
      continuations across structs instead of inlining them everywhere
- [ ] mise's shadow from usage's KDL spec rather than from usage's shadow
- [ ] wild: its parser and integration tests, as for mold
- [ ] brush: `dirs`/`pushd`/`popd` `+N` indices (parsed; brush implements them
      in no engine), `help -m`; brush's `getopts` on winnow-args' lexer, if it pays

## Not gaps

- Prefix inference: usage and winnow-args both refuse it.
- Help templates and clap's styles: `help::Style` and `Theme` instead.
- usage's `flagset`: `flatten`, checked at compile time.
- usage's `value_enum` and `var`: the field's type says both.

# winnow-args checklist

What a CLI argument parser has to get right, in the order we intend to get it
right. Behaviour follows usage's argv grammar (`../usage/docs/spec/argv.md`),
which is also what its conformance corpus (`../usage/corpus/`) tests, unless a
line says otherwise.

Legend: `[x]` done in phase 1 · `[ ]` not yet · **decision** marks a place
where the references disagree and we picked a side.

## 0. Foundations

- [x] Workspace: `winnow-args` (runtime), `winnow-args-derive` (proc-macro), `bench`
- [x] Input is `&[&BStr]`: argv borrowed word by word, never copied, joined or
      re-split; words may hold spaces or NUL
- [x] Custom `Stream` (`Argv`) whose tokens are words and whose checkpoint is the
      whole state, including the lexer mode (word start / inside a short bundle /
      after `--`), so `alt` backtracking is sound
- [x] Offsets count bytes plus one separator per word, so reading one letter of
      a bundle is progress for `repeat`
- [x] Error type implements `ParserError` + `ModalError` directly (no `ErrMode`
      wrapper); backtracking errors do not allocate
- [x] No `unsafe` in `winnow-args`
- [ ] `no_std` + `alloc` (winnow supports it; only `Args::parse` needs std)
- [x] No allocation per parse when the caller holds `&[&OsStr]` (Unix)

## 1. Long options

- [x] `--name` switch
- [x] `--name=value` attached value; only the first `=` splits (`--set=a=b` → `a=b`)
- [x] `--name value` detached value
- [x] `--name=` binds the empty string (present-but-empty ≠ absent)
- [x] Exact name match, no abbreviation (`--verb` ≠ `--verbose`)
- [x] Detached value must not be flag-like: `--path --verbose` is a missing value
- [x] A lone `-` is a value (`--path -` works)
- [x] Missing value at end of line is an error
- [x] **decision** `--verbose=x` on a switch is an error (clap, bpaf), not silently
      dropped (usage)
- [x] Several long names: `alias = "…"` / `alias("…", …)`; `Named<N>::longs([…])`
- [ ] Negative numbers as detached values (`--offset -1`)
- [ ] `allow_hyphen_values`, `require_equals`, `default_missing`
- [ ] `--no-name` negation
- [ ] Non-UTF-8 names are rejected cleanly (names are ASCII-ish in practice)

## 2. Short options

- [x] `-v` switch
- [x] Bundles: `-vq` sets both
- [x] `-p value`, `-pvalue`, `-p=value` (one `=` stripped; `-p==x` → `=x`)
- [x] Value-taking short ends the bundle: `-vpfile` → `v`, `p=file`
- [x] `-` alone is a value
- [x] Multi-byte short names (`-é`) decoded as one `char`
- [ ] **decision** usage rejects a whole bundle containing an unknown letter
      before applying any; we are strict (unknown flag is an error), so a
      partially-applied bundle is never observed. Revisit with lenient mode.
- [ ] Negative numbers vs digit shorts (`-1` is a value unless `-1` is declared)
- [ ] Several short names

## 3. Occurrence semantics (post-binding)

- [x] Switch absent → `false`, present → `true`, repeated → still `true`
- [x] `Option<T>` option: absent → `None`
- [x] Required option: absent → `missing_required_flag`
- [x] Repeated single-value option: last one wins (usage, clap)
- [x] Counting switch (`-vvv` → 3), `#[arg(count)]` on any integer, saturating
- [x] Repeatable option into `Vec<T>`, one value per occurrence, in order
- [x] Value delimiters: `delimiter = ','` on `Vec` flags and positionals, each
      piece converted alone; `Named::arguments_as`, `Arg::values_as`, `token::split`
- [ ] Variadic option (`--include a b`); `dont_delimit_trailing_values`
- [x] `default = "…"` and `env = "VAR"` on flags and positionals: command line >
      environment > default; `with_env` for deterministic tests
- [x] Value enums: `#[derive(ValueEnum)]` (a `FromArg` match on bytes; `name`, `alias`)
- [x] `choices("a", "b")` on string flags and positionals; `invalid_choice` lists the names
- [x] `conflicts`, `overrides`, `requires`, struct-level `group("name", required,
      multiple)` + `group = "name"`, `required`, `required_unless`; exclusivity
      judged on what was supplied, requiredness on what has a value
- [ ] Selectors naming an ancestor's global flag (usage resolves those at runtime)
- [ ] `requires_if`, `required_if_eq`, `default_if`, `exclusive`
- [ ] `value_optional` + `default_missing` (3)
- [ ] `restart_token` (2, on mise's `run`)

## 4. Positionals

- [x] `--` stops flag parsing; only the first `--` is a separator
- [x] A word where nothing accepts one is `unexpected_arg`
- [x] Positional fields, declaration order, interleaved with flags
- [x] Required / optional / trailing `Vec` positionals; `missing_required_arg`
- [x] `positional::<T>(name)` occurrence parser for the combinators
- [x] **decision** the derive requires required → optional → one `Vec` last, at
      compile time. usage instead lets an optional before a required one reserve
      the last word, which needs lookahead.
- [ ] `var_min` / `var_max`; a variadic that is not last
- [ ] `double_dash` modes: `required` (7 in mise), `automatic` (6), `preserve`

## 5. Subcommands

- [x] Enum of subcommands (`#[derive(Subcommand)]`), one word selects, only
      before any positional and never after `--`; the child takes the rest
- [x] Required (`E`) or optional (`Option<E>`) subcommand field; unit and
      `Box<T>` variants; the enum is itself `Args`
- [x] Nested subcommands
- [x] `command(name, inner)` and `Word::after_separator` for the combinators
- [x] Global flags (`#[arg(global)]`): accepted after the subcommand word at
      any depth, bundles included; a subcommand's own declaration wins
- [x] `Globals` trait / `globals(closure)` for the combinators
- [x] Subcommand aliases: `#[arg(alias = "…")]` on a variant; `command(["name", "alias"], …)`
- [ ] `default_subcommand` and `arg_required_else_help` (mise's root has both);
      external subcommands; hidden aliases (help only)
- [ ] Multicall (argv[0] selects)

## 6. Values

- [x] Values are `&BStr` borrowed from the buffer
- [x] `FromArg` conversion: `String`, `PathBuf`, `OsString`, integers, floats, `char`
- [x] `PathBuf`/`OsString` are lossless for non-UTF-8 on Unix
- [ ] Lossless non-UTF-8 on Windows (WTF-8) without `unsafe`
- [ ] Custom value parsers in the derive (`parse_with = ...`)

## 7. Errors

- [x] Error classes mirror usage's codes: `unknown_flag`, `missing_flag_value`,
      `missing_required_flag`, `unexpected_arg`, plus `unexpected_value`,
      `invalid_value`
- [x] Byte offset of the offending token (for carets)
- [ ] Rendered diagnostics with the command line and a caret
- [ ] "did you mean" suggestions
- [ ] Lenient mode: unknown flag-like tokens become words (usage default)

## 8. Help, version, completion

- [ ] `-h/--help`, `-V/--version` supplied unless declared
- [ ] Help rendering from derive doc comments
- [ ] Shell completions; emit a usage KDL spec

## 9. Derive (`winnow-args-derive`)

- [x] `#[derive(Args)]` on named-field structs
- [x] `#[arg(short, long)]` with defaults from the field name (`short` = first
      char, `long` = kebab-case)
- [x] `#[arg(short = 'x', long = "name")]` explicit names
- [x] Field types: `bool`, `Option<T>`, `T` where `T: FromArg`
- [x] Codegen is a single `match` over the lexed token (no `alt` chain), so a
      flag lookup is a compiled string match
- [x] Positionals: `#[arg(positional, value_name = "…")]`, `T` / `Option<T>` / `Vec<T>`
- [x] Subcommands: `#[arg(subcommand)]`, `#[derive(Subcommand)]`, `#[arg(name = "…")]`
- [ ] Flattening, doc-comment help

## 10. Conformance and performance

- [x] `bench/` with usage's methodology: `PARSE_N` binaries, cold parse = N=1
      minus N=0 instruction counts, same argv for every framework
- [x] Timings next to the counts: cold (first parse in a fresh process, median
      over processes) and warm (min / median in a hot loop)
- [x] Compared against usage (the zero-alloc reference), bpaf 0.10 (local
      checkout) and clap 4; falls back to `perf stat` medians where valgrind
      cannot run (this aarch64 host)
- [ ] Run usage's `binding` corpus vectors (needs a runtime/table-driven mode, or a
      generator from the KDL spec)
- [ ] Add a `mise-winnow-args` shadow to `../usage/benches/shadows` via
      `xtask gen-shadow` once subcommands and positionals exist

## Audit: usage's mise shadow (45 attribute keys)

Parsing semantics still missing, by use count: ~~`conflicts` 77, `overrides` 37,
`requires` 23, `group` 20~~, `double_dash` 13, ~~`required` 10, `required_unless` 7~~,
`value_optional`/`default_missing` 3, `restart_token` 2, `default_subcommand` 1,
`arg_required_else_help` 1; plus `-h/--help` and `-V/--version` everywhere
(`disable_help_flag` 3, `disable_version_flag` 1).

Help and metadata only, needed for help output but not for parsing: `help`,
`long_help`, `after_long_help`, `help_heading`, `hide`, `alias_hidden`,
`hide_default_value`, `hide_env`, `about`, `author`, `bin`, `effect` (usage's
side-effect annotation, 198 uses).

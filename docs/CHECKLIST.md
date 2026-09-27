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
- [ ] Several long names (aliases)
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
- [ ] Repeatable option into `Vec<T>`
- [ ] Variadic option (`--include a b`)
- [ ] Defaults, env fallback
- [ ] Choices / value enums
- [ ] Conflicts, requires, groups

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
- [ ] `double_dash` modes (required / preserve / automatic)

## 5. Subcommands

- [ ] Enum of subcommands, one word selects
- [ ] Nested subcommands, global/inherited flags
- [ ] Aliases, default subcommand, external subcommands
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
- [ ] Subcommands (`enum`), flattening, doc-comment help

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

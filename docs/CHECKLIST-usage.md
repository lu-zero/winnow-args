# What usage-rs has that winnow-args does not

usage's Rust framework (`../usage/docs/rust/`) and the spec it emits
(`../usage/docs/spec/reference/`), against winnow-args. Read for behaviour, never
copied. Items already open in [CHECKLIST.md](./CHECKLIST.md) are listed here too,
so this page is the whole gap.

Legend: `[ ]` missing · `[~]` partly there · **skip?** a candidate to leave out,
for a ruling.

## Values and cardinality

- [ ] `var_min` / `var_max` / `num_args = a..=b`: bounds on a `Vec`'s values
      (`VarTooFew`, `VarTooMany`); ours has only `values = N`, exact
- [ ] `variadic`: a flag that takes every following word (`--include a b c`)
- [ ] `value_terminator = ";"`: a variadic ends at this word, not stored
- [ ] `choices_strict = false`: choices as completion suggestions, any value accepted
- [ ] `bool_value`: `--flag=true` / `--flag=false` on a `bool`
- [ ] `Option<Option<T>>`: absent, bare and valued told apart without a sentinel
- [ ] `value_names = ["A", "B"]`: a placeholder per word of a `values = N` flag
- [ ] Custom value parser in the derive (`parse_with = f`); usage leaves it out
      on purpose (a closure has no portable spelling)

## Defaults and environment

- [ ] Bare `env`: the variable's name inferred from the field
- [ ] `env_fallback("A", "B")`: more variables, in order
- [ ] `deprecated_env("OLD")`: tried last, and reported when it supplied the value
- [ ] `default_fn = f` (+ `default_note = "…"` for help): a default computed at parse time
- [ ] `default_if("--json", "true")`: a default when another flag is given or has a value
- [ ] Repeated `default = "…"` for a `Vec` field

## Relations and validation

- [ ] `required_if`, `required_if_eq`, `required_if_eq_any` / `_all`,
      `required_unless_all`
- [ ] `requires_if`: requires another flag only for some values
- [ ] `exclusive`: given alone or not at all
- [ ] `arg_group`: an enum (`#[derive(ArgGroup)]`) whose variants are the
      group's flags; `Option<E>` optional, `E` required, `multiple` keeps an
      ordered stream (ours is close: `#[arg(sequence)]` with `Occurrence`)
- [ ] `validate = "…"` expressions (usage's `validation` feature): portable
      cross-field rules
- [ ] Typed finalization: a hook run on the built struct, its error reported as a usage error
- [ ] Selectors naming an ancestor's global flag (resolved at runtime in usage)
- [ ] Relations on positionals and across `flatten`

## Command policies

- [ ] `args_override_self = false`: a repeated scalar flag is an error (ours: last wins)
- [ ] `subcommand_negates_reqs`: selecting a subcommand lifts the parent's requirements
- [ ] `args_conflicts_with_subcommands`: once the parent took an argument, no subcommand
- [ ] `subcommand_precedence_over_arg`: a subcommand name ends a variadic
- [ ] `allow_missing_positional`: an optional positional before a required one
      (ours: a compile-time order rule, **decision** in CHECKLIST.md)
- [ ] `dont_delimit_trailing_values`
- [ ] `default_subcommand_on_empty`, `default_subcommand_flags`
- [ ] External subcommands: an unknown word becomes `Vec<OsString>` (`cargo foo`)
- [ ] Multicall: `argv[0]`'s basename selects the subcommand (busybox)
- [ ] Executable views: `view("bin", root = "cmd")`, one command as another
      executable's root
- [ ] Sigil arguments: a positional told apart by a prefix (`+node@22`), stripped
- [ ] Clauses: a repeatable group of flags and positionals, split by a separator
      (`:::`) or a terminal positional (ours: `restart_token` only)
- [ ] Mounts and dynamic commands: subcommands from a plugin's spec, at runtime

## Deprecation

- [ ] `deprecated = "…"` on a flag, a command and an env variable; a warning when
      used, `deprecated_warn_at` / `deprecated_remove_at` releases
- [ ] Warnings channel: parse succeeds and returns what to warn about

## Help and presentation

- [ ] `long_version`: `--version` longer than `-V`
- [ ] `usage = "…"`: a verbatim synopsis
- [ ] `before_help` / `before_long_help`
- [ ] `display_order = n`
- [ ] `verbatim_doc_comment`: keep the doc comment's line breaks
- [ ] `note = "…"` / `warning = "…"`: semantic blocks in long help
- [ ] Examples: spec- and command-level examples in help
- [ ] Logo on the help page
- [ ] `help <topic>` for one section of a page
- [ ] A subcommand's help lists the global flags it inherits
- [ ] Rendered diagnostics with the command line and a caret (usage: miette)
- [ ] "did you mean" suggestions
- [ ] `hide_default_value`, `hide_env`
- [ ] `surface = "…"`, `available_if(…)`, `effect = "…"`, `author`: metadata,
      no effect on parsing

## Completion

- [ ] `value_hint = FilePath | DirPath | …`, and `extensions("toml")`: which
      paths to offer (ours: files, or not)
- [ ] `complete = f`: a completion function for a value
- [ ] Descriptions on value candidates
- [~] Scripts: bash and zsh tested end to end; fish, elvish, PowerShell never
      run in their shells

## Configuration

- [ ] Settings: `setting = "key"` binds a flag to a config-file key, resolved
      command line > env > config file > default; `config = Settings` emits the block

## Program surface

- [ ] `update_from` / `try_update_from`: merge another command line into a held value
- [ ] Generated dispatch: a `run` per subcommand with a shared context, sync or async
- [ ] Testing helpers: what a command writes, completions offered, help per page
- [ ] Outputs and exit codes declared per command (`output "json" …`)
- [ ] Emit the usage KDL spec (round-trip), so usage's tools (markdown, man
      pages, completions, SDKs, spec diff) work from a winnow-args program
- [ ] Markdown and man page generation (brush's `gen` example does it by hand
      from `help::Command`)
- [ ] `no_std` + `alloc`

## Not gaps

- Prefix inference: neither has it, on purpose.
- Help templates and clap styles: usage drops them; ours has `help::Style` and `Theme`.
- `flagset`: ours is `#[arg(flatten)]`, checked at compile time.
- Response files, `restart_token`, `double_dash` modes, `require_equals`,
  `allow_negative_numbers`, `default_missing`, `unknown_flags`, groups,
  `conflicts` / `requires` / `overrides` / `required_unless`, `global`,
  `alias_hidden`, `arg_required_else_help`, `default_subcommand`: done.

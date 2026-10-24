# Changelog

What changed in each release of `winnow-args`, `winnow-args-derive`,
`winnow-args-spec`, `winnow-args-man` and `winnow-args-markdown`, which are
released together. Versions follow [semver](https://semver.org).

## 0.5.0 (2026-10-24)

- **Added**: what a choice means. A `ValueEnum` variant's doc comment or
  `help = "…"`, and `choices("fast" = "…", "slow")` on a field: the long
  help and the pages list each value with it, one a line, as clap's long
  help does; the short help keeps `[possible values: …]`. The pages show
  the text as written, the field's before the type's. Help asks
  `FromArg::choice_help(index, default)`, `default` being the field's text,
  so a `FromArg` written by hand can answer from what only the running
  program has, as another crate's enum's docs, when it sets
  `DESCRIBES_CHOICES`. `winnow_args_spec::Item::listed_choices` gives them
  to a page.
- **Added**: `hide_choices` on a field: help and the pages leave its choices
  out, which are still the only values accepted, and still completed.
- **Added**: `help = EXPR` on a field, a `&'static str` constant: what help
  shows, short and long, as a text made per target; `long_help` beside it
  is an error. The pages keep the doc comment.
- **Added**: `brief_help` on a struct: `--help`, and `help <command>` from
  the command above, print the short page, as `-h` does, and its row says
  "Print help".
- **Changed**: `help::Item` has two more fields (`choice_help`,
  `hide_choices`), `help::Command` one (`brief_help`), and
  `winnow_args_spec::Item` two and `Command` one of the same names. A value
  written by hand that ends in `..Item::EMPTY`, `..Command::EMPTY` or
  `..Default::default()` is unaffected. `choice_help` is a function, `None`
  unless something describes the choices: mise's binary is 8 KB larger,
  with 2 more relocations, and parsing takes no more instructions.
- **Added**: `deprecated`, or `deprecated = "use --new"`, on a flag, a
  positional or a subcommand variant, with `deprecated_warn_at` and
  `deprecated_remove_at`. Giving it records a `warn::Warning`: `Args::parse`
  prints those due on standard error, and `Args::warnings` returns them
  after any other entry point. One whose `deprecated_warn_at` is a later
  release than the program's `version` is quiet. Help marks a flag or a
  positional `[deprecated]`, and its long help and the pages say why and
  until when; a command is `(deprecated)` in the list, and its own page
  says why. The `warn` module has `Warning`, `WarningKind` and
  `Deprecation`; `winnow_args_spec` has `Deprecated`, and
  `Command::sub_about` gives a command's line with `(default)` and
  `(deprecated)`.
- **Added**: `env_fallback("A", "B")` on a field: more variables, read in
  order after `env`; and `deprecated_env("OLD")`, read last, whose value
  records a warning naming `env`. The pages list every name.
- **Changed**: `help::Item` and `help::Sub` have the field `deprecated`, and
  `winnow_args_spec::Item` the fields `deprecated`, `env_fallback` and
  `deprecated_env`, and `Sub` `deprecated`. mise's binary is 8 KB larger
  for it, the pointer on each item; parsing takes no more instructions.
- **Changed**: `color::Depth`, and `winnow_args_spec`'s `Effect`, `Mark` and
  `Word` are `#[non_exhaustive]`, so a later release can add to them: a
  `match` on one needs a `_` arm. `token::Arg` and `Kind` stay exhaustive,
  as every derived parser matches on them.

## 0.4.0 (2026-10-24)

- **Added**: `#[arg(external)]` on a `Subcommand` variant holding a `Vec`:
  an external subcommand, as `cargo foo` runs `cargo-foo`. A word that names
  no other variant and is not a flag, where nothing else takes it, is its
  first value, and the words after it the rest, as typed.
  `Subcommand::EXTERNAL` says an enum has one. A struct with
  `default_subcommand` cannot hold such an enum. Completion offers nothing
  after its name, and the pages say a word that names no command is an
  external one.
- **Added**: `multicall` on a struct or a `Subcommand` enum: run under the
  name of one of its subcommands, as busybox's applets are linked, the
  program is that subcommand. Its help names it by the name it was run as,
  and an error's offset counts from the first word after that name. Under
  any other name the line is read as usual. `parse`, `try_parse` and
  `parse_from_argv` do this; `parse_words` has no program name. Completion
  follows the name typed, and `completion_script` completes each command's
  name and shown aliases too, asking the program itself, so a name not
  linked to it never runs. The pages say so.
- **Added**: `complete::scripts(&[program, names…], shell)`: one script for
  a program and the names it is linked as, each asking the program.
- **Added**: a `multicall` program's commands have pages of their own name,
  `ls.1` and `ls.md` in place of `busybox-ls.1`, whose synopsis gives
  `ls` and `busybox ls`; in a man page, a shown alias is a page that sources
  its command's with `.so`. Such a page's `path` starts at the command's
  name, and its `full_path` at the program's. `Page` has the methods
  `usage` and `sub_stem`. `applet_aliases(root, &pages)` lists the shown
  aliases and the command each stands for, and is an error when one is a
  page's name.
- **Added**: `bool_value` on a `bool` flag: it also takes `--flag=true` and
  `--flag=false`, attached, and alone is true. Help shows `--flag[=<BOOL>]`
  and its two values. `Arg::switch_value` reads one.
- **Added**: `value_names("X", "Y")` on a `Vec` flag: one placeholder a word,
  `--point <X> <Y>`, and as many words each time.
- **Added**: `long_version = "…"` on a struct: what `--version` prints,
  where `-V` keeps `version`. `Error::long_version` is its request.
- **Added**: `before_help` and `before_long_help` on a struct: text before
  the description, in help and the pages.
- **Added**: `usage = "…"` on a struct: the usage lines as written, in help
  and the pages, in place of the ones made from the fields.
- **Added**: a bare `env` on a field reads the variable of the field's name
  in capitals: `jobs` reads `JOBS`.
- **Added**: `default` repeated on a `Vec`, one value each, in order. Help
  shows them all.
- **Added**: `required_if_all(…)`, written as `required_if` and repeated:
  the field is required only when every one of them holds.
- **Added**: `visible_alias` on an `Args` flag or an `Occurrence` variant:
  another long name, listed in help, suggestions and completion. `alias` and
  a second `long` stay hidden.
- **Added**: `requires` names a group. When the field is supplied, one of the
  group's members has a value. `message` works as for a field: `{0}` is this
  field and `{1}` the members. The error is `Error::one_of_required_by`, of
  the kind `MissingOneOf`, with the group's name as its token. If a field has
  the group's name, the field is meant.
- **Added**: `conflicts` and `requires` take `message = "…"` after their
  names, and `group` takes it among its options: the program's own words in
  place of the default error, `{0}` and `{1}` the two names. The error's kind
  is unchanged.
- **Added**: the `unicode` feature measures help in terminal columns, so a
  wide character takes two and a combining mark none, as clap's does.
- **Added**: `verbatim_doc_comment` keeps a doc comment's lines as written,
  on a field, a struct, a `Subcommand` enum or a variant.
- **Changed**: smaller generated parsers: a non-ASCII short letter is decoded
  out of line, and the lexer has no inlined panic paths. The bench's mise
  binary is 5 % smaller, and no slower.
- **Changed**: every required flag and positional left without a value is
  reported at once, under "the following required arguments were not
  provided", each as `--help` lists it: its spellings, value, description
  and possible values. The error's kind and token are the first one's.
- **Changed**: `winnow_args_spec::Page` is `#[non_exhaustive]`, and has the
  field `full_path`: it is no longer built by a struct literal.
- **Changed**: `help::Command` has six more fields (`before_help`,
  `before_long_help`, `usage`, `long_version`, `multicall`,
  `external_subcommands`) and
  `winnow_args_spec::Command` five (`before_help`, `before_long_help`,
  `usage`, `external_subcommands`, `multicall`). A value written by hand
  that ends in `..Command::EMPTY` or `..Command::default()` is unaffected.
- **Changed**: `help::Item` and `winnow_args_spec::Item` have two more
  fields, `visible_aliases` and `value_names`. A value written by hand that
  ends in `..Item::EMPTY` or `..Item::default()` is unaffected.
- **Fixed**: a doc comment's paragraph is wrapped to the terminal, not where
  its source lines break; an indented line is kept as written.

## 0.3.3 (2026-10-18)

- **Added**: `Args` derives for a unit struct (`struct Noop;`), which takes
  no words.
- **Fixed**: deriving `Args` on a struct with a field named with a leading
  `_`, such as `_args`, no longer raises a `non_snake_case` warning in the
  crate that derives it.

## 0.3.2 (2026-10-17)

- **Added**: `examples/gallery.rs` uses every item help renders.
- **Changed**: help paints an example's line word by word: the program, its
  subcommands, its flags, `<NAME>` placeholders and a `# comment`. The text
  is unchanged.
- **Changed**: a man page sets an example's line as a synopsis: the
  program, its subcommands and flags in bold, `<NAME>` in italic, the rest
  roman, where the whole line was bold. `winnow_args_spec::words` reads the
  line as help does.
- **Changed**: a `*emphasized*` or `**strong**` span loses its stars where
  it is rendered: in painted help, where it was painted with its stars kept;
  in a man page, now in italic or bold; and in Markdown, where it was
  escaped and is now Markdown's own. A `` `code` `` span keeps its
  backticks, in bold on a man page. Unpainted help is unchanged.
- **Changed**: with `default_subcommand_help`, the default subcommand's page
  follows a `Default command: name` line, without the sentence that
  explained it or the description its line in the list gives. `--help`
  keeps what the description says beyond that line.
- **Fixed**: a `Vec` flag with `values = 1..` is shown as `--tag <TAG>...`
  in help and the pages, not with a second `...`.
- **Fixed**: on a man or Markdown page, a sentence the page adds after a
  description (`Warning: …`, `The default.`) follows a period when the
  description ends in none, as help's descriptions often do.

## 0.3.1 (2026-10-17)

- **Added**: `validate = function` on a struct, called with the struct once
  it is built: the program's own check of what the rules do not say. A
  variant with fields takes it too, and its function is given the enum.
  `Error::invalid` is the error for a fault that is not one value's, of the
  new kind `ErrorKind::Invalid`.
- **Added**: `required_if("--flag")` and `required_if("--flag", "value")` on
  a field: required when that flag was given, or given that value; repeated,
  any one condition is enough. `required_unless_all(…)`: required unless all
  of those have a value.
- **Added**: `requires_if("x", "--other")` on a field: when its value is `x`,
  `--other` needs one. `exclusive`: no other field of the struct is given
  with it.
- **Added**: `default_from(field, …, function)` on a field: a default made
  from other fields' values by a function or a closure, after their own
  defaults. The field keeps its type, and help says what it is made from.
- **Added**: `default_subcommand_on_empty` on a struct: a line that names no
  subcommand and gives no positional and no `--`, its own flags aside, runs
  the default subcommand, as in usage.
- **Added**: `hide_default_value` and `hide_env` on a field leave its default
  or its environment variable out of help and the pages; `display_order = n`
  moves a flag in help.
- **Changed**: help paints the flag named in the help flag's own description
  (`see a summary with '-h'`) as a flag.

## 0.3.0 (2026-10-12)

- **Added**: a subcommand's help lists the global flags it inherits, under
  `Global options:`. A parse through `parse_words` and what is built on it
  knows them; a page rendered with `help::render` from one command alone
  does not.
- **Added**: help marks the default subcommand `(default)`, and
  `default_subcommand_help` has its page follow the command's own.
  Completion offers what the default subcommand takes where it is not
  named: its first word, and its flags under `default_subcommand_flags`.
- **Added**: `example = "…"` and `example("…", header = "…", help = "…")` on
  a struct: an Examples section after the lists.
- **Added**: `next_help_heading = "…"` on a struct, the heading of its fields
  that name none, and `heading("Title", help = "…")`, a paragraph `--help`
  shows under a heading.
- **Added**: `flatten_help` on a struct with subcommands: each one's usage
  line, description and arguments stand where their list would.
- **Added**: `warning = "…"` on a field, shown by `--help`.
- **Added**: a `Subcommand` variant may have named fields. They are parsed as
  a struct's are, with the variant's doc comment and options as the
  command's: `Build { #[arg(long)] release: bool }`.
- **Added**: `effect`, `output(…)` and `exit_code(…)` on a struct, and
  `effect` on a field, for the man and markdown pages, which also show the
  examples, the paragraphs under headings, the warnings and the default
  subcommand.
- **Added**: `from_str` on a field: its value is parsed with the type's
  `FromStr`, for a type of another crate that is not `FromArg`, with no
  `Parsed<T>` in the field.
- **Fixed**: after a `double_dash = "automatic"` positional stopped flags, a
  `--` before the next automatic positional has a value is a separator and
  not that value: `tool name -- -x` leaves `-x`. Only the first is: a
  second `--` is a word.
- **Fixed**: a markdown page had trailing spaces on the blank line between
  two paragraphs of a description, and backticks in a flag's spelling ended
  its code span: a span or a fence is now longer than any run it holds.
- **Changed**: `help::Command` has six more fields (`default_subcommand`,
  `default_subcommand_flags`, `default_subcommand_help`, `flatten_help`,
  `examples`, `headings`) and `help::Item` one (`warning`). Both now have a
  public `EMPTY`: a value written by hand ends in `..Command::EMPTY` or
  `..Item::EMPTY`, and later fields do not break it. What a derive writes is
  unaffected.
- **Changed**: `winnow_args_spec::Command` and `Item` have more fields; both
  have a `Default`.

## 0.2.1 (2026-10-11)

- **Added**: `default_subcommand_flags` on a struct with a
  `default_subcommand`. A flag the struct does not take selects the default
  subcommand, which reads it as its own: `em -pu @world` is
  `em emerge -pu @world`. Every later word is then that subcommand's.
- **Added**: a flattened struct may have `global` flags. They are global
  from the struct that flattens it: its subcommands take them at any depth,
  as they take its own.
- **Added**: `#[arg(run)]` and `#[arg(run_async)]` on a `Subcommand` enum,
  with the traits `run::Run` and `run::RunAsync`. Each command implements the
  trait, and the enum implements it by calling the one it holds, with
  whatever context the commands take. A parse pays nothing for it.

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

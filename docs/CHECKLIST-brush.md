# Checklist: brush's bash builtins

What [brush](../../brush-abstract-arg-parsing) needs to parse bash builtin
arguments with winnow-args. That branch already abstracts builtin parsing over
engines (clap, bpaf, usage; `brush-builtins/src/*/{clap,bpaf,usage}.rs`) and
records a shootout between them, so winnow-args would be a fourth engine.
Behaviour is bash 5.3's, checked by running it; `[x]` means winnow-args covers
it today (with the feature that does), `[ ]` that it does not yet.

Builtins run on every call, inside the scripts' loops (`read` in a `while`,
`echo`, `[`), so per-call parse cost matters as much as for a CLI's one parse.

## 1. Option zone, then operands

- [x] Options stop at the first operand: `echo hi -n` prints `hi -n`.
      (`double_dash = "automatic"` on the operand positional)
- [x] One leading `--` ends the options and is dropped; a later `--` is data:
      `printf -- --` prints `--`. (the lexer's `--`, `Mode::Stopped`)
- [x] `--` kept as data where bash keeps it: `echo -- -n` prints `-- -n`.
      (`double_dash = "preserve"`)
- [x] A lone `-` is an operand (`cd -`, `trap - USR1`, `set -`).
- [ ] `set -` also ends the options *and* turns off `-x`/`-v`: the builtin needs
      to see that the `-` was given, not only what follows it.
- [x] Everything after the option zone verbatim, flag-like or not: `command`,
      `builtin`, `exec`, `eval`, `printf`'s arguments after the format.
      (`double_dash = "automatic"` + `allow_hyphen_values` operands)

## 2. What counts as an option word

- [x] A word is an option only if every letter is one of the builtin's:
      `echo -nx hi` prints `-nx hi`, `echo -n -e` takes both.
      (`unknown_flags = "value"`: a bundle is checked whole before any letter binds)
- [x] Unknown option words become operands where bash does so (`echo`), and
      are errors elsewhere (`set -q`: `invalid option`). (per struct)
- [x] Negative numbers as operands: `exit -1` (255), `return -2` (254),
      `shift -1` (parsed, then out of range). (`allow_negative_numbers`)
- [x] Negative numbers as option values: `history -d -1`.
- [ ] `+N`/`-N` directory-stack indices: `dirs +1`, `dirs -0`, `pushd +2`,
      `popd -1`. `-N` needs to reach the operand even where `-0`… would be a
      digit flag elsewhere.
- [ ] Signal specs as options: `kill -9 pid`, `kill -TERM pid`,
      `kill -SIGTERM pid`, beside `kill -s TERM`, `-n 9`, `-l`. A dash word
      naming a signal must win over reading it as the bundle `-s IGTERM`.

## 3. `+` options

- [ ] `+x` turns an option off where `-x` turns it on: `set +e`, `set +o
      pipefail`, `declare +i`, `declare +x`, `typeset +r`, `shopt`-style pairs.
      brush's usage engine rewrites `+abc` into `--+a --+b --+c` before parsing
      (`args/usage_support.rs`); winnow-args could lex `+` bundles natively
      beside `-` bundles.
- [ ] `+` bundles stop at the same boundary as `-` ones (the first operand,
      `-`, `--`); `+` words after it are data.
- [x] `negate`-style pairs for the long forms (`--no-name`). (`negate`)

## 4. Values

- [x] Value-taking letter ending a bundle takes the next word:
      `set -euxo pipefail`. (tested)
- [x] Attached values: `read -rp prompt: x`, `read -d: x`, `-rdX`.
- [x] Empty-string values: `read -d '' x`, `mapfile -d ''`. (tested)
- [x] Values that look like options: `complete -P -x`, `printf -v var`.
      (`allow_hyphen_values`)
- [x] Optional values: `set -o` / `set +o` alone list the options,
      `set -o name` sets one; `ulimit -n` queries, `ulimit -n 1024` sets.
      (`value_optional` + `default_missing`)
- [ ] `ulimit` values are numbers or `unlimited`/`hard`/`soft`, and several
      resource letters combine with `-S`/`-H`: `ulimit -Sn 1024`.
- [x] Repeated value options accumulate: `complete -o a -o b`, `set -o x -o y`.

## 5. Semantics bash gives some pairs

- [x] Last one wins: `command -v`/`-V`, `-e`/`-E` in `echo`. (`overrides`)
- [ ] `-o`/`+o` with the same name in one command: order of application.
- [x] Assignments as operands: `declare -i n=3`, `export A=1 B`, `local x=`.

## 6. Not an option grammar

- [ ] `test` / `[`: an expression language (`-n`, `-z`, `-f` are operators,
      `[` needs a closing `]`); take the words raw, as brush does.
- [ ] `getopts optstring name [args]`: the builtin *is* a getopt; its own
      arguments are plain. Could reuse the lexer for the parsing it performs.
- [ ] `let`, `:`, `true`, `false`: no options at all; `--help` is data.

## 7. Help and errors

- [ ] bash's error shape: `bash: set: -q: invalid option`, then
      `set: usage: set [-abefhkmnptuvxBCEHPT] [-o option-name] [--] [-] [arg ...]`,
      status 2. winnow-args' `Error` has the kind and token; brush needs a
      renderer for this shape (and the one-line synopsis per builtin).
- [ ] `help -d`, `help -s`, `help -m` read the builtins' descriptions:
      the `help::Command` data, rendered bash's way.
- [x] Builtins that must not treat `--help` specially (`echo --help` prints it).
      (`disable_help_flag`)

## 8. Integration and measurement

- [ ] A winnow-args engine beside clap/bpaf/usage in brush-builtins, behind
      its own feature, passing brush's builtin test suite.
- [ ] The shootout's numbers for it: per-invocation cost of `echo`, `read`,
      `printf`, `set`, `declare` in a loop, and binary size of brush.

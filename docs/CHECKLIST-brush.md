# Checklist: brush's bash builtins

What [brush](../../brush-abstract-arg-parsing) needs to parse bash builtin
arguments with winnow-args, as a fourth engine beside the clap, bpaf and
usage ones on its `abstract-arg-parsing` branch. Each builtin there has one
file per engine (`brush-builtins/src/<builtin>/{clap,bpaf,usage}.rs`), picked
by a cargo feature (`parser-clap`, `parser-bpaf`, `parser-usage`); a
`parser-winnow` feature and `winnow.rs` files would follow the same pattern.

Legend: `[x]` winnow-args covers it today (the feature that does is named),
`[ ]` not yet. Behaviour is bash 5.3's, checked by running it.

## How it is checked

- **Conformance**: brush's compat suite runs each case's script in brush and
  in bash and compares the output: 905 cases under
  `brush-shell/tests/cases/compat/builtins/*.yaml` (162 marked as known
  failures or skipped for some engine). The target is the clap engine's
  baseline: every case it passes, the winnow engine passes. Case names below
  are the vectors for each item (`file: "case"`).
- **Speed**: `benchmarks/three-way.py` runs bash and each engine interleaved,
  pinned, with oracle parity (byte-identical output). The recorded shootout
  (`benchmarks/results/backends-c986f537/`) is the bar:

  | workload | bash | clap | bpaf | usage |
  |---|---:|---:|---:|---:|
  | startup | 2.41 ms | 4.69 | 4.70 | 4.85 |
  | interp-loop | 154 ms | 285 | 290 | 233 |
  | wordops | 344 ms | 111 | 116 | 142 |
  | config-lint-500 (parse-densest) | 25.5 ms | 107 | 79 | 62 |

  Binary size of `brush`: clap 6 535 448, bpaf 6 601 496, usage 7 007 208 bytes.
- Builtins parse on every call, inside scripts' loops (`read` in a `while`,
  `echo`, `[`), so per-call cost matters as much as for a CLI's one parse.

## Phase 1: the engine, and builtins with ordinary grammar

- [ ] `parser-winnow` feature in brush-core, brush-builtins and brush-shell;
      an `impl_winnow_parse!` like `impl_usage_parse!`
      (`brush-builtins/src/args/usage_support.rs`) mapping a derived
      `winnow_args::Args` to brush's `FromArgs`.
- [ ] Errors in bash's shape: `bash: set: -q: invalid option`, then
      `set: usage: set [-abefhkmnptuvxBCEHPT] [-o option-name] [--] [-] [arg ...]`,
      status 2. winnow-args' `Error` has the kind and the token; the engine
      renders them, with each builtin's one-line synopsis.
- [ ] Builtins with no special grammar ported and passing their compat files:
      `alias`, `unalias`, `hash`, `pwd`, `times`, `true_false`, `colon`,
      `caller`, `enable`, `jobs`, `wait`, `type`, `unset`, `readonly`,
      `mapfile`, `read`.
- [x] Attached and bundled values: `read -rp prompt: x`, `-rdX`.
      (`read: "read -a with empty lines"` and the other 66 `read` cases)
- [x] Empty-string values: `read -d '' x`, `mapfile -d ''`.
- [x] Repeated value options accumulate: `complete -o a -o b`.
- [x] Builtins that must not treat `--help` specially. (`disable_help_flag`)
- [ ] Measure: startup and interp-loop within noise of the other engines.

## Phase 2: option zone, then verbatim operands

- [x] Options stop at the first operand: `echo hi -n` prints `hi -n`.
      (`double_dash = "automatic"`)
- [x] A word is an option only if every letter is the builtin's:
      `echo -nx hi` prints `-nx hi`. (`unknown_flags = "value"`, whole-bundle check)
- [x] `--` kept as data where bash keeps it. (`double_dash = "preserve"`;
      `echo: "echo with only --"`, `"echo with -- and args"`)
- [x] One leading `--` dropped, later ones data: `printf -- --`.
      (`printf: "printf format string starting with hyphen via --"`,
      `"printf with -- among format arguments"`)
- [ ] `printf` accepts only `-v` before the format; anything else leading is
      an invalid option. (`printf: "printf with -v as a format arg"`,
      `"printf with hyphen-prefixed format string (invalid option)"`,
      `"printf with option-like format arguments and attached values"`)
- [ ] Everything after the option zone verbatim: `command`, `builtin`,
      `exec`, `eval`, `.`/`source`, `let`.
      (`builtin: "valid builtin with hyphen args"`, `command: "command with --"`,
      `"command -v with multiple operands"`, `exec: "exec -a"`, `"exec -c"`)
- [x] Last one wins: `command -v`/`-V`. (`overrides`; `command: "command -V"`)
- [x] A lone `-` is an operand: `cd -`, `trap - SIG`.
- [ ] Measure: wordops and config-lint-500 against the shootout.

## Phase 3: `+` options

- [ ] Lex `+x` bundles natively beside `-x` ones: `set +e`, `set +o pipefail`,
      `declare +i`, `declare +x`, `typeset +r`. brush's usage engine rewrites
      `+abc` into `--+a --+b --+c` before parsing; winnow-args should not need
      that. (`set: "set with options"`, `"set with multiple combined options"`;
      `declare: "Removing integer attribute prevents arithmetic evaluation"`)
- [ ] `+` bundles stop at the same boundary as `-` ones (first operand, `-`,
      `--`). (`set: "set with option-looking args"`)
- [ ] `set -`: ends the options and turns off `-x`/`-v`; `set --` clears the
      positional parameters. (`set: "set with -"`, `"set with --"`,
      `"set clearing args"`)
- [x] Value letter ending a bundle takes the next word: `set -euxo pipefail`.
- [x] Optional values: bare `set -o`/`set +o` list the options.
      (`value_optional` + `default_missing`; `set: "set with no args"`)
- [ ] Assignments as operands for the declaration builtins: `declare -i n=3`,
      `export A=1 B`, `local x=`. (the 129 `declare` cases, `local`, `export`)
- [ ] `complete`/`compgen` `-o`/`+o` option names.
      (`complete: "Roundtrip: complete -o options"`, `compgen: "compgen -o plusdirs"`)

## Phase 4: numeric and signal operands

- [x] Negative numbers as operands: `exit -1`, `return -2`, `shift -1`.
      (`allow_negative_numbers`; `exit: "Exit with i64 min"`)
- [x] Negative numbers as option values: `history -d -1`, `fc -l -3`.
      (`fc: "fc -l with negative indices"`, `"fc -s with negative offset"`)
- [ ] `+N`/`-N` directory-stack indices: `dirs +1`, `dirs -0`, `pushd +2`,
      `popd -1`. (`pushd_popd_dirs`; brush has TODOs for these in all engines)
- [ ] Signal specs as options: `kill -9 pid`, `kill -TERM pid`,
      `kill -SIGTERM pid`, beside `-s TERM`, `-n 9`, `-l`. A dash word naming a
      signal wins over the bundle `-s IGTERM`.
      (`kill: "kill -sigspec"`, `"kill -sigspec (numeric)"`,
      `"kill -sigspec (numeric, full name resolution)"`, `"kill -sigspec (numeric, invalid)"`)
- [ ] `ulimit`: resource letters with optional values that are numbers or
      `unlimited`/`hard`/`soft`, combined with `-S`/`-H`: `ulimit -Sn 1024`.
      (`ulimit: "ulimit -n"`, `"ulimit -d unlimited"`)
- [ ] `trap`: `trap -- 'cmd' SIG`, `trap - SIG`, `trap -p`, `trap -l`.
      (`trap: "trap -l - lists all signal names"`, `"trap unregistering"`)

## Phase 5: not an option grammar

- [ ] `test` / `[`: an expression language; take the words raw.
      (`test: "test: -- string"`, `"test: -o operator (shell option)"`)
- [ ] `getopts`: brush implements bash's getopt for scripts; its 43 cases pin
      the grammar (bundles, `-ovalue`, `--`, a lone `-`, a leading `:` in the
      optstring, `OPTIND` resets). winnow-args' lexer could implement it.
      (`getopts: "getopts: multiple options in one token"`,
      `"getopts: combined flags with arg-taking option consumes next arg"`,
      `"getopts: option value looks like another option"`, …)
- [ ] `help -d`, `help -s`, `help -m`: bash's layout of the builtins'
      descriptions, from the `help::Command` data. (`help: "Topic-specific help"`)

## Phase 6: the shootout

- [ ] `benchmarks/three-way.py` with a fourth binary built with
      `--features basic,reedline,minimal,parser-winnow`, oracle parity held.
- [ ] Every compat case the clap baseline passes, passing.
- [ ] Binary size of `brush` reported beside the other engines'.

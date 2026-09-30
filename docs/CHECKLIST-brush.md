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

Done on brush branch `winnow-args-engine` (local, `c8b308a0`).

- [x] `parser-winnow` feature in brush-builtins and brush-shell, and
      `arg_impl!(T, winnow)`: a ported builtin uses winnow-args with the
      feature on, every other builtin keeps the engine selected before, so
      builtins move one at a time. `impl_winnow_args!(T, "synopsis")` maps a
      derived `winnow_args::Args` to brush's `FromArgs` and `builtins::Command`
      (`brush-builtins/src/args/winnow_support.rs`).
- [x] Errors in bash's shape: `hash: -x: invalid option`, then
      `hash: usage: hash [-lr] [-p pathname] [-dt] [name ...]`, status 2;
      bash's own usage line where the builtin gives one (`help -s`), else one
      derived from the help data. brush prints the message as is (bash's
      `bash: line N:` prefix is the shell's, not the builtin's).
- [x] `--help` asks for help only as the first word, as in bash, and prints
      the usage line and description; `-h` is not help.
- [x] `help NAME` content (detailed, short usage, short description) from the
      derive's static help data.
- [x] Ported, compat results identical to clap's: `pwd`, `alias`, `unalias`,
      `hash`, `times`, `caller`, `enable`, `jobs`, `wait`, `mapfile`, `read`.
      The whole suite with the feature on: 2442 cases, 1971 passing, as with
      clap.
- [ ] Still on the old engine among the plain ones: `unset` (parses its
      options by hand), `type`, `readonly`, `true_false`, `colon`.
- [ ] Where bash is laxer than every brush engine: `pwd extra` ignores the
      operand, `caller notanumber` fails silently with status 1.
- [x] Attached and bundled values: `read -rp prompt: x`, `-rdX`.
      (`read: "read -a with empty lines"` and the other 66 `read` cases)
- [x] Empty-string values: `read -d '' x`, `mapfile -d ''`.
- [x] Repeated value options accumulate: `complete -o a -o b`.
- [x] Builtins that must not treat `--help` specially. (`disable_help_flag`)
- [x] Measure: `three-way.py` on node 3 / cpu 96, 15 samples: every workload
      within noise of clap (startup 4.56 ms vs 4.61, interp-loop 286 vs 288,
      wordops 121 ± 7 vs 115 ± 8, config-lint-500 106 ± 8 vs 103 ± 4,
      deploy-sim 56 vs 56). The workloads barely touch the ported builtins;
      `echo`, `printf` and `[` come in later phases. Binary +24 KB with both
      parsers linked.

## Phase 2: option zone, then verbatim operands

Done on brush branch `winnow-args-engine` (local, `402dfeb7`): `echo`,
`printf`, `exec`, `eval`, `let`, `.`/`source`. `command` and `builtin` move to
phase 3: brush parses them as declaration builtins, like `declare`.

- [x] Options stop at the first operand: `echo hi -n` prints `hi -n`.
      (`double_dash = "automatic"`)
- [x] A word is an option only if every letter is the builtin's:
      `echo -nx hi` prints `-nx hi`. (`unknown_flags = "value"`, whole-bundle check)
- [x] `echo`'s mix: a leading `--` kept as data and the first operand ending
      the options. (`double_dash = "preserve", stop_flags`)
- [x] `--` kept as data where bash keeps it. (`double_dash = "preserve"`;
      `echo: "echo with only --"`, `"echo with -- and args"`)
- [x] One leading `--` dropped, later ones data: `printf -- --`.
      (`printf: "printf format string starting with hyphen via --"`,
      `"printf with -- among format arguments"`)
- [x] `printf` accepts only `-v` before the format; anything else leading is
      an invalid option; no arguments prints the usage line alone. brush's
      other engines scan this by hand; winnow-args needs no special code. (`printf: "printf with -v as a format arg"`,
      `"printf with hyphen-prefixed format string (invalid option)"`,
      `"printf with option-like format arguments and attached values"`)
- [x] Everything after the option zone verbatim: `exec`, `eval`,
      `.`/`source`, `let` (no options at all: `let -x=1` is an expression).
- [x] …and `command`, `builtin` (brush hands `builtin` its words unparsed;
      `builtin -x` looking for a builtin named `-x` is the same under clap).
- [x] `--help` is data for `echo` (and will be for `true`, `test`):
      `impl_winnow_args!(…, no_help)`.
      (`builtin: "valid builtin with hyphen args"`, `command: "command with --"`,
      `"command -v with multiple operands"`, `exec: "exec -a"`, `"exec -c"`)
- [x] Last one wins: `command -v`/`-V`. (`overrides`; `command: "command -V"`)
- [x] A lone `-` is an operand: `cd -`, `trap - SIG`.
- [x] Measure: compat suite unchanged (2442 cases, 1971 passing, as with
      clap); `three-way.py` within noise (−1 % to +3 %). The workloads spend
      little of their time in argument parsing, so per-call loops tell the
      difference: `printf -v x %s y` 9.9 → 7.2 µs, `echo -n` 7.2 → 4.5,
      `pwd -P` 11.8 → 9.4, `read -r x <<< a` 18.2 → 11.7, `:` 3.1 → 3.1 (the
      control). winnow-args saves 2.4–6.5 µs a call over clap.

## Phase 3: `+` options

Done on brush branch `winnow-args-engine` (local, `c499caf0`): `set`,
`declare` (also `local`, `readonly`, `typeset`), `export`, `command`,
`builtin`. Left: `complete`/`compgen`.

- [x] Lex `+x` bundles natively beside `-x` ones (`#[arg(plus_options)]`;
      a tri-state `Option<bool>` field with `short = 'x', plus = 'x'`): `set +e`, `set +o pipefail`,
      `declare +i`, `declare +x`, `typeset +r`. brush's usage engine rewrites
      `+abc` into `--+a --+b --+c` before parsing; winnow-args should not need
      that. (`set: "set with options"`, `"set with multiple combined options"`;
      `declare: "Removing integer attribute prevents arithmetic evaluation"`)
- [x] `+` bundles stop at the same boundary as `-` ones (first operand, `-`,
      `--`). (`set: "set with option-looking args"`)
- [x] `set -`: ends the options and turns off `-x`/`-v`; `set --` clears the
      positional parameters (the operand list keeps its leading `-`/`--`:
      `double_dash = "preserve", stop_flags`). (`set: "set with -"`, `"set with --"`,
      `"set clearing args"`)
- [x] Value letter ending a bundle takes the next word: `set -euxo pipefail`.
- [x] Optional values: bare `set -o`/`set +o` list the options.
      (`value_optional` + `default_missing`; `set: "set with no args"`)
- [x] Assignments as operands for the declaration builtins (brush splits
      them off before parsing; `#[arg(skip)]` holds them, and
      `impl_winnow_args!(…, declarations = field)` stores them): `declare -i n=3`,
      `export A=1 B`, `local x=`. (the 129 `declare` cases, `local`, `export`)
- [ ] `complete`/`compgen` `-o`/`+o` option names.
- [x] Each name of a shared builtin gets its own bash usage line
      (`local: usage: local [option] name[=value] ...`).
- [x] Measure: compat suite unchanged (1971 passing, as with clap).
      `three-way.py`: config-lint-500 107 → 70 ms (−35 %), interp-loop
      285 → 232 ms (−19 %), the others within noise. Per call: `set -f +f`
      120 → 5.2 µs, `set +o noglob` 47 → 5.7, `declare -i n=1` 30 → 5.5,
      `command true` 9.4 → 5.5, `export E=1` 6.9 → 4.5; the control `:` 3.1
      both. The binary is 24 KB smaller than with clap alone.
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

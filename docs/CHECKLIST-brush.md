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
- [x] Still on the old engine among the plain ones: `unset`, `type`,
      `readonly`, `true_false`, `colon`: all ported by phase 6 (`readonly` is
      `declare`'s; `true`, `false`, `:` parse nothing).
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

Done on brush branch `winnow-args-engine` (local, `c499caf0`, `fe431505`):
`set`, `declare` (also `local`, `readonly`, `typeset`), `export`, `command`,
`builtin`, `complete`, `compgen`, `compopt`.

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
- [x] `complete`/`compgen`/`compopt` `-o`/`+o` option names: brush-core's
      completion enums derive `winnow_args::ValueEnum` under a
      `parser-winnow` feature, spelled `#[winnow_args(rename_all = "lowercase")]`
      because clap's derive on the same enums claims `#[arg]`. Per call:
      `compgen -W "a b" a` 26.2 → 10.6 µs, `complete -o nospace -W x cmd`
      29.8 → 8.4 µs.
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

Done on brush branch `winnow-args-engine` (local, `fa5bc249`): `kill`,
`ulimit`, `dirs`, `pushd`, `popd`, `trap`, `history`, `fc`, `exit`, `return`,
`shift`, `break`, `continue`. Compat suite unchanged (1971 passing). Per call:
`ulimit -n` 20.0 → 8.4 µs, `kill -0` 10.4 → 7.6, `trap -p` 8.6 → 5.6,
`shift 0` 5.6 → 4.4.

- [x] Negative numbers as operands: `exit -1`, `return -2`, `shift -1`.
      (`allow_negative_numbers`; `exit: "Exit with i64 min"`)
- [x] Negative numbers as option values: `history -d -1`, `fc -l -3`.
      (`fc: "fc -l with negative indices"`, `"fc -s with negative offset"`)
- [ ] `+N`/`-N` directory-stack indices: `dirs +1`, `dirs -0`, `pushd +2`,
      `popd -1`. The parsing is there (`+1` is a word, `-1` a negative number
      where `allow_negative_numbers`); brush implements the stack indices in
      none of its engines (TODOs in `dirs`, `pushd`, `popd`).
- [x] Signal specs as options: `kill -9 pid`, `kill -TERM pid`,
      `kill -SIGTERM pid`, beside `-s TERM`, `-n 9`, `-l`. A dash word naming a
      signal wins over the bundle `-s IGTERM`: `kill` is lenient, so a word
      with a letter it lacks is an operand kill.rs reads as a signal;
      `-l`/`-L` are short aliases.
      (`kill: "kill -sigspec"`, `"kill -sigspec (numeric)"`,
      `"kill -sigspec (numeric, full name resolution)"`, `"kill -sigspec (numeric, invalid)"`)
- [x] `ulimit`: resource letters with optional values that are numbers or
      `unlimited`/`hard`/`soft`, combined with `-S`/`-H`: `ulimit -Sn 1024`.
      (`ulimit: "ulimit -n"`, `"ulimit -d unlimited"`)
- [x] `trap`: `trap -- 'cmd' SIG`, `trap - SIG`, `trap -p`, `trap -l`.
      (`trap: "trap -l - lists all signal names"`, `"trap unregistering"`)
- [ ] Where bash's messages come from the builtins' own code, not parsing:
      `shift -1`/`continue -1` "out of range", `kill`'s "No such process".

## Phase 5: not an option grammar

Done on brush branch `winnow-args-engine` (local, `38c51eaa`): `test`/`[`,
`getopts`, `help`. Compat suite unchanged (1971 passing). Per call, loop
included: `[ a = a ]` 9.1 → 7.3 µs, `test -n x` 7.4 → 6.0, `getopts ab o -a`
10.7 → 7.7 (bash: 3.0, 2.8, 3.2).

- [x] `test` / `[`: an expression language; take the words raw
      (`unknown_flags = "value"`, `double_dash = "preserve"`, no `--help`).
      (`test: "test: -- string"`, `"test: -o operator (shell option)"`)
- [x] `getopts`: its own operands taken raw, the script's words after `name`
      handed over whole, `--` included; brush's getopt then runs its 43 cases
      as before (41 pass, 2 known to fail in both engines).
      (`getopts: "getopts: multiple options in one token"`,
      `"getopts: combined flags with arg-taking option consumes next arg"`,
      `"getopts: option value looks like another option"`, …)
- [ ] brush's getopt on winnow-args' lexer (bundles, `-ovalue`, `--`, a lone
      `-`): it works as it is, so only if it pays.
- [x] `help -s`: bash's `name: usage` line, identical to bash for every ported
      builtin. `help -d`: `name - about`, the text brush's: help need not
      match bash's wording.
      (`help: "Topic-specific help"`)
- [ ] `help -m`: man-page layout, unimplemented in every engine.

## Phase 6: the shootout

Done on brush branch `winnow-args-engine` (local, `10119107`, results in
`benchmarks/results/winnow-phase6-fe1019ba`): every builtin has a winnow-args
port (`bg`, `fg`, `cd`, `umask`, `type`, `shopt`, `suspend`, `unset`, `bind`,
`unimp` last).

- [x] `benchmarks/three-way.py`, bash / clap / winnow, 15 samples pinned:
      startup 4.39 / 4.48 ms (noise), interp-loop 286 → 246 ms,
      config-lint-500 103 → 59 ms (0.57×), deploy-sim 54 → 51 ms,
      wordops 115 → 114 ms.
- [x] Every compat case the clap baseline passes, passing: 1971 of 2442, 0
      failed, in both engines.
- [x] Binary size: clap 6 535 448, winnow 6 468 584 bytes (−65 KiB). clap
      stays linked: brush-shell's own command line and brush-builtins' direct
      dependency.
- [x] A clap-free build: on `winnow-port` (phase 7), clap remains only
      under uucore (`printf`'s formatter).

## Phase 7: on brush's engine-neutral contracts

brush's `args-abstracted-on-main` (`bb0b35b1`) has brush-core's
engine-neutral `FromArgs`/`HelpContent` and moves clap out of brush-core.
Local branch `winnow-port` (`../brush-winnow-port`) sits directly on it:
`6648e1aa` (the `brush-builtin-winnow` adapter), `072dda5a` (every builtin),
`c495e548` (brush's own command line, brushctl, gen, xtask, migration
guides). The earlier route through usage-port is kept as
`winnow-port-on-usage`.

clap is left in the dependency graph only through uucore, which `printf`
formats with and which requires clap in every release (0.10 to 0.12); its
`format` module does not use it. No clap code reaches the binary (an
unstripped release build has no clap symbol; uucore is 65 KB), so it stays a
compile-time dependency, by choice. The optional coreutils builtins are
uutils' clap-based tools.

Measured against its own base (`args-abstracted-on-main`, clap) and
usage-port (`ab991c00`, the same base with usage-rs), release builds, pinned
(node 3, core 96):

- Binary: clap 6 904 560, usage 6 733 808, winnow 6 464 048 bytes.
- three-way.py, 15 samples, bash / clap / usage / winnow (ms): startup
  2.35 / 4.51 / 4.49 / 4.48; config-lint-500 24.3 / 118.1 / 72.6 / 62.6;
  deploy-sim 21.2 / 55.4 / 52.5 / 53.6; wordops 302 / 160 / 152 / 156.
  interp-loop (no builtin parsing) 149 / 294 / 236 / 255 is an artifact:
  a byte-identical copy of the script at another path gives usage 1.697 G
  and winnow 1.675 G instructions (1.688 G / 1.814 G at the original path);
  the gap follows the path, not the parser.
- Per call, 100 000 iterations in a function, loop included (`:` is the
  loop), µs:

  | command | bash | clap | usage | winnow |
  |---|---|---|---|---|
  | `:` | 2.3 | 4.1 | 4.0 | 4.0 |
  | `set -f +f` | 3.1 | 132.0 | 83.2 | 5.8 |
  | `declare -i n=1` | 3.2 | 31.4 | 6.7 | 6.5 |
  | `local` | 2.4 | 27.6 | 4.6 | 4.5 |
  | `compgen -W "a b" a` | 4.2 | 25.6 | 9.6 | 9.2 |
  | `ulimit -n` | 3.3 | 18.1 | 6.6 | 6.4 |
  | `shopt -q extglob` | 2.9 | 12.6 | 6.6 | 6.2 |
  | `unset -v x` | 2.9 | 11.3 | 6.2 | 6.1 |
  | `getopts ab o -a` | 3.2 | 11.1 | 7.7 | 7.2 |
  | `cd .` | 5.6 | 11.2 | 6.6 | 6.3 |
  | `kill -0 $$` | 3.3 | 10.6 | 7.5 | 7.2 |
  | `command true` | 2.7 | 10.3 | 6.4 | 6.2 |
  | `[ a = a ]` | 3.0 | 9.4 | 7.7 | 7.2 |
  | `printf %s x` | 3.2 | 8.6 | 6.6 | 6.6 |
  | `echo -n` | 2.6 | 8.1 | 5.4 | 5.1 |
  | `export E=1` | 2.8 | 7.9 | 5.2 | 5.2 |
  | `trap -p` | 2.9 | 7.8 | 5.2 | 5.0 |
  | `test -n x` | 2.9 | 7.6 | 6.2 | 6.0 |
  | `shift 0` | 2.6 | 6.7 | 5.2 | 5.0 |
  | `type -t ls` | 10.6 | 20.3 | 14.8 | 14.5 |
  | `read -r v <<<x` | 11.1 | 19.0 | 12.5 | 12.2 |

After `ef4941e` (positional `Vec` sized once) and brush `f3b149ee` (the
adapter's word list on the stack), instructions per script, averaged over
four copies at different paths, usage / winnow: wordops 498.0 M / 498.7 M,
config-lint-500 427.1 M / 413.2 M, deploy-sim 26.41 M / 26.26 M. Per call:
`echo a b c d e f` 77 990 / 73 431; `printf -v x` with 1, 6 and 12 operands
54 784 / 54 407, 86 258 / 86 696, 129 751 / 131 919: usage-port parses
`printf` by hand over `String`s, winnow validates each operand's UTF-8
(about 100 to 200 instructions a word, under 2 % of `printf`).
With `printf` parsed by hand too (brush `c541c37b`), moving the
operands out of the word list instead of copying them: 53 877, 85 775 and
127 587 instructions for 1, 6 and 12 operands, under usage-port's at each;
wordops 497.9 M against usage-port's 498.4 M.

- [x] `brush-builtin-winnow`: `winnow_builtin!(T)`, `trailing_args = f`,
      `declarations = f`, as `usage_builtin!`; bash-shaped errors.
- [x] Every builtin; `-x`/`+x` pairs are `Option<bool>` fields
      (`usage_minus_or_plus_flag_arg!` removed); `complete`/`compgen` share
      their options with `#[arg(flatten)]`.
- [x] brush's command line: `+o`/`+O` native (the pre-parse rewrite gone),
      `--help` without `-h` (`disable_help_short`), `#[cfg]` fields.
- [x] Completion scripts (bash, zsh, fish, elvish, PowerShell) answered by
      `brush __complete_word__ --shell S --line L` from the help data
      (`winnow_args::complete`); bash and zsh checked end to end.
- [x] gen: man page and markdown from `CommandLineArgs::HELP`.
- [x] Compat suite 2106 of 2572, 0 failed (as usage-port); integration 48/48.
- [x] Release binary 6 472 000 bytes against usage-port's 6 733 808 (−256 KiB).
      three-way.py (15 samples) level with usage-port within noise; per call
      winnow is 0.1–0.6 µs faster, and `set -f +f` 77.8 → 5.0 µs.
- [x] `flatten` (`d2c7f5f`): `complete`/`compgen` share
      `CommonCompleteCommandArgs` as on usage-port (`be93f772`).
- [ ] fish, elvish and PowerShell scripts run in their shells.

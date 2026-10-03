# Contributing

Read [`docs/DESIGN.md`](./docs/DESIGN.md) first and keep it true as the design
changes. [`docs/CHECKLIST.md`](./docs/CHECKLIST.md) is the status: tick an item
only when a test covers it, and record a deliberate divergence from usage's
grammar there as a **decision**. Work happens on the `draft` branch.

## Tasks

The tasks are recipes of [`just`](https://github.com/casey/just); `just --list`
shows them all.

```
cargo test                    # the library: unit, integration and doctests
just check                    # every feature set and profile, clippy, docs, tests
just perf [argv...]           # parse cost against usage, bpaf and clap
PROFILE=release-lto just perf # one codegen unit and fat LTO: for sizes
BPAF010=1 just perf           # with the unreleased bpaf 0.10 as well
just bench-examples           # warm instructions per parse, the two examples
just bench-shell NAME=SHELL…  # a builtin call, and scripts, in bash-compatible shells
just bench-ld NAME=LINKER…    # parsing a link line, per word
just gen --help               # regenerate the mold port, the examples, the mise shadow
```

`just check` passes before a commit. Plain `cargo test` runs the doctests;
`cargo nextest` does not.

## Layout

- `winnow-args` — the runtime. On the parse path: `stream` (`Argv` over
  `&[&BStr]` words), `token` (the lexer), `combinator`, `value` (`FromArg`),
  `error`. Off it: `help`, `color`, `complete`, `response`, `env`.
- `winnow-args-derive` — the derives. What only generated code calls is under
  `winnow_args::__private`.
- `bench` (unpublished) — the same CLI, and mise's, in winnow-args, usage, bpaf
  and clap; `bench/argv.txt` and `bench/mise-argv.txt` are the lines.
- `xtask` (unpublished) — the generators. They edit text, so what they carry
  over stays verbatim; the `justfile` runs tools, it does not write Rust.

## Performance

Parsing a command line is the product; these rules decide between designs.

- **Fast beats lean.** Parse time comes first. A size reduction is taken only
  if it costs no time; shared code is left to LTO to factor, not moved out of
  line by hand (two such attempts were measured slower and reverted).
- **A feature costs nothing to those who do not use it.** A struct that does
  not declare it generates the same loop as before: compare the bench lines
  before and after, and say so in the commit.
- **The hot path is the generated loop**, `token` and `stream`: no
  indirection, allocation or out-of-line helper there without a measurement.
  `Argv` stays 32 bytes (asserted). Help, errors and completion are off the
  path: write them for clarity.
- **Measure, then quote.** Every feature gets an entry in
  [`docs/PERF.md`](./docs/PERF.md); a `perf:` commit quotes its numbers.
  Prefer instruction counts to wall time: warm (`PARSE_N=n` minus `PARSE_N=0`,
  divided by `n`) for a parse, cold (`just perf`) for a process. Pin to one
  core.
- **Numbers compare only on the same host and tool.** `just perf` uses
  cachegrind where valgrind works and `perf stat` medians otherwise; say which.

## Dependencies

The tree builds from a fresh clone, on releases only: every dependency comes
from crates.io with a semver requirement (`version = "1"`, never an exact pin),
the bench's too. No git or path dependency and no `[patch]` to a local
checkout: what is built, tested and measured is what a user gets.

One exception, outside the workspace: `bench/bpaf010` takes the unreleased bpaf
0.10 from git, and only `BPAF010=1 just perf` builds it, so its going out of
sync breaks nothing.

Until the first release the workspace tracks the latest stable Rust and
dependencies; no crate is pinned back for a lower MSRV.

## Other projects' code

usage, bpaf, clap and winnow are read for behaviour and API, never copied
from. The one exception is usage's generated mise shadows, vendored unmodified
in `bench/shadows/` with usage's license.

## Style

- `rustfmt`; no dead code, no unused dependencies; no `unsafe` in `winnow-args`.
- Doc comments on every public item; rustdoc clean under `-D warnings`;
  doctests green.
- Comments say why, briefly: no line over 150 characters, no paragraph over 5
  lines, nothing that restates the code or cites a task, commit or issue.
- Unit tests in a `#[cfg(test)] mod tests`; behaviour that must hold for both
  the combinators and the derive is tested in `winnow-args/tests/` with both.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat`, `fix`,
`refactor`, `perf` (quote the numbers), `docs`, `test`, `build`, `ci`, `chore`.
The body follows the comment limits: no line over 150 characters, no paragraph
over 5 lines.

A commit significantly assisted by an AI tool says so with an `Assisted-by:`
trailer in the kernel's format (`Assisted-by: Claude:claude-opus-5-5`), not
`Co-Authored-By:`. [`AGENTS.md`](./AGENTS.md) has the rules for coding agents.

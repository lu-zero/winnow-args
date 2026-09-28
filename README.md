# winnow-args

Command line parsing built from [winnow](../winnow) parsers over a `BStr`.

- `winnow-args/` — runtime: the `Argv` stream, lexer (`token`), bpaf-style
  combinators (`combinator`), `FromArg` values, `Args` trait.
- `winnow-args-derive/` — `#[derive(Args)]`.
- `bench/` — the same CLI in usage, winnow-args, bpaf 0.10 and clap; run
  `tasks/perf.sh [argv...]`.
- `docs/CHECKLIST.md` — what is done and what is next; `docs/DESIGN.md` — why;
  `docs/PERF.md` — measurements, one entry per feature.

Cargo features of `winnow-args`: `derive` (the derives) and `help-text` (the
prose of derived help; without it help still lists commands, flags, values and
defaults, and mise's shadow is 12 % smaller). Both are on by default.

```
cargo test
cargo run --example example -- -vp /tmp
tasks/perf.sh -v --path /tmp/x
```

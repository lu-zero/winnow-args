# winnow-args

Command line parsing built from [winnow](../winnow) parsers over a `BStr`.

- `winnow-args/` — runtime: the `Argv` stream, lexer (`token`), bpaf-style
  combinators (`combinator`), `FromArg` values, `Args` trait.
- `winnow-args-derive/` — `#[derive(Args)]`.
- `bench/` — the same CLI in usage, winnow-args, bpaf 0.10 and clap; run
  `tasks/perf.sh [argv...]`.
- `docs/CHECKLIST.md` — what is done and what is next; `docs/DESIGN.md` — why.

```
cargo test
cargo run --example example -- -vp /tmp
tasks/perf.sh -v --path /tmp/x
```
